use state_zen::prelude::*;
use state_zen::{AspectId, StateAspect, StateValue, Zone, Transition};
use state_zen::transition::EventId;
use state_zen::blueprint::BlueprintBuilder;
use state_zen::active_in::ActiveIn;
use state_zen::update::Update;

fn main() {
    // Define state aspects (dimensions of the state vector)
    let mode_aspect = StateAspect::new(
        AspectId(0),
        "mode",
        StateValue::String("idle".to_string()),
    );

    let battery_aspect = StateAspect::new(
        AspectId(1),
        "battery",
        StateValue::Integer(100),
    );

    let is_charging_aspect = StateAspect::new(
        AspectId(2),
        "is_charging",
        StateValue::Bool(false),
    );

    // Create a zone that activates when charging
    let charging_zone = Zone::new(
        "charging_zone",
        ActiveIn::aspect_bool(AspectId(2), true),
    )
    .with_on_enter(|| {
        println!("🔋 Started charging - entering charging zone");
    })
    .with_on_exit(|| {
        println!("🔋 Stopped charging - exiting charging zone");
    });

    // Create a low battery zone
    let low_battery_zone = Zone::new(
        "low_battery_zone",
        ActiveIn::aspect_lt(AspectId(1), 20),
    )
    .with_on_enter(|| {
        println!("⚠️  Low battery warning!");
    })
    .with_on_exit(|| {
        println!("✓ Battery level normal");
    });

    // Create a running zone
    let running_zone = Zone::new(
        "running_zone",
        ActiveIn::aspect_string_eq(AspectId(0), "running"),
    )
    .with_on_enter(|| {
        println!("▶️  System started");
    })
    .with_on_exit(|| {
        println!("⏸️  System stopped");
    });

    // Create transitions
    let start_transition = Transition::new(
        "start",
        ActiveIn::aspect_string_eq(AspectId(0), "idle"),
        EventId::new("start_button"),
        Update::set_string(AspectId(0), "running"),
    )
    .with_on_tran(|| {
        println!("🎯 Start button pressed - transitioning to running");
    });

    let stop_transition = Transition::new(
        "stop",
        ActiveIn::aspect_string_eq(AspectId(0), "running"),
        EventId::new("stop_button"),
        Update::set_string(AspectId(0), "idle"),
    )
    .with_on_tran(|| {
        println!("⏹️  Stop button pressed - transitioning to idle");
    });

    let charge_transition = Transition::new(
        "charge",
        ActiveIn::always(),
        EventId::new("charge"),
        Update::set_bool(AspectId(2), true),
    )
    .with_on_tran(|| {
        println!("🔌 Charger connected");
    });

    let uncharge_transition = Transition::new(
        "uncharge",
        ActiveIn::always(),
        EventId::new("uncharge"),
        Update::set_bool(AspectId(2), false),
    )
    .with_on_tran(|| {
        println!("🔌 Charger disconnected");
    });

    let consume_battery_transition = Transition::new(
        "consume_battery",
        ActiveIn::aspect_string_eq(AspectId(0), "running"),
        EventId::new("tick"),
        Update::compose(vec![
            Update::conditional_else(
                |s| s.get(AspectId(2)).map_or(false, |v| matches!(v, StateValue::Bool(true))),
                Update::increment(AspectId(1)), // charging: increase battery
                Update::decrement(AspectId(1)), // running: decrease battery
            ),
            Update::conditional(
                |s| s.get(AspectId(1)).map_or(false, |v| matches!(v, StateValue::Integer(i) if *i <= 0)),
                Update::compose(vec![
                    Update::set_string(AspectId(0), "idle"),
                    Update::set_int(AspectId(1), 0),
                ]),
            ),
        ]),
    );

    // Build the state machine blueprint
    let blueprint = BlueprintBuilder::new()
        .id("device_controller")
        .aspect(mode_aspect)
        .aspect(battery_aspect)
        .aspect(is_charging_aspect)
        .zone(charging_zone)
        .zone(low_battery_zone)
        .zone(running_zone)
        .transition(start_transition)
        .transition(stop_transition)
        .transition(charge_transition)
        .transition(uncharge_transition)
        .transition(consume_battery_transition)
        .build()
        .unwrap();

    // Print blueprint statistics
    let stats = blueprint.stats();
    println!("\n📊 Blueprint Statistics:");
    println!("  Aspects: {}", stats.aspect_count);
    println!("  Zones: {}", stats.zone_count);
    println!("  Transitions: {}", stats.transition_count);
    println!("  Events: {}", stats.event_count);

    // Create initial state
    let mut state = blueprint.create_initial_state();
    println!("\n📍 Initial State:");
    print_state(&blueprint, &state);

    println!("{}", "=".repeat(50));

    // Simulate state machine events
    println!("\n🎬 Simulation:\n");

    // Start the device
    simulate_event(&blueprint, &mut state, &EventId::new("start_button"));
    simulate_event(&blueprint, &mut state, &EventId::new("tick"));
    simulate_event(&blueprint, &mut state, &EventId::new("tick"));
    simulate_event(&blueprint, &mut state, &EventId::new("tick"));

    // Connect charger
    simulate_event(&blueprint, &mut state, &EventId::new("charge"));
    simulate_event(&blueprint, &mut state, &EventId::new("tick"));
    simulate_event(&blueprint, &mut state, &EventId::new("tick"));

    // Disconnect charger
    simulate_event(&blueprint, &mut state, &EventId::new("uncharge"));

    // Continue running until low battery
    simulate_event(&blueprint, &mut state, &EventId::new("tick"));
    simulate_event(&blueprint, &mut state, &EventId::new("tick"));
    simulate_event(&blueprint, &mut state, &EventId::new("tick"));

    // Stop the device
    simulate_event(&blueprint, &mut state, &EventId::new("stop_button"));

    println!("\n✅ Simulation complete!");
}

fn simulate_event(blueprint: &StateMachineBlueprint, state: &mut State, event: &EventId) {
    println!("📨 Event: {:?}", event.0);

    // Find and apply matching transitions
    for transition in blueprint.transitions() {
        if transition.event == *event && transition.is_active(state) {
            println!("  → Transition '{}' activated", transition.id);
            transition.trigger();
            *state = transition.apply(state.clone());
            break;
        }
    }

    // Check zone activations
    check_zone_activations(blueprint, state);

    println!("  Current State:");
    print_state(blueprint, state);
    println!();
}

fn check_zone_activations(blueprint: &StateMachineBlueprint, state: &State) {
    let active_zones: Vec<_> = blueprint
        .zones()
        .iter()
        .filter(|z| z.is_active(state))
        .map(|z| z.id.as_str())
        .collect();

    if !active_zones.is_empty() {
        println!("  Active zones: {}", active_zones.join(", "));
    }
}

fn print_state(blueprint: &StateMachineBlueprint, state: &State) {
    for aspect in blueprint.aspects() {
        if let Some(value) = state.get(aspect.id) {
            println!("    {}: {}", aspect.name, value);
        }
    }
}
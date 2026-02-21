use state_zen::{AspectId, AspectBlueprint, Zone, ZoneId, Transition, TransitionId, StateMachineRuntime};
use state_zen::transition::EventId;
use state_zen::active_in::ActiveInFactory;
use state_zen::update::Update;
use state_zen::StateMachineBlueprint;

fn main() {
    // Define state aspects (dimensions of the state vector)
    let mode_aspect = AspectBlueprint::new(
        AspectId(0),
        "mode",
        "idle".to_string(),
    );

    let battery_aspect = AspectBlueprint::new(
        AspectId(1),
        "battery",
        100i64,
    )
    .with_range(0i64, 100i64);

    let is_charging_aspect = AspectBlueprint::new(
        AspectId(2),
        "is_charging",
        false,
    );

    // Create a zone that activates when charging
    let charging_zone = Zone::new(
        ZoneId(0),
        "charging_zone",
        ActiveInFactory::aspect_bool(AspectId(2), true),
    )
    .with_on_enter(|| {
        println!("🔋 Started charging - entering charging zone");
    })
    .with_on_exit(|| {
        println!("🔋 Stopped charging - exiting charging zone");
    });

    // Create a low battery zone
    let low_battery_zone = Zone::new(
        ZoneId(1),
        "low_battery_zone",
        ActiveInFactory::aspect_lt(AspectId(1), 20i64),
    )
    .with_on_enter(|| {
        println!("⚠️  Low battery warning!");
    })
    .with_on_exit(|| {
        println!("✓ Battery level normal");
    });

    // Create a running zone
    let running_zone = Zone::new(
        ZoneId(2),
        "running_zone",
        ActiveInFactory::aspect_string_eq(AspectId(0), "running"),
    )
    .with_on_enter(|| {
        println!("▶️  System started");
    })
    .with_on_exit(|| {
        println!("⏸️  System stopped");
    });

    // Create transitions
    let start_transition = Transition::new(
        TransitionId(0),
        "start",
        ActiveInFactory::aspect_string_eq(AspectId(0), "idle"),
        EventId::new("start_button"),
        Update::set_string(AspectId(0), "running"),
    )
    .with_on_tran(|| {
        println!("🎯 Start button pressed - transitioning to running");
    });

    let stop_transition = Transition::new(
        TransitionId(1),
        "stop",
        ActiveInFactory::aspect_string_eq(AspectId(0), "running"),
        EventId::new("stop_button"),
        Update::set_string(AspectId(0), "idle"),
    )
    .with_on_tran(|| {
        println!("⏹️  Stop button pressed - transitioning to idle");
    });

    let charge_transition = Transition::new(
        TransitionId(2),
        "charge",
        ActiveInFactory::always(),
        EventId::new("charge"),
        Update::set_bool(AspectId(2), true),
    )
    .with_on_tran(|| {
        println!("🔌 Charger connected");
    });

    let uncharge_transition = Transition::new(
        TransitionId(3),
        "uncharge",
        ActiveInFactory::always(),
        EventId::new("uncharge"),
        Update::set_bool(AspectId(2), false),
    )
    .with_on_tran(|| {
        println!("🔌 Charger disconnected");
    });

    let consume_battery_transition = Transition::new(
        TransitionId(4),
        "consume_battery",
        ActiveInFactory::aspect_string_eq(AspectId(0), "running"),
        EventId::new("tick"),
        Update::compose(vec![
            Update::conditional_else(
                // Charging and battery < 100: increase, otherwise decrease
                |s| s.get_as::<bool>(AspectId(2)).map_or(false, |&v| v)
                     && s.get_as::<i64>(AspectId(1)).map_or(false, |&v| v < 100),
                Update::modify_typed::<i64, _>(AspectId(1), |v| v + 1), // charging: increase battery
                Update::modify_typed::<i64, _>(AspectId(1), |v| v - 1), // running: decrease battery
            ),
            Update::conditional(
                |s| s.get_as::<i64>(AspectId(1)).map_or(false, |&v| v <= 0),
                Update::compose(vec![
                    Update::set_string(AspectId(0), "idle"),
                    Update::set_int(AspectId(1), 0),
                ]),
            ),
        ]),
    );

    // Build the state machine blueprint
    let mut blueprint = StateMachineBlueprint::new("device_controller");
    blueprint.add_aspect(mode_aspect);
    blueprint.add_aspect(battery_aspect);
    blueprint.add_aspect(is_charging_aspect);
    blueprint.add_zone(charging_zone);
    blueprint.add_zone(low_battery_zone);
    blueprint.add_zone(running_zone);
    blueprint.add_transition(start_transition);
    blueprint.add_transition(stop_transition);
    blueprint.add_transition(charge_transition);
    blueprint.add_transition(uncharge_transition);
    blueprint.add_transition(consume_battery_transition);

    // Create runtime state machine instance
    let mut runtime = StateMachineRuntime::new(blueprint);

    println!("\n📍 Initial State:");
    print_state(&runtime);

    println!("{}", "=".repeat(50));

    // Simulate state machine events
    println!("\n🎬 Simulation:\n");

    // Start the device
    runtime.dispatch_str("start_button");
    print_runtime_state(&runtime);

    // Consume battery
    runtime.dispatch_str("tick");
    print_runtime_state(&runtime);

    runtime.dispatch_str("tick");
    print_runtime_state(&runtime);

    runtime.dispatch_str("tick");
    print_runtime_state(&runtime);

    // Connect charger
    runtime.dispatch_str("charge");
    print_runtime_state(&runtime);

    runtime.dispatch_str("tick");
    print_runtime_state(&runtime);

    runtime.dispatch_str("tick");
    print_runtime_state(&runtime);

    // Disconnect charger
    runtime.dispatch_str("uncharge");
    print_runtime_state(&runtime);

    // Continue running until low battery
    runtime.dispatch_str("tick");
    print_runtime_state(&runtime);

    runtime.dispatch_str("tick");
    print_runtime_state(&runtime);

    runtime.dispatch_str("tick");
    print_runtime_state(&runtime);

    // Stop the device
    runtime.dispatch_str("stop_button");
    print_runtime_state(&runtime);

    println!("\n✅ Simulation complete!");
}

fn print_state(runtime: &StateMachineRuntime) {
    for aspect in runtime.blueprint().aspects() {
        if let Some(value) = runtime.state().get(aspect.id) {
            // Try to format common types
            if let Some(b) = value.downcast_ref::<bool>() {
                println!("    {}: {}", aspect.name, b);
            } else if let Some(i) = value.downcast_ref::<i64>() {
                println!("    {}: {}", aspect.name, i);
            } else if let Some(f) = value.downcast_ref::<f64>() {
                println!("    {}: {}", aspect.name, f);
            } else if let Some(s) = value.downcast_ref::<String>() {
                println!("    {}: {}", aspect.name, s);
            } else {
                println!("    {}: {:?}", aspect.name, value);
            }
        }
    }
}

fn print_runtime_state(runtime: &StateMachineRuntime) {
    println!("📨 Event dispatched");

    let active_zones = runtime.active_zones();
    if !active_zones.is_empty() {
        println!("  Active zones: {}", active_zones.iter().map(|id| format!("{:?}", id)).collect::<Vec<_>>().join(", "));
    }

    println!("  Current State:");
    print_state(runtime);
    println!();
}

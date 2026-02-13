use state_zen::{AspectId, StateAspect, StateValue, Zone, Transition, StateMachineRuntime};
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
    )
    .with_range(StateValue::Integer(0), StateValue::Integer(100));

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
                // Charging and battery < 100: increase, otherwise decrease
                |s| s.get_as::<bool>(AspectId(2)).map_or(false, |&v| v)
                     && s.get_as::<i64>(AspectId(1)).map_or(false, |&v| v < 100),
                Update::increment(AspectId(1)), // charging: increase battery
                Update::decrement(AspectId(1)), // running: decrease battery
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
        println!("  Active zones: {}", active_zones.join(", "));
    }
    
    println!("  Current State:");
    print_state(runtime);
    println!();
}
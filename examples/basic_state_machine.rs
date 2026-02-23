use state_zen::{AspectId, AspectBlueprint, ZoneBlueprint, ZoneId, TransitionBlueprint, TransitionId, StateMachineRuntime};
use state_zen::core::EventId;
use state_zen::active_in::ActiveInBlueprint;
use state_zen::update::{Update, UpdateBlueprint};
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
    let charging_zone_blueprint = ZoneBlueprint::new(
        ZoneId(0),
        "charging_zone",
        ActiveInBlueprint::aspect_bool(AspectId(2), true),
    );

    // Create a low battery zone
    let low_battery_zone_blueprint = ZoneBlueprint::new(
        ZoneId(1),
        "low_battery_zone",
        ActiveInBlueprint::aspect_lt(AspectId(1), 20i64),
    );

    // Create a running zone
    let running_zone_blueprint = ZoneBlueprint::new(
        ZoneId(2),
        "running_zone",
        ActiveInBlueprint::aspect_string_eq(AspectId(0), "running"),
    );

    // Create transitions
    let start_transition_blueprint = TransitionBlueprint::new(
        TransitionId(0),
        "start",
        ActiveInBlueprint::aspect_string_eq(AspectId(0), "idle"),
        EventId::new("start_button"),
        UpdateBlueprint::set_string(AspectId(0), "running"),
    );

    let stop_transition_blueprint = TransitionBlueprint::new(
        TransitionId(1),
        "stop",
        ActiveInBlueprint::aspect_string_eq(AspectId(0), "running"),
        EventId::new("stop_button"),
        UpdateBlueprint::set_string(AspectId(0), "idle"),
    );

    let charge_transition_blueprint = TransitionBlueprint::new(
        TransitionId(2),
        "charge",
        ActiveInBlueprint::always(),
        EventId::new("charge"),
        UpdateBlueprint::set_bool(AspectId(2), true),
    );

    let uncharge_transition_blueprint = TransitionBlueprint::new(
        TransitionId(3),
        "uncharge",
        ActiveInBlueprint::always(),
        EventId::new("uncharge"),
        UpdateBlueprint::set_bool(AspectId(2), false),
    );

    let consume_battery_transition_blueprint = TransitionBlueprint::new(
        TransitionId(4),
        "consume_battery",
        ActiveInBlueprint::aspect_string_eq(AspectId(0), "running"),
        EventId::new("tick"),
        UpdateBlueprint::noop(), // Will be replaced with custom update in runtime
    );

    // Build the state machine blueprint
    let mut blueprint = StateMachineBlueprint::new("device_controller");
    blueprint.add_aspect(mode_aspect);
    blueprint.add_aspect(battery_aspect);
    blueprint.add_aspect(is_charging_aspect);
    blueprint.add_zone(charging_zone_blueprint);
    blueprint.add_zone(low_battery_zone_blueprint);
    blueprint.add_zone(running_zone_blueprint);
    blueprint.add_transition(start_transition_blueprint);
    blueprint.add_transition(stop_transition_blueprint);
    blueprint.add_transition(charge_transition_blueprint);
    blueprint.add_transition(uncharge_transition_blueprint);
    blueprint.add_transition(consume_battery_transition_blueprint);

    // Create runtime state machine instance
    let mut runtime = StateMachineRuntime::new(blueprint)
        // Add zone handlers
        .with_zone_on_enter(ZoneId(0), || {
            println!("🔋 Started charging - entering charging zone");
        })
        .with_zone_on_exit(ZoneId(0), || {
            println!("🔋 Stopped charging - exiting charging zone");
        })
        .with_zone_on_enter(ZoneId(1), || {
            println!("⚠️  Low battery warning!");
        })
        .with_zone_on_exit(ZoneId(1), || {
            println!("✓ Battery level normal");
        })
        .with_zone_on_enter(ZoneId(2), || {
            println!("▶️  System started");
        })
        .with_zone_on_exit(ZoneId(2), || {
            println!("⏸️  System stopped");
        })
        // Add transition handlers
        .with_transition_on_tran(TransitionId(0), || {
            println!("🎯 Start button pressed - transitioning to running");
        })
        .with_transition_on_tran(TransitionId(1), || {
            println!("⏹️  Stop button pressed - transitioning to idle");
        })
        .with_transition_on_tran(TransitionId(2), || {
            println!("🔌 Charger connected");
        })
        .with_transition_on_tran(TransitionId(3), || {
            println!("🔌 Charger disconnected");
        })
        // Custom update for consume_battery transition
        .with_transition_update(TransitionId(4), Update::compose(vec![
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
        ]));

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
            if let Some(b) = value.as_any().downcast_ref::<bool>() {
                println!("    {}: {}", aspect.name, b);
            } else if let Some(i) = value.as_any().downcast_ref::<i64>() {
                println!("    {}: {}", aspect.name, i);
            } else if let Some(f) = value.as_any().downcast_ref::<f64>() {
                println!("    {}: {}", aspect.name, f);
            } else if let Some(s) = value.as_any().downcast_ref::<String>() {
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

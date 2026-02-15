/// state-zen 使用指南
///
/// 本示例展示如何使用 state-zen 创建和运行状态机

use state_zen::{AspectId, Aspect, Zone, ZoneId, Transition, TransitionId, StateMachineRuntime, StateMachineBlueprint};
use state_zen::transition::EventId;
use state_zen::active_in::ActiveIn;
use state_zen::update::Update;

fn main() {
    println!("=== state-zen 使用指南 ===\n");

    // ============================================
    // 第一步：定义状态面 (StateAspect)
    // ============================================
    println!("1️⃣ 定义状态面");

    // 定义设备的状态面
    let mode: Aspect<String> = Aspect::new(
        AspectId(0),
        "mode",
        "idle".to_string()
    );

    let battery: Aspect<i64> = Aspect::new(
        AspectId(1),
        "battery",
        100
    )
    .with_range(0, 100);

    let is_charging: Aspect<bool> = Aspect::new(
        AspectId(2),
        "is_charging",
        false
    );

    println!("   ✓ 定义了 3 个状态面: mode, battery, is_charging\n");
    
    // ============================================
    // 第二步：定义区域 (Zone)
    // ============================================
    println!("2️⃣ 定义区域");
    
    // 低电量警告区域
    let low_battery_zone = Zone::new(
        ZoneId(0),
        "low_battery",
        ActiveIn::aspect_lt_typed(AspectId(1), 20)
    )
    .with_on_enter(|| {
        println!("   ⚠️ 警告：电量低于 20%！");
    })
    .with_on_exit(|| {
        println!("   ✓ 电量恢复正常");
    });

    // 充电区域
    let charging_zone = Zone::new(
        ZoneId(1),
        "charging",
        ActiveIn::aspect_eq_typed(AspectId(2), true)
    )
    .with_on_enter(|| {
        println!("   🔌 开始充电");
    })
    .with_on_exit(|| {
        println!("   🔌 停止充电");
    });

    // 运行区域
    let running_zone = Zone::new(
        ZoneId(2),
        "running",
        ActiveIn::aspect_eq_typed(AspectId(0), "running".to_string())
    )
    .with_on_enter(|| {
        println!("   ▶️ 设备启动");
    })
    .with_on_exit(|| {
        println!("   ⏸️ 设备停止");
    });
    
    println!("   ✓ 定义了 3 个区域: low_battery, charging, running\n");
    
    // ============================================
    // 第三步：定义转移 (Transition)
    // ============================================
    println!("3️⃣ 定义转移");
    
    // 启动设备
    let start_transition = Transition::new(
        TransitionId(0),
        "start",
        ActiveIn::aspect_eq_typed(AspectId(0), "idle".to_string()),
        EventId::new("start"),
        Update::set(AspectId(0), Box::new("running".to_string()))
    );

    // 停止设备
    let stop_transition = Transition::new(
        TransitionId(1),
        "stop",
        ActiveIn::aspect_eq_typed(AspectId(0), "running".to_string()),
        EventId::new("stop"),
        Update::set(AspectId(0), Box::new("idle".to_string()))
    );

    // 连接充电器
    let charge_transition = Transition::new(
        TransitionId(2),
        "charge",
        ActiveIn::always(),
        EventId::new("charge"),
        Update::set(AspectId(2), Box::new(true))
    );

    // 断开充电器
    let uncharge_transition = Transition::new(
        TransitionId(3),
        "uncharge",
        ActiveIn::always(),
        EventId::new("uncharge"),
        Update::set(AspectId(2), Box::new(false))
    );
    
    // 消耗电量
    let consume_transition = Transition::new(
        TransitionId(4),
        "consume",
        ActiveIn::aspect_eq_typed(AspectId(0), "running".to_string()),
        EventId::new("tick"),
        Update::compose(vec![
            Update::conditional_else(
                // 如果在充电且电量 < 100，增加电量；否则减少电量
                |s| s.get_as::<bool>(AspectId(2)).map_or(false, |&v| v)
                     && s.get_as::<i64>(AspectId(1)).map_or(false, |&v| v < 100),
                Update::modify_typed::<i64, _>(AspectId(1), |v| v + 1),
                Update::modify_typed::<i64, _>(AspectId(1), |v| v - 1)
            ),
            // 如果电量耗尽，自动停止
            Update::conditional(
                |s| s.get_as::<i64>(AspectId(1)).map_or(false, |&v| v <= 0),
                Update::compose(vec![
                    Update::set(AspectId(0), Box::new("idle".to_string())),
                    Update::set(AspectId(1), Box::new(0i64)),
                ]),
            ),
        ])
    );
    
    println!("   ✓ 定义了 5 个转移: start, stop, charge, uncharge, consume\n");
    
    // ============================================
    // 第四步：构建蓝图 (Blueprint)
    // ============================================
    println!("4️⃣ 构建蓝图");
    
    let mut blueprint = StateMachineBlueprint::new("device");
    blueprint.add_aspect(mode);
    blueprint.add_aspect(battery);
    blueprint.add_aspect(is_charging);
    blueprint.add_zone(low_battery_zone);
    blueprint.add_zone(charging_zone);
    blueprint.add_zone(running_zone);
    blueprint.add_transition(start_transition);
    blueprint.add_transition(stop_transition);
    blueprint.add_transition(charge_transition);
    blueprint.add_transition(uncharge_transition);
    blueprint.add_transition(consume_transition);
    
    let stats = blueprint.stats();
    println!("   ✓ 蓝图构建完成");
    println!("     - 状态面: {}", stats.aspect_count);
    println!("     - 区域: {}", stats.zone_count);
    println!("     - 转移: {}", stats.transition_count);
    println!("     - 事件: {}", stats.event_count);
    println!();
    
    // ============================================
    // 第五步：创建运行时实例
    // ============================================
    println!("5️⃣ 创建运行时实例");
    
    let mut runtime = StateMachineRuntime::new(blueprint);
    
    println!("   ✓ 初始状态:");
    print_state(&runtime);
    println!();
    
    // ============================================
    // 第六步：使用状态机
    // ============================================
    println!("6️⃣ 使用状态机 - 事件分发");
    println!();
    
    // 启动设备
    println!("📨 事件: start");
    runtime.dispatch_str("start");
    print_state(&runtime);
    println!("   活跃区域: {:?}", runtime.active_zones());
    println!();
    
    // 消耗电量
    println!("📨 事件: tick (消耗电量)");
    for _ in 0..3 {
        runtime.dispatch_str("tick");
    }
    print_state(&runtime);
    println!("   活跃区域: {:?}", runtime.active_zones());
    println!();
    
    // 连接充电器
    println!("📨 事件: charge (连接充电器)");
    runtime.dispatch_str("charge");
    print_state(&runtime);
    println!("   活跃区域: {:?}", runtime.active_zones());
    println!();
    
    // 充电
    println!("📨 事件: tick (充电)");
    for _ in 0..5 {
        runtime.dispatch_str("tick");
    }
    print_state(&runtime);
    println!("   活跃区域: {:?}", runtime.active_zones());
    println!();
    
    // 断开充电器
    println!("📨 事件: uncharge (断开充电器)");
    runtime.dispatch_str("uncharge");
    print_state(&runtime);
    println!("   活跃区域: {:?}", runtime.active_zones());
    println!();
    
    // 停止设备
    println!("📨 事件: stop (停止设备)");
    runtime.dispatch_str("stop");
    print_state(&runtime);
    println!("   活跃区域: {:?}", runtime.active_zones());
    println!();
    
    println!("✅ 演示完成！");
}

// 辅助函数：打印当前状态
fn print_state(runtime: &StateMachineRuntime) {
    for aspect in runtime.blueprint().aspects() {
        if let Some(value) = runtime.state().get(aspect.id) {
            // Try to format common types
            if let Some(b) = value.downcast_ref::<bool>() {
                println!("     {} = {}", aspect.name, b);
            } else if let Some(i) = value.downcast_ref::<i64>() {
                println!("     {} = {}", aspect.name, i);
            } else if let Some(f) = value.downcast_ref::<f64>() {
                println!("     {} = {}", aspect.name, f);
            } else if let Some(s) = value.downcast_ref::<String>() {
                println!("     {} = {}", aspect.name, s);
            } else {
                println!("     {} = {:?}", aspect.name, value);
            }
        }
    }
}

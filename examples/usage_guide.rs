/// state-zen 使用指南
///
/// 本示例展示如何使用 state-zen 创建和运行状态机

use state_zen::{AspectId, AspectBlueprint, ZoneBlueprint, ZoneId, TransitionBlueprint, TransitionId, StateMachineRuntime, StateMachineBlueprint};
use state_zen::core::EventId;
use state_zen::active_in::ActiveInBlueprint;
use state_zen::update::{Update, UpdateBlueprint};

fn main() {
    println!("=== state-zen 使用指南 ===\n");

    // ============================================
    // 第一步：定义状态面 (StateAspect)
    // ============================================
    println!("1️⃣ 定义状态面");

    // 定义设备的状态面
    let mode = AspectBlueprint::new(
        AspectId(0),
        "mode",
        "idle".to_string()
    );

    let battery = AspectBlueprint::new(
        AspectId(1),
        "battery",
        100i64
    )
    .with_range(0i64, 100i64);

    let is_charging = AspectBlueprint::new(
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
    let low_battery_zone = ZoneBlueprint::new(
        ZoneId(0),
        "low_battery",
        ActiveInBlueprint::aspect_lt(AspectId(1), 20i64)
    );

    // 充电区域
    let charging_zone = ZoneBlueprint::new(
        ZoneId(1),
        "charging",
        ActiveInBlueprint::aspect_bool(AspectId(2), true)
    );

    // 运行区域
    let running_zone = ZoneBlueprint::new(
        ZoneId(2),
        "running",
        ActiveInBlueprint::aspect_string_eq(AspectId(0), "running")
    );

    println!("   ✓ 定义了 3 个区域: low_battery, charging, running\n");

    // ============================================
    // 第三步：定义转移 (Transition)
    // ============================================
    println!("3️⃣ 定义转移");

    // 启动设备
    let start_transition = TransitionBlueprint::new(
        TransitionId(0),
        "start",
        ActiveInBlueprint::aspect_string_eq(AspectId(0), "idle"),
        EventId::new("start"),
        UpdateBlueprint::set_string(AspectId(0), "running")
    );

    // 停止设备
    let stop_transition = TransitionBlueprint::new(
        TransitionId(1),
        "stop",
        ActiveInBlueprint::aspect_string_eq(AspectId(0), "running"),
        EventId::new("stop"),
        UpdateBlueprint::set_string(AspectId(0), "idle")
    );

    // 连接充电器
    let charge_transition = TransitionBlueprint::new(
        TransitionId(2),
        "charge",
        ActiveInBlueprint::always(),
        EventId::new("charge"),
        UpdateBlueprint::set_bool(AspectId(2), true)
    );

    // 断开充电器
    let uncharge_transition = TransitionBlueprint::new(
        TransitionId(3),
        "uncharge",
        ActiveInBlueprint::always(),
        EventId::new("uncharge"),
        UpdateBlueprint::set_bool(AspectId(2), false)
    );

    // 消耗电量
    let consume_transition = TransitionBlueprint::new(
        TransitionId(4),
        "consume",
        ActiveInBlueprint::aspect_string_eq(AspectId(0), "running"),
        EventId::new("tick"),
        UpdateBlueprint::noop() // 将在运行时替换为复杂的条件更新
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

    println!("   ✓ 蓝图构建完成");
    println!();

    // ============================================
    // 第五步：创建运行时实例
    // ============================================
    println!("5️⃣ 创建运行时实例");

    let mut runtime = StateMachineRuntime::new(blueprint)
        // 添加区域副作用处理器
        .with_zone_on_enter(ZoneId(0), || {
            println!("   ⚠️ 警告：电量低于 20%！");
        })
        .with_zone_on_exit(ZoneId(0), || {
            println!("   ✓ 电量恢复正常");
        })
        .with_zone_on_enter(ZoneId(1), || {
            println!("   🔌 开始充电");
        })
        .with_zone_on_exit(ZoneId(1), || {
            println!("   🔌 停止充电");
        })
        .with_zone_on_enter(ZoneId(2), || {
            println!("   ▶️ 设备启动");
        })
        .with_zone_on_exit(ZoneId(2), || {
            println!("   ⏸️ 设备停止");
        })
        // 添加自定义更新操作
        .with_transition_update(TransitionId(4), Update::compose(vec![
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
                    Update::set_string(AspectId(0), "idle"),
                    Update::set_int(AspectId(1), 0),
                ]),
            ),
        ]));

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
            if let Some(b) = value.as_any().downcast_ref::<bool>() {
                println!("     {} = {}", aspect.name, b);
            } else if let Some(i) = value.as_any().downcast_ref::<i64>() {
                println!("     {} = {}", aspect.name, i);
            } else if let Some(f) = value.as_any().downcast_ref::<f64>() {
                println!("     {} = {}", aspect.name, f);
            } else if let Some(s) = value.as_any().downcast_ref::<String>() {
                println!("     {} = {}", aspect.name, s);
            } else {
                println!("     {} = {:?}", aspect.name, value);
            }
        }
    }
}
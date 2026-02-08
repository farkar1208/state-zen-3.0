# state-zen

一个面向组合性与表达力的状态机蓝图系统，采用 Rust 实现。支持高维状态向量、谓词驱动的行为绑定，以及声明式状态演化。

## 核心概念

### StateAspect（状态面）
状态的独立维度，每个 `StateAspect` 有独立的类型和取值范围。

### ActiveIn（激活条件）
谓词函数 `(State → bool)`，定义行为在哪些状态下被激活。替代传统状态机的硬编码源状态。

### Zone（区域）
状态行为容器，包含：
- `activeIn`: 定义覆盖的状态集合
- `on_enter`: 进入区域时的副作用
- `on_exit`: 离开区域时的副作用

### Transition（转移）
事件驱动的状态变更，包含：
- `activeIn`: 何时监听事件
- `event`: 监听的事件类型
- `update`: 如何计算新状态（纯函数）
- `on_tran`: 转移发生时的副作用

### Update（状态更新）
纯函数式的状态变换算子，支持：
- 基本设置（set, increment, decrement, toggle）
- 条件更新（conditional）
- 组合操作（compose）

## 快速开始

```rust
use state_zen::prelude::*;
use state_zen::{BlueprintBuilder, Zone, Transition, EventId};
use state_zen::active_in::ActiveIn;
use state_zen::update::Update;

// 定义状态面
let mode = StateAspect::new(AspectId(0), "mode", StateValue::String("idle".to_string()));
let battery = StateAspect::new(AspectId(1), "battery", StateValue::Integer(100));

// 定义区域
let low_battery_zone = Zone::new(
    "low_battery",
    ActiveIn::aspect_lt(AspectId(1), 20),
)
.with_on_enter(|| println!("⚠️  Low battery!"));

// 定义转移
let start_transition = Transition::new(
    "start",
    ActiveIn::aspect_string_eq(AspectId(0), "idle"),
    EventId::new("start"),
    Update::set_string(AspectId(0), "running"),
)
.with_on_tran(|| println!("Starting..."));

// 构建蓝图
let blueprint = BlueprintBuilder::new()
    .id("device")
    .aspect(mode)
    .aspect(battery)
    .zone(low_battery_zone)
    .transition(start_transition)
    .build()
    .unwrap();

// 创建初始状态
let mut state = blueprint.create_initial_state();
```

## ActiveIn 算子

```rust
// 基本谓词
ActiveIn::aspect_bool(id, true)           // 布尔值等于
ActiveIn::aspect_eq(id, 42)                // 整数等于
ActiveIn::aspect_lt(id, 10)                // 小于
ActiveIn::aspect_gt(id, 0)                 // 大于
ActiveIn::aspect_in_range(id, 0, 100)      // 范围
ActiveIn::aspect_string_eq(id, "active")   // 字符串等于

// 逻辑组合
let predicate = ActiveIn::aspect_bool(id1, true)
    .and(ActiveIn::aspect_lt(id2, 10));
```

## Update 算子

```rust
// 基本操作
Update::set(id, StateValue::Bool(true))   // 设置值
Update::set_bool(id, true)                 // 设置布尔
Update::set_int(id, 42)                    // 设置整数
Update::increment(id)                      // 自增
Update::decrement(id)                      // 自减
Update::add(id, 5)                         // 加法
Update::toggle(id)                         // 切换布尔

// 条件更新
Update::conditional(
    |s| s.get(id).map_or(false, |v| matches!(v, StateValue::Integer(i) if *i < 10)),
    Update::increment(id),
)

// 组合操作
Update::compose(vec![
    Update::toggle(id1),
    Update::increment(id2),
])
```

## 运行示例

```bash
cargo run --example basic_state_machine
```

## 项目结构

```
src/
├── lib.rs          # 库入口，导出公共 API
├── aspect.rs       # StateAspect 和 State 实现
├── active_in.rs    # ActiveIn 谓词算子
├── zone.rs         # Zone 区域定义
├── transition.rs   # Transition 转移定义
├── update.rs       # Update 状态更新算子
└── blueprint.rs    # StateMachineBlueprint 蓝图
```

## 设计哲学

- **分离关注点**: 每个 `StateAspect` 独立
- **声明式行为绑定**: 通过 `activeIn` 谓词
- **纯状态演化**: `update` 无副作用
- **高可组合性**: 支持谓词和更新的组合操作

## 运行测试

```bash
cargo test
```

## 编译检查

```bash
cargo check
```

## 注意事项

当前实现是状态机蓝图层，不包含运行时状态机实例化。蓝图可以通过编译/验证后用于实例化可运行的状态机（待实现）。
# state-zen

一个面向组合性与表达力的状态机蓝图系统，采用 Rust 实现。支持高维状态向量、谓词驱动的行为绑定，以及声明式状态演化。

## 核心概念

### StateAspect（状态面）
状态的独立维度，每个 `StateAspect` 有独立的类型和取值范围。

### ActiveIn（激活条件）
谓词函数 `(State → bool)`，定义行为在哪些状态下被激活。替代传统状态机的硬编码源状态。

### Zone（区域）
状态行为容器，分为蓝图层和运行时层：
- **蓝图层 (`ZoneBlueprint`)**: 定义覆盖的状态集合，不包含副作用处理器
- **运行时层 (`Zone`)**: 包含生命周期副作用（`on_enter`、`on_exit`）

### Transition（转移）
事件驱动的状态变更，分为蓝图层和运行时层：
- **蓝图层 (`TransitionBlueprint`)**: 定义激活条件、事件类型和状态更新，不包含副作用处理器
- **运行时层 (`Transition`)**: 包含转移发生时的副作用（`on_tran`）

### Update（状态更新）
纯函数式的状态变换算子，支持：
- 基本设置（set, increment, decrement, toggle）
- 条件更新（conditional）
- 组合操作（compose）

### StateMachine（状态机）
分为蓝图层和运行时层：
- **蓝图层 (`StateMachineBlueprint`)**: 声明式定义状态机结构，可序列化存储或传输
- **运行时层 (`StateMachineRuntime`)**: 从蓝图创建可执行实例，支持事件分发和状态追踪

## 快速开始

```rust
use state_zen::prelude::*;
use state_zen::{AspectBlueprint, ZoneBlueprint, TransitionBlueprint, StateMachineRuntime, StateMachineBlueprint};
use state_zen::active_in::ActiveInBlueprint;
use state_zen::update::UpdateBlueprint;

// 定义状态面（蓝图）
let mode = AspectBlueprint::new(AspectId(0), "mode", "idle".to_string());
let battery = AspectBlueprint::new(AspectId(1), "battery", 100i64);

// 定义区域蓝图（声明式，不包含副作用）
let low_battery_zone = ZoneBlueprint::new(
    ZoneId(0),
    "low_battery",
    ActiveInBlueprint::aspect_lt(AspectId(1), 20i64),
);

// 定义转移蓝图（声明式，不包含副作用）
let start_transition = TransitionBlueprint::new(
    TransitionId(0),
    "start",
    ActiveInBlueprint::aspect_string_eq(AspectId(0), "idle"),
    EventId::new("start"),
    UpdateBlueprint::set_string(AspectId(0), "running"),
);

// 构建状态机蓝图
let mut blueprint = StateMachineBlueprint::new("device");
blueprint.add_aspect(mode);
blueprint.add_aspect(battery);
blueprint.add_zone(low_battery_zone);
blueprint.add_transition(start_transition);

// 创建运行时实例并添加副作用处理器
let mut runtime = StateMachineRuntime::new(blueprint)
    .with_zone_on_enter(ZoneId(0), || println!("⚠️  Low battery!"))
    .with_transition_on_tran(TransitionId(0), || println!("Starting..."));

// 分发事件
runtime.dispatch_str("start");
```

## ActiveIn 算子

### 蓝图层（声明式）
```rust
use state_zen::active_in::ActiveInBlueprint;

// 基本谓词
ActiveInBlueprint::aspect_bool(id, true)           // 布尔值等于
ActiveInBlueprint::aspect_eq(id, 42)                // 整数等于
ActiveInBlueprint::aspect_lt(id, 10)                // 小于
ActiveInBlueprint::aspect_gt(id, 0)                 // 大于
ActiveInBlueprint::aspect_in_range(id, 0, 100)      // 范围
ActiveInBlueprint::aspect_string_eq(id, "active")   // 字符串等于

// 逻辑组合
let predicate = ActiveInBlueprint::aspect_bool(id1, true)
    .and(ActiveInBlueprint::aspect_lt(id2, 10));
```

### 运行时层（可执行）
```rust
use state_zen::active_in::ActiveInFactory;

// 使用工厂方法创建运行时 ActiveIn
let predicate = ActiveInFactory::aspect_bool(id1, true)
    .and(ActiveInFactory::aspect_lt(id2, 10));
```

## Update 算子

### 蓝图层（声明式）
```rust
use state_zen::update::UpdateBlueprint;

// 基本操作
UpdateBlueprint::set_bool(id, true)   // 设置布尔
UpdateBlueprint::set_int(id, 42)      // 设置整数
UpdateBlueprint::increment(id)        // 自增
UpdateBlueprint::decrement(id)        // 自减
```

### 运行时层（可执行）
```rust
use state_zen::update::Update;

// 基本操作
Update::set_bool(id, true)   // 设置布尔
Update::set_int(id, 42)      // 设置整数
Update::increment(id)        // 自增
Update::decrement(id)        // 自减

// 条件更新（仅运行时支持）
Update::conditional(|state| state.get_as::<bool>(id).map_or(false, |&v| v), Update::noop())
```

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
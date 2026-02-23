# state-zen

一个面向组合性与表达力的状态机系统，采用 Rust 实现。支持高维状态向量、谓词驱动的行为绑定，以及声明式状态演化。
项目尚处于开发阶段，测试覆盖尚不全面，且API可能变动，请勿用于生产环境中。

## 核心概念

### State（状态向量）
运行时状态容器，表示系统的完整状态作为高维状态向量。使用 `HashMap<AspectId, Box<dyn ClonableAny>>` 存储类型擦除的值，支持类型安全的查询和更新。

### AspectBlueprint（状态面蓝图）
状态的独立维度蓝图定义。每个 `AspectBlueprint` 有独立的类型、取值范围和约束条件。蓝图层不包含运行时行为，可序列化存储或传输。

### ActiveIn（激活条件）
谓词函数 `(State → bool)`，定义行为在哪些状态下被激活。替代传统状态机的硬编码源状态。分为蓝图层（`ActiveInBlueprint`）和运行时层（`ActiveIn`）。

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
- 类型化修改（modify_typed）
- 条件更新（conditional）
- 组合操作（compose）

### StateMachine（状态机）
分为蓝图层和运行时层：
- **蓝图层 (`StateMachineBlueprint`)**: 声明式定义状态机结构，包含状态面、区域蓝图和转移蓝图，可序列化存储或传输
- **运行时层 (`StateMachineRuntime`)**: 从蓝图创建可执行实例，支持事件分发、状态追踪和区域激活管理

## 快速开始

```rust
use state_zen::prelude::*;

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

// 验证蓝图
blueprint.validate().expect("Invalid blueprint");

// 创建运行时实例并添加副作用处理器
let mut runtime = StateMachineRuntime::new(blueprint)
    .with_zone_on_enter(ZoneId(0), || println!("⚠️  Low battery!"))
    .with_transition_on_tran(TransitionId(0), || println!("Starting..."));

// 分发事件
runtime.dispatch_str("start");

// 查询状态
let current_state = runtime.state();
let active_zones = runtime.active_zones();
```

## ActiveIn 算子

### 蓝图层（声明式 - ActiveInBlueprint）
```rust
use state_zen::active_in::ActiveInBlueprint;

// 基本谓词
ActiveInBlueprint::always()                         // 总是为真
ActiveInBlueprint::never()                          // 总是为假
ActiveInBlueprint::aspect_bool(id, true)            // 布尔值等于
ActiveInBlueprint::aspect_eq(id, 42)                // 整数等于
ActiveInBlueprint::aspect_lt(id, 10)                // 小于
ActiveInBlueprint::aspect_gt(id, 0)                 // 大于
ActiveInBlueprint::aspect_in_range(id, 0, 100)      // 范围内 [min, max]
ActiveInBlueprint::aspect_string_eq(id, "active")   // 字符串等于

// 逻辑组合
let predicate = ActiveInBlueprint::aspect_bool(id1, true)
    .and(ActiveInBlueprint::aspect_lt(id2, 10));
let predicate = ActiveInBlueprint::aspect_lt(id, 10)
    .or(ActiveInBlueprint::aspect_gt(id, 20));
let predicate = ActiveInBlueprint::always().not();
```

### 运行时层（可执行 - ActiveIn）
```rust
use state_zen::active_in::ActiveInFactory;

// 使用工厂方法创建运行时 ActiveIn
let predicate = ActiveInFactory::aspect_bool(id1, true)
    .and(ActiveInFactory::aspect_lt(id2, 10));

// 泛型类型比较
let predicate = ActiveInFactory::aspect_lt_typed(id, 5.0f64);
let predicate = ActiveInFactory::aspect_eq_typed(id, "value".to_string());
```

## Update 算子

### 蓝图层（声明式 - UpdateBlueprint）
```rust
use state_zen::update::UpdateBlueprint;

// 基本操作
UpdateBlueprint::noop()                             // 无操作
UpdateBlueprint::set_bool(id, true)                 // 设置布尔
UpdateBlueprint::set_int(id, 42)                    // 设置整数
UpdateBlueprint::set_float(id, 3.14)                // 设置浮点数
UpdateBlueprint::set_string(id, "value")            // 设置字符串
UpdateBlueprint::increment(id)                      // 自增
UpdateBlueprint::decrement(id)                      // 自减
UpdateBlueprint::add(id, 5)                         // 加法
UpdateBlueprint::toggle(id)                         // 切换布尔

// 组合操作
UpdateBlueprint::compose(vec![
    UpdateBlueprint::toggle(id1),
    UpdateBlueprint::increment(id2),
])

// 条件更新
UpdateBlueprint::conditional(
    ActiveInBlueprint::aspect_lt(id, 10),
    UpdateBlueprint::increment(id),
)
```

### 运行时层（可执行 - Update）
```rust
use state_zen::update::Update;

// 基本操作
Update::set_bool(id, true)                          // 设置布尔
Update::set_int(id, 42)                             // 设置整数
Update::set_float(id, 3.14)                         // 设置浮点数
Update::set_string(id, "value")                     // 设置字符串
Update::increment(id)                               // 自增
Update::decrement(id)                               // 自减
Update::add(id, 5)                                  // 加法
Update::toggle(id)                                  // 切换布尔

// 类型化修改
Update::modify_typed(id, |v: i64| v * 2)            // 类型安全的修改

// 条件更新（运行时支持动态谓词）
Update::conditional(
    |state| state.get_as::<i64>(id).map_or(false, |&v| v < 10),
    Update::increment(id),
)

// 条件更新带 else 分支
Update::conditional_else(
    |state| state.get_as::<bool>(id).map_or(false, |&v| v),
    Update::set_int(id, 100),
    Update::set_int(id, 0),
)

// 组合操作
Update::compose(vec![
    Update::toggle(id1),
    Update::increment(id2),
])
```

## 运行示例

```bash
# 基础状态机示例
cargo run --example basic_state_machine

# 完整使用指南示例
cargo run --example usage_guide
```

## 项目结构

```
src/
├── lib.rs              # 库入口，导出公共 API 和 prelude
├── core.rs             # 核心类型定义（ClonableAny、AspectId、EventId）
├── aspect.rs           # AspectBlueprint 和 State 实现
├── state.rs            # State 和 StateBuilder 实现
├── active_in.rs        # ActiveIn 和 ActiveInBlueprint 谓词算子
├── zone.rs             # Zone 和 ZoneBlueprint 区域定义
├── transition.rs       # Transition 和 TransitionBlueprint 转移定义
├── update.rs           # Update 和 UpdateBlueprint 状态更新算子
└── statemachine.rs     # StateMachineBlueprint 和 StateMachineRuntime 状态机实现
```

## API 文档

详细的 API 文档请参考 `/doc` 目录：
- `core.md` - 核心类型（ClonableAny、AspectId、EventId）
- `aspect.md` - AspectBlueprint 和 State
- `state.md` - State 和 StateBuilder
- `active_in.md` - ActiveIn 谓词算子
- `zone.md` - Zone 区域定义
- `transition.md` - Transition 转移定义
- `update.md` - Update 状态更新算子
- `statemachine.md` - StateMachine 蓝图和运行时
- `lib.md` - 库入口和 prelude

## 设计哲学

- **蓝图-运行时分离**: 蓝图层提供声明式定义和可序列化能力，运行时层提供可执行实例
- **分离关注点**: 每个 `AspectBlueprint` 独立，支持正交的状态面设计
- **声明式行为绑定**: 通过 `activeIn` 谓词实现灵活的行为激活
- **纯状态演化**: `update` 算子无副作用，副作用通过处理器分离
- **高可组合性**: 支持谓词和更新的组合操作，避免状态爆炸
- **类型安全**: 运行时类型检查确保类型一致性

## 运行测试

```bash
cargo test
```

## 编译检查

```bash
cargo check
```

## 清理构建产物

```bash
cargo clean
```

## 贡献

别贡献，项目没完工，懒得维护。

## 许可证

请查看 [LICENSE](LICENSE) 文件了解详细信息。
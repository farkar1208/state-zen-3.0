# state-zen 项目指南

## 项目概述

**state-zen** 是一个使用 Rust 实现的面向组合性与表达力的状态机系统。该项目采用创新的**高维状态向量**模型，通过谓词驱动的行为绑定和声明式状态演化，为复杂系统提供灵活的状态管理解决方案。

### 核心设计理念

- **高维状态建模**：状态不是单一标量，而是由多个正交的 `StateAspect` 组成的向量
- **谓词驱动激活**：通过 `activeIn` 谓词函数替代传统状态机的硬编码源状态
- **纯状态演化**：状态更新通过纯函数实现，副作用与状态变更分离
- **高可组合性**：支持谓词和更新操作的组合，避免状态爆炸
- **蓝图-运行时分离**：蓝图层定义状态机结构，运行时层提供实例化和事件分发

### 核心概念

- **StateAspect（状态面）**：状态的独立维度，每个有独立的类型和取值范围
- **ActiveIn（激活条件）**：谓词函数 `(State → bool)`，定义行为在哪些状态下被激活
- **Zone（区域）**：状态行为容器，包含激活条件、进入/离开时的副作用
- **Transition（转移）**：事件驱动的状态变更，包含激活条件、事件监听、状态更新和副作用
- **Update（状态更新）**：纯函数式的状态变换算子，支持基本操作、条件更新和组合
- **StateMachineRuntime（运行时）**：状态机的可执行实例，支持事件分发和状态追踪

### 技术栈

- **语言**：Rust 2021 Edition
- **构建工具**：Cargo
- **架构**：模块化设计，核心功能分为 7 个独立模块
- **DSL 支持**：SZD (StateZen Define DSL) 用于声明式状态机定义

---

## 项目结构

```
state-zen 3.0/
├── src/
│   ├── lib.rs              # 库入口，导出公共 API 和 prelude
│   ├── aspect.rs           # StateAspect 和 State 实现
│   ├── active_in.rs        # ActiveIn 谓词算子
│   ├── zone.rs             # Zone 区域定义
│   ├── transition.rs       # Transition 转移定义
│   ├── update.rs           # Update 状态更新算子
│   ├── blueprint.rs        # StateMachineBlueprint 蓝图
│   └── runtime.rs          # StateMachineRuntime 运行时实例
├── examples/
│   ├── basic_state_machine.rs  # 基础状态机示例
│   └── usage_guide.rs          # 完整使用指南示例
├── Cargo.toml              # 项目配置和依赖
├── README.md               # 项目文档
├── define.md               # 详细的术语和模型规范
└── grammar.md              # SZD DSL 语法规范
```

### 模块说明

| 模块 | 职责 | 主要类型 |
|------|------|----------|
| `aspect.rs` | 状态面定义和状态向量管理 | `StateAspect`, `State`, `StateValue`, `StateBuilder` |
| `active_in.rs` | 谓词函数和激活条件 | `ActiveIn`, `ActiveInBlueprint`, `Predicate` |
| `zone.rs` | 状态区域定义和生命周期管理 | `Zone`, `ZoneBlueprint`, `ZoneId` |
| `transition.rs` | 状态转移和事件处理 | `Transition`, `TransitionBlueprint`, `TransitionId`, `EventId` |
| `update.rs` | 状态更新操作 | `Update`, `UpdateBlueprint` |
| `blueprint.rs` | 状态机蓝图构建和验证 | `StateMachineBlueprint`, `BlueprintBuilder`, `AspectDescriptor` |
| `runtime.rs` | 状态机运行时实例和事件分发 | `StateMachineRuntime` |

---

## 构建和运行

### 构建项目

```bash
cargo build
```

### 运行测试

```bash
cargo test
```

### 编译检查（不生成二进制）

```bash
cargo check
```

### 运行示例

```bash
# 基础状态机示例
cargo run --example basic_state_machine

# 完整使用指南示例
cargo run --example usage_guide
```

### 清理构建产物

```bash
cargo clean
```

---

## 开发约定

### 代码风格

- **命名规范**：
  - 结构体和枚举：大驼峰命名（PascalCase）
  - 函数和方法：蛇形命名（snake_case）
  - 常量：全大写下划线分隔（SCREAMING_SNAKE_CASE）
- **模块组织**：每个核心概念独立一个模块，通过 `lib.rs` 统一导出
- **API 设计**：提供 `prelude` 模块以便于常用类型的导入

### 测试实践

- 使用 Rust 内置的 `#[cfg(test)]` 模块进行单元测试
- 测试与实现代码放在同一文件中
- 确保所有核心功能都有对应的测试覆盖

### 扩展指南

#### 添加新的 StateAspect

```rust
let new_aspect = AspectBlueprint::new(
    AspectId(3),  // 确保 ID 唯一
    "aspect_name",
    "default_value".to_string()
).with_range(min_value, max_value);  // 可选：设置范围约束
```

#### 创建新的 ActiveIn 谓词

在 `active_in.rs` 中添加新的谓词构造方法，或使用组合操作：

```rust
let predicate = ActiveIn::aspect_bool(id1, true)
    .and(ActiveIn::aspect_lt(id2, 10))
    .or(ActiveIn::aspect_string_eq(id3, "special"));
```

#### 定义新的 Update 操作

在 `update.rs` 中添加新的更新算子，或使用组合：

```rust
Update::compose(vec![
    Update::increment(id1),
    Update::set_bool(id2, true),
    Update::modify_typed(id3, |v: i64| v * 2),  // 类型化修改
])
```

#### 构建状态机蓝图

使用 `StateMachineBlueprint` 构建方法：

```rust
let mut blueprint = StateMachineBlueprint::new("machine_name");
blueprint.add_aspect(aspect1);
blueprint.add_aspect(aspect2);
blueprint.add_zone(zone1);
blueprint.add_transition(transition1);
```

#### 创建和使用运行时实例

```rust
// 从蓝图创建运行时实例
let mut runtime = StateMachineRuntime::new(blueprint);

// 分发事件
runtime.dispatch(&EventId::new("start"));
runtime.dispatch_str("start");  // 使用字符串事件名

// 查询状态
let current_state = runtime.state();
let active_zones = runtime.active_zones();

// 重置状态机
runtime.reset();
```

### SZD DSL 使用

项目支持 SZD (StateZen Define DSL) 语法进行声明式状态机定义。详细语法规范请参考 `grammar.md`。

#### DSL 结构示例

```
Aspect
  mode enum {idle running stopped} idle
  battery 0 <= i64 <= 100 100
  is_charging bool false

Zone
  low_battery battery < 20
  charging is_charging == true
  running mode == running

Transition
  start start
    ActiveIn mode == idle
    Update mode := running
  stop stop
    ActiveIn mode == running
    Update mode := stopped
```

---

## 常用操作速查

### ActiveIn 谓词算子

```rust
ActiveIn::aspect_bool(id, true)           // 布尔值等于
ActiveIn::aspect_eq(id, 42)                // 整数等于
ActiveIn::aspect_lt(id, 10)                // 小于
ActiveIn::aspect_gt(id, 0)                 // 大于
ActiveIn::aspect_le(id, 10)                // 小于等于
ActiveIn::aspect_ge(id, 0)                 // 大于等于
ActiveIn::aspect_in_range(id, 0, 100)      // 范围
ActiveIn::aspect_string_eq(id, "active")   // 字符串等于
ActiveIn::always()                         // 总是激活
ActiveIn::never()                          // 从不激活

// 组合操作
predicate.and(other)                       // 逻辑与
predicate.or(other)                        // 逻辑或
predicate.not()                            // 逻辑非
```

### Update 更新算子

```rust
Update::set(id, StateValue::Bool(true))   // 设置值
Update::set_bool(id, true)                 // 设置布尔
Update::set_int(id, 42)                    // 设置整数
Update::set_string(id, "value")            // 设置字符串
Update::increment(id)                      // 自增
Update::decrement(id)                      // 自减
Update::add(id, 5)                         // 加法
Update::subtract(id, 3)                    // 减法
Update::toggle(id)                         // 切换布尔
Update::modify_typed(id, |v: i64| v * 2)   // 类型化修改
Update::compose(vec![...])                 // 组合操作
Update::conditional(predicate, update)     // 条件更新
Update::conditional_else(predicate, true_update, false_update)  // 条件分支
```

### Zone 区域操作

```rust
Zone::new(id, "name", predicate)           // 创建区域
  .with_on_enter(|| { /* 副作用 */ })      // 进入时触发
  .with_on_exit(|| { /* 副作用 */ })       // 离开时触发
```

### Transition 转移操作

```rust
Transition::new(
    id,                    // TransitionId
    "name",                // 转移名称
    active_in,             // ActiveIn 激活条件
    event_id,              // EventId 事件ID
    update,                // Update 状态更新
)
  .with_on_trigger(|| { /* 触发副作用 */ })
```

### Runtime 运行时操作

```rust
// 创建运行时
let mut runtime = StateMachineRuntime::new(blueprint);

// 事件分发
runtime.dispatch(&EventId::new("event"));
runtime.dispatch_str("event");

// 状态查询
runtime.state()                    // 获取当前状态
runtime.blueprint()                // 获取蓝图引用
runtime.active_zones()             // 获取活跃区域列表
runtime.is_zone_active(zone_id)    // 检查区域是否活跃

// 状态管理
runtime.reset()                    // 重置到初始状态
```

---

## 核心特性

### 蓝图层 (Blueprint Layer)

- **声明式定义**：使用 `StateMachineBlueprint` 声明式地定义状态机结构
- **类型安全**：编译时类型检查，确保状态值在约束范围内
- **可序列化**：蓝图可以序列化存储或传输

### 运行时层 (Runtime Layer)

- **事件驱动**：通过事件分发触发状态转移
- **自动区域管理**：自动追踪和更新区域激活状态
- **状态查询**：提供当前状态和活跃区域的查询接口
- **可重置性**：支持重置到初始状态

### 组合性

- **谓词组合**：支持 AND、OR、NOT 等逻辑组合
- **更新组合**：支持多个更新操作的顺序组合
- **条件更新**：支持基于当前状态的条件分支更新

---

## 参考资源

- **详细文档**：`README.md` - 包含完整的 API 文档和示例
- **模型规范**：`define.md` - 详细的术语词典和模型规范文档
- **语法规范**：`grammar.md` - SZD DSL 语法规则和示例
- **基础示例**：`examples/basic_state_machine.rs` - 基础状态机示例
- **使用指南**：`examples/usage_guide.rs` - 完整的使用指南和最佳实践

---

## 最佳实践

### 状态面设计

1. **正交性**：每个状态面应该代表一个独立的维度
2. **类型选择**：根据需求选择合适的类型（bool、enum、数值类型）
3. **范围约束**：为数值类型设置合理的范围约束

### 谓词设计

1. **简单优先**：优先使用简单的单条件谓词
2. **组合使用**：复杂条件通过组合简单谓词实现
3. **避免过度复杂**：过复杂的谓词会降低可读性和维护性

### 状态更新

1. **纯函数优先**：尽量使用无副作用的纯函数更新
2. **原子性**：相关更新应该放在一个 `compose` 中确保原子性
3. **类型安全**：使用 `modify_typed` 确保类型安全

### 区域使用

1. **生命周期感知**：合理使用进入/离开副作用
2. **避免重叠**：设计清晰的区域边界
3. **监控友好**：区域激活状态可用于监控和调试
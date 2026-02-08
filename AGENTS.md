# state-zen 项目指南

## 项目概述

**state-zen** 是一个使用 Rust 实现的面向组合性与表达力的状态机蓝图系统。该项目采用创新的**高维状态向量**模型，通过谓词驱动的行为绑定和声明式状态演化，为复杂系统提供灵活的状态管理解决方案。

### 核心设计理念

- **高维状态建模**：状态不是单一标量，而是由多个正交的 `StateAspect` 组成的向量
- **谓词驱动激活**：通过 `activeIn` 谓词函数替代传统状态机的硬编码源状态
- **纯状态演化**：状态更新通过纯函数实现，副作用与状态变更分离
- **高可组合性**：支持谓词和更新操作的组合，避免状态爆炸

### 核心概念

- **StateAspect（状态面）**：状态的独立维度，每个有独立的类型和取值范围
- **ActiveIn（激活条件）**：谓词函数 `(State → bool)`，定义行为在哪些状态下被激活
- **Zone（区域）**：状态行为容器，包含激活条件、进入/离开时的副作用
- **Transition（转移）**：事件驱动的状态变更，包含激活条件、事件监听、状态更新和副作用
- **Update（状态更新）**：纯函数式的状态变换算子，支持基本操作、条件更新和组合

### 技术栈

- **语言**：Rust 2021 Edition
- **构建工具**：Cargo
- **架构**：模块化设计，核心功能分为 6 个独立模块

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
│   └── blueprint.rs        # StateMachineBlueprint 蓝图
├── examples/
│   └── basic_state_machine.rs  # 基础状态机示例
├── Cargo.toml              # 项目配置和依赖
├── README.md               # 项目文档
└── define.md               # 详细的术语和模型规范
```

### 模块说明

| 模块 | 职责 | 主要类型 |
|------|------|----------|
| `aspect.rs` | 状态面定义和状态向量管理 | `StateAspect`, `State`, `StateValue` |
| `active_in.rs` | 谓词函数和激活条件 | `ActiveIn`, `Predicate` |
| `zone.rs` | 状态区域定义和生命周期管理 | `Zone` |
| `transition.rs` | 状态转移和事件处理 | `Transition`, `EventId` |
| `update.rs` | 状态更新操作 | `Update` |
| `blueprint.rs` | 状态机蓝图构建和验证 | `StateMachineBlueprint`, `BlueprintBuilder` |

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
cargo run --example basic_state_machine
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
let new_aspect = StateAspect::new(
    AspectId(3),  // 确保 ID 唯一
    "aspect_name",
    StateValue::Integer(0),  // 初始值
);
```

#### 创建新的 ActiveIn 谓词

在 `active_in.rs` 中添加新的谓词构造方法，或使用组合操作：

```rust
let predicate = ActiveIn::aspect_bool(id1, true)
    .and(ActiveIn::aspect_lt(id2, 10));
```

#### 定义新的 Update 操作

在 `update.rs` 中添加新的更新算子，或使用组合：

```rust
Update::compose(vec![
    Update::increment(id1),
    Update::set_bool(id2, true),
])
```

#### 构建状态机蓝图

使用 `BlueprintBuilder` 构建器模式：

```rust
let blueprint = BlueprintBuilder::new()
    .id("machine_name")
    .aspect(aspect1)
    .aspect(aspect2)
    .zone(zone1)
    .transition(transition1)
    .build()?;
```

### 当前限制

- 当前实现是状态机蓝图层，不包含运行时状态机实例化
- 蓝图可以通过编译/验证后用于实例化可运行的状态机（待实现）
- 无外部依赖，所有功能为纯 Rust 实现

---

## 常用操作速查

### ActiveIn 谓词算子

```rust
ActiveIn::aspect_bool(id, true)           // 布尔值等于
ActiveIn::aspect_eq(id, 42)                // 整数等于
ActiveIn::aspect_lt(id, 10)                // 小于
ActiveIn::aspect_gt(id, 0)                 // 大于
ActiveIn::aspect_in_range(id, 0, 100)      // 范围
ActiveIn::aspect_string_eq(id, "active")   // 字符串等于
ActiveIn::always()                         // 总是激活
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
Update::toggle(id)                         // 切换布尔
Update::compose(vec![...])                 // 组合操作
Update::conditional(predicate, update)     // 条件更新
```

---

## 参考资源

- **详细文档**：`README.md` - 包含完整的 API 文档和示例
- **模型规范**：`define.md` - 详细的术语词典和模型规范文档
- **示例代码**：`examples/basic_state_machine.rs` - 完整的状态机示例
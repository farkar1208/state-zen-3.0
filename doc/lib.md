# lib.rs API 文档

## 功能介绍
`lib.rs` 是 state-zen 库的入口文件，负责导出所有公共 API 和提供便捷导入的 prelude 模块。

## 功能实现思路
通过 `pub use` 语句重新导出各子模块的核心类型，并提供 prelude 模块集中导出常用类型，方便用户批量导入。注意：为了保持向后兼容性，`StateMachineBlueprint`、`AspectDescriptor` 和 `StateMachineRuntime` 从 `statemachine` 模块重新导出，而不是从 `blueprint` 和 `runtime` 模块。

---

## 模块导出

### `pub mod core`
核心类型定义模块（ClonableAny、AspectId、EventId）

### `pub mod aspect`
状态面和状态向量管理模块

### `pub mod active_in`
谓词函数和激活条件模块

### `pub mod zone`
状态区域定义和生命周期管理模块

### `pub mod transition`
状态转移和事件处理模块

### `pub mod update`
状态更新操作模块

### `pub mod statemachine`
状态机模块，包含蓝图层和运行时层

### `pub mod prelude`
便捷导入模块，集中导出所有常用类型

---

## 公开类型导出

从各模块重新导出的核心类型：

- **ClonableAny**, **AspectId**, **EventId** (从 core 模块)
- **State**, **StateBuilder**, **AspectBlueprint**, **AspectBoundsBlueprint** (从 aspect 模块)
- **clone_any**, **eq_any**, **any_value** (从 aspect 模块)
- **ActiveIn**, **ActiveInBlueprint**, **ActiveInFactory**, **Predicate** (从 active_in 模块)
- **Zone**, **ZoneBlueprint**, **ZoneId** (从 zone 模块)
- **Transition**, **TransitionBlueprint**, **TransitionId**, **EventId** (从 transition 模块)
- **Update**, **UpdateBlueprint** (从 update 模块)
- **StateMachineBlueprint**, **StateMachineRuntime**, **ValidationError** (从 statemachine 模块)
  - `StateMachineRuntime` 提供以下 builder 方法用于在运行时添加副作用：
    - `with_zone_on_enter` - 添加区域进入副作用处理器
    - `with_zone_on_exit` - 添加区域离开副作用处理器
    - `with_transition_on_tran` - 添加转移触发副作用处理器
    - `with_transition_update` - 替换转移的更新操作

---

## Review 意见

1. **向后兼容性**：`StateMachineBlueprint` 和 `StateMachineRuntime` 从 `statemachine` 模块重新导出。建议在 README 中说明这一设计。

2. **模块重构**：`blueprint` 和 `runtime` 模块已合并为 `statemachine` 模块。建议用户直接从 `statemachine` 模块导入。

3. **验证支持**：`ValidationError` 已添加到公共 API，用户可以在创建运行时实例前验证蓝图。

3. **文档注释**：建议为模块添加更详细的 Rust doc 注释（`///`），说明每个模块的用途和主要类型。

4. **prelude 模块**：prelude 模块集中导出了所有常用类型，这是很好的设计。建议在 README 中强调使用 prelude 的好处。
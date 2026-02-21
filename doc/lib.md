# lib.rs API 文档

## 功能介绍
`lib.rs` 是 state-zen 库的入口文件，负责导出所有公共 API 和提供便捷导入的 prelude 模块。

## 功能实现思路
通过 `pub use` 语句重新导出各子模块的核心类型，并提供 prelude 模块集中导出常用类型，方便用户批量导入。

---

## 模块导出

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

### `pub mod blueprint`
状态机蓝图构建和验证模块

### `pub mod runtime`
状态机运行时实例和事件分发模块

### `pub mod prelude`
便捷导入模块，集中导出所有常用类型

---

## 公开类型导出

从各模块重新导出的核心类型：

- **AspectId**, **State**, **StateBuilder**, **AspectBlueprint**, **AspectBoundsBlueprint**
- **clone_any**, **eq_any**, **any_value**
- **ActiveIn**, **ActiveInBlueprint**, **ActiveInFactory**, **Predicate**
- **Zone**, **ZoneBlueprint**, **ZoneId**
- **Transition**, **TransitionBlueprint**, **TransitionId**, **EventId**
- **Update**, **UpdateBlueprint**
- **StateMachineBlueprint**, **AspectDescriptor**
- **StateMachineRuntime**

---

## Review 意见
暂无
# transition.rs API 文档

## 功能介绍
`transition.rs` 模块定义了状态转移和事件处理，Transition 描述系统如何响应事件并演化状态。转移只在 `active_in` 条件为真时监听事件。

## 功能实现思路
- **蓝图层**：`TransitionBlueprint` 定义转移结构（声明式，不包含副作用）
- **运行时层**：`Transition` 包含事件触发时的副作用处理器（`on_tran`）
- **激活机制**：通过 `is_active` 方法判断转移是否应该监听事件
- **状态更新**：通过 `apply` 方法应用状态更新
- **构建器模式**：提供 `TransitionBuilder` 支持链式构建
- **相等性**：仅基于 `TransitionId` 判断相等性

---

## Structs

### TransitionId
转移的唯一标识符

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransitionId(pub usize);
```

**字段：**
- `pub usize` - 内部 usize 值

---

### EventId
事件类型标识符

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventId(pub String);
```

**字段：**
- `pub String` - 内部字符串值

**方法：**
- `pub fn new(id: impl Into<String>) -> Self` - 创建事件 ID

---

### TransitionBlueprint
转移蓝图（声明层，不包含副作用处理器）

```rust
#[derive(Debug, Clone)]
pub struct TransitionBlueprint {
    pub id: TransitionId,
    pub name: String,
    pub active_in: ActiveInBlueprint,
    pub event: EventId,
    pub update: UpdateBlueprint,
}
```

**字段：**
- `pub id: TransitionId` - 转移标识符
- `pub name: String` - 转移名称
- `pub active_in: ActiveInBlueprint` - 定义转移何时应监听事件
- `pub event: EventId` - 要监听的事件类型
- `pub update: UpdateBlueprint` - 如何计算新状态（纯函数）

**方法：**
- `pub fn new(id: TransitionId, name: impl Into<String>, active_in: ActiveInBlueprint, event: EventId, update: UpdateBlueprint) -> Self` - 创建新的转移蓝图

---

### Transition
运行时转移，描述事件驱动的状态变化

```rust
pub struct Transition {
    pub id: TransitionId,
    pub name: String,
    pub active_in: ActiveIn,
    pub event: EventId,
    pub update: Update,
    pub on_tran: Option<TransitionHandler>,
}
```

**字段：**
- `pub id: TransitionId` - 转移标识符
- `pub name: String` - 转移名称
- `pub active_in: ActiveIn` - 定义转移何时应监听事件
- `pub event: EventId` - 要监听的事件类型
- `pub update: Update` - 如何计算新状态（纯函数）
- `pub on_tran: Option<TransitionHandler>` - 转移发生时触发的副作用

**方法：**
- `pub fn new(id: TransitionId, name: impl Into<String>, active_in: ActiveIn, event: EventId, update: Update) -> Self` - 创建新的转移
- `pub fn from_blueprint(blueprint: TransitionBlueprint) -> Self` - 从蓝图创建
- `pub fn with_on_tran<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static` - 设置触发处理器
- `pub fn is_active(&self, state: &State) -> bool` - 检查转移是否应监听事件
- `pub fn apply(&self, state: State) -> State` - 应用状态更新
- `pub fn trigger(&self)` - 执行触发处理器

**Trait 实现：**
- `Debug` - 显示转移基本信息（不显示处理器内容）
- `PartialEq` - 仅基于 `id` 判断相等
- `Eq` - 完全相等语义

---

### TransitionBuilder
转移构建器

```rust
pub struct TransitionBuilder {
    id: Option<TransitionId>,
    name: Option<String>,
    active_in: Option<ActiveIn>,
    event: Option<EventId>,
    update: Option<Update>,
    on_tran: Option<TransitionHandler>,
}
```

**字段：**
- `id: Option<TransitionId>` - 转移标识符
- `name: Option<String>` - 转移名称
- `active_in: Option<ActiveIn>` - 激活条件
- `event: Option<EventId>` - 事件类型
- `update: Option<Update>` - 状态更新
- `on_tran: Option<TransitionHandler>` - 触发处理器

**方法：**
- `pub fn new() -> Self` - 创建构建器
- `pub fn id(mut self, id: TransitionId) -> Self` - 设置 ID
- `pub fn name(mut self, name: impl Into<String>) -> Self` - 设置名称
- `pub fn active_in(mut self, active_in: ActiveIn) -> Self` - 设置激活条件
- `pub fn event(mut self, event: EventId) -> Self` - 设置事件
- `pub fn event_str(mut self, event: impl Into<String>) -> Self` - 设置事件（字符串便捷方法）
- `pub fn update(mut self, update: Update) -> Self` - 设置状态更新
- `pub fn on_tran<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static` - 设置触发处理器
- `pub fn build(self) -> Result<Transition, String>` - 构建转移，返回 `Result`

**Trait 实现：**
- `Default` - 默认为空构建器

---

## Type Aliases

### TransitionHandler
转移副作用处理器类型

```rust
pub type TransitionHandler = Box<dyn Fn() + Send + Sync>;
```

---

## Review 意见

1. **Debug 实现**：`Transition` 的 `Debug` 实现只显示处理器是否存在，不显示 `active_in` 和 `update` 的内容。虽然这些可能较复杂，但可以考虑添加更详细的调试信息。

2. **相等性语义**：`PartialEq` 仅基于 `id` 判断相等，忽略 `name`、`active_in`、`event`、`update`、`on_tran`。这可能导致混淆，建议在文档中明确说明此行为。

3. **错误处理**：`TransitionBuilder::build` 返回 `Result<Transition, String>`，但错误信息是硬编码字符串。建议考虑使用自定义错误类型或 `thiserror` 等库。

4. **处理器无参数**：`TransitionHandler` 不接受任何参数，无法访问事件参数、旧状态或新状态。如果需要访问这些信息，可能需要重新设计。

5. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

6. **Clone 缺失**：`Transition` 没有实现 `Clone`，可能因为 `TransitionHandler` 无法克隆。如果需要克隆 Transition，可以考虑使用 `Arc` 包装处理器。

7. **EventId 设计**：`EventId` 内部使用 `String`，每次创建都会分配内存。如果事件类型数量有限且已知，可以考虑使用枚举或静态字符串来优化性能。
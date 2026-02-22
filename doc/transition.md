# transition.rs API 文档

## 功能介绍
`transition.rs` 模块定义了状态转移（`Transition`）和转移蓝图（`TransitionBlueprint`），实现事件驱动的状态变更机制，支持激活条件、状态更新和副作用处理。

## 功能实现思路
- **蓝图层**：`TransitionBlueprint` 定义声明式的转移结构，不包含副作用处理器
- **运行时层**：`Transition` 封装可执行的转移逻辑，包括激活条件评估、状态更新和副作用触发
- **事件驱动**：转移只在激活条件满足时监听事件
- **纯状态演化**：状态更新通过 `Update` 纯函数实现
- **副作用分离**：副作用通过 `on_tran` 处理器与状态更新分离

---

## Structs

### TransitionBlueprint
转移蓝图（声明层，无副作用处理器）

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
- `pub id: TransitionId` - 转移的唯一标识符
- `pub name: String` - 转移名称
- `pub active_in: ActiveInBlueprint` - 转移应该监听事件的条件
- `pub event: EventId` - 要监听的事件类型
- `pub update: UpdateBlueprint` - 如何计算新状态（纯函数）

**方法：**
- `pub fn new(id: TransitionId, name: impl Into<String>, active_in: ActiveInBlueprint, event: EventId, update: UpdateBlueprint) -> Self` - 创建新的转移蓝图

**Trait 实现：**
- `Debug` - 显示转移蓝图信息
- `Clone` - 支持克隆

---

### Transition
运行时转移实例，表示事件触发的状态转移

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
- `pub id: TransitionId` - 转移的唯一标识符
- `pub name: String` - 转移名称
- `pub active_in: ActiveIn` - 转移应该监听事件的条件
- `pub event: EventId` - 要监听的事件类型
- `pub update: Update` - 如何计算新状态（纯函数）
- `pub on_tran: Option<TransitionHandler>` - 转移发生时触发的副作用

**方法：**
- `pub fn new(id: TransitionId, name: impl Into<String>, active_in: ActiveIn, event: EventId, update: Update) -> Self` - 创建新的转移
- `pub fn from_blueprint(blueprint: TransitionBlueprint) -> Self` - 从蓝图创建转移
- `pub fn with_on_tran<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static` - 设置 on_tran 处理器
- `pub fn is_active(&self, state: &State) -> bool` - 检查转移在给定状态下是否应该激活（监听事件）
- `pub fn apply(&self, state: &mut State)` - 应用更新到状态（直接修改状态）
- `pub fn trigger(&self)` - 执行 on_tran 处理器（如果存在）

**Trait 实现：**
- `Debug` - 显示基本信息，不显示闭包内容
- `PartialEq` - 仅基于 `id` 判断相等，忽略其他字段
- `Eq` - 相等关系的完整实现

---

## Type Aliases

### TransitionId
转移的唯一标识符

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransitionId(pub usize);
```

**Trait 实现：**
- `Debug` - 显示 ID
- `Clone` - 支持克隆
- `Copy` - 支持复制
- `PartialEq` - 相等比较
- `Eq` - 相等关系的完整实现
- `Hash` - 支持哈希

---

### TransitionHandler
转移事件的副作用处理器

```rust
pub type TransitionHandler = Box<dyn Fn() + Send + Sync>;
```

---

## Review 意见

1. **性能考虑**：`Transition::apply` 方法现在接受 `&mut State` 并直接修改状态，避免了克隆开销。这是对性能的重大改进。

2. **副作用处理**：`on_tran` 处理器在 `trigger` 方法中执行，与状态更新分离。建议在文档中明确说明执行顺序（先 `apply` 后 `trigger`）。

3. **激活条件**：转移只在 `active_in` 条件满足时监听事件。建议添加示例说明如何使用激活条件避免不必要的处理。

4. **类型安全**：`TransitionBlueprint` 和 `Transition` 的类型参数都使用了适当的 trait bounds，确保类型安全。

5. **API 一致性**：`TransitionBlueprint` 和 `Transition` 提供了相似的 API，保持了蓝图层和运行时层的一致性。

6. **Debug 实现**：`Transition` 的 `Debug` 实现不显示闭包内容，这是合理的，因为闭包的内容通常不便于显示。

7. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

8. **错误处理**：当前实现没有显式的错误处理。建议考虑在状态更新失败时提供错误信息。

9. **并发安全**：`TransitionHandler` 要求 `Send + Sync`，确保可以在多线程环境中安全使用。

10. **命名约定**：`on_tran` 使用下划线前缀表示事件处理器，建议在文档中说明这个命名约定。
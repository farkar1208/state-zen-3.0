# zone.rs API 文档

## 功能介绍
`zone.rs` 模块定义了状态区域（Zone）和生命周期管理，Zone 表示状态空间中的一个区域，支持在进入和离开该区域时触发副作用。

## 功能实现思路
- **蓝图层**：`ZoneBlueprint` 定义区域结构（声明式，不包含副作用）
- **运行时层**：`Zone` 包含生命周期回调（`on_enter`、`on_exit`）
- **激活判断**：通过 `is_active` 方法评估区域是否处于激活状态
- **构建器模式**：提供 `ZoneBuilder` 支持链式构建
- **相等性**：仅基于 `ZoneId` 判断相等性

---

## Structs

### ZoneId
区域的唯一标识符

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZoneId(pub usize);
```

**字段：**
- `pub usize` - 内部 usize 值

---

### ZoneBlueprint
区域蓝图（声明层，不包含副作用处理器）

```rust
#[derive(Debug, Clone)]
pub struct ZoneBlueprint {
    pub id: ZoneId,
    pub name: String,
    pub active_in: ActiveInBlueprint,
}
```

**字段：**
- `pub id: ZoneId` - 区域标识符
- `pub name: String` - 区域名称
- `pub active_in: ActiveInBlueprint` - 定义该区域覆盖的状态集合

**方法：**
- `pub fn new(id: ZoneId, name: impl Into<String>, active_in: ActiveInBlueprint) -> Self` - 创建新的区域蓝图

---

### Zone
运行时区域，包含生命周期语义

```rust
pub struct Zone {
    pub id: ZoneId,
    pub name: String,
    pub active_in: ActiveIn,
    pub on_enter: Option<ZoneHandler>,
    pub on_exit: Option<ZoneHandler>,
}
```

**字段：**
- `pub id: ZoneId` - 区域标识符
- `pub name: String` - 区域名称
- `pub active_in: ActiveIn` - 定义该区域覆盖的状态集合
- `pub on_enter: Option<ZoneHandler>` - 进入该区域时触发的副作用
- `pub on_exit: Option<ZoneHandler>` - 离开该区域时触发的副作用

**方法：**
- `pub fn new(id: ZoneId, name: impl Into<String>, active_in: ActiveIn) -> Self` - 创建新的区域
- `pub fn from_blueprint(blueprint: ZoneBlueprint) -> Self` - 从蓝图创建
- `pub fn with_on_enter<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static` - 设置进入处理器
- `pub fn with_on_exit<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static` - 设置离开处理器
- `pub fn is_active(&self, state: &crate::aspect::State) -> bool` - 检查区域是否激活
- `pub fn enter(&self)` - 执行进入处理器
- `pub fn exit(&self)` - 执行离开处理器

**Trait 实现：**
- `Debug` - 显示区域基本信息（不显示处理器内容）
- `PartialEq` - 仅基于 `id` 判断相等
- `Eq` - 完全相等语义

---

### ZoneBuilder
区域构建器

```rust
pub struct ZoneBuilder {
    id: Option<ZoneId>,
    name: Option<String>,
    active_in: Option<ActiveIn>,
    on_enter: Option<ZoneHandler>,
    on_exit: Option<ZoneHandler>,
}
```

**字段：**
- `id: Option<ZoneId>` - 区域标识符
- `name: Option<String>` - 区域名称
- `active_in: Option<ActiveIn>` - 激活条件
- `on_enter: Option<ZoneHandler>` - 进入处理器
- `on_exit: Option<ZoneHandler>` - 离开处理器

**方法：**
- `pub fn new() -> Self` - 创建构建器
- `pub fn id(mut self, id: ZoneId) -> Self` - 设置 ID
- `pub fn name(mut self, name: impl Into<String>) -> Self` - 设置名称
- `pub fn active_in(mut self, active_in: ActiveIn) -> Self` - 设置激活条件
- `pub fn on_enter<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static` - 设置进入处理器
- `pub fn on_exit<F>(mut self, handler: F) -> Self where F: Fn() + Send + Sync + 'static` - 设置离开处理器
- `pub fn build(self) -> Result<Zone, String>` - 构建区域，返回 `Result`

**Trait 实现：**
- `Default` - 默认为空构建器

---

## Type Aliases

### ZoneHandler
区域副作用处理器类型

```rust
pub type ZoneHandler = Box<dyn Fn() + Send + Sync>;
```

---

## Review 意见

1. **Debug 实现**：`Zone` 的 `Debug` 实现只显示处理器是否存在（`has_on_enter`、`has_on_exit`），而不显示实际内容。这是合理的，因为闭包无法直接显示。但可以考虑添加更多调试信息。

2. **相等性语义**：`PartialEq` 仅基于 `id` 判断相等，忽略 `name`、`active_in`、`on_enter`、`on_exit`。这可能导致混淆，建议在文档中明确说明此行为。

3. **错误处理**：`ZoneBuilder::build` 返回 `Result<Zone, String>`，但错误信息是硬编码字符串。建议考虑使用自定义错误类型或 `thiserror` 等库。

4. **处理器无参数**：`ZoneHandler` 不接受任何参数，无法访问状态或上下文。如果需要在处理器中访问状态，可能需要重新设计。

5. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

6. **Clone 缺失**：`Zone` 没有实现 `Clone`，可能因为 `ZoneHandler` 无法克隆。如果需要克隆 Zone，可以考虑使用 `Arc` 包装处理器。
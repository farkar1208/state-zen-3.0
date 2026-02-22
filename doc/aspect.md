# aspect.rs API 文档

## 功能介绍
`aspect.rs` 模块定义了状态面蓝图（`AspectBlueprint`）和状态面约束蓝图（`AspectBoundsBlueprint`），用于声明式地定义状态面的结构和约束条件。

## 功能实现思路
- **蓝图层**：`AspectBlueprint` 和 `AspectBoundsBlueprint` 是声明式定义，不包含验证逻辑或运行时行为
- **类型擦除**：使用 `Box<dyn ClonableAny>` 存储类型擦除的值和约束
- **类型一致性验证**：在构建时验证类型一致性，确保默认值和约束的类型匹配
- **链式 API**：通过 `with_*` 方法提供流畅的构建体验

---

## Structs

### AspectBoundsBlueprint
类型擦除的状态面约束定义（蓝图层），用于定义数值范围等约束

```rust
#[derive(Debug)]
pub struct AspectBoundsBlueprint {
    pub type_id: TypeId,
    pub type_name: String,
    pub min_value: Option<Box<dyn Any + Send + Sync>>,
    pub max_value: Option<Box<dyn Any + Send + Sync>>,
}
```

**字段：**
- `pub type_id: TypeId` - 约束类型的 TypeId
- `pub type_name: String` - 类型名称
- `pub min_value: Option<Box<dyn Any + Send + Sync>>` - 最小值（类型擦除）
- `pub max_value: Option<Box<dyn Any + Send + Sync>>` - 最大值（类型擦除）

**方法：**
- `pub fn new<T: 'static>() -> Self` - 创建新的约束蓝图
- `pub fn with_min<T: 'static + Send + Sync>(mut self, min: T) -> Self` - 设置最小值（类型必须匹配）
- `pub fn with_max<T: 'static + Send + Sync>(mut self, max: T) -> Self` - 设置最大值（类型必须匹配）
- `pub fn with_range<T: 'static + Send + Sync>(mut self, min: T, max: T) -> Self` - 设置范围（类型必须匹配）
- `pub fn is_type<T: 'static>(&self) -> bool` - 检查是否为特定类型

---

### AspectBlueprint
状态面蓝图定义（声明层）

```rust
#[derive(Debug)]
pub struct AspectBlueprint {
    pub id: AspectId,
    pub name: String,
    pub default_value: Box<dyn ClonableAny>,
    pub default_type_id: TypeId,
    pub default_type_name: String,
    pub bounds: Option<AspectBoundsBlueprint>,
}
```

**字段：**
- `pub id: AspectId` - 状态面标识符
- `pub name: String` - 状态面名称
- `pub default_value: Box<dyn ClonableAny>` - 默认值（类型擦除）
- `pub default_type_id: TypeId` - 默认值的 TypeId
- `pub default_type_name: String` - 默认值的类型名称
- `pub bounds: Option<AspectBoundsBlueprint>` - 约束条件（可选）

**方法：**
- `pub fn new<T: Any + Send + Sync + Clone + PartialEq + std::fmt::Debug + 'static>(id: AspectId, name: impl Into<String>, default_value: T) -> Self` - 创建新的状态面蓝图
- `pub fn with_bounds(mut self, bounds: AspectBoundsBlueprint) -> Self` - 设置约束（类型必须匹配）
- `pub fn with_range<T: 'static + Send + Sync + Clone + PartialEq + std::fmt::Debug>(mut self, min: T, max: T) -> Self` - 设置范围约束（便捷方法）
- `pub fn with_min<T: 'static + Send + Sync + Clone + PartialEq + std::fmt::Debug>(mut self, min: T) -> Self` - 设置最小值约束
- `pub fn with_max<T: 'static + Send + Sync + Clone + PartialEq + std::fmt::Debug>(mut self, max: T) -> Self` - 设置最大值约束
- `pub fn is_type<T: 'static>(&self) -> bool` - 检查是否为特定类型
- `pub fn get_default_as<T: 'static>(&self) -> Option<&T>` - 安全获取默认值的特定类型引用

**Trait 实现：**
- `Clone` - 深拷贝所有字段，包括类型擦除的值

---

## Type Aliases

（无公开类型别名）

---

## Functions

（无公共函数）

---

## Review 意见

1. **错误处理**：当类型不匹配时，`with_*` 方法使用 `panic!` 处理。对于生产环境，建议使用 `Result` 类型返回错误，让调用者决定如何处理。

2. **类型验证时机**：类型一致性验证在构建时进行，但运行时使用时可能仍需检查。建议考虑提供验证方法。

3. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

4. **约束验证**：`AspectBoundsBlueprint` 定义了约束但没有验证逻辑。建议考虑在运行时层添加验证机制。

5. **类型擦除开销**：使用 `Box<dyn ClonableAny>` 有 vtable 开销。对于大量小型值，建议考虑使用枚举或其他零成本抽象。

6. **API 一致性**：`with_min`、`with_max` 和 `with_range` 的类型约束完全相同，建议考虑统一类型参数或提供更灵活的 API。
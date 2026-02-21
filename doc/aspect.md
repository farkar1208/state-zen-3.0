# aspect.rs API 文档

## 功能介绍
`aspect.rs` 模块定义了状态面（StateAspect）和状态向量（State）管理，实现了类型擦除的状态容器，支持多种类型的值存储和运行时类型检查。

## 功能实现思路
- 使用 `HashMap<AspectId, Box<dyn Any + Send + Sync>>` 存储类型擦除的状态值
- 通过 `TypeId` 进行运行时类型检查，确保同一 AspectId 始终存储相同类型的值
- 蓝图层（`AspectBlueprint`、`AspectBoundsBlueprint`）定义声明式结构
- 运行时层（`State`）提供不可变的状态更新，每次 `set` 操作返回新的状态实例
- 提供 `StateBuilder` 便捷构建状态
- 实现 `Clone` 和 `PartialEq` trait 支持状态克隆和比较

---

## Structs

### AspectId
状态面的唯一标识符

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AspectId(pub usize);
```

**字段：**
- `pub usize` - 内部 usize 值

---

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
    pub default_value: Box<dyn Any + Send + Sync>,
    pub default_type_id: TypeId,
    pub default_type_name: String,
    pub bounds: Option<AspectBoundsBlueprint>,
}
```

**字段：**
- `pub id: AspectId` - 状态面标识符
- `pub name: String` - 状态面名称
- `pub default_value: Box<dyn Any + Send + Sync>` - 默认值（类型擦除）
- `pub default_type_id: TypeId` - 默认值的 TypeId
- `pub default_type_name: String` - 默认值的类型名称
- `pub bounds: Option<AspectBoundsBlueprint>` - 约束条件（可选）

**方法：**
- `pub fn new<T: Any + Send + Sync + 'static>(id: AspectId, name: impl Into<String>, default_value: T) -> Self` - 创建新的状态面蓝图
- `pub fn with_bounds(mut self, bounds: AspectBoundsBlueprint) -> Self` - 设置约束（类型必须匹配）
- `pub fn with_range<T: 'static + Send + Sync>(mut self, min: T, max: T) -> Self` - 设置范围约束（便捷方法）
- `pub fn with_min<T: 'static + Send + Sync>(mut self, min: T) -> Self` - 设置最小值约束
- `pub fn with_max<T: 'static + Send + Sync>(mut self, max: T) -> Self` - 设置最大值约束
- `pub fn is_type<T: 'static>(&self) -> bool` - 检查是否为特定类型
- `pub fn get_default_as<T: 'static>(&self) -> Option<&T>` - 安全获取默认值的特定类型引用

---

### State
运行时状态容器（高维状态向量）

```rust
#[derive(Debug, Default)]
pub struct State {
    values: HashMap<AspectId, Box<dyn Any + Send + Sync>>,
    type_ids: HashMap<AspectId, TypeId>,
}
```

**字段：**
- `values: HashMap<AspectId, Box<dyn Any + Send + Sync>>` - 状态面 ID 到类型擦除值的映射
- `type_ids: HashMap<AspectId, TypeId>` - 状态面 ID 到 TypeId 的映射（运行时类型检查）

**方法：**
- `pub fn new() -> Self` - 创建空状态
- `pub fn get(&self, aspect_id: AspectId) -> Option<&(dyn Any + Send + Sync)>` - 获取类型擦除的值
- `pub fn get_as<T: 'static>(&self, aspect_id: AspectId) -> Option<&T>` - 获取类型化值
- `pub fn set(&self, aspect_id: AspectId, value: Box<dyn Any + Send + Sync>) -> Self` - 设置值（返回新状态）
- `pub fn set_typed<T: Any + Send + Sync + 'static>(&self, aspect_id: AspectId, value: T) -> Self` - 设置类型化值
- `pub fn has(&self, aspect_id: AspectId) -> bool` - 检查是否包含某个状态面
- `pub fn aspect_ids(&self) -> impl Iterator<Item = AspectId> + '_` - 获取所有状态面 ID
- `pub fn len(&self) -> usize` - 获取状态面数量
- `pub fn is_empty(&self) -> bool` - 检查是否为空
- `pub fn get_type_id(&self, aspect_id: AspectId) -> Option<TypeId>` - 获取值的 TypeId
- `pub fn get_as_checked<T: 'static>(&self, aspect_id: AspectId, expected_type_id: TypeId) -> Option<&T>` - 带类型检查的获取

---

### StateBuilder
状态构建器，提供链式 API 构建状态

```rust
pub struct StateBuilder {
    values: HashMap<AspectId, Box<dyn Any + Send + Sync>>,
    type_ids: HashMap<AspectId, TypeId>,
}
```

**字段：**
- `values: HashMap<AspectId, Box<dyn Any + Send + Sync>>` - 状态面 ID 到类型擦除值的映射
- `type_ids: HashMap<AspectId, TypeId>` - 状态面 ID 到 TypeId 的映射

**方法：**
- `pub fn new() -> Self` - 创建构建器
- `pub fn set(mut self, aspect_id: AspectId, value: Box<dyn Any + Send + Sync>) -> Self` - 设置类型擦除值
- `pub fn set_typed<T: Any + Send + Sync + 'static>(mut self, aspect_id: AspectId, value: T) -> Self` - 设置类型化值
- `pub fn set_bool(self, aspect_id: AspectId, value: bool) -> Self` - 设置布尔值（便捷方法）
- `pub fn set_int(self, aspect_id: AspectId, value: i64) -> Self` - 设置整数（便捷方法）
- `pub fn set_float(self, aspect_id: AspectId, value: f64) -> Self` - 设置浮点数（便捷方法）
- `pub fn set_string(self, aspect_id: AspectId, value: impl Into<String>) -> Self` - 设置字符串（便捷方法）
- `pub fn build(self) -> State` - 构建状态

---

## Functions

### `pub fn any_value<T: Any + Send + Sync>(value: T) -> Box<dyn Any + Send + Sync>`
从常见类型创建类型擦除的值

### `pub fn clone_any(value: &Box<dyn Any + Send + Sync>) -> Box<dyn Any + Send + Sync>`
克隆类型擦除的值，支持常见类型，不支持的类型返回 `()` 作为后备

### `pub fn eq_any(a: &Box<dyn Any + Send + Sync>, b: &Box<dyn Any + Send + Sync>) -> bool`
比较两个类型擦除的值是否相等，支持常见类型，不支持的类型返回 `false`

---

## Review 意见

1. **类型擦除限制**：`clone_any` 和 `eq_any` 只支持有限的常见类型，对于自定义类型会返回后备值（`()` 或 `false`）。建议考虑使用 trait 约束要求类型实现 `Clone` 和 `PartialEq`，或者提供更完善的错误处理机制。

2. **错误处理**：当类型不匹配时，代码使用 `panic!` 处理。对于生产环境，建议使用 `Result` 类型返回错误，让调用者决定如何处理。

3. **get_type_id_of_any**：此函数内部实现了一个类型匹配链，对于新增类型需要修改代码。建议考虑使用更通用的方法或宏来减少维护成本。

4. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

5. **性能考虑**：`State::clone` 会深拷贝所有值，对于大型状态可能有性能影响。建议考虑引用计数（`Arc`）或其他优化策略。

6. **类型安全**：`set` 和 `set_typed` 方法在运行时进行类型检查，无法在编译时保证类型安全。这是类型擦除的权衡，但建议在文档中明确说明。
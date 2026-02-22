# state.rs API 文档

## 功能介绍
`state.rs` 模块定义了运行时状态容器（`State`）和状态构建器（`StateBuilder`），实现了高维状态向量管理，支持类型擦除的多类型值存储和运行时类型检查。

## 功能实现思路
- **运行时层**：`State` 提供可变的状态更新，`set` 和 `set_typed` 方法直接修改状态，避免克隆开销
- **类型安全**：通过 `TypeId` 进行运行时类型检查，确保同一 `AspectId` 始终存储相同类型的值
- **类型擦除**：使用 `HashMap<AspectId, Box<dyn ClonableAny>>` 存储类型擦除的值
- **构建器模式**：`StateBuilder` 提供链式 API 便捷构建状态
- **Clone 实现**：利用 `ClonableAny` trait 实现状态的深拷贝
- **PartialEq 实现**：基于类型和值的语义相等性比较

---

## Structs

### State
运行时状态容器，表示系统的完整状态作为高维状态向量

```rust
#[derive(Debug, Default)]
pub struct State {
    values: HashMap<AspectId, Box<dyn ClonableAny>>,
    type_ids: HashMap<AspectId, TypeId>,
}
```

**字段：**
- `values: HashMap<AspectId, Box<dyn ClonableAny>>` - 状态面 ID 到类型擦除值的映射
- `type_ids: HashMap<AspectId, TypeId>` - 状态面 ID 到 TypeId 的映射（运行时类型检查）

**方法：**
- `pub fn new() -> Self` - 创建空状态
- `pub fn get(&self, aspect_id: AspectId) -> Option<&(dyn ClonableAny)>` - 获取类型擦除的值
- `pub fn get_as<T: 'static>(&self, aspect_id: AspectId) -> Option<&T>` - 获取类型化值
- `pub fn set(&mut self, aspect_id: AspectId, value: Box<dyn ClonableAny>)` - 设置值，执行运行时类型检查。如果 `AspectId` 已存在且类型不同则 panic
- `pub fn set_typed<T: Any + Send + Sync + Clone + PartialEq + std::fmt::Debug + 'static>(&mut self, aspect_id: AspectId, value: T)` - 设置类型化值，执行运行时类型检查。如果 `AspectId` 已存在且类型不同则 panic
- `pub fn has(&self, aspect_id: AspectId) -> bool` - 检查是否包含某个状态面
- `pub fn aspect_ids(&self) -> impl Iterator<Item = AspectId> + '_` - 获取所有状态面 ID 的迭代器
- `pub fn len(&self) -> usize` - 获取状态面数量
- `pub fn is_empty(&self) -> bool` - 检查是否为空
- `pub fn get_type_id(&self, aspect_id: AspectId) -> Option<TypeId>` - 获取值的 TypeId
- `pub fn get_as_checked<T: 'static>(&self, aspect_id: AspectId, expected_type_id: TypeId) -> Option<&T>` - 带类型检查的获取，如果 TypeId 不匹配则返回 `None`

**Trait 实现：**
- `Default` - 提供空状态的默认实现
- `Debug` - 显示基本状态信息
- `Clone` - 深拷贝所有状态值
- `PartialEq` - 基于所有值的语义相等性比较

---

### StateBuilder
状态构建器，提供链式 API 构建状态

```rust
pub struct StateBuilder {
    values: HashMap<AspectId, Box<dyn ClonableAny>>,
    type_ids: HashMap<AspectId, TypeId>,
}
```

**字段：**
- `values: HashMap<AspectId, Box<dyn ClonableAny>>` - 状态面 ID 到类型擦除值的映射
- `type_ids: HashMap<AspectId, TypeId>` - 状态面 ID 到 TypeId 的映射

**方法：**
- `pub fn new() -> Self` - 创建构建器
- `pub fn set(mut self, aspect_id: AspectId, value: Box<dyn ClonableAny>) -> Self` - 设置类型擦除值
- `pub fn set_typed<T: Any + Send + Sync + Clone + PartialEq + std::fmt::Debug + 'static>(mut self, aspect_id: AspectId, value: T) -> Self` - 设置类型化值
- `pub fn set_bool(self, aspect_id: AspectId, value: bool) -> Self` - 设置布尔值（便捷方法）
- `pub fn set_int(self, aspect_id: AspectId, value: i64) -> Self` - 设置整数（便捷方法）
- `pub fn set_float(self, aspect_id: AspectId, value: f64) -> Self` - 设置浮点数（便捷方法）
- `pub fn set_string(self, aspect_id: AspectId, value: impl Into<String>) -> Self` - 设置字符串（便捷方法）
- `pub fn build(self) -> State` - 构建状态

**Trait 实现：**
- `Default` - 提供空构建器的默认实现

---

## Type Aliases

（无公开类型别名）

---

## Functions

（无公共函数）

---

## Review 意见

1. **性能考虑**：`State::set` 和 `State::set_typed` 现在使用可变引用，避免了每次更新都需要克隆整个状态的开销。这是对性能的重大改进。

2. **错误处理**：当类型不匹配时，`set` 和 `set_typed` 方法使用 `panic!` 处理。对于生产环境，建议使用 `Result` 类型返回错误，让调用者决定如何处理。

3. **类型安全**：类型检查在运行时进行，无法在编译时保证类型安全。这是类型擦除的权衡，但建议在文档中明确说明。

4. **内存开销**：每个 `Box<dyn ClonableAny>` 都有 vtable 开销。对于大量小型值，建议考虑使用枚举或其他零成本抽象。

5. **API 一致性**：`set` 和 `set_typed` 的行为一致，但 `set` 需要手动装箱，建议在文档中说明两者的使用场景。

6. **测试覆盖**：当前测试覆盖了基本功能，建议添加更多边界情况测试，如空状态、重复设置不同类型值等。

7. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

8. **类型推断**：`set_bool`、`set_int` 等便捷方法使用了特定类型（`i64`、`f64`），限制了灵活性。建议考虑使用泛型或提供更多类型变体。
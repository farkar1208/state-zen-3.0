# update.rs API 文档

## 功能介绍
`update.rs` 模块定义了状态更新操作，Update 是纯函数，接收当前状态并返回新状态。支持基本操作、条件更新和组合操作。

## 功能实现思路
- **蓝图层**：使用 `UpdateBlueprint` 枚举作为 AST 表示更新结构，可序列化
- **运行时层**：使用 `Update` 结构体封装 `Arc<UpdateOp>` 进行实际评估
- **编译**：`Update::from_blueprint` 将蓝图编译为运行时操作
- **组合**：支持 `Compose` 顺序组合多个更新操作
- **条件更新**：支持基于状态的分支更新（`Conditional`）
- **类型擦除**：使用 `Box<dyn Any + Send + Sync>` 支持多种类型

---

## Enums

### UpdateBlueprint
蓝图层更新操作 AST（抽象语法树）

```rust
#[derive(Debug, Clone)]
pub enum UpdateBlueprint {
    Noop,
    Set { aspect_id: AspectId, value: BlueprintValue },
    Modify { aspect_id: AspectId, type_id: std::any::TypeId, op: ModifyOp },
    Compose(Vec<UpdateBlueprint>),
    Conditional { predicate: ActiveInBlueprint, then_update: Box<UpdateBlueprint>, else_update: Option<Box<UpdateBlueprint>> },
}
```

**变体：**
- `Noop` - 无操作
- `Set { aspect_id, value }` - 设置特定值
- `Modify { aspect_id, type_id, op }` - 使用变换函数修改值
- `Compose(Vec<UpdateBlueprint>)` - 顺序组合多个更新
- `Conditional { predicate, then_update, else_update }` - 基于状态谓词的条件更新

**方法：**
- `pub fn noop() -> Self` - 创建无操作更新
- `pub fn set(aspect_id: AspectId, value: BlueprintValue) -> Self` - 设置值（类型擦除）
- `pub fn set_bool(aspect_id: AspectId, value: bool) -> Self` - 设置布尔值
- `pub fn set_int(aspect_id: AspectId, value: i64) -> Self` - 设置整数
- `pub fn set_float(aspect_id: AspectId, value: f64) -> Self` - 设置浮点数
- `pub fn set_string(aspect_id: AspectId, value: impl Into<String>) -> Self` - 设置字符串
- `pub fn increment(aspect_id: AspectId) -> Self` - 自增整数
- `pub fn decrement(aspect_id: AspectId) -> Self` - 自减整数
- `pub fn add(aspect_id: AspectId, delta: i64) -> Self` - 加法
- `pub fn toggle(aspect_id: AspectId) -> Self` - 切换布尔值
- `pub fn compose(updates: Vec<UpdateBlueprint>) -> Self` - 组合更新（自动优化空/单元素情况）
- `pub fn conditional(predicate: ActiveInBlueprint, then_update: UpdateBlueprint) -> Self` - 条件更新
- `pub fn conditional_else(predicate: ActiveInBlueprint, then_update: UpdateBlueprint, else_update: UpdateBlueprint) -> Self` - 条件更新（带 else）

---

### BlueprintValue
蓝图值类型（类型擦除表示）

```rust
#[derive(Debug, Clone)]
pub enum BlueprintValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
}
```

**变体：**
- `Bool(bool)` - 布尔值
- `Integer(i64)` - 整数
- `Float(f64)` - 浮点数
- `String(String)` - 字符串

---

### ModifyOp
修改操作类型

```rust
#[derive(Debug, Clone)]
pub enum ModifyOp {
    Increment,
    Decrement,
    Add(i64),
    Toggle,
}
```

**变体：**
- `Increment` - 自增 1
- `Decrement` - 自减 1
- `Add(i64)` - 加上指定值
- `Toggle` - 切换布尔值

---

## Structs

### Update
运行时更新操作

```rust
pub struct Update {
    operation: Arc<UpdateOp>,
}
```

**字段：**
- `operation: Arc<UpdateOp>` - 内部操作表示（使用 Arc 共享）

**方法：**
- `pub fn noop() -> Self` - 创建无操作更新
- `pub fn from_blueprint(blueprint: UpdateBlueprint) -> Self` - 从蓝图编译
- `pub fn set(aspect_id: AspectId, value: Box<dyn Any + Send + Sync>) -> Self` - 设置值（类型擦除）
- `pub fn set_typed<T: Any + Send + Sync>(aspect_id: AspectId, value: T) -> Self` - 设置类型化值
- `pub fn modify<F>(aspect_id: AspectId, f: F) -> Self where F: Fn(Box<dyn Any + Send + Sync>) -> Box<dyn Any + Send + Sync> + Send + Sync + 'static` - 使用函数修改
- `pub fn set_bool(aspect_id: AspectId, value: bool) -> Self` - 设置布尔值
- `pub fn set_int(aspect_id: AspectId, value: i64) -> Self` - 设置整数
- `pub fn set_float(aspect_id: AspectId, value: f64) -> Self` - 设置浮点数
- `pub fn set_string(aspect_id: AspectId, value: impl Into<String>) -> Self` - 设置字符串
- `pub fn increment(aspect_id: AspectId) -> Self` - 自增整数
- `pub fn decrement(aspect_id: AspectId) -> Self` - 自减整数
- `pub fn add(aspect_id: AspectId, delta: i64) -> Self` - 加法
- `pub fn toggle(aspect_id: AspectId) -> Self` - 切换布尔值
- `pub fn modify_typed<T, F>(aspect_id: AspectId, f: F) -> Self where T: Any + Send + Sync + Clone, F: Fn(T) -> T + Send + Sync + 'static` - 类型化修改
- `pub fn compose(updates: Vec<Update>) -> Self` - 组合更新（自动优化空/单元素情况）
- `pub fn conditional<F>(predicate: F, then_update: Update) -> Self where F: Fn(&State) -> bool + Send + Sync + 'static` - 条件更新
- `pub fn conditional_else<F>(predicate: F, then_update: Update, else_update: Update) -> Self where F: Fn(&State) -> bool + Send + Sync + 'static` - 条件更新（带 else）
- `pub fn apply(&self, state: State) -> State` - 应用更新到状态

**Trait 实现：**
- `Clone` - 使用 Arc 共享内部操作

---

## Functions

### `fn compile_update(blueprint: UpdateBlueprint) -> UpdateOp`
编译 `UpdateBlueprint` AST 为运行时 `UpdateOp`（内部使用）

---

## Review 意见

1. **类型限制**：`apply` 方法中的类型匹配链只支持有限类型（`bool`、`i64`、`f64`、`String`、`i32`）。对于其他类型，修改操作会返回原值，可能导致静默失败。建议考虑使用更通用的方法或错误处理。

2. **性能考虑**：`UpdateOp::Modify` 在应用时需要克隆整个状态以获取可变引用，对于大型状态可能有性能开销。建议考虑优化策略。

3. **类型安全**：`modify_typed` 要求类型实现 `Clone`，这增加了性能开销。如果不需要克隆，可以考虑使用引用。

4. **错误处理**：当前实现中，类型不匹配时静默返回原值。这可能导致难以发现的 bug。建议考虑使用 `Result` 类型或添加日志。

5. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

6. **API 一致性**：`UpdateBlueprint` 和 `Update` 的 API 非常相似，但有一些细微差异（如 `conditional` 的谓词类型不同）。建议在文档中清晰说明区别。

7. **空优化**：`compose` 方法自动处理空和单元素情况，这是很好的设计，但建议在文档中明确说明。

8. **类型擦除开销**：使用 `Box<dyn Any + Send + Sync>` 有运行时开销。如果性能是关键因素，可以考虑使用泛型实现，但这会增加复杂度。
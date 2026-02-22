# update.rs API 文档

## 功能介绍
`update.rs` 模块定义了状态更新操作（`Update`）和更新蓝图（`UpdateBlueprint`），实现了纯函数式的状态变换算子，支持基本操作、条件更新和组合。

## 功能实现思路
- **蓝图层**：`UpdateBlueprint` 定义声明式的更新操作结构（AST），支持序列化
- **运行时层**：`Update` 编译蓝图为可执行的更新算子，使用闭包实现实际更新逻辑
- **纯函数变换**：状态更新通过纯函数实现，与副作用分离
- **可组合性**：支持多个更新操作的顺序组合
- **条件分支**：支持基于状态谓词的条件更新

---

## Enums

### UpdateBlueprint
更新操作的蓝图层定义（AST）

```rust
#[derive(Debug, Clone)]
pub enum UpdateBlueprint {
    Noop,
    Set {
        aspect_id: AspectId,
        value: BlueprintValue,
    },
    Modify {
        aspect_id: AspectId,
        type_id: std::any::TypeId,
        op: ModifyOp,
    },
    Compose(Vec<UpdateBlueprint>),
    Conditional {
        predicate: ActiveInBlueprint,
        then_update: Box<UpdateBlueprint>,
        else_update: Option<Box<UpdateBlueprint>>,
    },
}
```

**变体：**
- `Noop` - 无操作
- `Set` - 设置状态面为特定值
- `Modify` - 使用变换函数修改状态面的值
- `Compose` - 组合多个更新操作
- `Conditional` - 基于状态谓词的条件更新

**方法：**
- `pub fn noop() -> Self` - 创建无操作更新
- `pub fn set(aspect_id: AspectId, value: BlueprintValue) -> Self` - 设置类型擦除值
- `pub fn set_bool(aspect_id: AspectId, value: bool) -> Self` - 设置布尔值
- `pub fn set_int(aspect_id: AspectId, value: i64) -> Self` - 设置整数
- `pub fn set_float(aspect_id: AspectId, value: f64) -> Self` - 设置浮点数
- `pub fn set_string(aspect_id: AspectId, value: impl Into<String>) -> Self` - 设置字符串
- `pub fn increment(aspect_id: AspectId) -> Self` - 自增整数
- `pub fn decrement(aspect_id: AspectId) -> Self` - 自减整数
- `pub fn add(aspect_id: AspectId, delta: i64) -> Self` - 加法操作
- `pub fn toggle(aspect_id: AspectId) -> Self` - 切换布尔值
- `pub fn compose(updates: Vec<UpdateBlueprint>) -> Self` - 组合多个更新
- `pub fn conditional(predicate: ActiveInBlueprint, then_update: UpdateBlueprint) -> Self` - 条件更新
- `pub fn conditional_else(predicate: ActiveInBlueprint, then_update: UpdateBlueprint, else_update: UpdateBlueprint) -> Self` - 条件更新带 else 分支

**Trait 实现：**
- `Debug` - 显示更新操作结构
- `Clone` - 支持克隆

---

### BlueprintValue
蓝图值（类型擦除表示）

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
- `Integer(i64)` - 整数值
- `Float(f64)` - 浮点数值
- `String(String)` - 字符串值

**Trait 实现：**
- `Debug` - 显示值
- `Clone` - 支持克隆

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

**Trait 实现：**
- `Debug` - 显示操作类型
- `Clone` - 支持克隆

---

## Structs

### Update
运行时更新算子，表示状态如何响应事件而演化

```rust
pub struct Update {
    operation: Arc<UpdateOp>,
}
```

**字段：**
- `operation: Arc<UpdateOp>` - 内部更新操作表示（运行时层）

**方法：**
- `pub fn noop() -> Self` - 创建无操作更新
- `pub fn from_blueprint(blueprint: UpdateBlueprint) -> Self` - 从蓝图编译为运行时更新
- `pub fn set(aspect_id: AspectId, value: Box<dyn ClonableAny>) -> Self` - 设置类型擦除值
- `pub fn set_typed<T: Any + Send + Sync + Clone + PartialEq + std::fmt::Debug>(aspect_id: AspectId, value: T) -> Self` - 设置类型化值
- `pub fn modify<F>(aspect_id: AspectId, f: F) -> Self where F: Fn(Box<dyn ClonableAny>) -> Box<dyn ClonableAny> + Send + Sync + 'static` - 使用变换函数修改
- `pub fn set_bool(aspect_id: AspectId, value: bool) -> Self` - 设置布尔值
- `pub fn set_int(aspect_id: AspectId, value: i64) -> Self` - 设置整数
- `pub fn set_float(aspect_id: AspectId, value: f64) -> Self` - 设置浮点数
- `pub fn set_string(aspect_id: AspectId, value: impl Into<String>) -> Self` - 设置字符串
- `pub fn increment(aspect_id: AspectId) -> Self` - 自增整数
- `pub fn decrement(aspect_id: AspectId) -> Self` - 自减整数
- `pub fn add(aspect_id: AspectId, delta: i64) -> Self` - 加法操作
- `pub fn toggle(aspect_id: AspectId) -> Self` - 切换布尔值
- `pub fn modify_typed<T, F>(aspect_id: AspectId, f: F) -> Self where T: Any + Send + Sync + Clone + PartialEq + std::fmt::Debug, F: Fn(T) -> T + Send + Sync + 'static` - 类型化修改
- `pub fn compose(updates: Vec<Update>) -> Self` - 组合多个更新
- `pub fn conditional<F>(predicate: F, then_update: Update) -> Self where F: Fn(&State) -> bool + Send + Sync + 'static` - 条件更新
- `pub fn conditional_else<F>(predicate: F, then_update: Update, else_update: Update) -> Self where F: Fn(&State) -> bool + Send + Sync + 'static` - 条件更新带 else 分支
- `pub fn apply(&self, state: &mut State)` - 应用更新到状态（直接修改状态）

**Trait 实现：**
- `Clone` - 支持克隆（使用 `Arc` 共享操作）

---

## Type Aliases

（无公开类型别名）

---

## Functions

（无公共函数）

---

## Review 意见

1. **性能考虑**：`Update::apply` 方法现在接受 `&mut State` 并直接修改状态，避免了克隆开销。这是对性能的重大改进。

2. **纯函数语义**：虽然 `apply` 方法现在使用可变引用，但更新操作本身仍然是纯函数式的。所有副作用应该在 `on_tran` 处理器中实现。

3. **错误处理**：`modify` 和 `modify_typed` 方法在类型不匹配时静默跳过（返回原值）。建议考虑返回 `Result` 或记录警告。

4. **类型安全**：`BlueprintValue` 只支持基本类型（bool、i64、f64、String）。对于复杂类型，用户需要使用 `modify_typed`。

5. **API 一致性**：`UpdateBlueprint` 和 `Update` 提供了相似的 API，保持了蓝图层和运行时层的一致性。

6. **组合操作**：`Compose` 操作现在使用顺序循环而不是 `fold`，更直接地反映可变状态的操作语义。

7. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

8. **闭包捕获**：`conditional` 和 `conditional_else` 使用闭包捕获谓词，需要注意生命周期和内存使用。

9. **Arc 使用**：`Update` 使用 `Arc<UpdateOp>` 共享操作，允许多个 `Update` 实例共享相同的操作逻辑。

10. **类型推断**：`modify_typed` 方法需要显式类型参数 `T`，在某些情况下可能不够灵活。建议考虑使用 `Into` trait 或其他方式改善类型推断。
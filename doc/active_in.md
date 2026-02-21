# active_in.rs API 文档

## 功能介绍
`active_in.rs` 模块定义了谓词函数和激活条件，支持蓝图层（AST）和运行时层（闭包）两种表示方式，用于判断行为（Zone 或 Transition）在给定状态下是否应被激活。

## 功能实现思路
- **蓝图层**：使用 `ActiveInBlueprint` 枚举作为 AST（抽象语法树）表示谓词结构，可序列化
- **运行时层**：使用 `ActiveIn` 结构体封装 `Arc<dyn Fn(&State) -> bool>` 闭包进行实际评估
- **编译**：`ActiveIn::from_blueprint` 将蓝图编译为运行时闭包
- **组合**：支持 AND、OR、NOT 等逻辑组合操作，实现了 `Not` trait
- **泛型支持**：提供类型化的比较方法（`aspect_lt_typed`、`aspect_gt_typed`、`aspect_eq_typed`）

---

## Enums

### ActiveInBlueprint
蓝图层激活条件 AST（抽象语法树）

```rust
#[derive(Debug, Clone)]
pub enum ActiveInBlueprint {
    Always,
    Never,
    AspectBool { aspect_id: AspectId, value: bool },
    AspectEq { aspect_id: AspectId, value: i64 },
    AspectLt { aspect_id: AspectId, value: i64 },
    AspectGt { aspect_id: AspectId, value: i64 },
    AspectInRange { aspect_id: AspectId, min: i64, max: i64 },
    AspectStringEq { aspect_id: AspectId, value: String },
    And(Vec<ActiveInBlueprint>),
    Or(Vec<ActiveInBlueprint>),
    Not(Box<ActiveInBlueprint>),
}
```

**变体：**
- `Always` - 总是为真
- `Never` - 总是为假
- `AspectBool { aspect_id, value }` - 布尔值等于
- `AspectEq { aspect_id, value }` - 整数等于
- `AspectLt { aspect_id, value }` - 小于
- `AspectGt { aspect_id, value }` - 大于
- `AspectInRange { aspect_id, min, max }` - 范围内 [min, max]
- `AspectStringEq { aspect_id, value }` - 字符串等于
- `And(Vec<ActiveInBlueprint>)` - 逻辑与
- `Or(Vec<ActiveInBlueprint>)` - 逻辑或
- `Not(Box<ActiveInBlueprint>)` - 逻辑非

**方法：**
- `pub fn always() -> Self` - 创建总是为真的谓词
- `pub fn never() -> Self` - 创建总是为假的谓词
- `pub fn aspect_bool(aspect_id: AspectId, value: bool) -> Self` - 布尔值等于
- `pub fn aspect_eq(aspect_id: AspectId, value: i64) -> Self` - 整数等于
- `pub fn aspect_lt(aspect_id: AspectId, value: i64) -> Self` - 小于
- `pub fn aspect_gt(aspect_id: AspectId, value: i64) -> Self` - 大于
- `pub fn aspect_in_range(aspect_id: AspectId, min: i64, max: i64) -> Self` - 范围内
- `pub fn aspect_string_eq(aspect_id: AspectId, value: impl Into<String>) -> Self` - 字符串等于
- `pub fn and(self, other: ActiveInBlueprint) -> Self` - 逻辑与（自动展平）
- `pub fn or(self, other: ActiveInBlueprint) -> Self` - 逻辑或（自动展平）
- `pub fn all(predicates: Vec<ActiveInBlueprint>) -> Self` - 多个谓词的与
- `pub fn any(predicates: Vec<ActiveInBlueprint>) -> Self` - 多个谓词的或

**Not trait 实现：**
- `fn not(self) -> Self` - 返回 `Not(Box::new(self))`

---

## Type Aliases

### Predicate
谓词函数类型别名

```rust
pub type Predicate = Arc<dyn Fn(&State) -> bool + Send + Sync>;
```

---

## Structs

### ActiveIn
运行时激活条件

```rust
#[derive(Clone)]
pub struct ActiveIn {
    predicate: Predicate,
}
```

**字段：**
- `predicate: Predicate` - 闭包形式的谓词函数

**方法：**
- `pub fn new<F>(f: F) -> Self where F: Fn(&State) -> bool + Send + Sync + 'static` - 从函数创建
- `pub fn from_blueprint(blueprint: ActiveInBlueprint) -> Self` - 从蓝图编译
- `pub fn evaluate(&self, state: &State) -> bool` - 评估谓词
- `pub fn always() -> Self` - 总是为真
- `pub fn never() -> Self` - 总是为假
- `pub fn aspect_bool(aspect_id: AspectId, value: bool) -> Self` - 布尔值等于
- `pub fn aspect_eq(aspect_id: AspectId, value: i64) -> Self` - 整数等于
- `pub fn aspect_lt(aspect_id: AspectId, value: i64) -> Self` - 小于
- `pub fn aspect_gt(aspect_id: AspectId, value: i64) -> Self` - 大于
- `pub fn aspect_in_range(aspect_id: AspectId, min: i64, max: i64) -> Self` - 范围内
- `pub fn aspect_string_eq(aspect_id: AspectId, value: impl Into<String> + Clone) -> Self` - 字符串等于
- `pub fn aspect_lt_typed<T>(aspect_id: AspectId, value: T) -> Self where T: std::cmp::PartialOrd + Send + Sync + 'static` - 泛型小于
- `pub fn aspect_gt_typed<T>(aspect_id: AspectId, value: T) -> Self where T: std::cmp::PartialOrd + Send + Sync + 'static` - 泛型大于
- `pub fn aspect_eq_typed<T>(aspect_id: AspectId, value: T) -> Self where T: std::cmp::PartialEq + Send + Sync + 'static` - 泛型等于
- `pub fn and(self, other: ActiveIn) -> Self` - 逻辑与
- `pub fn or(self, other: ActiveIn) -> Self` - 逻辑或
- `pub fn all(predicates: Vec<ActiveIn>) -> Self` - 多个谓词的与
- `pub fn any(predicates: Vec<ActiveIn>) -> Self` - 多个谓词的或

**Not trait 实现：**
- `fn not(self) -> Self` - 返回取反后的谓词

---

## Functions

### `pub(crate) fn evaluate_blueprint(blueprint: &ActiveInBlueprint, state: &State) -> bool`
评估蓝图谓词 AST 并返回结果（内部使用）

---

## Review 意见

1. **API 一致性**：`ActiveInBlueprint::aspect_string_eq` 接受 `impl Into<String>`，但 `ActiveIn::aspect_string_eq` 接受 `impl Into<String> + Clone`。建议统一签名。

2. **性能考虑**：`ActiveIn::from_blueprint` 在每次评估时都会递归调用 `evaluate_blueprint`，对于复杂的谓词可能有性能开销。可以考虑在编译时将 AST 转换为更高效的闭包结构。

3. **错误处理**：当状态中不存在指定的 aspect_id 时，当前行为是返回 `false`。这可能隐藏配置错误。建议考虑添加验证机制或文档说明此行为。

4. **类型支持**：当前只支持 `bool`、`i64`、`String` 三种类型的特定谓词，但提供了泛型方法。建议在文档中更清晰地说明何时使用泛型方法。

5. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

6. **命名**：`evaluate_blueprint` 是 `pub(crate)`，但可能对测试和调试有用。考虑是否应该暴露为公开 API 或提供更好的调试接口。
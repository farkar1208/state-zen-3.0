# core.rs API 文档

## 功能介绍
`core.rs` 模块定义了 state-zen 项目的核心基础类型和 trait，包括类型擦除值的克隆和比较能力（`ClonableAny`）以及各类标识符类型（`AspectId`、`EventId`）。

## 功能实现思路
- `ClonableAny` trait 为类型擦除的值提供类型安全的克隆和相等比较能力
- 通过 blanket implementation 自动为所有满足约束的类型实现 `ClonableAny`
- `AspectId` 使用 `usize` 作为内部表示，支持 `Copy` 和 `Hash`
- `EventId` 使用 `String` 作为内部表示，提供灵活的事件命名

---

## Traits

### `pub trait ClonableAny: Any + Send + Sync + std::fmt::Debug`
类型擦除的值支持克隆和相等比较的 trait。

此 trait 自动为所有满足以下约束的类型实现：
- `Any + Send + Sync + Clone + PartialEq + std::fmt::Debug + 'static`

用户只需为自定义类型 derive `Clone` 和 `PartialEq` 即可获得支持。

**方法：**
- `fn clone_box(&self) -> Box<dyn ClonableAny>` - 克隆此类型擦除的值到新的 boxed 值
- `fn eq_box(&self, other: &dyn ClonableAny) -> bool` - 与另一个类型擦除的值比较相等性。如果类型不同则返回 `false`
- `fn as_any(&self) -> &dyn Any` - 访问底层 `Any` trait 用于 downcasting

**使用示例：**
```rust
// 基础类型自动实现
let bool_val: Box<dyn ClonableAny> = Box::new(true);
let cloned = bool_val.clone_box();  // 克隆
assert!(cloned.eq_box(bool_val.as_ref()));

// 自定义类型只需 derive Clone 和 PartialEq
#[derive(Clone, PartialEq, Debug)]
struct MyStruct { x: i32 }
let custom_val: Box<dyn ClonableAny> = Box::new(MyStruct { x: 100 });
```

---

## Structs

### AspectId
状态面（Aspect）的唯一标识符

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AspectId(pub usize);
```

**字段：**
- `pub usize` - 内部 `usize` 值

**Trait 实现：**
- `Copy` - 支持复制语义
- `Hash` - 可用作 `HashMap` 和 `HashSet` 的键
- `PartialEq/Eq` - 基于内部 `usize` 值判断相等

---

### EventId
事件类型的唯一标识符

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventId(pub String);
```

**字段：**
- `pub String` - 内部字符串值

**方法：**
- `pub fn new(id: impl Into<String>) -> Self` - 创建新的事件 ID

**Trait 实现：**
- `Clone` - 支持克隆
- `Hash` - 可用作 `HashMap` 和 `HashSet` 的键
- `PartialEq/Eq` - 基于字符串内容判断相等

---

## Functions

（无公共函数）

---

## Review 意见

1. **类型约束**：`ClonableAny` 要求类型实现 `Clone` 和 `PartialEq`，这限制了只能用于可克隆和可比较的类型。这是合理的权衡，但文档中应明确说明。

2. **类型安全**：`eq_box` 在类型不匹配时返回 `false` 而不是错误，这可能导致静默失败。建议考虑在文档中强调类型匹配的重要性。

3. **EventId 设计**：`EventId` 使用 `String` 而非 `&str`，增加了内存开销。建议考虑使用 `Arc<str>` 或其他零拷贝方案（如果性能是关键）。

4. **命名一致性**：`AspectId` 和 `EventId` 的内部字段命名不一致（`pub usize` vs `pub String`）。建议考虑统一使用具名字段或保持匿名元组风格。

5. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

6. **测试覆盖**：当前测试覆盖了基本功能，建议添加更多边界情况测试，如空字符串的 `EventId`、零值的 `AspectId` 等。
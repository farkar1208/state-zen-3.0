# blueprint.rs API 文档

## 功能介绍
`blueprint.rs` 模块定义了状态机蓝图（StateMachineBlueprint），用于声明式地定义状态机结构。蓝图包含状态面、区域、转移和事件类型，可以编译/验证后实例化为可运行的状态机。

## 功能实现思路
- **蓝图模式**：`StateMachineBlueprint` 是状态机的声明式定义，不包含运行时行为
- **类型擦除**：`AspectDescriptor` 使用类型擦除存储状态面信息
- **初始状态**：`create_initial_state` 从蓝图创建初始状态
- **构建器模式**：通过链式调用 `add_aspect`、`add_zone`、`add_transition` 构建蓝图
- **事件追踪**：自动收集所有引用的事件类型

---

## Structs

### AspectDescriptor
类型擦除的状态面描述符

```rust
#[derive(Debug)]
pub struct AspectDescriptor {
    pub id: AspectId,
    pub name: String,
    pub type_id: TypeId,
    pub default_value: Box<dyn Any + Send + Sync>,
    pub has_min: bool,
    pub has_max: bool,
}
```

**字段：**
- `pub id: AspectId` - 状态面标识符
- `pub name: String` - 状态面名称
- `pub type_id: TypeId` - 类型的 TypeId
- `pub default_value: Box<dyn Any + Send + Sync>` - 默认值（类型擦除）
- `pub has_min: bool` - 是否有最小值约束
- `pub has_max: bool` - 是否有最大值约束

**方法：**
- `pub fn from_blueprint(blueprint: &AspectBlueprint) -> Self` - 从蓝图创建描述符

**Trait 实现：**
- `Clone` - 支持克隆（使用类型匹配链处理常见类型）

---

### StateMachineBlueprint
状态机蓝图（声明式定义）

```rust
#[derive(Debug)]
pub struct StateMachineBlueprint {
    pub id: String,
    aspects: HashMap<AspectId, AspectDescriptor>,
    zones: Vec<Zone>,
    transitions: Vec<Transition>,
    events: HashSet<EventId>,
}
```

**字段：**
- `pub id: String` - 蓝图唯一标识符
- `aspects: HashMap<AspectId, AspectDescriptor>` - 所有状态面定义（类型擦除）
- `zones: Vec<Zone>` - 所有区域定义
- `transitions: Vec<Transition>` - 所有转移定义
- `events: HashSet<EventId>` - 所有引用的事件类型

**方法：**
- `pub fn new(id: impl Into<String>) -> Self` - 创建空蓝图
- `pub fn add_aspect(&mut self, blueprint: AspectBlueprint) -> &mut Self` - 添加状态面（使用 AspectBlueprint）
- `pub fn add_zone(&mut self, zone: Zone) -> &mut Self` - 添加区域
- `pub fn add_transition(&mut self, transition: Transition) -> &mut Self` - 添加转移（自动收集事件类型）
- `pub fn aspects(&self) -> impl Iterator<Item = &AspectDescriptor>` - 获取所有状态面描述符
- `pub fn get_aspect(&self, id: AspectId) -> Option<&AspectDescriptor>` - 根据 ID 获取状态面描述符
- `pub fn zones(&self) -> &[Zone]` - 获取所有区域
- `pub fn transitions(&self) -> &[Transition]` - 获取所有转移
- `pub fn events(&self) -> &HashSet<EventId>` - 获取所有事件类型
- `pub fn create_initial_state(&self) -> State` - 从蓝图的默认值创建初始状态

---

## Review 意见

1. **类型擦除限制**：`AspectDescriptor::clone` 和 `create_initial_state` 中的类型匹配链只支持有限类型（`bool`、`i64`、`f64`、`String`、`i32`、`usize`、`u32`、`u64`、`char` 及其 Vec 变体）。对于不支持的类型，`clone` 返回 `()`，`create_initial_state` 跳过该状态面。这可能导致静默失败，建议考虑使用更通用的方法或错误处理。

2. **验证缺失**：`StateMachineBlueprint` 没有验证逻辑，例如：
   - 检查 AspectId 是否唯一
   - 检查 ZoneId 是否唯一
   - 检查 TransitionId 是否唯一
   - 检查转移引用的状态面是否存在
   - 建议添加 `validate()` 方法

3. **构建器返回类型**：`add_aspect`、`add_zone`、`add_transition` 返回 `&mut Self`，支持链式调用。但这是可变借用模式，如果需要不可变的构建器模式，可以考虑使用单独的 `BlueprintBuilder`。

4. **事件收集**：`add_transition` 自动收集事件类型，这是很好的设计。但如果转移被移除，事件类型不会被清理。如果需要动态修改，建议考虑使用计数或其他机制。

5. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

6. **Clone 实现**：`StateMachineBlueprint` 没有实现 `Clone`，可能因为包含 `Zone` 和 `Transition`（它们的处理器无法克隆）。如果需要克隆蓝图，可以考虑使用 `Arc` 或重新设计。

7. **序列化支持**：蓝图应该支持序列化（存储/传输），但当前实现使用了 `Box<dyn Any>`，无法直接序列化。建议考虑使用枚举或其他可序列化的表示方式。

8. **性能考虑**：`create_initial_state` 对每个状态面都进行类型匹配和克隆，对于大量状态面可能有性能开销。可以考虑优化或缓存。
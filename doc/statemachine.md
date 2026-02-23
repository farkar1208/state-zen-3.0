# statemachine.rs API 文档

## 功能介绍
`statemachine.rs` 模块定义了状态机的蓝图层和运行时层。蓝图层（`StateMachineBlueprint`）提供声明式的状态机定义，包含状态面、区域蓝图和转移蓝图；运行时层（`StateMachineRuntime`）从蓝图创建可执行实例，将蓝图转换为运行时对象，维护当前状态、追踪区域激活状态并提供事件分发功能。

## 功能实现思路

### 蓝图层（Blueprint Layer）
- **蓝图模式**：`StateMachineBlueprint` 是状态机的声明式定义，不包含运行时行为和副作用处理器
- **类型擦除**：`AspectDescriptor` 使用类型擦除存储状态面信息
- **蓝图存储**：存储 `ZoneBlueprint` 和 `TransitionBlueprint`，而非运行时对象
- **初始状态**：`create_initial_state` 从蓝图创建初始状态
- **构建器模式**：通过链式调用 `add_aspect`、`add_zone`、`add_transition` 构建蓝图
- **事件追踪**：自动收集所有引用的事件类型
- **可序列化**：蓝图层不包含闭包，理论上可序列化存储或传输

### 运行时层（Runtime Layer）
- **蓝图编译**：从 `StateMachineBlueprint` 创建运行时实例时，自动将 `ZoneBlueprint` 和 `TransitionBlueprint` 编译为运行时对象
- **状态管理**：维护当前状态，支持状态查询和重置
- **事件分发**：通过 `dispatch` 方法分发事件，触发匹配的状态转移
- **区域追踪**：自动追踪和更新区域激活状态，触发进入/离开副作用
- **生命周期管理**：提供 `reset` 方法重置状态机到初始状态
- **副作用注入**：通过 builder 方法在运行时添加副作用处理器（`on_enter`、`on_exit`、`on_tran`）
- **自定义更新**：支持在运行时替换转移的更新操作

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
    pub default_value: Box<dyn ClonableAny>,
    pub has_min: bool,
    pub has_max: bool,
}
```

**字段：**
- `pub id: AspectId` - 状态面标识符
- `pub name: String` - 状态面名称
- `pub type_id: TypeId` - 类型的 TypeId
- `pub default_value: Box<dyn ClonableAny>` - 默认值（类型擦除）
- `pub has_min: bool` - 是否有最小值约束
- `pub has_max: bool` - 是否有最大值约束

**方法：**
- `pub fn from_blueprint(blueprint: &AspectBlueprint) -> Self` - 从蓝图创建描述符

**Trait 实现：**
- `Clone` - 支持克隆（使用 `ClonableAny::clone_box()` 方法）
- `Debug` - 显示基本信息

---

### StateMachineBlueprint
状态机蓝图（声明式定义）

```rust
#[derive(Debug)]
pub struct StateMachineBlueprint {
    pub id: String,
    aspects: HashMap<AspectId, AspectDescriptor>,
    zones: Vec<ZoneBlueprint>,
    transitions: Vec<TransitionBlueprint>,
    events: HashSet<EventId>,
}
```

**字段：**
- `pub id: String` - 蓝图唯一标识符
- `aspects: HashMap<AspectId, AspectDescriptor>` - 所有状态面定义（类型擦除）
- `zones: Vec<ZoneBlueprint>` - 所有区域蓝图定义（不包含副作用处理器）
- `transitions: Vec<TransitionBlueprint>` - 所有转移蓝图定义（不包含副作用处理器）
- `events: HashSet<EventId>` - 所有引用的事件类型

**方法：**
- `pub fn new(id: impl Into<String>) -> Self` - 创建空蓝图
- `pub fn add_aspect(&mut self, blueprint: AspectBlueprint) -> &mut Self` - 添加状态面（使用 AspectBlueprint）
- `pub fn add_zone(&mut self, zone: ZoneBlueprint) -> &mut Self` - 添加区域蓝图
- `pub fn add_transition(&mut self, transition: TransitionBlueprint) -> &mut Self` - 添加转移蓝图（自动收集事件类型）
- `pub fn aspects(&self) -> impl Iterator<Item = &AspectDescriptor>` - 获取所有状态面描述符
- `pub fn get_aspect(&self, id: AspectId) -> Option<&AspectDescriptor>` - 根据 ID 获取状态面描述符
- `pub fn zones(&self) -> &[ZoneBlueprint]` - 获取所有区域蓝图
- `pub fn transitions(&self) -> &[TransitionBlueprint]` - 获取所有转移蓝图
- `pub fn events(&self) -> &HashSet<EventId>` - 获取所有事件类型
- `pub fn create_initial_state(&self) -> State` - 从蓝图的默认值创建初始状态

**Trait 实现：**
- `Debug` - 显示基本信息

---

### StateMachineRuntime
状态机运行时实例，是状态机蓝图的可执行实例

```rust
pub struct StateMachineRuntime {
    blueprint: StateMachineBlueprint,
    state: State,
    zones: Vec<Zone>,
    transitions: Vec<Transition>,
    zone_activations: HashMap<ZoneId, bool>,
}
```

**字段：**
- `blueprint: StateMachineBlueprint` - 蓝图引用
- `state: State` - 当前状态
- `zones: Vec<Zone>` - 运行时区域实例（从蓝图编译）
- `transitions: Vec<Transition>` - 运行时转移实例（从蓝图编译）
- `zone_activations: HashMap<ZoneId, bool>` - 区域激活追踪（zone_id -> active）

**方法：**
- `pub fn new(blueprint: StateMachineBlueprint) -> Self` - 从蓝图创建新的运行时实例，自动编译蓝图为运行时对象
- `pub fn state(&self) -> &State` - 获取当前状态
- `pub fn blueprint(&self) -> &StateMachineBlueprint` - 获取蓝图引用
- `pub fn dispatch(&mut self, event: &EventId) -> bool` - 分发事件到状态机，如果触发转移返回 true，否则返回 false
- `pub fn dispatch_str(&mut self, event: &str) -> bool` - 通过字符串分发事件
- `pub fn active_zones(&self) -> Vec<ZoneId>` - 获取当前活跃的区域 ID 列表
- `pub fn is_zone_active(&self, zone_id: ZoneId) -> bool` - 检查特定区域是否活跃
- `pub fn reset(&mut self)` - 重置状态机到初始状态
- `pub fn with_zone_on_enter<F>(self, zone_id: ZoneId, handler: F) -> Self` - 添加区域进入副作用处理器（builder 模式）
- `pub fn with_zone_on_exit<F>(self, zone_id: ZoneId, handler: F) -> Self` - 添加区域离开副作用处理器（builder 模式）
- `pub fn with_transition_on_tran<F>(self, transition_id: TransitionId, handler: F) -> Self` - 添加转移触发副作用处理器（builder 模式）
- `pub fn with_transition_update(self, transition_id: TransitionId, update: Update) -> Self` - 替换转移的更新操作（builder 模式）

---

## Review 意见

1. **类型擦除限制**：`AspectDescriptor::clone` 和 `create_initial_state` 中的类型匹配链只支持有限类型。**✅ 已解决**：现在使用 `ClonableAny::clone_box()` 方法，支持所有实现了 `ClonableAny` trait 的类型（包括用户自定义类型）。

2. **蓝图-运行时分离**：**✅ 已实现**：`StateMachineBlueprint` 现在存储 `ZoneBlueprint` 和 `TransitionBlueprint`，而非运行时对象。运行时层在实例化时自动编译蓝图为运行时对象。

3. **验证缺失**：`StateMachineBlueprint` 没有验证逻辑，例如：
   - 检查 AspectId 是否唯一
   - 检查 ZoneId 是否唯一
   - 检查 TransitionId 是否唯一
   - 检查转移引用的状态面是否存在
   - 建议添加 `validate()` 方法

4. **构建器返回类型**：`add_aspect`、`add_zone`、`add_transition` 返回 `&mut Self`，支持链式调用。但这是可变借用模式，如果需要不可变的构建器模式，可以考虑使用单独的 `BlueprintBuilder`。

5. **事件收集**：`add_transition` 自动收集事件类型，这是很好的设计。但如果转移被移除，事件类型不会被清理。如果需要动态修改，建议考虑使用计数或其他机制。

6. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

7. **Clone 实现**：`StateMachineBlueprint` 现在可以克隆，因为 `ZoneBlueprint` 和 `TransitionBlueprint` 都是可克隆的。建议实现 `Clone` trait。

8. **序列化支持**：蓝图应该支持序列化（存储/传输），但当前实现使用了 `Box<dyn Any>`，无法直接序列化。建议考虑使用枚举或其他可序列化的表示方式。

9. **性能考虑**：`create_initial_state` 对每个状态面都进行类型匹配和克隆，对于大量状态面可能有性能开销。可以考虑优化或缓存。

10. **事件处理顺序**：当前实现只触发第一个匹配的转移（`break`）。建议在文档中明确说明这个行为，或考虑允许多个转移同时触发。

11. **区域激活更新**：`update_zone_activations` 在状态变更后自动调用，确保区域激活状态与当前状态同步。

12. **副作用执行顺序**：在 `dispatch` 中，先执行 `transition.trigger()`，然后应用状态更新，最后更新区域激活。建议在文档中明确说明这个执行顺序。

13. **错误处理**：当状态更新导致值超出定义的范围约束时，会 panic。建议考虑返回 `Result` 类型，让调用者决定如何处理错误。

14. **并发安全**：当前实现不是线程安全的。如果需要多线程访问，建议添加 `Arc<Mutex<>>` 包装或使用消息传递。

15. **状态查询**：`state()` 方法返回不可变引用，确保外部无法直接修改状态。所有状态修改必须通过 `dispatch` 方法进行。

16. **重置语义**：`reset` 方法重置状态和区域激活，然后初始化区域激活。建议在文档中说明这个两步过程。

17. **API 一致性**：`dispatch` 和 `dispatch_str` 提供了相似的 API，保持了一致性。`dispatch_str` 是一个便捷方法，内部调用 `dispatch`。

18. **区域激活追踪**：使用 `HashMap<ZoneId, bool>` 追踪区域激活状态，支持快速查询。建议在文档中说明这种数据结构的选择。

19. **初始化行为**：`new` 方法创建运行时实例时初始化状态和区域激活，但没有立即调用 `update_zone_activations`。这是合理的设计，因为初始状态应该由蓝图定义。

20. **副作用注入**：**✅ 已实现**：通过 `with_zone_on_enter`、`with_zone_on_exit`、`with_transition_on_tran` 等方法在运行时添加副作用处理器，实现了蓝图层和运行时层的清晰分离。

21. **自定义更新**：**✅ 已实现**：通过 `with_transition_update` 方法可以在运行时替换转移的更新操作，支持复杂的条件更新逻辑。

22. **Builder 模式**：副作用注入方法使用 builder 模式，支持链式调用。这是良好的 API 设计，但需要注意 `self` 是移动语义，不能在同一个引用上多次调用。

23. **蓝图编译**：运行时层在实例化时自动编译蓝图，这是透明的。建议在文档中说明这个过程，以便用户理解性能开销。

24. **区域和转移索引**：运行时层使用 `Vec` 存储区域和转移，通过 ID 查找需要遍历。如果性能成为问题，可以考虑使用 `HashMap`。
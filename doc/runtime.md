# runtime.rs API 文档

## 功能介绍
`runtime.rs` 模块定义了状态机运行时实例（`StateMachineRuntime`），提供状态机的可执行实例，维护当前状态、追踪区域激活状态并提供事件分发功能。

## 功能实现思路
- **蓝图实例化**：从 `StateMachineBlueprint` 创建可执行的运行时实例
- **状态管理**：维护当前状态，支持状态查询和重置
- **事件分发**：通过 `dispatch` 方法分发事件，触发匹配的状态转移
- **区域追踪**：自动追踪和更新区域激活状态，触发进入/离开副作用
- **生命周期管理**：提供 `reset` 方法重置状态机到初始状态

---

## Structs

### StateMachineRuntime
状态机运行时实例，是状态机蓝图的可执行实例

```rust
pub struct StateMachineRuntime {
    blueprint: StateMachineBlueprint,
    state: State,
    zone_activations: HashMap<ZoneId, bool>,
}
```

**字段：**
- `blueprint: StateMachineBlueprint` - 蓝图引用
- `state: State` - 当前状态
- `zone_activations: HashMap<ZoneId, bool>` - 区域激活追踪（zone_id -> active）

**方法：**
- `pub fn new(blueprint: StateMachineBlueprint) -> Self` - 从蓝图创建新的运行时实例
- `pub fn state(&self) -> &State` - 获取当前状态
- `pub fn blueprint(&self) -> &StateMachineBlueprint` - 获取蓝图引用
- `pub fn dispatch(&mut self, event: &EventId) -> bool` - 分发事件到状态机，如果触发转移返回 true，否则返回 false
- `pub fn dispatch_str(&mut self, event: &str) -> bool` - 通过字符串分发事件
- `pub fn active_zones(&self) -> Vec<ZoneId>` - 获取当前活跃的区域 ID 列表
- `pub fn is_zone_active(&self, zone_id: ZoneId) -> bool` - 检查特定区域是否活跃
- `pub fn reset(&mut self)` - 重置状态机到初始状态

**Trait 实现：**
- （无公开 trait 实现）

---

## Type Aliases

（无公开类型别名）

---

## Functions

（无公共函数）

---

## Review 意见

1. **性能考虑**：`dispatch` 方法现在直接修改状态（`transition.apply(&mut self.state)`），避免了克隆开销。这是对性能的重大改进。

2. **事件处理顺序**：当前实现只触发第一个匹配的转移（`break`）。建议在文档中明确说明这个行为，或考虑允许多个转移同时触发。

3. **区域激活更新**：`update_zone_activations` 在状态变更后自动调用，确保区域激活状态与当前状态同步。

4. **副作用执行顺序**：在 `dispatch` 中，先执行 `transition.trigger()`，然后应用状态更新，最后更新区域激活。建议在文档中明确说明这个执行顺序。

5. **错误处理**：当状态更新导致值超出定义的范围约束时，会 panic。建议考虑返回 `Result` 类型，让调用者决定如何处理错误。

6. **并发安全**：当前实现不是线程安全的。如果需要多线程访问，建议添加 `Arc<Mutex<>>` 包装或使用消息传递。

7. **状态查询**：`state()` 方法返回不可变引用，确保外部无法直接修改状态。所有状态修改必须通过 `dispatch` 方法进行。

8. **重置语义**：`reset` 方法重置状态和区域激活，然后初始化区域激活。建议在文档中说明这个两步过程。

9. **文档注释**：部分公开 API 缺少 Rust doc 注释（`///`），建议补充以便生成更好的文档。

10. **API 一致性**：`dispatch` 和 `dispatch_str` 提供了相似的 API，保持了一致性。`dispatch_str` 是一个便捷方法，内部调用 `dispatch`。

11. **区域激活追踪**：使用 `HashMap<ZoneId, bool>` 追踪区域激活状态，支持快速查询。建议在文档中说明这种数据结构的选择。

12. **初始化行为**：`new` 方法创建运行时实例时初始化状态和区域激活，但没有立即调用 `update_zone_activations`。这是合理的设计，因为初始状态应该由蓝图定义。
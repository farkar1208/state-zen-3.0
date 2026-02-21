# runtime.rs API 文档

## 功能介绍
`runtime.rs` 模块定义了状态机运行时实例（StateMachineRuntime），是状态机蓝图的执行实例。它维护当前状态、追踪区域激活状态，并提供事件分发功能。

## 功能实现思路
- **实例化**：从蓝图创建运行时实例，初始化状态和区域激活追踪
- **事件分发**：`dispatch` 查找匹配的转移并执行状态更新
- **区域管理**：自动追踪和更新区域激活状态，触发进入/离开事件
- **重置支持**：支持重置到初始状态
- **单次触发**：每次事件只触发第一个匹配的转移

---

## Structs

### StateMachineRuntime
运行时状态机实例

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
- `zone_activations: HashMap<ZoneId, bool>` - 区域激活状态追踪（zone_id -> active）

**方法：**
- `pub fn new(blueprint: StateMachineBlueprint) -> Self` - 从蓝图创建运行时实例
- `pub fn state(&self) -> &State` - 获取当前状态引用
- `pub fn blueprint(&self) -> &StateMachineBlueprint` - 获取蓝图引用
- `pub fn dispatch(&mut self, event: &EventId) -> bool` - 分发事件，返回是否触发了转移
- `pub fn dispatch_str(&mut self, event: &str) -> bool` - 分发事件（字符串便捷方法）
- `pub fn active_zones(&self) -> Vec<ZoneId>` - 获取当前活跃的区域 ID 列表
- `pub fn is_zone_active(&self, zone_id: ZoneId) -> bool` - 检查特定区域是否活跃
- `pub fn reset(&mut self)` - 重置到初始状态

**内部方法：**
- `fn update_zone_activations(&mut self)` - 更新区域激活状态并触发进入/离开处理器

---

## Review 意见

1. **单次触发策略**：`dispatch` 方法在找到第一个匹配的转移后立即 `break`，只执行一个转移。这是设计决策，但建议在文档中明确说明此行为，并考虑是否支持多转移触发策略。

2. **事件分发顺序**：当前使用 `blueprint.transitions()` 的顺序，没有定义明确的事件分发优先级。如果需要优先级控制，建议添加优先级字段或排序机制。

3. **状态克隆开销**：`dispatch` 中调用 `self.state.clone()` 以避免在转移失败时修改状态，但这有性能开销。可以考虑使用 `Cow` 或其他优化策略。

4. **初始化时的区域激活**：`new` 方法中区域激活初始化为 `false`，然后调用 `reset` 触发 `update_zone_activations`。但实际代码中 `new` 并没有调用 `update_zone_activations`，初始状态下的区域激活状态可能不正确。需要检查初始化逻辑。

5. **错误处理**：当前实现中，如果状态更新失败（如类型不匹配），可能会 panic。建议考虑使用 `Result` 类型返回错误。

6. **文档注释**：`dispatch` 方法的文档注释提到"Panics if a state update results in a value outside the defined range constraints"，但实际代码中没有范围约束验证。需要验证此断言是否正确。

7. **线程安全**：`StateMachineRuntime` 没有实现 `Send` 或 `Sync`，不是线程安全的。如果需要在多线程环境使用，需要添加同步机制（如 `Mutex`）。

8. **并发事件处理**：`dispatch` 是可变方法，同一时间只能处理一个事件。如果需要并发事件处理，需要重新设计。

9. **调试支持**：建议添加日志记录或事件追踪功能，便于调试状态机行为。

10. **Clone 支持**：`StateMachineRuntime` 没有实现 `Clone`，这可能是因为 `State` 中的 `Box<dyn Any>` 克隆限制。如果需要克隆，可以考虑重新设计。
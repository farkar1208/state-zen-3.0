# Bug Report - 2026-02-23
## 任务：将 State 从不可变改为可变以提升性能

---

## 问题 1: update.rs 中仍使用旧的 apply 签名

### 现象
```rust
error[E0308]: mismatched types
   --> src\update.rs:647:38
    |
647 |         let new_state = update.apply(state);
    |                                ----- ^^^^^ expected `&mut State`, found `State`
    |
    |                                arguments to this method are incorrect
```

### 原因
- `Update::apply()` 方法签名已从 `fn apply(&self, state: State) -> State` 改为 `fn apply(&self, state: &mut State)`
- 但 `update.rs` 的 `test_runtime_typed` 测试中仍使用旧的调用方式

### 解决方案
```rust
// 之前
let update = Update::modify_typed(id, |v: i32| v + 10);
let new_state = update.apply(state);
assert_eq!(new_state.get_as::<i32>(id), Some(&52));

// 之后
let update = Update::modify_typed(id, |v: i32| v + 10);
update.apply(&mut state);
assert_eq!(state.get_as::<i32>(id), Some(&52));
```

---

## 问题 2: 编译通过但需要更新所有测试

### 现象
- 编译错误修复后，所有测试都需要更新以适配新的可变 API
- 涉及 `state.rs`、`update.rs`、`transition.rs` 的测试

### 原因
- `State::set()` 和 `State::set_typed()` 改为 `&mut self`
- `Update::apply()` 改为接受 `&mut State`
- `Transition::apply()` 改为接受 `&mut State`
- 所有使用这些方法的测试都需要相应修改

### 解决方案
将所有测试中的 `let new_state = ...apply(state)` 模式改为 `...apply(&mut state)`，并更新断言使用修改后的 state。

---

## 总结

### 核心问题
1. **API 变更**：State、Update、Transition 的方法签名从不可变改为可变
2. **测试更新**：所有使用旧 API 的测试都需要更新

### 解决方案
1. 更新 `Update::apply()` 方法签名为 `fn apply(&self, state: &mut State)`
2. 更新 `Transition::apply()` 方法签名为 `fn apply(&self, state: &mut State)`
3. 更新 `StateMachineRuntime::dispatch()` 方法直接修改状态
4. 更新所有测试以使用新的可变 API

### 测试结果
- **76/76 测试全部通过** ✅
- 示例代码运行正常 ✅

### 文件变更清单
**修改文件：**
- `src/state.rs` - 更新 `set` 和 `set_typed` 方法签名及测试
- `src/update.rs` - 更新 `apply` 方法签名及测试
- `src/transition.rs` - 更新 `apply` 方法签名及测试
- `src/runtime.rs` - 更新 `dispatch` 方法实现
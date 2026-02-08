## 📘 一、**state-zen 核心术语词典**

| 术语 | 类型 | 含义 |
|------|------|------|
| **`StateAspect`** | 概念 / 类型单元 | 状态的一个正交维度（关注面）。每个 `StateAspect` 有独立的取值类型（如枚举、整数等）。完整状态由多个 `StateAspect` 组成的高维向量表示。 |
| **`activeIn`** | 谓词函数（`State → boolean`） | 描述一个行为（`Zone` 或 `Transition`）在哪些状态下被激活。它替代了传统状态机中的“源状态”，通过布尔条件隐式定义状态集合，而非显式枚举。 |
| **`Zone`** | 行为容器 | 一个状态行为区域，包含：<br>• `activeIn`：定义该区域覆盖的状态集合<br>• `on_enter`：当状态进入该集合时触发的副作用<br>• `on_exit`：当状态离开该集合时触发的副作用 |
| **`update`** | 状态变更描述 | 在 `Transition` 中定义状态如何演化的纯函数或声明式结构。它接收当前状态，返回新状态（通常为不可变更新），不包含副作用。 |

---

## 📄 二、**state-zen 模型规范文档（使用新术语）**

# state-zen：高维并行状态机模型

state-zen 是一种面向组合性与表达力的状态机模型，适用于复杂系统中多维度状态协同演化的场景。其核心思想是将状态建模为高维向量，并通过谓词驱动的行为绑定实现灵活控制。

---

### 1. 状态表示：`StateAspect`

- 系统状态不是单一标量，而是由若干 **`StateAspect`** 组成的向量。
- 每个 `StateAspect` 代表状态的一个正交维度，具有独立的类型和语义（例如：`mode: enum{Idle, Running}`、`batteryLevel: u8`、`networkStatus: bool`）。
- 状态整体是不可变的；任何变化都通过生成新状态实现。

> ✦ 示例：  
> ```ts
> // State = { mode, battery, isCharging }
> // 其中 mode, battery, isCharging 均为 StateAspect
> ```

---

### 2. 行为激活机制：`activeIn`

- 所有行为（无论是区域还是转移）都通过 **`activeIn`** 谓词决定其适用范围。
- `activeIn: (state: State) → boolean` 是一个纯函数，用于判断当前状态是否“激活”该行为。
- 它取代了传统状态机中对具体源状态的硬编码，支持动态、组合式的状态匹配。

> ✦ 优势：  
> - 支持跨 `StateAspect` 的复合条件（如 `mode === 'Running' && battery < 10`）  
> - 避免状态爆炸（无需为每个状态组合显式定义节点）

---

### 3. 状态区域：`Zone`

- 一个 **`Zone`** 表示状态空间中的一个逻辑区域，具有生命周期语义。
- 结构：
  ```ts
  interface Zone {
    activeIn: (state: State) => boolean;
    on_enter?: () => void;  // 进入该区域时执行
    on_exit?: () => void;   // 离开该区域时执行
  }
  ```
- `Zone` 用于封装与特定状态上下文相关的副作用（如 UI 更新、资源启停）。
- 多个 `Zone` 可重叠；系统可同时处于多个 `Zone` 中。

> ✦ 示例：  
> ```ts
> const chargingZone = {
>   activeIn: s => s.isCharging,
>   on_enter: () => startChargingAnimation(),
>   on_exit: () => stopChargingAnimation()
> };
> ```

---

### 4. 状态转移：`Transition`

- **`Transition`** 描述系统如何响应事件并演化状态。
- 结构：
  ```ts
  interface Transition {
    activeIn: (state: State) => boolean;     // 何时监听此事件
    event: EventType;                        // 监听的事件类型
    update: (state: State) => State;         // 如何计算新状态（纯函数）
    on_tran?: () => void;                    // 转移发生时的副作用
  }
  ```
- 只有当 `activeIn(state) === true` 时，该 `Transition` 才会监听对应事件。
- `update` 必须是无副作用的；所有副作用应放在 `on_tran` 中。

> ✦ 示例：  
> ```ts
> const startTransition = {
>   activeIn: s => s.mode === 'Idle',
>   event: 'StartButtonPressed',
>   update: s => ({ ...s, mode: 'Running', startTime: Date.now() }),
>   on_tran: () => playSound('start')
> };
> ```

---

### 5. 整体执行模型

1. 系统维护一个当前 **状态向量**（由 `StateAspect` 构成）。
2. 所有 `Zone` 持续评估其 `activeIn`：
   - 若从 `false` → `true`，触发 `on_enter`
   - 若从 `true` → `false`，触发 `on_exit`
3. 所有 `Transition` 在 `activeIn` 为真时监听对应事件。
4. 当事件发生且匹配某个 `Transition`：
   - 调用其 `update` 生成新状态
   - 触发 `on_tran`
   - 用新状态替换当前状态，并重新评估所有 `Zone` 和 `Transition`

---

> ✅ **设计哲学**：  
> state-zen 强调 **分离关注点**（每个 `StateAspect` 独立）、**声明式行为绑定**（通过 `activeIn`）、**纯状态演化**（`update` 无副作用），从而实现高可组合性与可维护性。
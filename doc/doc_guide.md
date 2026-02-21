# state-zen 文档编写指南

本文档说明如何为 state-zen 项目编写和维护 API 文档。

---

## 文档格式规范

### 基本结构

每个模块的 API 文档应遵循以下结构：

```
# <module_name>.rs API 文档

## 功能介绍
简短描述模块的功能和用途。

## 功能实现思路
说明模块的核心设计理念和实现方式，包括：
- 蓝图层与运行时层的区分（如适用）
- 主要的设计模式
- 关键的技术选型

---

## Enums
（如果有枚举类型）

### EnumName
枚举的简要描述

```rust
#[derive(Debug, Clone)]
pub enum EnumName {
    // 变体列表
}
```

**变体：**
- `Variant` - 变体描述

**方法：**
- `pub fn method_name(args) -> ReturnType` - 方法描述

**Trait 实现：**
- `TraitName` - 描述该 trait 的实现方式

---

## Structs
（如果有结构体类型）

### StructName
结构体的简要描述

```rust
#[derive(Debug, Clone)]
pub struct StructName {
    // 字段列表
}
```

**字段：**
- `pub field_name: Type` - 字段描述

**方法：**
- `pub fn method_name(args) -> ReturnType` - 方法描述

**Trait 实现：**
- `TraitName` - 描述该 trait 的实现方式

---

## Type Aliases
（如果有类型别名）

### TypeName
类型别名的简要描述

```rust
pub type TypeName = ActualType;
```

---

## Functions
（如果有独立函数）

### `pub fn function_name(args) -> ReturnType`
函数的简要描述

---

## Review 意见
列出对当前代码的改进建议和潜在问题。
```

---

## 编写规则

### 1. 代码块格式

所有代码块使用 rust 语法高亮：

```rust
#[derive(Debug, Clone)]
pub struct Example {
    pub field: String,
}
```

### 2. 方法描述格式

- **公开方法**：必须包含完整的签名和描述
- **私有方法**：不需要记录在文档中
- **内部函数**：使用 `pub(crate)` 标记

格式：
```
- `pub fn method_name(args) -> ReturnType` - 方法描述
```

示例：
```
- `pub fn new(id: AspectId, name: impl Into<String>, active_in: ActiveIn) -> Self` - 创建新的区域
```

### 3. 字段描述格式

只记录公开字段：

```
**字段：**
- `pub field_name: Type` - 字段描述
```

### 4. Trait 实现

记录所有公开的 trait 实现：

```
**Trait 实现：**
- `Debug` - 显示基本信息
- `Clone` - 支持克隆
- `PartialEq` - 基于 id 判断相等
```

### 5. Review 意见

列出代码中需要注意的问题或改进建议：

```
## Review 意见

1. **问题类别**：具体问题描述
2. **问题类别**：具体问题描述
```

常见的 Review 意见类别：
- **性能考虑**：关于性能的讨论
- **错误处理**：关于错误处理的建议
- **类型安全**：关于类型安全的讨论
- **文档注释**：关于 Rust doc 注释的建议
- **命名**：关于命名的建议
- **API 一致性**：关于 API 设计一致性的讨论

---

## 命名规范

### 文件命名

文档文件名与源文件名一致，使用小写和下划线：

```
源文件: active_in.rs
文档文件: active_in.md
```

### 标题规范

- 一级标题 `#`：文件标题
- 二级标题 `##`：主要章节（功能介绍、功能实现思路、类型定义等）
- 三级标题 `###`：类型名称（EnumName、StructName 等）

### 描述规范

- **功能介绍**：1-2 句话概括模块功能
- **功能实现思路**：使用项目符号列出关键点
- **变体/字段/方法描述**：简洁明了，避免冗余

---

## 内容要求

### 必须包含的内容

1. **功能介绍**：简短描述模块做什么
2. **功能实现思路**：说明设计理念
3. **所有公开类型**：Enums、Structs、Type Aliases、Functions
4. **类型签名**：使用代码块展示
5. **字段和方法列表**：所有公开成员
6. **Review 意见**：至少列出 3-5 个改进建议

### 不需要包含的内容

1. **私有字段和方法**：不在文档中记录
2. **测试代码**：测试不放入 API 文档
3. **使用示例**：示例代码应在单独的示例文件中
4. **内部实现细节**：除非对理解 API 有帮助

---

## 示例参考

参考以下已完成的文档作为标准：

- `doc/zone.md` - 区域模块文档
- `doc/transition.rs` - 转移模块文档
- `doc/update.rs` - 更新模块文档
- `doc/active_in.md` - 激活条件模块文档（最新）

---

## 文档更新流程

1. **代码变更时**：修改公开 API 后，同步更新对应文档
2. **新增模块时**：创建新模块时，同步创建对应文档
3. **重构时**：重构 API 时，检查并更新所有相关文档
4. **定期审查**：定期检查文档与代码的一致性

---

## 常见问题

### Q: 如何处理复杂的泛型约束？

A: 完整保留泛型约束，帮助用户理解类型要求：

```
- `pub fn aspect_lt_typed<T>(aspect_id: AspectId, value: T) -> ActiveIn where T: std::cmp::PartialOrd + Send + Sync + 'static` - 泛型小于
```

### Q: 如何描述 trait 实现？

A: 简要说明该 trait 的实现方式和特殊行为：

```
**Trait 实现：**
- `PartialEq` - 仅基于 `id` 判断相等，忽略其他字段
- `Debug` - 显示基本信息，不显示闭包内容
```

### Q: Review 意见应该写什么？

A: 记录对代码的改进建议，包括：
- 设计问题
- 性能考虑
- API 一致性问题
- 错误处理建议
- 文档注释缺失

### Q: 如何处理蓝图层和运行时层的区分？

A: 在"功能实现思路"中明确说明，并在类型描述中标注：

```
- **蓝图层**：`ActiveInBlueprint` 定义声明式结构
- **运行时层**：`ActiveIn` 封装闭包进行实际评估
```

---

## 质量检查清单

在提交文档前，检查以下项目：

- [ ] 文件名与源文件名一致
- [ ] 包含功能介绍和功能实现思路
- [ ] 所有公开类型都已记录
- [ ] 所有公开字段和方法都已列出
- [ ] 代码块使用 rust 语法高亮
- [ ] 方法签名完整准确
- [ ] Review 意见至少包含 3 条
- [ ] 格式与现有文档保持一致
- [ ] 没有包含私有成员
- [ ] 没有包含使用示例代码

---

## 版本历史

- 2026-02-21: 初始版本，建立文档编写规范
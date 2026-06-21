# design.md "L1~L5 自动化测试设计" 章节模板

> **强制章节**（[AGENTS.md §7](../../AGENTS.md#7-feature-自动化测试设计强制规范) 条款 1）。本章在每个 Feature 的 `design.md` 中编码启动前完成，覆盖本 Feature 全部 FR。
>
> **引用**：
> - [`docs/test-level-matrix.md`](../test-level-matrix.md)（等级分工矩阵）
> - [`docs/principles/testing-principles.md`](./testing-principles.md) §8（强制规范详细条款 + 等级原则）
>
> **来源**：F115（2026-06-18 立项）；首案例自验证见 [`features/115-ux-e2e-infrastructure/design.md`](../../features/115-ux-e2e-infrastructure/design.md) §N。

---

## 阈值与简化规则

| 触发条件 | 模板版本 | 必填子节 | 可选子节 |
|---------|---------|---------|---------|
| FR 总数 > 5 | **完整版**（下方结构） | N.1 / N.2.1~N.2.5 / N.5 | N.3 / N.4 |
| FR 总数 ≤ 5 | **简化版**（单表） | 单表（含 FR ID / 能力 / L1~L5 / 关联 FR） | N.3 / N.4 / N.5 |

> 简化版阈值依据 spec OQ-9 决策：FR ≤ 5 时单表足以承载测试设计意图，无需展开 5 层子节。

---

## 完整版结构（FR > 5）

### N.1 测试等级矩阵填充

- 列出本 Feature 在 [`docs/test-level-matrix.md`](../test-level-matrix.md) 中新增的行（至少含 FR ID + L1~L5 等级标注）
- 引用 spec FR-2.3.1-R4 的 8 条等级决策原则（见 test-level-matrix.md §1）

### N.2 测试用例设计（逐层）

#### N.2.1 L1（Rust / TS 单元测试）

| 测试函数 | 模块 | 断言 | 覆盖率目标 |
|---------|------|------|----------|
| ... | ... | ... | ... |

#### N.2.2 L2（FR 验收测试）

| 测试函数 | 文件路径 | 可观测结果 | 关联 FR |
|---------|---------|----------|--------|
| ... | ... | ... | ... |

#### N.2.3 L3（契约 / 管道集成测试）

| 测试函数 | 类型（契约/管道）| 关键断言 | 关联 FR |
|---------|----------------|---------|--------|
| ... | ... | ... | ... |

#### N.2.4 L4（vitest + RTL，UX 用例写法）

| spec 文件 | 操作序号 | 操作描述 | 期望结果 | 关联 FR |
|---------|---------|---------|---------|--------|
| ... | 1 | ...（用户动作） | ...（可观测结果） | ... |

> **UX 写法约束（? id:06 处理结果，2026-06-19）**：L4/L5 涉及前端用户交互的用例**必须**使用通用 UX 测试用例写法（操作序号 + 操作描述 + 期望结果），不再使用抽象的 "describe/it + 渲染场景" 描述。操作序号 / 操作描述 / 期望结果为必填列。

#### N.2.5 L5（e2e spec，UX 用例写法）

| spec 文件 | 操作序号 | 操作描述 | 期望结果 | 关联 FR |
|---------|---------|---------|---------|--------|
| e2e/specs/\<feature-id\>/... | 1 | ...（用户动作） | ...（可观测结果） | ... |

> 跨 spec 性质（如 "日志仅一次 newSession"）或 grep 类断言不在 spec 内执行，由 finalize 阶段度量脚本（如 `benchmark.sh`）验证。

### N.3 测试数据（可选）

- 共享 fixtures（`e2e/fixtures/`）vs Feature 私有 fixtures（`features/<NNN>/fixtures/`）
- 测试用 site_id / mock 数据清单

### N.4 测试脚本入口（可选）

- 单 Feature 全量测试：`pnpm test:feature -- <id>`
- 单 Feature e2e：`pnpm test:e2e:feature -- <id>`
- 全套：`pnpm test:all`

### N.5 TDD 执行顺序（必填）

- 列出 RED → GREEN → REFACTOR 的实施顺序（按 FR 优先级）
- 每个 task 标注关联 FR 与所属测试等级，便于 finalize 阶段对照

---

## 简化版结构（FR ≤ 5）

### N. L1~L5 自动化测试设计（简化版，FR ≤ 5）

| FR ID | 能力 | L1 | L2 | L3 | L4 | L5 | 关联 FR | 依据 |
|-------|------|----|----|----|----|----|--------|------|
| ... | ... | 函数名 / N/A | 函数名 / N/A | 函数名 / N/A | spec:it / N/A | spec:it / N/A | ... | ... |

附（可选）：测试数据 / 脚本入口 / TDD 顺序（按需附加）

---

## 使用约束

1. **章节位置**：design.md 中作为独立顶层章节 `## N. L1~L5 自动化测试设计`（N 为该 design.md 中本章的实际编号）。
2. **必填 vs 可选**：完整版 N.1 / N.2.1~N.2.5 / N.5 必填；N.3 / N.4 当 Feature 有特殊测试数据或脚本入口时填写，否则可省略（省略时注明"无特殊要求"）。
3. **不可跳层**：TDD 执行时不允许跳过 L1~L5 任意层（testing-principles.md §8.2 HF 检查点表）。
4. **finalize 校验**：本 Feature 行内 `<TBD` 计数 = 0；声明测试均已实现且通过（testing-principles.md §8.2 `hf-finalize` 行）。

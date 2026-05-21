# Spec-Design-Implementation Drift 根因分析与 HF 框架修补方案

- **Date**: 2026-05-21
- **Trigger**: F002 Coordinated Mode 契约断裂（spec+design+ADR 承诺双侧，实现只做了单侧）
- **Type**: 方法论复盘 → HF 框架改进

## 根因分析

### 直接原因

`deployment_manager.rs` 使用 `#[cfg(target_os)]` 条件编译替代了 ADR-0005 承诺的运行时动态选择。这是**实现阶段的技术决策偏离了设计阶段的架构决策**。

### 为什么审查链没有捕获？

HF 框架当前有 6 道审查门：

| 门 | 职责 | 此案例中未捕获的原因 |
|----|------|-------------------|
| hf-spec-review | 需求完整性 | ✅ 正确通过——spec 本身描述完整 |
| hf-design-review | 设计覆盖需求 | ✅ 正确通过——design 正确映射了 spec |
| hf-tasks-review | 任务计划合理性 | ⚠️ 未验证——任务粒度是"实现 X 适配器"，未要求验证"协同模式是否真正创建两个适配器" |
| hf-test-driven-dev | 代码实现 | ❌ 缺失——**无 ADR conformance 检查** |
| hf-finalize | 质量门 | ❌ 缺失——**无 spec↔code 追溯验证** |
| hf-code-review | 代码审查 | ❌ 未执行——F002 未走独立 code-review 节点 |

### 本质：HF 框架缺少的两道防线

**防线 1：hf-tasks-review 缺少"可验证性"检查**

当前 tasks-review 检查维度是：完整性、依赖关系、粒度、估算等。但**不检查任务的验收标准是否能追溯到 spec/ADR 的具体承诺**。

如果 tasks-review 要求 "T5.1 DeploymentManager 的验收标准必须包含：Coordinated 模式下同时返回 WindowsAdapter 和 WslAdapter"，这个断层在任务计划阶段就会被发现。

**防线 2：hf-test-driven-dev 缺少 "ADR conformance check"**

`architectural-health-during-tdd.md` 定义了 conformance check：

> 实现是否仍遵循 hf-design 中的依赖方向、模块边界、接口契约

但这条纪律**没有被编码为可执行的检查项**——它是文档中的原则，不是 gate 中的 hard check。实现者（Agent）在 TDD 循环中不会主动对照 ADR 验证自己的技术选型。

**防线 3：hf-finalize 缺少 "spec↔code 追溯"**

hf-finalize 检查 evidence matrix（review 记录、测试结果、clippy），但**不检查 spec 的成功标准是否被测试覆盖**。

如果 hf-finalize 要求 "SC-2（协同部署下 Windows 与 WSL 代理状态一致）必须有对应的测试证据"，这个断层在关闭前就会被发现。

## 框架修补方案

### 方案评估

| 方案 | 描述 | 优势 | 劣势 | 结论 |
|------|------|------|------|------|
| A. 仅记录到 memory | 记录经验教训 | 快速 | 不可执行、不可验证、下次 Agent 不一定读 memory | ❌ 不够 |
| B. 新增 HF 节点 | 增加 `hf-adr-conformance` 节点 | 独立检查 | 流程变重、与现有 review 重叠 | ❌ 过重 |
| **C. 强化现有 3 个节点** | 在 tasks-review、test-driven-dev、finalize 中增加检查项 | 最小改动、嵌入现有流程 | 需要更新原则文档 | ✅ 选择 |

### 方案 C：三节点强化

#### 强化 1：hf-tasks-review 增加 "Traceability to Spec/ADR" 检查维度

在 tasks-review 的检查清单中新增：

> **T-TRACE**: 每个任务的验收标准是否可追溯到 spec 的 FR/SC 和 ADR 的具体承诺？
> 对于 spec 中标记为关键路径的需求（如协同模式、跨平台），验收标准必须包含显式的验证条件。

**落地方式**：更新 `docs/principles/hf-sdd-tdd-skill-design.md` 的 tasks-review 检查维度。

#### 强化 2：hf-test-driven-dev 的 REFACTOR 步增加 "ADR Conformance Hard Gate"

将 `architectural-health-during-tdd.md` 中已有的 conformance check 从"原则"升级为"hard gate"：

> **ADR-CONFORMANCE (Hard Gate)**: 实现**不得**使用被 ADR 排除的技术方案。如果发现 ADR 排除的方案比选定方案更简单，**必须先 escalate 回 hf-design 修改 ADR**，然后才能使用。

**落地方式**：更新 `docs/principles/architectural-health-during-tdd.md` 的 Escalation Boundary 表格。

#### 强化 3：hf-finalize 增加 "Spec Success Criteria Coverage" 检查项

在 hf-finalize 的 evidence matrix 中新增：

> **SC-COVERAGE**: 每个 spec Success Criteria (SC-N) 是否有对应的测试证据？列出 SC → 测试/命令 的映射表。如果 SC-N 没有测试证据，**不得关闭**。

**落地方式**：更新 hf-finalize 质量门检查清单。

### 不做的事

- 不修改 spec/design/closeout 的工件格式（保持向后兼容）
- 不新增独立节点（保持流程轻量）
- 不要求 Agent 在每次 RED/GREEN 前读 ADR（太重，只要求在 REFACTOR 步检查）

## 实施步骤

1. 将本分析记录到 `docs/insights/`
2. 更新 `docs/principles/hf-sdd-tdd-skill-design.md`：tasks-review 增加 T-TRACE 维度
3. 更新 `docs/principles/architectural-health-during-tdd.md`：REFACTOR 步增加 ADR-CONFORMANCE hard gate
4. 更新 `docs/principles/sdd-artifact-layout.md`（如需）：hf-finalize evidence matrix 增加 SC-COVERAGE
5. 将经验教训写入项目 memory
6. 提交为 hf-framework-improvement commit

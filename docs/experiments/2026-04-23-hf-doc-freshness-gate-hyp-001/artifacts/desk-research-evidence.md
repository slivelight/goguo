# Desk research evidence for HYP-001 (A1)

按 probe-plan §5 五维度逐项收集证据。引用格式：`<file>:<line> "<quoted text>"`。

## E1. 角色分离纪律

> *"`entry`、`route`、`authoring`、`implementation` 的职责不混。叶子 skill 不会顺手接管 orchestrator 逻辑。review 和 gate 能保持独立性。"*
> — `docs/principles/hf-sdd-tdd-skill-design.md` §"不做'一个总 skill'，而做 family" (line 109-111)

> *"Separation of Author/Reviewer Roles | 不自审自交 | 单人合并作者 + 评审"*
> — `docs/principles/methodology-coherence.md` §评审层 (line 90)

> *"`hf-finalize`：消费该 [completion gate] 结论并做状态 / 文档 / closeout 收口"*
> — `skills/hf-finalize/SKILL.md` §"和其他 Skill 的区别" (line 225)

**判定**:

- **A1（新独立 gate）**：把"判断对外可见文档是否需要刷新 + 给 verdict"作为独立 gate 节点，与既有 `hf-regression-gate` / `hf-completion-gate` 同 tier；author（实现节点）/ reviewer（review 节点）/ gate（本节点）/ closeout（finalize）四角色清晰。**严格遵守**角色分离。
- **A2（扩 finalize + 加合同到 task closeout）**：让 `hf-finalize`（角色 = closeout 收口）同时承担"判断文档漂移并给 verdict"（角色 = gate）。**违反**「`hf-finalize` 消费 gate 结论」的既有定位（line 225）。
- **A3（嵌入 review）**：让 `hf-code-review` / `hf-traceability-review`（角色 = reviewer）同时承担 gate 职责。**违反** Fagan 角色分离纪律（line 90）与 HF "review 节点不替 gate 下结论"的纪律。

→ **A1 命中（+1）**

## E2. 既有"不允许替代"规则模式

> *"`hf-bug-patterns`（mandatory gate） | `hf-bug-patterns`（可选经验沉淀） | `hf-bug-patterns` 只是**可选旁路**，不得写成 `hf-test-review` 前的 mandatory gate"*
> — `docs/principles/methodology-coherence.md` §二 不允许替代清单 (line 137)

> *"`hf-experiment` | `hf-test-driven-dev` | experiment 是在 spec 之前或之中的**假设**验证；TDD 是在已批准 spec / design 之后的**实现**验证；**互相不得替代**"*
> — 同上 (line 136)

> *"Fagan inspection full protocol | HF review skills | HF 只取'独立 reviewer'纪律，不搬 Fagan 完整流程"*
> — 同上 (line 138)

**模式归纳**：HF 既有"不允许替代"清单的核心模式之一是 **"用既有节点的次要职责取代独立节点的主要职责"被显式禁止**（如 `hf-bug-patterns` 不得被升级为 `hf-test-review` 前置 mandatory gate；`hf-test-driven-dev` 不得替代 `hf-experiment`）。

**判定**:

- **A2（扩 finalize 取代独立 gate）**：把 `hf-finalize` 的"sync-on-presence 同步"次要职责（其主要职责是 closeout）扩展成"判断对外可见文档漂移"的强 mandatory gate 角色，**命中**既有禁止模式的同构变种。
- **A3（嵌入 review 取代独立 gate）**：把 reviewer 节点的次要 checklist 升级为 gate verdict，**命中**既有禁止模式的同构变种。
- **A1**：建立独立节点，**不触发**既有禁止模式。

→ **A1 命中（+1）**

## E3. Gate 节点形态可比性（与既有 gate 三段合同对齐）

既有 `hf-regression-gate` / `hf-completion-gate` 都遵循 **"Hard Gates + Verification + fresh evidence"** 三段合同（参见 `skills/hf-regression-gate/SKILL.md`、`skills/hf-completion-gate/SKILL.md`）。

> *"Evidence Bundle Pattern | reviews + gates + 交接块 必须齐全 | 单点证据"*
> — `docs/principles/methodology-coherence.md` §验证 / 门禁层 (line 99)

> *"finalize 只能消费已落盘的 completion / regression / release artifacts，不能把对话记忆当成 closeout evidence"*
> — `skills/docs/hf-workflow-shared-conventions.md` §长期资产同步规则 (line 221)

**判定**:

- **A1**：可直接套既有 gate 三段合同（Hard Gates: "未给出 verdict 不得 closeout"；Verification: 文档载体清单逐项；Fresh Evidence: `features/<active>/verification/doc-freshness-YYYY-MM-DD.md` + `features/<active>/evidence/doc-freshness-diff-*.log`）。形态完全对齐。
- **A2**：`hf-finalize` 自身的合同是 "Project Closeout (PMBOK) + Release Readiness Review + Handoff Pack Pattern"，**不是** gate 形态；强行加 gate 合同会让 `hf-finalize` 同时承担两套合同形态，破坏一致性。
- **A3**：`hf-code-review` / `hf-traceability-review` 是 review 形态（structured walkthrough + checklist + verdict），与 gate 形态不同；嵌入会让 review 节点同时承担两套合同形态。

→ **A1 命中（+1）**

## E4. Profile 分级机制可继承

> *"Profile-Aware Rigor | full / standard / lightweight 不同证据量 | 代替质量纪律"*
> — `docs/principles/methodology-coherence.md` §验证 / 门禁层 (line 100)

> *"Escalation Pattern | profile 渐进升级（lightweight → standard → full） | 降级回退"*
> — 同上 §路由层 (line 80)

> *"`hf-finalize` 同步范围按当前 `docs/` 实际存在的子目录决定，不要求未启用的资产被新建" / "未启用的可选资产不构成 `blocked` 依据"*
> — `skills/hf-finalize/SKILL.md` §4 (line 112-137)

**判定**:

- **A1**：可继承既有的 sync-on-presence + profile 分级两条纪律——`lightweight` 至少强制仓库根 README + Conventional Commits `docs:` 标记自检；`standard` 加 README + 公共 API + 已存在 i18n 副本；`full` 全表。**完全可继承**。
- **A2**：`hf-finalize` 现行合同的"必须同步项"已经按 tier 分层（档 0/1 必须 vs 档 2 按需）；如果再加"`task closeout` 也强制刷 README"，意味着 lightweight 项目的每个 task closeout 都被强制走文档同步，**直接破坏 `lightweight` 的密度承诺**（lightweight 的本意是 "task closeout 走最小路径"）。
- **A3**：嵌入 review 后，`hf-code-review` / `hf-traceability-review` 在 lightweight profile 下原本就允许压缩，加入 docs drift checklist 会让"压缩规则"变得复杂（哪些必查、哪些可省），破坏既有的 profile 分级清晰度。

→ **A1 命中（+1）**

## E5. Task closeout 既有合同不被破坏

> *"`task closeout` 不要求额外人工确认；它只是把当前任务收口后交回 router。"*
> — `skills/hf-finalize/SKILL.md` §3A (line 102)

> *"task closeout：`Next Action Or Recommended Skill` 必须是 `hf-workflow-router`"*
> — 同上 §"Output Contract" (line 207)

`task closeout` 的既有合同语义是 **"轻量、不阻塞、把状态交回 router"**。

**判定**:

- **A1**：在 router transition 中加一个 `hf-doc-freshness-gate` 节点（位于 `hf-traceability-review` 之后、`hf-regression-gate` / `hf-completion-gate` 同 tier），**完全不动 `hf-finalize` 与 `task closeout` 的既有合同**。
- **A2**：直接修改 `hf-finalize` 的 §3A "task closeout 不要求额外人工确认" 与 §4 "必须同步项"，把强制 docs sync 加到 task closeout 路径——**直接破坏** task closeout 既有"轻量"承诺。
- **A3**：不直接动 `hf-finalize`，但 review 节点本身的轻量性会被 docs drift checklist 拖重；间接影响。

→ **A1 命中（+1）**

## 命中汇总

| 维度 | A1 命中 | A2 命中 | A3 命中 |
|---|:-:|:-:|:-:|
| E1 角色分离纪律 | ✅ | ❌ | ❌ |
| E2 既有"不允许替代"规则模式 | ✅ | ❌ | ❌ |
| E3 Gate 节点形态可比性 | ✅ | ❌ | ❌ |
| E4 Profile 分级机制可继承 | ✅ | ❌ | ❌ |
| E5 Task closeout 既有合同不被破坏 | ✅ | ❌ | ⚠ 间接 |
| **合计** | **5 / 5** | **0 / 5** | **0 / 5** |

**门槛对照**：probe-plan §5 阈值 = ≥ 3 / 5 → **5 / 5 大幅超过门槛 → Pass**。

## 反向证据扫描

按 probe-plan §6 Failure Threshold 要求："出现任一 HF 既有规则**反对** A1 的硬证据视为 Fail"。

扫描了以下文件：

- `docs/principles/methodology-coherence.md` 全文（§二 "不允许替代清单" 12 条）
- `docs/principles/hf-sdd-tdd-skill-design.md` §"HF 的关键设计判断"（4 条）
- `docs/principles/sdd-artifact-layout.md` 全文
- `skills/hf-workflow-router/SKILL.md`（路由治理）
- `skills/docs/hf-workflow-shared-conventions.md`（共享约定）

**未发现**任何条文要求 "新增 gate 必须先经 finalize 扩展验证" 或 "禁止新增独立 gate" 的硬证据。

唯一接近的相邻规则是 "Phase 0 不引入完整威胁建模流程，留 Phase 2"（`methodology-coherence.md` line 51）——这是**节奏控制**而非反对独立节点本身；当前主题 wedge 与 Phase 1 `hf-release` 同节奏（与 Conventional Commits 的 `docs:` 标记天然耦合），与该节奏控制规则不冲突。

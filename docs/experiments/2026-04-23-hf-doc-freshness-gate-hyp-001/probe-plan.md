# HYP-001 (A1) Probe Plan — User preference: new gate vs extend `hf-finalize`

- 状态: 已结束（与 probe-result.md 同步发布）
- 主题: 在 HF 主链上引入"对外可见文档刷新合同"时，**用户偏好**新增独立 gate（`hf-doc-freshness-gate`）还是扩展 `hf-finalize` 同步范围 + 把合同加到 `task closeout`
- 上游锚点:
  - Discovery: `docs/insights/2026-04-23-hf-doc-freshness-gate-discovery.md` §6 A1 / §11 OST Solution A1 vs A2
  - Spec: 尚未起草，本 probe 在 spec 起草前完成

## 1. 本轮 probe 覆盖的假设

| ID | Restatement | Type | Blocking? | Confidence (probe 前) |
|---|---|---|---|---|
| HYP-001 (= discovery §6 A1) | HF 用户在 "task / 增量级文档刷新合同应该长在哪里" 这个问题上，**偏好**新增独立 `hf-doc-freshness-gate`（与既有 `hf-regression-gate` / `hf-completion-gate` 同 tier），而**不**偏好把合同硬塞回 `hf-finalize`（扩 finalize 同步范围 + 把强制同步加到 `task closeout`） | Desirability | 是（spec 通过评审前必须关闭） | medium |

本轮 probe **不覆盖**的假设（显式写出）：

- HYP-002 (= discovery §6 U2 责任边界稳定) → 留给 spec 阶段通过显式责任矩阵 + reviewer 判定关闭，不走 `hf-experiment`
- HYP-003 (= discovery §6 A1-V router FSM 复杂度) → 已在 review 中降级为次要假设，留给 design 阶段 transition map dry-run
- HYP-004 (= discovery §6 U1 lightweight 不退化) → P1，留给 design 阶段 dry-run

## 2. 验证方式

- **Method**: **Desk research**（对项目自身文档与既有方法论合同进行证据汇总）
- 为什么选这种方式：
  - 这是 HF skill pack 自身的演进决策，不是面向终端用户产品功能；首要"用户"是**HF 自身的设计纪律**与已经写下的方法论合同
  - HF 已有 `docs/principles/methodology-coherence.md` 与 `docs/principles/hf-sdd-tdd-skill-design.md` 两份**项目级治理文档**，对"author / gate / reviewer 角色分离"、"不允许替代"、"什么属于 mandatory gate vs 可选旁路" 有显式硬规则；HYP-001 的本质是判断这条偏好与既有规则是否一致
  - 访谈 / 问卷在当前 cloud agent 环境下不可执行（无真实用户接入通道），且会引入比 desk research 更弱的证据等级（"我喜欢 / 我觉得"），不如直接以**项目治理规则**作为权威证据
  - Lowest-cost first 原则：`hf-experiment` Step 3 推荐方法表把 desk research 列在最前

## 3. 样本 / 参与者 / 数据范围

- 数量: 项目级治理文档与既有 skill SKILL.md 共 6 份核心文件
- 来源:
  - `docs/principles/methodology-coherence.md`（治理文档：分工地图 + 不允许替代清单）
  - `docs/principles/hf-sdd-tdd-skill-design.md`（顶层设计原则）
  - `docs/principles/sdd-artifact-layout.md`（双根目录 + Promotion Rules + Skill 容错语义）
  - `skills/hf-finalize/SKILL.md`（被对照的"扩展候选"）
  - `skills/hf-completion-gate/SKILL.md` + `skills/hf-regression-gate/SKILL.md`（被对照的"同 tier 既有 gate"模板）
  - `skills/hf-code-review/SKILL.md` + `skills/hf-traceability-review/SKILL.md`（被对照的"嵌入 review" 备选 A3 / C3）
- 招募 / 筛选规则: 不适用（desk research）

## 4. Setup

- 需要准备什么: 无；只读已存在的仓库文档
- 是否涉及生产代码 / 产品 UI: 否
- 一次性原型: 否

## 5. Success Threshold（事先写死）

判定准则：HYP-001 视为 Pass 当且仅当**至少 3 条独立证据**指向"新独立 gate (Solution A1)"在 HF 现行治理框架下比"扩 finalize (Solution A2)"或"嵌 review (Solution A3)"**更一致**。具体阈值：

| 证据维度 | 通过阈值 |
|---|---|
| **角色分离纪律**（author / gate / reviewer 三者不交叉） | A1 严格遵守 ↔ A2 / A3 至少一处违反；命中即 +1 |
| **既有"不允许替代"规则模式**（`methodology-coherence.md` §二） | A2 / A3 命中既有"扩展现有节点取代独立节点"被显式禁止的同构模式；命中即 +1 |
| **gate 节点形态可比性**（与既有 `hf-regression-gate` / `hf-completion-gate` 三段合同对齐） | A1 可直接套既有 gate 模板（Hard Gates + Verification + fresh evidence），A2 / A3 不能；命中即 +1 |
| **profile 分级机制可继承**（lightweight 不退化为跳过） | A1 可继承 sync-on-presence + profile 分级两条既有纪律；A2 把强制同步加到 `task closeout` 会破坏 lightweight；命中即 +1 |
| **task closeout 既有合同不被破坏** | A1 不动 `hf-finalize`；A2 直接改 `hf-finalize` 的合同形状；命中即 +1 |

通过门槛：**5 项中 ≥ 3 项命中 A1 优于 A2 / A3** → Pass。

## 6. Failure Threshold（事先写死）

| 判定 | 条件 |
|---|---|
| **Fail** | 5 项中 ≤ 1 项命中 A1 优于 A2 / A3，或出现任一 HF 既有规则**反对** A1 的硬证据（例如有显式条文要求"新增 gate 必须先经 finalize 扩展验证"——预期不存在，但若存在视为 Fail） |
| **Inconclusive** | 5 项中正好 2 项命中（证据强度不足以单方面下结论），或 desk research 期间发现治理文档自身条文相互矛盾，无法用 desk research 单独决断 |
| **Pass** | 见 §5 |

## 7. Timebox

- 最大允许投入时间: 1 小时（cloud agent 单轮内可完成；超时视为 Inconclusive）
- 超时处理: 视为 Inconclusive，进入 result 回流

## 8. Evidence 归档

- 路径: `docs/experiments/2026-04-23-hf-doc-freshness-gate-hyp-001/artifacts/`
- 包含内容: 引用的项目治理文档片段（用 file:line 引用 + 关键句节选）

## 9. Rollback / Cleanup

- 一次性原型清理步骤: N/A（desk research 无副作用）
- 对仓库 / 生产 / 第三方服务的反向操作清单: N/A

## 10. 下游回流目标（事先声明）

| 结果 | 回流目标 | 行为 |
|---|---|---|
| **Pass** | `hf-specify` | 把 HYP-001 Confidence 从 medium 升至 high（带 desk research 证据指针），把 Blocking 从"是"翻为"否"；起草 `features/<NNN>-hf-doc-freshness-gate/spec.md` |
| **Fail** | `hf-product-discovery` | 修订 §11 OST：把 Solution A1 降级，把 A2 或 A3 提为主候选；修订 discovery §4 Wedge |
| **Inconclusive** | `hf-product-discovery` 或人工确认 | 显式接受为 "accepted-risk" 并钉在 spec Key Hypotheses 中（confidence: medium），由 `规格真人确认` 阶段判断是否接受；或追加一次更高强度 probe（用户访谈 / 问卷），需用户提供接入通道 |

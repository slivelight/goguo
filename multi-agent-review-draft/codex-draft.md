# Goguo Multi-Agent Review Panel 可实施方案草案

## 1. 方案结论

本项目建议采用：

> **HF 主流程不改 + goguo 项目级 Multi-Agent Review Panel 机制**

核心原则：

- HF 仍是流程引擎，`using-hf-workflow` 与 `hf-workflow-router` 继续承担入口和运行时编排职责。
- Husky 仍是主执行者，负责起草、实现、回修、成本量化和技术债登记。
- Collie 与 Teddy 不作为并行开发者参与所有实施节点，而是在 review/gate 节点作为带持久上下文和强制角色约束的独立评审者参与。
- 三方意见不直接驱动 HF router，而是先通过 `resolution` 工件收敛，再映射为 HF 可消费的 `conclusion` 与 `next_action_or_recommended_skill`。
- 当三方无法在角色边界内达成一致，且争议触及方向、取舍、标准时，提交真人终裁。Husky 按真人裁决执行。

本方案不修改 HF canonical route，不新增 HF 主链节点，不把多 Agent 机制写成新的流程状态机。

## 2. 背景与目标

goguo 项目遵循 HF 框架，并采用 `full` profile。当前 HF 已包含从 `hf-strategy-discovery`、`hf-product-discovery`、`hf-specify`、`hf-design`、`hf-tasks` 到 gated TDD、review、gate、finalize 的完整链路。

新增多 Agent 评审机制的目标不是替代 HF，而是在不改变 HF 主流程的前提下，引入 Collie 与 Teddy 两个外部 Agent 角色，形成多 Agent + 真人的人员职责监督与协同机制。

核心诉求：

- 暴露 Husky 单 Agent 实施中的模型偏见和盲点。
- 让 Collie 从用户价值、范围和验收标准角度形成独立压力。
- 让 Teddy 从质量、测试、证据、安全、隐私和回归角度形成独立压力。
- 允许多 Agent 在明确条件下妥协，并将延期项持久化为技术债或风险记录。
- 保持 HF 工件链、review/gate 语义、approval step 和 router 状态机稳定。

## 3. 总体架构

```text
using-hf-workflow
  -> hf-workflow-router
    -> HF canonical node
      -> Husky 起草 / 实现 / 修订工件
      -> 到达 review/gate 节点
        -> Multi-Agent Review Panel
          -> Collie: 价值 / 范围 / 验收评审
          -> Teddy: 质量 / 测试 / 风险评审
          -> Husky: 可行性回应 / 成本量化 / 技术债登记
          -> resolution: 三方收敛 + 分歧 + 最终 HF verdict
        -> router 消费 resolution
          -> 通过 / 需修改 / 阻塞 / 真人裁决
```

Multi-Agent Review Panel 是项目级 adapter，而不是 HF 新节点。它挂接在现有 `hf-*review` 与 `hf-*gate` 节点内部或外围，用于增强评审输入与决议质量。

## 3A. 挂接方式

推荐采用 **外围 adapter 挂接**，而不是直接修改每个 `hf-*review` 与 `hf-*gate` skill。

### 3A.1 推荐挂接：外围 adapter

外围 adapter 的含义是：

- HF router 仍然路由到原 canonical review/gate 节点。
- 父会话在进入该节点时，按 goguo 项目规则启动 Multi-Agent Review Panel。
- Collie、Teddy、Husky 分别产出角色评审记录。
- 父会话生成 `resolution-<node>-YYYYMMDD-HHMM.md`。
- HF router 消费 `resolution` 中兼容 `reviewer-return-contract.md` 的 `HF Verdict`。

此方式不要求修改现有 `hf-*review` / `hf-*gate` skill 文件。需要修改的是项目级配置和协议：

```text
AGENTS.md
docs/agent-configs/multi-agent-review-panel.md
docs/agent-configs/review-context-pack.md
docs/agent-configs/review-resolution-template.md
docs/agent-configs/changelog.md
```

### 3A.2 Review 节点挂接

对 `hf-spec-review`、`hf-design-review`、`hf-ui-review`、`hf-tasks-review`、`hf-test-review`、`hf-code-review`、`hf-traceability-review`：

```text
router -> canonical hf-*review
  -> parent session applies goguo Multi-Agent Review Panel adapter
    -> dispatch Collie reviewer request
    -> dispatch Teddy reviewer request
    -> collect role review records
    -> Husky feasibility response
    -> write resolution record
  -> return HF-compatible reviewer summary to router
```

这里的 `resolution` 是聚合 review record；它必须输出唯一 `conclusion` 和唯一 `next_action_or_recommended_skill`。

### 3A.3 Gate 节点挂接

HF 当前的 `review-dispatch-protocol.md` 明确覆盖 review 节点；gate 节点没有同等的一等 reviewer dispatch 协议。goguo 项目可采用同形外围挂接：

```text
router -> canonical hf-*gate
  -> gate skill 先按自身规则检查 fresh evidence / verdict
  -> parent session applies goguo Multi-Agent Review Panel adapter when gate result needs panel scrutiny
    -> Collie checks user-value impact of pass/fail/debt
    -> Teddy checks evidence sufficiency and gate rigor
    -> Husky responds with feasibility / cost / debt plan
    -> write resolution record
  -> parent session maps resolution HF Verdict back to router
```

Gate 节点不建议一开始修改 skill 本体。项目级规则只要求：当 gate 涉及技术债延期、证据不足争议、性能 / 安全 / 隐私 / 用户价值风险时，必须生成 panel resolution。普通全绿、无争议、无债务的 gate 可以只保留 gate 自身 verdict，并在 completion evidence bundle 中引用。

### 3A.4 何时才修改 HF skill 本体

初期不修改 HF 框架和 `hf-*review` / `hf-*gate` skill。只有在以下条件满足时，才考虑上收或修改：

- Multi-Agent Review Panel 已在 goguo 至少完成 2-3 个 feature 周期验证。
- 多个项目都需要同一机制，而不只是 goguo 的项目治理策略。
- 需要让 router 原生识别 `panel-required`、`panel-resolution-path` 等字段。
- 需要把 gate dispatch 也产品化为 HF 通用协议。

在此之前，Multi-Agent Review Panel 保持为 goguo 项目级 adapter。

## 4. 角色分工

| 角色 | 职责 | 不做什么 |
|---|---|---|
| Husky | 主执行、技术方案、代码实现、成本量化、技术债登记、按裁决回修 | 不自审自过，不压过价值或质量阻塞，不替真人做方向取舍 |
| Collie | 用户价值、范围、验收标准、零配置、不破坏直连、当前轮 wedge | 不替 Husky 做技术设计，不泛泛提体验建议，不凭空扩范围 |
| Teddy | 测试策略、fresh evidence、性能、安全、隐私、回归、门禁 | 不脱离证据做主观阻塞，不把所有质量建议都升级为阻塞 |
| 真人 | 方向、取舍、标准、争议终裁 | 不需要介入所有普通修订 |

## 5. 适用节点

强制启用 Multi-Agent Review Panel 的节点：

- `hf-discovery-review`
- `hf-spec-review`
- `hf-design-review`
- `hf-ui-review`，仅当 UI surface 激活
- `hf-tasks-review`
- `hf-test-review`
- `hf-code-review`
- `hf-traceability-review`
- `hf-regression-gate`
- `hf-doc-freshness-gate`
- `hf-completion-gate`

各节点的角色权重如下：

| 节点 | Collie 权重 | Teddy 权重 | Husky 权重 |
|---|---:|---:|---:|
| `hf-discovery-review` | 高 | 中 | 中 |
| `hf-spec-review` | 高 | 中 | 中 |
| `hf-design-review` | 高 | 高 | 高 |
| `hf-ui-review` | 高 | 高 | 中 |
| `hf-tasks-review` | 高 | 高 | 高 |
| `hf-test-review` | 中 | 高 | 高 |
| `hf-code-review` | 中 | 高 | 高 |
| `hf-traceability-review` | 中 | 高 | 高 |
| `hf-regression-gate` | 中 | 高 | 中 |
| `hf-doc-freshness-gate` | 中 | 高 | 中 |
| `hf-completion-gate` | 中 | 高 | 中 |

## 6. 固定 Review Context Pack

为避免 Collie 与 Teddy 因未参与实施节点而评审泛化，每次派发 review/gate 前，Husky 必须提供固定上下文包。

本上下文包不是替代 HF 既有 review dispatch 协议的新协议，而是对
`skills/hf-workflow-router/references/review-dispatch-protocol.md` 中
`review request.supporting_context_paths` 的项目级扩展。

也就是说：

- HF 仍使用现有 review request 字段派发 reviewer。
- Multi-Agent Review Panel 只规定 goguo 项目中 `artifact_paths`、`supporting_context_paths`、`expected_record_path` 应如何填充。
- reviewer 的结构化返回仍遵循 `skills/hf-workflow-router/references/reviewer-return-contract.md`。

### 6.1 最小上下文清单

默认前提：

- `AGENTS.md` 会在 Agent 会话启动时作为系统上下文注入。
- `AGENTS.md` 中引用的 `docs/principles/` 宪法层文档会随项目级注入规则进入上下文。
- 因此 Review Context Pack 不重复传入这些文档，避免上下文膨胀和重复约束。
- 只有当某个外部 Agent 平台无法确认已加载项目级注入上下文时，才把 `AGENTS.md` 与相关 `docs/principles/` 文档作为 fallback supporting context 显式传入。

常规最小清单如下：

```text
docs/agent-configs/<Agent>-*.md
features/<active>/README.md
features/<active>/progress.md
features/<active>/spec.md
features/<active>/design.md
features/<active>/ui-design.md
features/<active>/tasks.md
features/<active>/reviews/
features/<active>/approvals/
features/<active>/verification/
features/<active>/evidence/
docs/TECH-DEBT.md
```

其中 `ui-design.md`、`approvals/`、`verification/`、`evidence/`、`docs/TECH-DEBT.md` 按存在读取。不存在时，reviewer 应明确标注为 `N/A` 或 `missing evidence`，不得脑补。

### 6.2 上下文使用规则

- 缺少当前节点必需上下文时，reviewer 不得凭印象继续，应返回 `阻塞` 或 `需补上下文`。
- Collie/Teddy 的 finding 必须引用具体工件路径、章节、需求 ID、测试记录或 evidence 路径。
- 没有证据的问题只能写成“证据缺口”，不能写成事实断言。
- reviewer 不参与实施细节时，不应对低层实现作无证据断言；应要求 Husky 补充测试、benchmark、日志、traceability 或方案对比。

### 6.3 HF Review Request 映射

Multi-Agent Review Panel 不新增 router 字段。父会话仍按 HF 现有协议构造 review request，并通过以下映射承载多 Agent 信息：

```json
{
  "review_type": "spec|design|ui|tasks|test|code|traceability|regression|doc-freshness|completion",
  "review_skill": "hf-xxx-review 或 hf-xxx-gate",
  "topic": "本次评审主题 + reviewer role，例如 Collie value review",
  "artifact_paths": [
    "当前被评审的主工件"
  ],
  "supporting_context_paths": [
    "docs/agent-configs/<Agent>-*.md",
    "features/<active>/progress.md",
    "features/<active>/spec.md",
    "features/<active>/design.md",
    "features/<active>/tasks.md",
    "features/<active>/reviews/",
    "features/<active>/verification/",
    "features/<active>/evidence/"
  ],
  "expected_record_path": "features/<active>/reviews/review-<node>-<Agent>-YYYYMMDD-HHMM.md",
  "current_profile": "full",
  "design_execution_mode": "parallel|architecture-first|ui-first"
}
```

其中 `topic` 与 `expected_record_path` 用于区分 Collie、Teddy、Husky 三类角色评审；`supporting_context_paths` 负责传入角色配置和当前 feature 上下文。`AGENTS.md` 与 `docs/principles/` 宪法层文档默认依赖 Agent 会话启动时的项目级注入，不在常规 request 中重复列出。

## 7. 强制角色模板

### 7.1 Collie Review 模板

```markdown
# Review - <node> - Collie

## 总体结论

通过 / 需修改 / 阻塞

## 价值锚点检查

- 是否满足核心用户场景：
- 是否破坏零配置：
- 是否误伤直连网站：
- 是否偏离当前轮 wedge：
- 验收标准是否可观察：

## 价值风险

| ID | 严重度 | 风险 | 工件锚点 | 要求 |
|---|---|---|---|---|

## 对 Husky 的质疑

至少 2 条，除非明确说明无质疑原因。

## 对 Teddy 的质疑

按需提出，聚焦测试是否服务用户价值。

## 可妥协项

| 项 | 妥协条件 | 记录位置 |
|---|---|---|

## 不可妥协项

- 
```

### 7.2 Teddy Review 模板

```markdown
# Review - <node> - Teddy

## 总体结论

通过 / 需修改 / 阻塞

## 质量门禁检查

- fresh evidence：
- 测试覆盖：
- 回归风险：
- 性能 baseline：
- 安全 / 隐私：
- 文档同步：

## 风险 / 缺口

| ID | 严重度 | 风险 | 证据缺口 | 必须补什么 |
|---|---|---|---|---|

## 对 Husky 的质疑

至少 2 条，除非明确说明无质疑原因。

## 对 Collie 的质疑

按需提出，聚焦验收标准是否可测。

## 可妥协项

| 项 | 妥协条件 | 技术债记录 |
|---|---|---|

## 不可妥协项

- 
```

### 7.3 Husky Feasibility Response 模板

```markdown
# Feasibility Response - <node> - Husky

## 对 Collie Findings 的回应

| Finding | 接受 / 反驳 / 延期 / 升级真人 | 成本 | 处理计划 |
|---|---|---|---|

## 对 Teddy Findings 的回应

| Finding | 接受 / 反驳 / 延期 / 升级真人 | 成本 | 处理计划 |
|---|---|---|---|

## 技术债登记

| Debt ID | 来源 | 影响 | 修复版本 | 是否阻塞 |
|---|---|---|---|---|

## 最终建议

通过 / 需修改 / 阻塞 / 真人裁决
```

## 8. Finding 严重度与处理规则

| 严重度 | 含义 | 默认处理 |
|---|---|---|
| `blocking` | 违反底线、证据链断裂、无法进入下游节点 | 不得通过；需回修或真人裁决 |
| `important` | 本轮应修，影响交付质量或核心验收 | 默认回修；可经真人批准延期 |
| `debt-acceptable` | 可延期，但有明确影响范围 | 必须登记技术债或风险记录 |
| `minor` | 建议项，不影响本轮完成 | 不阻塞，可记录为改进建议 |

## 9. 角色不可妥协底线

### 9.1 Collie 底线

- 误伤直连网站。
- 默认增加用户配置负担。
- 偏离当前轮核心用户价值。
- 验收标准无法观察，导致用户价值无法验证。

### 9.2 Teddy 底线

- 隐私数据外发。
- 高危安全问题。
- 关键路径无 fresh evidence。
- 性能敏感任务无 baseline 或替代证据。
- 回归范围与当前实现不一致。

### 9.3 Husky 底线

- 技术方案不可实现。
- 成本明显超出当前轮范围且未获真人批准。
- 平台目标无法满足且未获真人批准。
- 为满足单点诉求引入不可控技术债或架构破坏。

## 10. 分歧、妥协与技术债

### 10.1 分歧处理矩阵

| 分歧类型 | 默认处理 |
|---|---|
| Collie 认为破坏核心用户价值，Husky 不接受 | 升级真人裁决 |
| Teddy 认为安全、隐私、数据丢失风险阻塞，Husky 不接受 | 默认阻塞，除非真人明确接受风险 |
| Teddy 要求补测试，但 Husky 认为成本高 | 可降级为技术债，但必须有替代验证证据 |
| Collie 要求新增体验能力，Husky 认为超范围 | 回到 `hf-increment` 或提交真人裁决 |
| 性能指标暂时无法自动化测量 | 可接受半自动 evidence，但必须记录 baseline 缺口 |
| 非核心路径代码质量问题 | 可记录技术债进入下一轮 |

### 10.2 妥协允许条件

只有同时满足以下条件，finding 才允许延期或降级：

1. 不触碰任何角色的不可妥协底线。
2. 有明确影响范围和风险说明。
3. 已落盘到 `docs/TECH-DEBT.md` 或 feature 风险 / 债务记录，并绑定后续任务或版本。

### 10.3 技术债记录格式

建议统一使用以下格式：

```markdown
| ID | 来源 | 描述 | 影响 | 计划修复版本 | 引入迭代 | 责任人 | 是否阻塞 |
|---|---|---|---|---|---|---|---|
| TD-001 | Teddy review | 示例：性能 baseline 暂缺自动化采集 | 需要人工复核性能回归 | v0.2 | 001 | Husky | 否 |
```

## 11. Resolution 工件

每个 review/gate 最终必须生成一份 resolution：

```text
features/<active>/reviews/resolution-<node>-YYYYMMDD-HHMM.md
```

`resolution` 是 Multi-Agent Review Panel 给 HF router 的唯一汇总输入。HF router 不直接消费 Collie、Teddy、Husky 三份原始意见。

从 HF 既有协议视角看，`resolution` 承担的是“聚合 reviewer”的 review record：

- Collie、Teddy、Husky 的原始评审仍是项目级附属记录。
- `resolution` 的 `HF Verdict` 必须完全兼容 `reviewer-return-contract.md`。
- 父会话消费 `resolution` 时，按 HF 现有字段读取 `conclusion`、`next_action_or_recommended_skill`、`record_path`、`key_findings`、`needs_human_confirmation`、`reroute_via_router`。
- 不新增新的 verdict 词表，不新增多个下一步，不允许把多个候选动作拼成一个字符串。

### 11.1 Resolution 模板

````markdown
# Resolution - <node>

## 输入工件

- Collie review:
- Teddy review:
- Husky response:

## 三方一致点

- 

## 三方分歧点

| ID | Collie | Teddy | Husky | 是否需真人裁决 |
|---|---|---|---|---|

## 已接受修订

- 

## 已延期技术债

| Debt ID | 来源 finding | 延期理由 | 修复版本 |
|---|---|---|---|

## 真人裁决

N/A

## HF Verdict

```json
{
  "conclusion": "通过|需修改|阻塞",
  "next_action_or_recommended_skill": "<canonical hf node>",
  "record_path": "features/<active>/reviews/resolution-<node>-YYYYMMDD-HHMM.md",
  "key_findings": [
    "关键发现 1",
    "关键发现 2"
  ],
  "needs_human_confirmation": false,
  "reroute_via_router": false
}
```
````

### 11.2 HF Verdict 映射规则

| Panel 结果 | HF `conclusion` | 下一步 |
|---|---|---|
| 三方通过，无 blocking / important 未处理 | `通过` | 按 HF 迁移表进入下一节点 |
| 存在 LLM 可修复缺口 | `需修改` | 回到对应上游 skill |
| 缺少必需上下文或证据链冲突 | `阻塞` | `hf-workflow-router` |
| 触及方向、取舍、标准争议 | `阻塞` 或等待真人裁决 | 真人裁决后由 Husky 执行 |
| 存在可延期债务且不触底线 | `通过` 或 `需修改` | 取决于是否仍需本轮回修 |

## 12. 真人裁决触发条件

以下情况必须升级真人：

- 三方对方向、范围、验收标准无法一致。
- Collie 标记核心价值阻塞，Husky 不接受。
- Teddy 标记安全、隐私、数据丢失、关键证据阻塞，Husky 不接受。
- 需要牺牲用户价值换进度。
- 需要把 `blocking` 降级为技术债。
- 任何与 `docs/principles/soul.md` 中“方向、取舍、标准最终权在用户”相关的问题。

真人裁决后，Husky 执行裁决。Collie 与 Teddy 后续只审查执行是否符合裁决，不继续争论同一裁决本身。

## 13. 文件与配置落地清单

建议新增：

```text
docs/agent-configs/multi-agent-review-panel.md
docs/agent-configs/review-context-pack.md
docs/agent-configs/review-resolution-template.md
docs/agent-configs/changelog.md
docs/TECH-DEBT.md
```

建议修订：

```text
AGENTS.md
docs/agent-configs/Husky-codex-goguo.md
docs/agent-configs/Collie-opencode-goguo.md
docs/agent-configs/Teddy-claude-goguo.md
```

修订重点：

- 在 `AGENTS.md` 增加 “Multi-Agent Review Panel” 项目级声明。
- 明确 `resolution` 是 HF router 的唯一多 Agent 汇总输入。
- 明确技术债格式、延期条件和真人裁决触发条件。

## 13A. 建议新增文档实例

以下实例用于创建项目级协议文件。实例以文本块形式给出，实际落盘时去掉外层围栏即可。

### 13A.1 `docs/agent-configs/multi-agent-review-panel.md`

````markdown
# Multi-Agent Review Panel

## Purpose

本文件定义 goguo 项目在 HF review/gate 节点启用 Collie、Teddy、Husky 三方评审面板的项目级规则。

本机制是项目级 adapter，不修改 HF canonical route，不新增 HF 主链节点，不替代真人 approval step。

## Relationship With HF

- HF router 仍是 runtime authority。
- `hf-*review` 与 `hf-*gate` 仍是 canonical 节点。
- Multi-Agent Review Panel 通过 HF 既有 review request / reviewer return contract 承载。
- `resolution-<node>-YYYYMMDD-HHMM.md` 是父会话交回 router 的聚合 review record。

## Roles

| Role | Responsibility | Hard Boundary |
|---|---|---|
| Husky | 主执行、技术方案、代码实现、成本量化、技术债登记、按裁决回修 | 不自审自过，不替真人决定方向 / 取舍 / 标准 |
| Collie | 用户价值、范围、验收标准、零配置、不破坏直连、当前轮 wedge | 不替 Husky 做技术设计，不凭空扩范围 |
| Teddy | 测试策略、fresh evidence、性能、安全、隐私、回归、门禁 | 不脱离证据做主观阻塞 |
| 真人 | 方向、取舍、标准、争议终裁 | N/A |

## Panel Required Nodes

- `hf-discovery-review`
- `hf-spec-review`
- `hf-design-review`
- `hf-ui-review`，仅当 UI surface 激活
- `hf-tasks-review`
- `hf-test-review`
- `hf-code-review`
- `hf-traceability-review`
- `hf-regression-gate`
- `hf-doc-freshness-gate`
- `hf-completion-gate`

## Hooking Mode

采用外围 adapter：

```text
router -> canonical hf-*review/gate
  -> parent session applies Multi-Agent Review Panel
  -> Collie review
  -> Teddy review
  -> Husky feasibility response
  -> resolution
  -> HF-compatible verdict returned to router
```

初期不修改 HF skill 本体。只有当该机制需要上收为 harness-flow 通用能力时，才考虑修改 router 或 review/gate skill。

## Severity

| Severity | Meaning | Default Handling |
|---|---|---|
| `blocking` | 违反底线、证据链断裂、无法进入下游节点 | 不得通过 |
| `important` | 本轮应修，影响交付质量或核心验收 | 默认回修 |
| `debt-acceptable` | 可延期，有明确影响范围 | 必须登记技术债 |
| `minor` | 建议项，不影响本轮完成 | 不阻塞 |

## Compromise Rules

Finding 只有同时满足以下条件才允许延期：

1. 不触碰任何角色的不可妥协底线。
2. 有明确影响范围和风险说明。
3. 已落盘到 `docs/TECH-DEBT.md` 或 feature 风险 / 债务记录。
4. 已声明未来偿还入口：`hf-increment` 或 `hf-hotfix`。

## Human Escalation

以下情况必须升级真人：

- 三方对方向、范围、验收标准无法一致。
- Collie 标记核心价值阻塞，Husky 不接受。
- Teddy 标记安全、隐私、数据丢失、关键证据阻塞，Husky 不接受。
- 需要牺牲用户价值换进度。
- 需要把 `blocking` 降级为技术债。
- 任何触及 `docs/principles/soul.md` 中“方向、取舍、标准最终权在用户”的问题。
````

### 13A.2 `docs/agent-configs/review-context-pack.md`

````markdown
# Review Context Pack

## Purpose

本文件定义 goguo Multi-Agent Review Panel 在派发 Collie、Teddy、Husky 角色评审时，应通过 HF `review request.supporting_context_paths` 传入的最小项目上下文。

## Default Assumption

- `AGENTS.md` 已在 Agent 会话启动时作为系统上下文注入。
- `AGENTS.md` 引用的 `docs/principles/` 宪法层文档已随项目级注入规则进入上下文。
- 常规 review request 不重复传入上述文档。
- 若某外部 Agent 平台无法确认已加载项目级注入上下文，才把 `AGENTS.md` 与相关 `docs/principles/` 作为 fallback supporting context。

## Common Supporting Context

```text
docs/agent-configs/<Agent>-*.md
features/<active>/README.md
features/<active>/progress.md
features/<active>/spec.md
features/<active>/design.md
features/<active>/ui-design.md
features/<active>/tasks.md
features/<active>/reviews/
features/<active>/approvals/
features/<active>/verification/
features/<active>/evidence/
docs/TECH-DEBT.md
```

按存在读取：`ui-design.md`、`approvals/`、`verification/`、`evidence/`、`docs/TECH-DEBT.md`。

## HF Review Request Shape

```json
{
  "review_type": "spec|design|ui|tasks|test|code|traceability|regression|doc-freshness|completion",
  "review_skill": "hf-xxx-review or hf-xxx-gate",
  "topic": "<node> + <Agent role>",
  "artifact_paths": [
    "当前被评审的主工件"
  ],
  "supporting_context_paths": [
    "docs/agent-configs/<Agent>-*.md",
    "features/<active>/progress.md",
    "features/<active>/spec.md",
    "features/<active>/design.md",
    "features/<active>/tasks.md",
    "features/<active>/reviews/",
    "features/<active>/verification/",
    "features/<active>/evidence/"
  ],
  "expected_record_path": "features/<active>/reviews/review-<node>-<Agent>-YYYYMMDD-HHMM.md",
  "current_profile": "full",
  "design_execution_mode": "parallel|architecture-first|ui-first"
}
```

## Evidence Rules

- 缺少当前节点必需上下文时，reviewer 不得脑补，应返回 `阻塞` 或 `需补上下文`。
- Finding 必须引用具体工件路径、章节、需求 ID、测试记录或 evidence 路径。
- 没有证据的问题只能写为“证据缺口”，不能写成事实断言。
````

### 13A.3 `docs/agent-configs/review-resolution-template.md`

`````markdown
# Review Resolution Template

## File Name

```text
features/<active>/reviews/resolution-<node>-YYYYMMDD-HHMM.md
```

## Template

````markdown
# Resolution - <node>

## 输入工件

- Collie review:
- Teddy review:
- Husky response:

## 三方一致点

- 

## 三方分歧点

| ID | Collie | Teddy | Husky | 是否需真人裁决 |
|---|---|---|---|---|

## 已接受修订

- 

## 已延期技术债

| Debt ID | 来源 finding | 延期理由 | 偿还入口 | 修复版本 |
|---|---|---|---|---|

## 真人裁决

N/A

## HF Verdict

```json
{
  "conclusion": "通过|需修改|阻塞",
  "next_action_or_recommended_skill": "<canonical hf node>",
  "record_path": "features/<active>/reviews/resolution-<node>-YYYYMMDD-HHMM.md",
  "key_findings": [
    "关键发现 1",
    "关键发现 2"
  ],
  "needs_human_confirmation": false,
  "reroute_via_router": false
}
```
````

## Contract

`HF Verdict` 必须兼容 `skills/hf-workflow-router/references/reviewer-return-contract.md`。

约束：

- `conclusion` 只能是 `通过`、`需修改`、`阻塞`。
- `next_action_or_recommended_skill` 必须是唯一 canonical HF 节点。
- 不允许把多个候选下一步拼成一个字符串。
- `record_path` 指向本 resolution 文件。
`````

### 13A.4 `docs/agent-configs/changelog.md`

````markdown
# Agent Configs Changelog

本文件记录 `docs/agent-configs/` 下项目级 Agent 配置与 Multi-Agent Review Panel 协议的变更。

## Format

| Date | Change | Files | Reason | Approved By |
|---|---|---|---|---|

## Entries

| Date | Change | Files | Reason | Approved By |
|---|---|---|---|---|
| 2026-05-06 | Introduce Multi-Agent Review Panel draft | `multi-agent-review-panel.md`, `review-context-pack.md`, `review-resolution-template.md` | 将 Collie/Teddy/Husky 多 Agent 评审制度化，并保持 HF 主流程不变 | 待真人确认 |
````

## 14. 实施步骤

### Step 1：建立项目级协议

新增 `docs/agent-configs/multi-agent-review-panel.md`，写入本方案中的角色、节点、上下文包、finding 严重度、妥协和 resolution 规则。

### Step 2：统一三方配置

修订 Husky、Collie、Teddy 的项目配置：

- 统一 review 文件命名。
- 增加各自的强制角色模板。
- 增加不可妥协底线。
- 增加与 `resolution` 的关系说明。

### Step 3：更新 `AGENTS.md`

在 `AGENTS.md` 中增加项目级声明：

- HF canonical route 不变。
- review/gate 节点启用 Multi-Agent Review Panel。
- `resolution` 是 router 消费的汇总结论。
- 真人裁决优先于任何 Agent 结论。

### Step 4：建立技术债文件

新增 `docs/TECH-DEBT.md`，并采用统一表格格式记录延期项。

### Step 5：试运行一个 review 节点

优先选择 `hf-spec-review` 或 `hf-design-review` 试运行：

1. Husky 准备 context pack。
2. Collie 产出价值评审。
3. Teddy 产出质量评审。
4. Husky 产出 feasibility response。
5. 生成 resolution。
6. 将 resolution 的 HF Verdict 交回 router。

### Step 6：根据试运行结果微调模板

只调整项目级协议和模板，不改 HF 主流程。若该机制在多个项目复用，再考虑上收为 harness-flow 的可选扩展。

## 15. 关键约束

- 多 Agent 评审不替代 HF review/gate 节点。
- 多 Agent 评审不替代真人 approval step。
- Collie/Teddy 不参与实施节点时，必须通过 context pack 恢复上下文。
- 任何 reviewer 不得用无证据判断替代工件证据。
- 技术债不是绕过质量的口子，只能用于不触碰底线且影响范围明确的延期项。
- 当争议触及方向、取舍、标准，必须回到真人。

## 16. 最终判断

这套方案的关键不是“多几个 Agent 发表意见”，而是把多 Agent 评审制度化为：

```text
角色化上下文
+ 强制挑战模板
+ 结构化分歧
+ 有条件妥协
+ 技术债落盘
+ 真人终裁
+ HF verdict 归一化
```

这样既能利用 Collie 与 Teddy 暴露 Husky 单 Agent 实施中的模型偏见，又不会把 HF 主流程改成不可维护的多 Agent 状态机。

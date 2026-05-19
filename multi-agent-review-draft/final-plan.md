# Goguo 多 Agent 评审最终方案

> 本文档是 codex-draft、claude-draft、opencode-draft 三份方案的取舍合并结果。
> 经三个 Agent 平台（Codex / OpenCode / Claude）各自提出方案后，由 Claude 汇总比较，
> 最终方案经真人确认后生效。

---

## 0. 修订摘要

### 0.1 Codex 落地性修订

本次修订不调整 Claude 汇总方案的整体方向与骨架，只补足实际落地时容易缺失或执行不稳的部分：

- 统一命名：将新增目录 / 文件示例统一为中划线风格，例如 `docs/agent-configs/`、`docs/TECH-DEBT.md`、`review-<节点>-Collie-YYYYMMDD-HHMM.md`、`resolution-<节点>-YYYYMMDD-HHMM.md`。
- 去重上下文：`AGENTS.md` 与其引用的 `docs/principles/` 宪法层文档默认由 Agent 会话启动注入，常规 Review Context Pack 不重复传入；仅在外部 Agent 平台无法确认注入时作为 fallback。
- 明确挂接方式：Multi-Agent Review Panel 采用项目级外围 adapter，不修改 `skills/hf-*`、`using-hf-workflow`、`hf-workflow-router`；review/gate 的 panel 介入由父会话按项目协议执行。
- 补齐新增文档实例：补充 `multi-agent-review-panel.md`、`review-context-pack.md`、`review-briefing-template.md`、`collie-review-template.md`、`teddy-review-template.md`、`husky-response-template.md`、`resolution-template.md`、`disagreement-record-template.md`、`tech-debt-template.md`、`changelog.md`、`goguo-review-orchestrator/SKILL.md` 的可落盘文本实例。
- 补齐 AGENTS.md 与三份 Agent 配置的最小修订块，避免只有”需要修改”但没有可复制内容。
- 补齐技术债偿还入口字段：技术债登记时必须声明未来偿还入口为 `hf-increment` 或 `hf-hotfix`。

### 0.2 HF Subagent 并行四输入模型修订

本版在 Codex 落地版基础上，明确 HF 框架原生 subagent review 与 Multi-Agent Review Panel 的关系：

- HF 框架的 reviewer subagent 是框架内置行为，不可跳过，不修改。
- Husky reviewer subagent 执行 `hf-spec-review` / `hf-design-review` / ... 等 canonical review skill，产出 **HF-native review record**。
- HF subagent review 与 Collie / Teddy 评审**并行执行**，不串联等待。
- Resolution 汇总**四类输入**：
  1. HF-native Husky reviewer subagent 结论（框架标准评审）
  2. Collie review（价值视角）
  3. Teddy review（质量视角）
  4. Husky feasibility response（对 2/3 的技术可行性回应）
- Resolution 输出唯一 HF Verdict；父会话从中提炼 return contract 交给 router。

### 0.3 阶段感知路径与 Gate 精简模式修订

补全两个落地缺口：

1. **Discovery 阶段路径**：`hf-discovery-review` 时 `features/<active>/` 尚不存在，Context Pack、评审输出、Resolution 均使用 `docs/reviews/` 替代。在 Briefing 模板、Context Pack、Resolution 模板中增加阶段感知路径规则。
2. **Gate 精简验证模式**：gate 节点（`hf-regression-gate`、`hf-doc-freshness-gate`、`hf-completion-gate`）是条件验证型判断，不适用论证型评审机制。gate 不使用反方立场、分歧发现量化、技术债妥协。新增 `gate-review-template.md` 和 `gate-resolution-template.md`。

### 0.4 HF 框架适配逻辑修订

补全与 HF 框架实际行为不一致的 7 个逻辑缺口：

1. **Gate 词汇修正**：`hf-regression-gate` 和 `hf-completion-gate` 使用 `通过/需修改/阻塞`（与 review 相同），不是 `pass/partial/N/A/blocked`（仅 `hf-doc-freshness-gate` 使用）。
2. **Gate subagent 差异**：`hf-regression-gate` 和 `hf-completion-gate` 不派发 subagent，在父会话内直接执行。并行四输入模型对这两个 gate 退化为三类输入。
3. **Approval Step 补全**：`hf-spec-review`/`hf-design-review`/`hf-tasks-review` 通过后需执行 HF canonical approval step（规格/设计/任务真人确认）。`hf-discovery-review` 通过后无 canonical approval step，改为项目级 panel hard stop / user checkpoint。新增 §4.16 Resolution 与 Return Contract 映射规则及 `needs_human_confirmation` 节点查表。
4. **Resolution ≠ return contract**：Resolution 是项目级工件，不是 `reviewer-return-contract.md` 替代品。父会话需从 Resolution 提炼结论，按 return contract 格式交回 router。
5. **HF subagent 命名**：HF subagent 的 review record 路径由各 `hf-*-review` skill 自行定义，不再硬编码命名模式。
6. **现有 Husky 配置整合**：§8.14 追加块增加与现有配置的整合说明。
7. **agent-configs 目录声明**：在 AGENTS.md 块中声明 `docs/agent-configs/` 为项目级档 2 自定义路径。

---

## 1. 三方案摘要

### 1.1 codex-draft（Husky 视角）

**核心定位**：HF 框架不改，Multi-Agent Review Panel 作为项目级 adapter 挂接在 review/gate 节点外围。

**关键设计**：
- 固定 Review Context Pack 提供最小上下文
- 三方角色模板（Collie 价值 / Teddy 质量 / Husky 可行性回应）
- Finding 严重度分级（blocking / important / debt-acceptable / minor）
- Resolution 工件作为 HF router 的唯一汇总结论
- 分歧处理矩阵 + 有条件妥协 + 技术债落盘
- 角色不可妥协底线

**框架侵入性**：不修改 HF。

### 1.2 claude-draft（Teddy 视角）

**核心定位**：不修改 HF 框架，通过项目级 Skill（`goguo-review-orchestrator`）+ Agent 配置实现多 Agent 协作。

**关键设计**：
- 三层评审质量机制：Review Briefing（上下文注入）→ 角色锚定清单（角色遵循）→ 强制反方立场（对抗认同倾向）
- 串行互读（Teddy 读 Collie 评审后再写自己的）
- 按节点×按角色定制锚定清单
- 结构化分歧记录模板（含触发条件 + Tech Debt ID）
- review/gate 调人信号（Husky 主动提示真人调 Collie/Teddy）
- 三方案比较与推荐分析

**框架侵入性**：不修改 HF。

### 1.3 opencode-draft（Collie 视角）

**核心定位**：扩展 HF 框架核心协议，使 Router 原生支持多评审者派发、盲审、自动妥协。

**关键设计**：
- Router 新增 Step 9A（多评审者盲审派发）/ Step 9B（分歧合并与仲裁）
- 扩展 `review-dispatch-protocol.md`、`reviewer-return-contract.md`
- 扩展所有 `hf-*review` skill 的 Workflow
- 盲审模式（评审者独立评审，不互读）
- 分歧发现量化评分（1-10 分制）
- 自动妥协判断（Router 按阈值自动决策）
- 红线检查（质量属性不可妥协底线）
- 角色锚定检查（强制执行，未完成则评审失败）
- 技术债持久化路径

**框架侵入性**：修改 HF（router、review skills、return contract）。

---

## 2. 多维度比较

### 2.1 比较维度表

| 维度 | codex-draft | claude-draft | opencode-draft |
|------|------------|-------------|----------------|
| **HF 框架独立性** | 不改 HF | 不改 HF | **修改 HF**（router + 7 个 review skill + return contract） |
| **真人操作复杂度** | 中（真人汇总裁决，但无工具辅助） | 中低（Skill 提供模板和 prompt，但真人仍需跨平台切换） | **低**（Router 自动派发合并，真人只看结论和裁决） |
| **评审针对性** | 中（Context Pack 提供静态上下文） | **高**（Briefing 含决策追溯 + 评审焦点 + 角色定向提示） | 高（Router 注入战略/feature/标准/ADR 上下文） |
| **角色遵循保障** | 中（角色模板 + 质疑模板） | **高**（三层机制：Briefing → 锚定清单 → 反方立场） | 高（Role Anchor Check 强制执行 + Divergence Discovery） |
| **对抗认同倾向** | 中（质疑模板要求 ≥2 条） | **高**（强制 3 条反方立场 + 角色立场回应） | 中（分歧发现任务 ≥3 个，但盲审模式无互读碰撞） |
| **拧毛巾张力效果** | 中（三方独立评审，真人汇总） | **高**（串行互读 + 反方立场 + 分歧记录） | 中（盲审独立，无互读，靠量化评分发现分歧） |
| **妥协机制严谨性** | 高（底线 + 矩阵 + 债务落盘） | 高（触发条件 + Tech Debt ID + 妥协有效性五条件） | **最高**（量化阈值 + 红线检查 + 成本收益分析 + 修复计划） |
| **HF 升级兼容性** | 好（直接覆盖 `skills/hf-*`） | 好（项目级 skill 独立于 `hf-*`） | **差**（需合并 router/review skill 冲突） |
| **实施成本** | 中（项目文档 + 配置修改） | 中高（项目文档 + Skill + 配置修改 + 锚定清单） | **高**（修改 10+ 个 HF 文件 + 回归测试） |
| **迭代适应性** | 好（调整项目文档即可） | 好（调整 skill 模板即可） | 差（改 HF 框架文件） |
| **跨项目复用性** | 中（项目级文档需重写） | 中（Skill 可复制但需定制） | 高（HF 框架级改动可复用） |
| **评审模式** | 未明确（隐含真人串行协调） | 串行互读（Teddy 读 Collie 后写） | 盲审并行（独立不互读） |

### 2.2 关键差异分析

**差异一：是否修改 HF 框架**

- codex / claude：不修改，多 Agent 机制完全在项目层实现
- opencode：修改 HF 核心协议，使 Router 原生支持多评审者

> **判定**：不修改 HF。理由：(1) HF 框架持续迭代，修改增加合并负担；(2) goguo 当前在 product-discovery 阶段，先验证机制再考虑是否上收到框架。

**差异二：评审模式 — 盲审 vs 串行互读**

- opencode：盲审并行（评审者不看对方意见，独立评审）
- claude：串行互读（Teddy 读 Collie 评审后再写自己的）
- codex：未明确指定

> **判定**：串行互读为主，可选盲审。理由：(1) 拧毛巾的核心是碰撞，串行互读让后评者直接回应前评者观点，形成真实碰撞；(2) 盲审发现的分歧是"潜在分歧"（猜测对方立场），而非"实际分歧"（回应对方观点），碰撞深度不足；(3) 但对于低风险节点，真人可选择跳过互读以加速。

**差异三：自动妥协 vs 真人裁决**

- opencode：Router 按量化阈值自动妥协（分歧差距 ≤ 4 时自动决策）
- codex / claude：真人裁决为主，妥协条件结构化记录

> **判定**：真人裁决为主，不自动妥协。理由：(1) 多平台 Agent 之间不存在自动通信通道，"自动妥协"在当前架构下无法实现；(2) 妥协的质量依赖真人对上下文的综合判断，量化阈值只能辅助不能替代；(3) 但采纳 opencode 的量化评分和红线检查作为真人裁决的输入，提升裁决效率。

---

## 3. 取舍决策

### 3.1 取

| 来源 | 取什么 | 理由 |
|------|--------|------|
| codex § 3 总体架构 | "项目级 adapter，HF 不改"的定位 | 架构清晰，框架独立 |
| codex § 4 角色分工 | 三方角色 + 职责 + 不做什么 | 简洁明确，边界清晰 |
| codex § 6 Review Context Pack | 固定最小上下文清单 | 确保评审者有项目背景 |
| codex § 7 强制角色模板 | Collie/Teddy/Husky 三方结构化评审模板 | 模板实用，覆盖核心检查项 |
| codex § 8 Finding 严重度 | blocking / important / debt-acceptable / minor 四级 | 分级清晰，处理规则明确 |
| codex § 9 角色底线 | 三方不可妥协底线 | 防止妥协越过红线 |
| codex § 11 Resolution 工件 | Resolution 作为 HF router 唯一汇总输入 | 接口简洁，router 只消费一个结论 |
| codex § 10 分歧处理矩阵 | 6 种典型分歧的默认处理 | 实用，覆盖常见场景 |
| claude § 3 Review Briefing | 作者产出定向简报（决策追溯 + 评审焦点 + 角色提示） | 解决评审者上下文缺失问题 |
| claude § 4 角色锚定清单 | 按节点×按角色定制的逐项检查清单 | 确保角色遵循，防止泛议 |
| claude § 5 强制反方立场 | 必填的反方论证 + 角色立场回应章节 | 对抗 LLM 认同倾向 |
| claude § 4.2 分歧记录模板 | 结构化分歧记录（立场 + 证据 + 妥协评估） | 分歧可追溯，妥协有原则 |
| claude § 6 评审执行流程 | 完整流程图（Briefing → Collie → Teddy → 真人 → Resolution） | 操作清晰 |
| claude § 7 goguo-review-orchestrator | 项目级 Skill 提供模板和 prompt | 降低真人操作成本 |
| opencode § 7.5 分歧发现量化 | 1-10 分制分歧强度评分 | 量化辅助裁决 |
| opencode § 7.6.3 红线定义 | 质量属性不可妥协红线 | 妥协前强制检查 |
| opencode 角色锚定检查概念 | 评审前强制执行检查，未完成则 blocked | 确保评审质量 |
| opencode 不跨权评审规则 | 各角色只评审自己职责范围 | 防止角色混权 |

### 3.2 舍

| 来源 | 舍什么 | 理由 |
|------|--------|------|
| opencode 全部框架修改 | 修改 router、review skills、return contract | 违反"不改 HF"约束 |
| opencode Router 自动派发 | Step 9A/9B 多评审者自动派发和合并 | 多平台无法自动派发，真人协调 |
| opencode 盲审模式 | 评审者互不看到对方意见 | 失去拧毛巾碰撞效果 |
| opencode 自动妥协 | Router 按阈值自动决策妥协 | 多平台无法自动执行，真人裁决更可靠 |
| opencode 角色重定义 | Collie=技术评审、Teddy=UX评审 | 与现有 agent 配置（Collie=价值、Teddy=质量）不一致 |
| claude 过多模板文件 | collie-checklist/ 和 teddy-checklist/ 下每个节点一份 | 初始阶段过度设计，改为通用清单 + 节点专属补充 |
| codex § 5 权重表 | "Collie 权重 高/中" 的模糊权重 | 改为明确的"必须/可选/不参与"参与矩阵 |

---

## 4. 最终合并方案

### 4.1 方案定位

**不修改 HF 框架**。多 Agent 评审机制作为项目级 adapter，通过以下三层实现：

1. **项目级 Skill**（`goguo-review-orchestrator`）— 提供模板、prompt、流程指引
2. **Agent 配置文件**（`docs/agent-configs/`）— 定义角色、检查清单、行为规范
3. **AGENTS.md 声明**（§ 7）— 项目级多 Agent 架构声明

### 4.2 角色分工

沿用现有 agent 配置的角色定义（与 codex-draft § 4 一致）：

| 角色 | 职责 | 不做什么 |
|------|------|----------|
| Husky | 主执行、技术方案、代码实现、Review Briefing 产出、可行性回应、技术债登记 | 不自审自过，不压过价值或质量阻塞，不替真人做方向取舍 |
| Collie | 用户价值、范围、验收标准、零配置、不破坏直连、当前轮 wedge | 不替 Husky 做技术设计，不泛泛提体验建议，不凭空扩范围 |
| Teddy | 测试策略、fresh evidence、性能、安全、隐私、回归、质量门禁 | 不脱离证据做主观阻塞，不把所有质量建议都升级为阻塞 |
| 真人 | 方向、取舍、标准、争议终裁 | 不需要介入所有普通修订 |

### 4.3 参与矩阵

| HF 评审 / 门禁节点 | 节点类型 | Collie（价值） | Teddy（质量） | Husky（可行性回应） |
|--------------------|----------|---------------|--------------|-------------------|
| `hf-discovery-review` | review | 必须 | 可选 | 必须 |
| `hf-spec-review` | review | 必须 | 必须 | 必须 |
| `hf-design-review` | review | 必须 | 必须 | 必须 |
| `hf-ui-review`（仅 UI surface 激活） | review | 必须 | 必须 | 必须 |
| `hf-tasks-review` | review | 必须 | 必须 | 必须 |
| `hf-test-review` | review | 可选 | 必须 | 必须 |
| `hf-code-review` | review | 不参与 | 必须 | 必须 |
| `hf-traceability-review` | review | 可选 | 必须 | 必须 |
| `hf-regression-gate` | **gate** | 不参与 | 必须 | 必须 |
| `hf-doc-freshness-gate` | **gate** | 不参与 | 可选 | 必须 |
| `hf-completion-gate` | **gate** | 可选 | 必须 | 必须 |

> **节点类型决定评审模式**：review 节点使用完整评审模式（含反方立场、分歧发现量化、技术债妥协）；gate 节点使用精简验证模式（见 § 4.15）。

### 4.4 评审执行流程

```
HF workflow 进入 review/gate 节点
    │
    ▼
确认工件路径阶段：
    - discovery 阶段（hf-discovery-review）→ 上下文/输出落到 docs/reviews/
    - feature 阶段（hf-spec-review 及之后）→ 上下文/输出落到 features/<active>/reviews/
    │
    ▼
Husky 产出工件 + Review Briefing
父会话在 review/gate 节点应用项目级外围 adapter，输出调人信号
    │
    ├─── [并行分支 1] Husky 主会话按 HF 协议派发 reviewer subagent
    │         执行 hf-spec-review / hf-design-review / ...（框架标准 skill）
    │         产出 HF-native review record（框架标准结论）
    │
    ├─── [并行分支 2] 真人切换到 Collie（opencode）
    │         读：Context Pack + Review Briefing + 被评审工件
    │         执行：角色锚定清单 + 强制反方立场 + 分歧发现量化
    │         写：review-<节点>-Collie-YYYYMMDD-HHMM.md
    │
    └─── [并行分支 3] 真人切换到 Teddy（claude）
              读：Context Pack + Review Briefing + 被评审工件
                  （可选串行互读：读 Collie 评审文档后再写）
              执行：角色锚定清单 + 强制反方立场 + 分歧发现量化
              对 Collie 结论冲突点（如有）：填入分歧记录
              写：review-<节点>-Teddy-YYYYMMDD-HHMM.md
    │
    ▼  三路并行评审完成
Husky 主会话产出 feasibility response
    读：Collie review + Teddy review
    对每条 finding 回应：接受 / 反驳 / 延期 / 升级真人
    量化成本，登记技术债（如有）
    写：response-<节点>-Husky-YYYYMMDD-HHMM.md
    │
    ▼  四类输入就绪
真人汇总四类输入，写 Resolution：
    (1) HF-native Husky reviewer subagent 结论
    (2) Collie review
    (3) Teddy review
    (4) Husky feasibility response
    检查：锚定清单完整性 / 反方立场实质度 / 分歧记录质量
    对分歧点量化评分（1-10 分制）
    执行红线检查
    裁决：共识 → 通过 | 分歧 → 妥协或升级
    写：resolution-<节点>-YYYYMMDD-HHMM.md
    │
    ▼
Husky 按 Resolution 执行
    修改工件（如有 ❌ 项）→ 重新提交
    或继续推进（全部通过 / 有条件通过已处理）
    │
    ▼
父会话从 Resolution 提炼结论，按 reviewer-return-contract.md 格式构建 return contract
    │
    ▼
若 needs_human_confirmation=true（spec/design/tasks-review；hf-ui-review 待联合汇合）：
    父会话执行 approval step（规格/设计/任务真人确认）
    │
    ▼
hf-workflow-router 消费 return contract
    按迁移表进入下一 canonical 节点
```

#### 并行执行要点

- **分支 1（HF subagent）**是 HF 框架内置行为，由 `hf-workflow-router` 按 `review-dispatch-protocol` 自动派发，不可跳过。它在 Husky 平台内独立执行，与分支 2/3 无依赖。
  - **注意**：`hf-regression-gate` 和 `hf-completion-gate` **不派发 subagent**，在父会话内直接执行。对这两个 gate，分支 1 不存在，并行模型退化为三类输入（Collie + Teddy + Husky response）。
  - `hf-doc-freshness-gate` 通过自己的 `reviewer-dispatch-handoff.md` 派发 subagent，分支 1 存在。
- **分支 2（Collie）和分支 3（Teddy）** 由真人在不同 Agent 平台（opencode / claude）手动触发。默认模式为**串行互读**：Teddy 在写评审前先读 Collie 的评审文档，形成拧毛巾碰撞。低风险节点可由真人选择**快速并行模式**：Teddy 与 Collie 同时开始，但 Teddy 必须在 Resolution 中补充分歧回应章节（对 Collie 结论的独立评判）。
- **Husky feasibility response** 在三路并行评审全部完成后产出，因为它需要回应 Collie 和 Teddy 的 findings。
- **Resolution** 是项目级汇总工件，父会话消费 Resolution 并从中提炼 `reviewer-return-contract.md` 格式的 return contract 交给 router。HF-native subagent 的 return contract 不直接驱动 router 后续路由，而是作为 Resolution 的输入之一参与真人综合裁决。
- 若 HF-native subagent 返回 `需修改/阻塞`，但 Collie/Teddy 均通过，真人有权在 Resolution 中综合判定。但以下类型的 finding **不得降级为技术债**，必须按 subagent 结论处理：`reroute_via_router=true`、缺少已批准上游工件、`stage/profile/route` 冲突、`gate fresh evidence` 缺口、`precheck` 阻塞。只有内容类、非 hard-gate、非 workflow/precheck 的 `minor/important` finding 可考虑延期为技术债（仍须满足 § 4.11 妥协有效性五条件）。Resolution 的 HF Verdict 是最终权威。
- **Resolution 不是 return contract 替代品**（见 § 4.16）。父会话需从 Resolution 提炼结论，按 `reviewer-return-contract.md` 格式交回 router。

### 4.5 上下文注入机制（两层）

**第一层：固定 Review Context Pack**（源自 codex-draft § 6）

每次评审的固定最小上下文，所有评审者都读取。

默认前提：

- `AGENTS.md` 会在 Agent 会话启动时作为系统上下文注入。
- `AGENTS.md` 中引用的 `docs/principles/` 宪法层文档会随项目级注入规则进入上下文。
- 常规 Review Context Pack 不重复传入这些文档，避免上下文膨胀和重复约束。
- 只有当某个外部 Agent 平台无法确认已加载项目级注入上下文时，才把 `AGENTS.md` 与相关 `docs/principles/` 文档作为 fallback supporting context 显式传入。

常规最小清单按阶段分为两种：

**Feature 阶段**（`hf-spec-review` 及之后，`features/<active>/` 已存在）：

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

**Discovery 阶段**（`hf-discovery-review`，`features/<active>/` 尚不存在）：

```text
docs/agent-configs/<Agent>-*.md
README.md
CHANGELOG.md
docs/architecture.md
docs/insights/                          # strategy-discovery / product-discovery 输出
docs/reviews/                           # discovery-review 落盘目录
docs/TECH-DEBT.md
当前被评审的 discovery 草稿路径         # 由 Review Briefing 指定
```

缺失文件按 `read-on-presence` 原则处理，不阻塞但标注为 `missing evidence`。

**第二层：Review Briefing**（源自 claude-draft § 3）

每轮评审动态生成的定向简报，由 Husky 产出：

```markdown
# Review Briefing — <节点名>

## 1. 当前工件
- 路径 / 状态 / HF 节点 / Profile

## 2. 上游关键决策（本工件继承的约束）
| 决策 ID | 内容 | 原因 | 来源工件 |

## 3. 本轮评审焦点（作者主动请求的关注点）
- [必答] ≥ 2 条
- [可选] ≥ 1 条

## 4. 角色定向提示
### 给 Collie（价值视角）：≥ 2 条定向检查
### 给 Teddy（质量视角）：≥ 2 条定向检查
```

### 4.6 角色锚定清单

每个评审者在写评审前必须逐项检查，每项给出 `✅` / `⚠️` / `❌` + 说明。

**通用锚定项（所有节点必填）**：

```markdown
### 通用角色锚定检查

- [ ] 已确认项目级注入上下文已加载；若无法确认，已从 fallback context 读取 `AGENTS.md` 与 `docs/principles/`
- [ ] 已读取 Review Context Pack 中的当前 feature 工件和角色配置
- [ ] 已读取 Review Briefing 中的上游关键决策
- [ ] 每条 finding 已锚定到工件具体章节/条目（不允许泛议）
- [ ] 不跨权评审（发现其他角色职责范围问题只标注"建议 XX 关注"）
```

**角色专属通用项**：

Collie（价值）：

```markdown
- [ ] 本工件是否与当前轮 wedge 对齐？
- [ ] 验收场景 AC01-AC04 是否仍可被覆盖？
- [ ] 是否触碰 soul.md 底线（误伤直连 / 增加配置负担）？
```

Teddy（质量）：

```markdown
- [ ] 是否存在无 fresh evidence 的关键断言？
- [ ] 是否存在安全/隐私红线被触碰的迹象？
- [ ] 与 TECH-DEBT.md 中已有债务的交互风险是否评估？
```

**节点专属补充项**（存放在 `goguo-review-orchestrator/templates/` 下，按需加载）：

每个 HF review 节点可以有额外的专属检查项。初始阶段只创建核心节点的清单（spec-review、design-review），其他节点在首次评审前按需补充。

### 4.7 强制反方立场

**仅适用于 review 节点。gate 节点不使用此机制（见 § 4.15）。**

每个评审文档的**必填章节**（源自 claude-draft § 5）：

```markdown
## 反方立场（必填）

假设你完全反对本工件方案，给出至少 3 条最强有力的反对理由：

### 反对理由 1：<标题>
- 论点：
- 依据（引用工件章节/条目）：
- 强度评估：高 / 中 / 低

### 反对理由 2 / 3：（同上格式）

### 角色立场回应
从你自己的角色出发，逐一回应：

| 反对理由 | 回应 | 处理 |
|----------|------|------|
| 理由 1 | ... | 接受 / 反驳（附依据）/ 可折衷（附条件） |
| 理由 2 | ... | ... |
| 理由 3 | ... | ... |
```

**评判标准**（真人使用）：

| 反方立场表现 | 含义 | 处理 |
|-------------|------|------|
| 全部"高强度" + 全部"反驳" | 缺乏自省 | 真人手动审查反对理由 |
| 全部"高强度" + 全部"接受" | 工件确实有严重问题 | 需回修 |
| 反对理由引用了具体工件章节 | 评审有针对性 | 采纳 |
| 反对理由是泛泛的 | 评审流于形式 | 标记低质量，要求补充 |

### 4.8 分歧发现量化（源自 opencode § 7.5）

**仅适用于 review 节点。gate 节点不使用此机制（见 § 4.15）。**

每个评审者在反方立场章节之后，增加**分歧发现量化**必填章节：

```markdown
## 分歧发现量化（必填）

从其他评审者角色视角，识别至少 3 个潜在分歧点：

| 分歧点 | 本方立场 | 预估他方立场 | 分歧强度(1-10) | 置信度 | 理由 |
|--------|---------|-------------|---------------|--------|------|
| ... | ... | ... | ... | ... | ... |
```

**分歧强度标准**：

| 强度 | 评分 | 含义 |
|------|------|------|
| 轻微 | 1-3 | 不影响核心决策，可妥协 |
| 中度 | 4-6 | 需权衡，真人裁决或结构化妥协 |
| 重大 | 7-10 | 核心冲突，必须真人裁决 |

### 4.9 Finding 严重度与处理规则（源自 codex-draft § 8）

| 严重度 | 含义 | 默认处理 |
|--------|------|----------|
| `blocking` | 违反底线、证据链断裂 | 不得通过；需回修或真人裁决 |
| `important` | 本轮应修，影响交付质量 | 默认回修；可经真人批准延期 |
| `debt-acceptable` | 可延期，有明确影响范围 | 必须登记技术债 |
| `minor` | 建议项，不影响完成 | 不阻塞，记录为改进建议 |

### 4.10 角色不可妥协底线（源自 codex-draft § 9）

**Collie 底线**：
- 误伤直连网站
- 默认增加用户配置负担
- 偏离当前轮核心用户价值
- 验收标准无法观察

**Teddy 底线**：
- 隐私数据外发
- 高危安全问题
- 关键路径无 fresh evidence
- 性能敏感任务无 baseline
- 回归范围与实现不一致

**Husky 底线**：
- 技术方案不可实现
- 成本明显超范围且未获真人批准
- 为满足单点诉求引入不可控架构破坏

### 4.11 分歧处理与妥协（合并 codex § 10 + opencode § 7.6）

#### 分歧处理矩阵

| 分歧类型 | 默认处理 |
|----------|----------|
| Collie 认为破坏核心价值，Husky 不接受 | 升级真人裁决 |
| Teddy 认为安全/隐私/数据丢失阻塞，Husky 不接受 | 默认阻塞，除非真人明确接受风险 |
| Teddy 要求补测试，Husky 认为成本高 | 可降级为技术债，须有替代验证证据 |
| Collie 要求新增能力，Husky 认为超范围 | 回 `hf-increment` 或提交真人裁决 |
| 非核心路径质量问题 | 可记录技术债进入下一轮 |

#### 红线检查（源自 opencode § 7.6.3）

妥协前**强制检查**，触碰任一红线则不可妥协，必须真人裁决：

| 红线类型 | 触发条件 |
|----------|----------|
| soul.md 底线 | 误伤直连 / 增加配置负担 / 用户数据外发 |
| 安全红线 | OWASP Top 10 漏洞 |
| 架构红线 | 违反 ADR 核心决策 |
| 性能红线 | 性能敏感路径无 baseline 证据 |
| 证据红线 | 关键路径无 fresh evidence |

#### 妥协有效性条件

只有**同时满足以下五条**，妥协才成立：

1. 不触碰任何角色的不可妥协底线
2. 有明确影响范围和风险说明
3. 填写了触发修复条件（不允许无条件延期）
4. 已落盘到 `docs/TECH-DEBT.md`，绑定后续任务或版本
5. 目标迭代不得超出 strategy-discovery 确定的 OPP 序列范围
6. 已声明未来偿还入口：`hf-increment` 或 `hf-hotfix`

#### 技术债记录格式

```markdown
| ID | 来源 | 描述 | 影响 | 触发修复条件 | 偿还入口 | 目标迭代 | 责任人 | 阻塞 |
|----|------|------|------|-------------|----------|----------|--------|------|
| TD-001 | Teddy review | ... | ... | 用户报告数 > X | hf-increment | OPP-004 | Husky | 否 |
```

### 4.12 Resolution 工件（源自 codex-draft § 11）

Resolution 是 Multi-Agent Review Panel 的**项目级汇总工件**。它汇总评审输入，输出 HF Verdict。

> **重要**：Resolution **不是** `reviewer-return-contract.md` 的替代品。Router 不直接消费 Resolution 文件。父会话需从 Resolution 提炼结论，按 `reviewer-return-contract.md` 格式构建 return contract 交回 router（见 § 4.16）。

**工件路径规则**：
- Discovery 阶段输入工件：由 Briefing 明确指定，通常来自 `docs/insights/`（discovery 草稿），不是 `docs/reviews/`。
- Discovery 阶段输出工件：Panel review / response / resolution 落到 `docs/reviews/`。
- Feature 阶段：输入和输出均落到 `features/<active>/reviews/`。
- 不使用"所有 `features/<active>/` 路径全局替换"规则；输入路径按 Briefing 指定，输出路径按阶段确定。

**输入数量规则**：
- 有 HF subagent 的节点（review 节点 + `hf-doc-freshness-gate`）：四类输入（HF-native record + Collie + Teddy + Husky response）
- 无 HF subagent 的节点（`hf-regression-gate` + `hf-completion-gate`）：三类输入（Collie + Teddy + Husky response）

````markdown
# Resolution — <节点名>

## 输入工件
- HF-native review: （由 HF subagent 的 `record_path` 返回值填充；对无 subagent 的 gate 填 N/A）
- Collie review: {features/<active>/reviews/ | docs/reviews/}review-<节点>-Collie-YYYYMMDD-HHMM.md
- Teddy review: {features/<active>/reviews/ | docs/reviews/}review-<节点>-Teddy-YYYYMMDD-HHMM.md
- Husky response: {features/<active>/reviews/ | docs/reviews/}response-<节点>-Husky-YYYYMMDD-HHMM.md

## 四方一致点
-

## HF-native subagent 与 Panel 结论差异（如有）
| 差异点 | HF subagent 结论 | Panel 综合结论 | 差异原因 | 真人判定 |
|--------|-----------------|---------------|----------|----------|

## 分歧点（含量化评分）
| 分歧点 | Collie 评分 | Teddy 评分 | 红线检查 | 处理 |
|--------|-----------|-----------|----------|------|

## 已接受修订
-

## 已延期技术债
| Debt ID | 来源 finding | 延期理由 | 触发条件 | 偿还入口 | 目标迭代 |
|---------|-------------|----------|----------|----------|----------|

## 真人裁决（如有）
- 裁决结论：
- 裁决理由：

## HF Verdict

```json
{
  "conclusion": "通过|需修改|阻塞",
  "next_action_or_recommended_skill": "<canonical hf node>",
  "needs_human_confirmation": false,
  "reroute_via_router": false,
  "record_path": "{features/<active>/reviews/ | docs/reviews/}resolution-<节点>-YYYYMMDD-HHMM.md",
  "key_findings": [
    "关键发现 1",
    "关键发现 2"
  ]
}
```
````

### 4.13 HF Verdict 映射

| Panel 结果 | HF conclusion | 下一步 |
|------------|---------------|--------|
| 三方通过，无 blocking/important 未处理 | `通过` | 按迁移表进入下一节点 |
| 存在可修复缺口 | `需修改` | 回到对应上游 skill |
| 缺少必需上下文或证据链冲突 | `阻塞` | `hf-workflow-router` |
| 方向/取舍/标准争议 | `阻塞` 或等待真人裁决 | 真人裁决后 Husky 执行 |
| 存在可延期债务且不触底线 | `通过` 或 `需修改` | 取决于是否仍需本轮回修 |

### 4.14 真人裁决触发条件

以下情况**必须**升级真人：

- 三方对方向/范围/验收标准无法一致
- Collie 标记核心价值阻塞，Husky 不接受
- Teddy 标记安全/隐私/数据丢失/关键证据阻塞，Husky 不接受
- 需要牺牲用户价值换进度
- 需要把 `blocking` 降级为技术债
- 任何与 soul.md"方向/取舍/标准最终权在用户"相关的问题

真人裁决后，Husky 执行裁决。Collie 与 Teddy 后续只审查执行是否符合裁决，不继续争论裁决本身。

### 4.15 Gate 节点精简验证模式

Gate 节点（`hf-regression-gate`、`hf-doc-freshness-gate`、`hf-completion-gate`）与 review 节点有本质差异，不适用完整的论证型评审机制，采用**精简验证模式**：

#### 4.15.1 Gate vs Review 差异

| 维度 | Review 节点 | Gate 节点 |
|------|------------|-----------|
| 关注点 | 工件质量（好不好） | 条件验证（满足没有） |
| 输入 | 被评审工件本身 | 上游 review/gate 记录 + 实际运行证据 |
| 评审本质 | 论证型（多角度碰撞） | 验证型（条件逐条检查） |
| 结论类型 | `通过/需修改/阻塞` + findings | `通过/需修改/阻塞` + fresh evidence（`hf-doc-freshness-gate` 例外：`pass/partial/N/A/blocked`） |
| 分歧处理 | 可妥协、可登记技术债 | **不妥协**，条件不满足即回退上游 |

#### 4.15.2 Gate 精简规则

**豁免的机制**：
- 不需要"反方立场"（§ 4.7）——gate 是条件检查，不存在"论证碰撞"的空间
- 不需要"分歧发现量化"（§ 4.8）——gate 结论基于 fresh evidence，不受主观评分影响
- 不走"技术债妥协"路径（§ 4.11）——gate 不通过 = 证据不足或条件不满足，必须回退上游补充，不可延期

**保留的机制**：
- Review Briefing（简化版：gate 的输入是上游记录和证据，不是被评审工件）
- 角色锚定清单（gate 专用精简版，见下）
- Finding 严重度（§ 4.9）
- Husky feasibility response（回应 Collie/Teddy 对 gate 证据的质疑）
- Resolution（使用 gate 专用模板）

#### 4.15.3 Gate 角色锚定清单

**通用项**：

```markdown
### Gate 角色锚定检查（精简版）

- [ ] 已读取上游 review/gate 记录作为输入（不是被评审工件）
- [ ] 已读取 Review Briefing 中的 gate 验证范围
- [ ] 每条 finding 已锚定到具体证据路径（测试输出、运行日志、验证命令结果）
- [ ] 不跨权评审
```

**Teddy（质量）gate 专属项**：

```markdown
- [ ] 上游 review/gate 记录是否齐全且结论一致
- [ ] fresh evidence 是否在当前会话内实际产生（不接受历史结果）
- [ ] 回归/验证范围是否与 traceability review 结论对齐
```

**Husky（可行性回应）gate 专属项**：

```markdown
- [ ] gate 不通过时，是否明确回退上游节点（不妥协、不登记技术债）
- [ ] evidence 缺口是否有明确的补充路径
```

#### 4.15.4 Gate Resolution 裁决规则

Gate 节点的 Resolution 不走"妥协→技术债"路径：

| Panel 结果 | HF conclusion | 下一步 |
|------------|---------------|--------|
| 条件全部满足，fresh evidence 齐全 | `通过` | 按迁移表进入下一节点 |
| evidence 部分缺失或命令有失败项，但可修复 | `需修改` | 回退上游节点补充证据（如 `hf-test-driven-dev`） |
| 环境或工具链阻塞导致验证无法完成 | `阻塞` | 回退本 gate 重试 |
| 证据冲突或阶段不清 | `阻塞` | `hf-workflow-router` |

> gate 不通过的 finding 不可降级为技术债。gate 是确定性条件检查，条件不满足就是事实，不能通过"妥协"绕过。
> `hf-doc-freshness-gate` 例外：该 gate 使用 `pass/partial/N/A/blocked` 词汇（FR-002 约束），不使用 `通过/需修改/阻塞`。

#### 4.15.5 Gate Panel 执行时机

Multi-Agent gate panel 是对 HF-native gate 的**二次项目级审查**，不替代 gate 执行：

- **有 subagent 的 gate**（`hf-doc-freshness-gate`）：Panel 在 HF subagent 完成 gate record 后执行审查。
- **无 subagent 的 gate**（`hf-regression-gate`、`hf-completion-gate`）：HF-native gate 由父会话在当前上下文内执行（运行命令、产生 verification record）。Panel 在 gate record 生成后执行审查。
- Panel 若判定证据不足，Resolution 可覆盖后续路由（如回退上游），但**不得伪造或修改 native gate evidence**。
- Panel 的 Collie/Teddy gate review 读取 gate record 和 fresh evidence 作为输入，不替代 gate 的命令执行和证据产生。

### 4.16 Resolution 与 Return Contract 的关系

Resolution 是项目级 Multi-Agent Panel 的汇总工件，**不是** HF 框架认知的 `reviewer-return-contract.md`。在"不修改 HF 框架"约束下，router 不直接消费 Resolution 文件。

**实际流程**：

```
HF subagent → return contract (分支 A，仅限有 subagent 的节点)
Collie/Teddy → Panel reviews (分支 B/C)
Husky → feasibility response
→ 真人汇总 → Resolution（项目级工件，落盘到 features/<active>/reviews/ 或 docs/reviews/）
→ 父会话从 Resolution 提炼结论，按 reviewer-return-contract.md 格式构建 return contract
→ router 消费 return contract，按迁移表进入下一 canonical 节点
→ 若 return contract 中 needs_human_confirmation=true，父会话执行 approval step 后再继续
```

**映射规则**：

| Resolution 字段 | Return Contract 字段 | 说明 |
|-----------------|---------------------|------|
| Resolution → HF Verdict → conclusion | conclusion | 直接映射 |
| Resolution → HF Verdict → next_action | next_action_or_recommended_skill | 直接映射 |
| Resolution 路径 | record_path | Resolution 文件路径 |
| Resolution → key_findings | key_findings | 直接映射 |
| 按节点类型查表 | needs_human_confirmation | 见下方表格 |
| Resolution 是否涉及 reroute | reroute_via_router | 触发条件见 § 4.13 |

**`needs_human_confirmation` 按节点类型**（对齐 `reviewer-return-contract.md` 行 91-98）：

| 节点 | conclusion=通过 时 | 说明 |
|------|-------------------|------|
| `hf-spec-review` | `true` | 需规格真人确认 |
| `hf-design-review` | `true` | 需设计真人确认 |
| `hf-tasks-review` | `true` | 需任务真人确认 |
| `hf-ui-review` | `true` | UI surface 激活时与 hf-design-review 联合设计真人确认（见 review-dispatch-protocol § 联合 design approval） |
| `hf-discovery-review` | `false` | HF 无 canonical approval step；项目级 panel hard stop / user checkpoint 由真人自行执行 |
| `hf-test-review` | `false` | |
| `hf-code-review` | `false` | |
| `hf-traceability-review` | `false` | |
| 所有 gate 节点 | `false` | gate 不需要 approval step |

**`hf-doc-freshness-gate` return contract 特例**：

该 gate 通过 `references/reviewer-dispatch-handoff.md` 使用 adapted return contract，verdict 词表为 `pass/partial/N/A/blocked`（FR-002），含额外的 `dimension_breakdown` 字段。Resolution 提炼 return contract 时**必须保留该 gate 原生 verdict 词表**，不得转换为 `通过/需修改/阻塞`。

### 5.1 新增文件

| 文件路径 | 用途 | 优先级 |
|----------|------|--------|
| `docs/agent-configs/multi-agent-review-panel.md` | 多 Agent 评审协议主文档（合并 codex-draft 核心内容） | P0 |
| `docs/agent-configs/review-context-pack.md` | Review Context Pack 规则，定义常规 supporting context 与 fallback context | P0 |
| `docs/agent-configs/templates/review-briefing-template.md` | Review Briefing 模板 | P0 |
| `docs/agent-configs/templates/collie-review-template.md` | Collie 评审文档模板（含锚定清单 + 反方立场 + 分歧量化） | P0 |
| `docs/agent-configs/templates/teddy-review-template.md` | Teddy 评审文档模板（含锚定清单 + 反方立场 + 分歧量化 + 串行互读） | P0 |
| `docs/agent-configs/templates/husky-response-template.md` | Husky 可行性回应模板 | P0 |
| `docs/agent-configs/templates/resolution-template.md` | Resolution 模板（review 节点用）；承担早期方案中的 `review-resolution-template.md` 职责，避免重复模板漂移 | P0 |
| `docs/agent-configs/templates/gate-review-template.md` | Gate 节点精简评审模板（Collie/Teddy/Teddy 共用，无反方立场和分歧量化） | P0 |
| `docs/agent-configs/templates/gate-resolution-template.md` | Gate 节点专用 Resolution 模板（通过/需修改/阻塞，无技术债妥协路径；doc-freshness-gate 例外用 pass/partial/N/A/blocked） | P0 |
| `docs/agent-configs/templates/disagreement-record-template.md` | 分歧记录模板 | P0 |
| `docs/agent-configs/templates/tech-debt-template.md` | 技术债注册模板 | P0 |
| `docs/TECH-DEBT.md` | 项目技术债总表 | P1 |
| `docs/agent-configs/changelog.md` | Agent 配置变更记录 | P2 |
| `skills/goguo-review-orchestrator/SKILL.md` | 项目级评审编排 Skill（可选，P1 阶段创建） | P1 |

### 5.2 修改文件

| 文件路径 | 修改内容 | 优先级 |
|----------|----------|--------|
| `AGENTS.md` | § 6 后增加 § 7 多 Agent 协作架构声明 | P0 |
| `docs/agent-configs/Husky-codex-goguo.md` | (1) 参与矩阵 (2) 工件发现路径 (3) Briefing 产出规范 (4) 暂停信号行为 (5) 可行性回应模板引用 (6) 统一命名 | P0 |
| `docs/agent-configs/Collie-opencode-goguo.md` | (1) 参与矩阵 (2) 工件发现路径 (3) 锚定清单引用 (4) 反方立场必填要求 (5) 统一命名（补时分） | P0 |
| `docs/agent-configs/Teddy-claude-goguo.md` | (1) 参与矩阵 (2) 工件发现路径 (3) 锚定清单引用 (4) 反方立场必填要求 (5) 统一命名（R3→Teddy） | P0 |

### 5.3 不修改的文件

`skills/hf-*` 下所有文件、`docs/principles/` 下所有文件、`skills/using-hf-workflow/`、`skills/hf-workflow-router/` —— 均不做任何修改。

---

## 6. 实施步骤

### Step 1：创建项目级协议主文档（P0）

将本方案 § 4.1-4.14 的核心协议内容写入 `docs/agent-configs/multi-agent-review-panel.md`。

### Step 2：创建模板文件（P0）

创建 `docs/agent-configs/review-context-pack.md` 与 `docs/agent-configs/templates/` 下的 7 个模板文件。

### Step 3：修改三份 Agent 配置（P0）

按 § 5.2 修改 Husky / Collie / Teddy 配置。

### Step 4：更新 AGENTS.md（P0）

增加 § 7 多 Agent 协作架构声明。

### Step 5：创建技术债文件（P1）

创建 `docs/TECH-DEBT.md`，初始为空表头。

### Step 6：创建项目级 Skill（P1，可选）

创建 `skills/goguo-review-orchestrator/SKILL.md`，将模板和 prompt 组织为可调用的 Skill。

### Step 7：试运行（P1）

选择 `hf-spec-review` 首次试运行，验证：
1. Briefing 质量
2. 锚定清单覆盖度
3. 反方立场实质度
4. 分歧发现有效性
5. Resolution 可操作性

### Step 8：根据试运行微调（P2）

调整模板、清单、参与矩阵，更新 `changelog.md`。

---

## 7. 关键约束（重申）

- 多 Agent 评审**不替代** HF review/gate 节点
- 多 Agent 评审**不替代**真人 approval step
- Collie/Teddy 不参与实施节点时，**必须**通过 Briefing 恢复上下文
- 任何 reviewer 不得用无证据判断替代工件证据
- 技术债**不是**绕过质量的口子，只能用于不触碰底线且影响范围明确的延期项
- 当争议触及方向/取舍/标准，**必须**回到真人

---

## 8. 可落盘文档实例

本节只补齐落地所需的文件实例，不改变前文方案骨架。实际创建文件时，复制对应文本块内容并去掉外层围栏。

### 8.1 `docs/agent-configs/multi-agent-review-panel.md`

````markdown
# Multi-Agent Review Panel

## Purpose

本文件定义 goguo 项目在 HF review/gate 节点启用 Husky、Collie、Teddy 三方评审面板的项目级规则。

本机制是项目级 adapter，不修改 HF canonical route，不新增 HF 主链节点，不替代真人 approval step。

## Relationship With HF

- HF router 仍是 runtime authority。
- `hf-*review` 与 `hf-*gate` 仍是 canonical 节点。
- Multi-Agent Review Panel 通过项目级外围 adapter 挂接，不修改 `skills/hf-*`。
- `resolution-<节点>-YYYYMMDD-HHMM.md` 是 Panel 汇总工件；父会话从中提炼 return contract 交给 router。
- `HF Verdict` 必须兼容 `skills/hf-workflow-router/references/reviewer-return-contract.md`。

## Participation Matrix

| HF 评审 / 门禁节点 | Collie | Teddy | Husky |
|---|---|---|---|
| `hf-discovery-review` | 必须 | 可选 | 必须 |
| `hf-spec-review` | 必须 | 必须 | 必须 |
| `hf-design-review` | 必须 | 必须 | 必须 |
| `hf-ui-review` | 必须 | 必须 | 必须 |
| `hf-tasks-review` | 必须 | 必须 | 必须 |
| `hf-test-review` | 可选 | 必须 | 必须 |
| `hf-code-review` | 不参与 | 必须 | 必须 |
| `hf-traceability-review` | 可选 | 必须 | 必须 |
| `hf-regression-gate` | 不参与 | 必须 | 必须 |
| `hf-doc-freshness-gate` | 不参与 | 可选 | 必须 |
| `hf-completion-gate` | 可选 | 必须 | 必须 |

## Hooking Mode

```text
router -> canonical hf-*review/gate
  -> parent session applies goguo project-level adapter
  -> Husky writes Review Briefing
  -> Collie writes role review when required
  -> Teddy writes role review when required
  -> Husky writes feasibility response
  -> parent session writes resolution
  -> parent session extracts return contract from Resolution for router
```

## Evaluation Mode

节点的评审模式由节点类型决定：

| 节点类型 | 评审模式 | 模板 | 核心特征 |
|----------|----------|------|----------|
| review | 完整评审模式 | `collie-review-template.md` / `teddy-review-template.md` | 含反方立场、分歧发现量化、技术债妥协 |
| gate | 精简验证模式 | `gate-review-template.md` | 无反方立场、无分歧量化、不妥协；结论同 review 使用 `通过/需修改/阻塞`（doc-freshness-gate 例外） |

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
3. 填写触发修复条件。
4. 已落盘到 `docs/TECH-DEBT.md`。
5. 已声明偿还入口：`hf-increment` 或 `hf-hotfix`。
6. 目标迭代不超出 strategy-discovery 确定的 OPP 序列范围。

## Human Escalation

以下情况必须升级真人：

- 三方对方向、范围、验收标准无法一致。
- Collie 标记核心价值阻塞，Husky 不接受。
- Teddy 标记安全、隐私、数据丢失、关键证据阻塞，Husky 不接受。
- 需要牺牲用户价值换进度。
- 需要把 `blocking` 降级为技术债。
- 任何触及 `docs/principles/soul.md` 中“方向、取舍、标准最终权在用户”的问题。
````

### 8.2 `docs/agent-configs/templates/review-briefing-template.md`

````markdown
# Review Briefing - <节点名>

## 1. Metadata

| Field | Value |
|---|---|
| Feature | `<features/<active>>` 或 `N/A（discovery 阶段，feature 尚未创建）` |
| HF Node | `<hf-*review / hf-*gate>` |
| Profile | `full` |
| Author | Husky |
| Date | `YYYY-MM-DD HH:MM` |

## 2. 当前工件

| Artifact | Path | Status |
|---|---|---|
| 主工件 |  |  |
| 支撑工件 |  |  |
| 最新 evidence |  |  |

## 3. 上游关键决策

| 决策 ID | 内容 | 原因 | 来源工件 |
|---|---|---|---|

## 4. 本轮评审焦点

### 必答

- 
- 

### 可选

- 

## 5. 给 Collie 的定向提示

- 
- 

## 6. 给 Teddy 的定向提示

- 
- 

## 7. 已知争议 / 风险

| ID | 描述 | 期望 reviewer 关注 |
|---|---|---|
````

### 8.3 `docs/agent-configs/review-context-pack.md`

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

### Feature 阶段（`hf-spec-review` 及之后）

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

### Discovery 阶段（`hf-discovery-review`）

`features/<active>/` 尚不存在，使用以下替代上下文：

```text
docs/agent-configs/<Agent>-*.md
README.md
CHANGELOG.md
docs/architecture.md
docs/insights/
docs/reviews/
docs/TECH-DEBT.md
当前被评审的 discovery 草稿路径（由 Review Briefing 指定）
```

## HF Review Request Shape

```json
{
  "review_type": "spec|design|ui|tasks|test|code|traceability|regression|doc-freshness|completion|discovery",
  "review_skill": "hf-xxx-review or hf-xxx-gate",
  "topic": "<node> + <Agent role>",
  "artifact_paths": [
    "当前被评审的主工件"
  ],
  "supporting_context_paths": [
    "docs/agent-configs/<Agent>-*.md",
    "features/<active>/progress.md  (feature阶段) 或 docs/insights/ (discovery阶段)",
    "features/<active>/spec.md      (feature阶段) 或 当前discovery草稿 (discovery阶段)",
    "features/<active>/design.md    (feature阶段，discovery阶段 N/A)",
    "features/<active>/tasks.md     (feature阶段，discovery阶段 N/A)",
    "features/<active>/reviews/     (feature阶段) 或 docs/reviews/ (discovery阶段)",
    "features/<active>/verification/ (feature阶段，discovery阶段 N/A)",
    "features/<active>/evidence/    (feature阶段，discovery阶段 N/A)"
  ],
  "expected_record_path": "各 hf-*-review skill 的 review-record-template 自行定义命名（feature阶段落到 features/<active>/reviews/，discovery阶段落到 docs/reviews/）",
  "current_profile": "full",
  "design_execution_mode": "parallel|architecture-first|ui-first"
}
```

## Evidence Rules

- 缺少当前节点必需上下文时，reviewer 不得脑补，应返回 `阻塞` 或 `需补上下文`。
- Finding 必须引用具体工件路径、章节、需求 ID、测试记录或 evidence 路径。
- 没有证据的问题只能写为“证据缺口”，不能写成事实断言。
````

### 8.4 `docs/agent-configs/templates/collie-review-template.md`

````markdown
# Review - <节点名> - Collie

## Metadata

| Field | Value |
|---|---|
| Review Node |  |
| Reviewed Artifact |  |
| Briefing |  |
| Date |  |

## 角色锚定检查

- [ ] 已确认项目级注入上下文已加载；若无法确认，已读取 fallback context。
- [ ] 已读取 Review Briefing 中的上游关键决策。
- [ ] 每条 finding 已锚定到工件具体章节 / 条目。
- [ ] 不跨权评审；技术实现问题只标注“建议 Husky/Teddy 关注”。
- [ ] 本工件是否与当前轮 wedge 对齐。
- [ ] 验收场景 AC01-AC04 是否仍可覆盖。
- [ ] 是否触碰 Collie 底线：误伤直连、增加配置负担、偏离核心价值、验收不可观察。

## 总体结论

通过 / 需修改 / 阻塞

## 价值 Findings

| ID | Severity | Finding | Evidence Anchor | Required Action |
|---|---|---|---|---|

## 反方立场（必填）

假设你完全反对本工件方案，给出至少 3 条最强有力的反对理由。

| 反对理由 | 依据 | 强度 | 角色立场回应 | 处理 |
|---|---|---|---|---|

## 分歧发现量化（必填）

| 分歧点 | Collie 立场 | 预估 Teddy/Husky 立场 | 分歧强度(1-10) | 置信度 | 理由 |
|---|---|---|---:|---|---|

## 可妥协项

| Finding | 妥协条件 | 是否触碰红线 | 技术债 ID |
|---|---|---|---|

## 不可妥协项

- 
````

### 8.5 `docs/agent-configs/templates/teddy-review-template.md`

````markdown
# Review - <节点名> - Teddy

## Metadata

| Field | Value |
|---|---|
| Review Node |  |
| Reviewed Artifact |  |
| Briefing |  |
| Collie Review |  |
| Date |  |

## 角色锚定检查

- [ ] 已确认项目级注入上下文已加载；若无法确认，已读取 fallback context。
- [ ] 已读取 Review Briefing 中的上游关键决策。
- [ ] 已读取 Collie review，并明确回应其中与质量 / 测试相关的观点。
- [ ] 每条 finding 已锚定到工件具体章节 / evidence。
- [ ] 不跨权评审；价值范围问题只标注“建议 Collie 关注”。
- [ ] 是否存在无 fresh evidence 的关键断言。
- [ ] 是否存在安全 / 隐私红线被触碰的迹象。
- [ ] 与 `docs/TECH-DEBT.md` 中已有债务的交互风险是否评估。

## 总体结论

通过 / 需修改 / 阻塞

## 质量 Findings

| ID | Severity | Finding | Evidence Gap | Required Action |
|---|---|---|---|---|

## 对 Collie 结论的回应

| Collie Finding | Teddy Response | 是否形成分歧 | 分歧强度(1-10) |
|---|---|---|---:|

## 反方立场（必填）

| 反对理由 | 依据 | 强度 | 角色立场回应 | 处理 |
|---|---|---|---|---|

## 分歧发现量化（必填）

| 分歧点 | Teddy 立场 | Collie/Husky 立场 | 分歧强度(1-10) | 置信度 | 理由 |
|---|---|---|---:|---|---|

## 可妥协项

| Finding | 替代验证证据 | 是否触碰红线 | 技术债 ID |
|---|---|---|---|

## 不可妥协项

- 
````

### 8.6 `docs/agent-configs/templates/husky-response-template.md`

````markdown
# Feasibility Response - <节点名> - Husky

## Metadata

| Field | Value |
|---|---|
| Review Node |  |
| Briefing |  |
| Collie Review |  |
| Teddy Review |  |
| Date |  |

## 对 Collie Findings 的回应

| Finding ID | 接受 / 反驳 / 延期 / 升级真人 | 成本 | 处理计划 | Evidence |
|---|---|---|---|---|

## 对 Teddy Findings 的回应

| Finding ID | 接受 / 反驳 / 延期 / 升级真人 | 成本 | 处理计划 | Evidence |
|---|---|---|---|---|

## 技术债登记建议

| Debt ID | 来源 Finding | 描述 | 影响 | 触发修复条件 | 偿还入口 | 目标迭代 | 阻塞 |
|---|---|---|---|---|---|---|---|

## 需要真人裁决的问题

| ID | 冲突 | Husky 最终立场 | 请求真人裁决点 |
|---|---|---|---|

## Husky 最终建议

通过 / 需修改 / 阻塞 / 真人裁决
````

### 8.7 `docs/agent-configs/templates/resolution-template.md`

`````markdown
# Resolution - <节点名>

## 输入工件

- HF-native review: （由 HF subagent 的 record_path 返回值填充；对 hf-regression-gate / hf-completion-gate 填 N/A）
- Review Briefing:
- Collie review:
- Teddy review:
- Husky feasibility response:

## 四方一致点

-

## HF-native subagent 与 Panel 结论差异

| 差异点 | HF subagent 结论 | Panel 综合结论 | 差异原因 | 真人判定 |
|---|---|---|---|---|

## 分歧点（含量化评分）

| 分歧点 | Collie 评分 | Teddy 评分 | Husky 立场 | 红线检查 | 处理 |
|---|---:|---:|---|---|---|

## 红线检查

| 红线类型 | 是否触发 | 证据 | 处理 |
|---|---|---|---|
| soul.md 底线 | 否 |  |  |
| 安全红线 | 否 |  |  |
| 架构红线 | 否 |  |  |
| 性能红线 | 否 |  |  |
| 证据红线 | 否 |  |  |

## 已接受修订

- 

## 已延期技术债

| Debt ID | 来源 Finding | 延期理由 | 触发条件 | 偿还入口 | 目标迭代 |
|---|---|---|---|---|---|

## 真人裁决（如有）

- 裁决结论：
- 裁决理由：

## HF Verdict

```json
{
  "conclusion": "通过|需修改|阻塞",
  "next_action_or_recommended_skill": "<canonical hf node>",
  "record_path": "features/<active>/reviews/resolution-<节点>-YYYYMMDD-HHMM.md  (feature阶段) 或 docs/reviews/resolution-<节点>-YYYYMMDD-HHMM.md  (discovery阶段)",
  "key_findings": [
    "关键发现 1",
    "关键发现 2"
  ],
  "needs_human_confirmation": "按 § 4.16 节点类型取 true/false",
  "reroute_via_router": false
}
```

## Return Contract 映射

父会话从本 Resolution 提炼以下字段，按 `reviewer-return-contract.md` 格式构建 return contract：

| 字段 | 来源 | 说明 |
|------|------|------|
| conclusion | HF Verdict → conclusion | 直接映射 |
| next_action_or_recommended_skill | HF Verdict → next_action | 直接映射 |
| record_path | 本 Resolution 文件路径 | |
| key_findings | HF Verdict → key_findings | 直接映射 |
| needs_human_confirmation | 按 § 4.16 表格查表 | spec/design/tasks/ui-review = true，其余 = false |
| reroute_via_router | HF Verdict → reroute_via_router | 直接映射 |
`````

### 8.7.5 `docs/agent-configs/templates/gate-review-template.md`

````markdown
# Gate Review - <节点名> - <Agent Role>

> 本模板仅用于 gate 节点（`hf-regression-gate`、`hf-doc-freshness-gate`、`hf-completion-gate`）。
> 不含反方立场、分歧发现量化等论证型章节。gate 是条件验证，不是论证碰撞。

## Metadata

| Field | Value |
|---|---|
| Gate Node |  |
| Agent Role | Collie / Teddy / Husky |
| Upstream Records |  |
| Date |  |

## Gate 角色锚定检查（精简版）

- [ ] 已读取上游 review/gate 记录作为输入（不是被评审工件）。
- [ ] 已读取 Review Briefing 中的 gate 验证范围。
- [ ] 每条 finding 已锚定到具体证据路径（测试输出、运行日志、验证命令结果）。
- [ ] 不跨权评审；非本角色职责范围问题只标注"建议 XX 关注"。

### Teddy 专属项（若角色为 Teddy）

- [ ] 上游 review/gate 记录是否齐全且结论一致。
- [ ] fresh evidence 是否在当前会话内实际产生。
- [ ] 回归/验证范围是否与 traceability review 结论对齐。

## Gate 条件检查

| 条件项 | 期望 | 实际 | Evidence | Pass / Fail |
|---|---|---|---|---|

## Findings

| ID | Severity | Finding | Evidence Path | Required Action |
|---|---|---|---|---|

## 总体结论

通过 / 需修改 / 阻塞

> **注意**：gate 节点不使用技术债妥协路径。`需修改` 表示证据有缺口但可修复，回退上游补充。`hf-doc-freshness-gate` 例外：使用 `pass / partial / N/A / blocked`（FR-002）。
````

### 8.7.6 `docs/agent-configs/templates/gate-resolution-template.md`

````markdown
# Gate Resolution - <节点名>

> 本模板仅用于 gate 节点。gate 不走"妥协→技术债"路径。结论使用 `通过/需修改/阻塞`（与 review 相同）；`hf-doc-freshness-gate` 例外使用 `pass/partial/N/A/blocked`。

## 输入工件

- HF-native gate record: （对 hf-doc-freshness-gate：由 subagent 的 record_path 填充；对 hf-regression-gate / hf-completion-gate：由父会话执行 gate skill 产生的 verification record 填充，非 subagent）
- Review Briefing:
- Collie gate review（如参与）:
- Teddy gate review（如参与）:
- Husky feasibility response:

## 条件检查汇总

| 条件项 | HF subagent 判定 | Collie 判定 | Teddy 判定 | Husky 判定 | 最终 |
|---|---|---|---|---|---|

## 证据缺口（如有）

| 缺口 | 需要回退到的上游节点 | 补充路径 |
|---|---|---|

## HF Verdict

```json
{
  "conclusion": "通过|需修改|阻塞",
  "next_action_or_recommended_skill": "<canonical hf node>",
  "record_path": "features/<active>/reviews/gate-resolution-<节点>-YYYYMMDD-HHMM.md",
  "key_findings": [
    "关键发现 1"
  ],
  "needs_human_confirmation": false,
  "reroute_via_router": false,
  "evidence_gaps": []
}
```

> gate 的 `conclusion` 为 `通过`、`需修改`、`阻塞`，与 HF 框架 review 节点使用相同词汇。不使用技术债妥协路径。`hf-doc-freshness-gate` 例外：使用 `pass/partial/N/A/blocked`（FR-002）。
````

### 8.8 `docs/agent-configs/templates/disagreement-record-template.md`

````markdown
# Disagreement Record - <节点名> - <ID>

## 分歧摘要

| Field | Value |
|---|---|
| Disagreement ID |  |
| Source Node |  |
| Related Findings |  |
| Severity | blocking / important / debt-acceptable / minor |

## 各方立场

| Role | Position | Evidence |
|---|---|---|
| Collie |  |  |
| Teddy |  |  |
| Husky |  |  |

## 量化评分

| Dimension | Score(1-10) | Notes |
|---|---:|---|
| 用户价值影响 |  |  |
| 质量 / 安全影响 |  |  |
| 实现成本 |  |  |
| 延期风险 |  |  |

## 红线检查

| Red Line | Triggered | Evidence |
|---|---|---|

## 处理结论

接受 / 回修 / 技术债延期 / 真人裁决

## 技术债引用

Debt ID: N/A
````

### 8.9 `docs/agent-configs/templates/tech-debt-template.md`

````markdown
# Tech Debt Register

| ID | 来源 | 描述 | 影响 | 触发修复条件 | 偿还入口 | 目标迭代 | 责任人 | 阻塞 |
|---|---|---|---|---|---|---|---|---|
| TD-001 |  |  |  |  | hf-increment / hf-hotfix |  | Husky | 否 |

## Field Rules

- `偿还入口` 只能填写 `hf-increment` 或 `hf-hotfix`。
- 已造成缺陷、安全、隐私、数据风险的债务优先进入 `hf-hotfix`。
- 计划性改进、补测试、补性能 baseline、重构、补文档优先进入 `hf-increment`。
- 技术债不得触碰任何不可妥协底线。
````

### 8.10 `docs/TECH-DEBT.md`

````markdown
# Technical Debt

本文件记录 Multi-Agent Review Panel 接受延期的非阻塞技术债。技术债登记不等于允许绕过质量门禁；偿还时必须经 `hf-workflow-router` 判断进入 `hf-increment` 或 `hf-hotfix`。

| ID | 来源 | 描述 | 影响 | 触发修复条件 | 偿还入口 | 目标迭代 | 责任人 | 阻塞 |
|---|---|---|---|---|---|---|---|---|
````

### 8.11 `docs/agent-configs/changelog.md`

````markdown
# Agent Configs Changelog

本文件记录 `docs/agent-configs/` 下项目级 Agent 配置与 Multi-Agent Review Panel 协议的变更。

| Date | Change | Files | Reason | Approved By |
|---|---|---|---|---|
| 2026-05-06 | Introduce Multi-Agent Review Panel | `multi-agent-review-panel.md`, `templates/*`, `docs/TECH-DEBT.md` | 将多厂家 Agent 评审制度化，并保持 HF 主流程不变 | 待真人确认 |
````

### 8.12 `skills/goguo-review-orchestrator/SKILL.md`

````markdown
---
name: goguo-review-orchestrator
description: 适用于 goguo 项目在 HF review/gate 节点需要组织 Husky、Collie、Teddy 多 Agent 评审面板、生成 Review Briefing、汇总 Resolution 的场景。不替代 hf-workflow-router，不修改 HF canonical route。
---

# Goguo Review Orchestrator

## Purpose

组织 goguo 项目级 Multi-Agent Review Panel。该 skill 只提供模板、prompt 与编排检查，不直接替代任何 `hf-*review` 或 `hf-*gate` 节点。

## Inputs

- 当前 HF node
- 当前 active feature（discovery 阶段为 N/A）
- 被评审工件路径
- `docs/agent-configs/multi-agent-review-panel.md`
- `docs/agent-configs/templates/`

## Workflow

1. 确认当前处于 HF review/gate 节点。
2. **解析工件路径阶段**：
   - 若当前节点为 `hf-discovery-review` → 评审上下文/输出落到 `docs/reviews/`，Context Pack 使用 discovery 阶段清单。
   - 其他节点 → 评审上下文/输出落到 `features/<active>/reviews/`，Context Pack 使用 feature 阶段清单。
3. **选择评审模式**：
   - **review 节点** → 完整评审模式（含反方立场、分歧发现量化、技术债妥协）。模板：`collie-review-template.md` / `teddy-review-template.md` / `resolution-template.md`。
   - **gate 节点** → 精简验证模式（无反方立场、无分歧量化、不妥协；结论用 `通过/需修改/阻塞`，doc-freshness-gate 例外）。模板：`gate-review-template.md` / `gate-resolution-template.md`。
4. 根据 HF 协议，由 Husky 主会话 fork reviewer subagent（执行对应的 `hf-spec-review` / `hf-design-review` / `hf-regression-gate` 等），产出 HF-native review/gate record。
5. 根据参与矩阵判断 Collie / Teddy 是否必须参与。
6. 生成 Review Briefing（discovery 阶段 Feature 字段填 `N/A`；gate 阶段简化为上游记录 + 验证范围）。
7. **并行阶段启动**（三项评审同时进行，无先后依赖）：
   - **分支 A**：Husky reviewer subagent 按原生 HF protocol 执行，产出 HF-native review/gate record。
   - **分支 B**：给出 Collie review request（按评审模式选择模板），Collie 产出价值评审。
   - **分支 C**：给出 Teddy review request（按评审模式选择模板；若串行互读启用，则带入 Collie review 路径），Teddy 产出质量评审。
8. 收集 Husky feasibility response（对 Collie/Teddy findings 的可行性回应）。
9. 使用对应 Resolution template 汇总**四类输入**：
   - 输入 1：HF-native review/gate record（Husky subagent 产出）
   - 输入 2：Collie review
   - 输入 3：Teddy review
   - 输入 4：Husky feasibility response
10. 输出兼容 `reviewer-return-contract.md` 的 HF Verdict（唯一 `next_action_or_recommended_skill`）。

## Hard Rules

- 不修改 HF router。
- 不修改 `skills/hf-*`。
- 不跳过真人 approval step。
- `resolution` 必须给出唯一 `next_action_or_recommended_skill`。
- gate 节点 `conclusion`：`hf-regression-gate` / `hf-completion-gate` 可用 `通过 / 需修改 / 阻塞`；`hf-doc-freshness-gate` 使用 `pass / partial / N/A / blocked`。所有 gate 不得走技术债妥协路径。
````

### 8.13 `AGENTS.md` 最小追加块

````markdown
## 7. Multi-Agent Review Panel（goguo 项目级 adapter）

- 本项目在 HF review/gate 节点启用 Multi-Agent Review Panel。
- 该机制为项目级外围 adapter，不修改 HF canonical route，不替代 `hf-*review` / `hf-*gate`，不替代真人 approval step。
- Husky 为主执行者，Collie 为价值评审者，Teddy 为质量评审者。
- **并行四输入模型**：Husky 主会话 fork 的 HF-native reviewer subagent 与 Collie/Teddy 评审并行执行；Resolution 汇总四类输入（HF-native review record、Collie review、Teddy review、Husky feasibility response），输出唯一 HF Verdict。
- 参与矩阵、模板、分歧处理、技术债规则以 `docs/agent-configs/multi-agent-review-panel.md` 为准。
- `resolution-<节点>-YYYYMMDD-HHMM.md` 是 Panel 汇总工件；父会话从中提炼 return contract 交给 router。
- **阶段感知路径**：discovery 阶段（`hf-discovery-review`）评审输出落到 `docs/reviews/`；feature 阶段（`hf-spec-review` 及之后）评审输出落到 `features/<active>/reviews/`。
- **评审模式区分**：review 节点使用完整评审模式（含反方立场、分歧量化、技术债妥协）；gate 节点使用精简验证模式（不妥协，结论同 review 词汇，详见 `multi-agent-review-panel.md` § Evaluation Mode）。
- 技术债统一记录到 `docs/TECH-DEBT.md`，偿还入口必须声明为 `hf-increment` 或 `hf-hotfix`。
- `docs/agent-configs/` 为 Multi-Agent Review Panel 的配置与模板目录（项目级档 2 自定义路径）。
````

### 8.14 Agent 配置最小追加块

#### `docs/agent-configs/Husky-codex-goguo.md`

> **整合说明**：以下内容追加到现有 §8 之后。现有 §5 技术债务记录规则改为引用 `docs/TECH-DEBT.md` + `tech-debt-template.md`（现有字段名需对齐新模板）。现有 §8.2 文件命名（`review-<节点名>-Husky-YYYYMMDD-HHMM.md`）是 Panel 评审文档命名；HF subagent 的 review 由各 `hf-*-review` skill 自行命名，两者不冲突。

````markdown
## Multi-Agent Review Panel 职责补充

- Husky 必须在进入 review/gate panel 前产出 Review Briefing。
- Husky 必须按 `husky-response-template.md` 回应 Collie / Teddy findings。
- Husky 不得自行压过 Collie 的核心价值阻塞或 Teddy 的安全 / 隐私 / 关键证据阻塞。
- Husky 可以提出成本、范围、技术可行性反驳，但触及方向 / 取舍 / 标准时必须升级真人。
- Husky 登记技术债时必须填写偿还入口：`hf-increment` 或 `hf-hotfix`。
- **gate 节点**：Husky 不得对 gate 不通过项登记技术债或提出妥协，必须回退上游补充证据。
````

#### `docs/agent-configs/Collie-opencode-goguo.md`

````markdown
## Multi-Agent Review Panel 职责补充

- Collie 只评审用户价值、范围、验收标准、零配置、不破坏直连和当前轮 wedge。
- Collie 不跨权评审底层实现；发现实现风险时标注“建议 Teddy/Husky 关注”。
- Collie review 必须包含角色锚定检查、反方立场、分歧发现量化（gate 节点使用精简版，不含后两项）。
- Collie 的核心价值阻塞若被 Husky 不接受，必须升级真人裁决。
````

#### `docs/agent-configs/Teddy-claude-goguo.md`

````markdown
## Multi-Agent Review Panel 职责补充

- Teddy 只评审测试策略、fresh evidence、性能、安全、隐私、回归和质量门禁。
- Teddy 不跨权决定产品范围；发现价值风险时标注”建议 Collie 关注”。
- Teddy review 必须包含角色锚定检查、反方立场、分歧发现量化（gate 节点使用精简版，不含后两项）。
- Teddy 在串行互读模式下必须读取 Collie review，并明确回应与质量相关的观点。
- Teddy 的安全 / 隐私 / 关键证据阻塞若被 Husky 不接受，必须升级真人裁决。
- **gate 节点**：Teddy 的 finding 必须锚定到具体 evidence 路径（测试输出、运行日志），不接受无证据阻塞。gate 结论使用 `通过/需修改/阻塞`（doc-freshness-gate 例外）。
````

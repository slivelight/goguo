# Goguo 多 Agent 评审最终方案

> 本文档是 codex-draft、claude-draft、opencode-draft 三份方案的取舍合并结果。
> 经三个 Agent 平台（Codex / OpenCode / Claude）各自提出方案后，由 Claude 汇总比较，
> 最终方案经真人确认后生效。

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
- Approval Step 暂停信号（Husky 主动提示真人调 Collie/Teddy）
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
2. **Agent 配置文件**（`docs/agent_configs/`）— 定义角色、检查清单、行为规范
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

| HF 评审 / 门禁节点 | Collie（价值） | Teddy（质量） | Husky（可行性回应） |
|--------------------|---------------|--------------|-------------------|
| `hf-discovery-review` | 必须 | 可选 | 必须 |
| `hf-spec-review` | 必须 | 必须 | 必须 |
| `hf-design-review` | 必须 | 必须 | 必须 |
| `hf-ui-review`（仅 UI surface 激活） | 必须 | 必须 | 必须 |
| `hf-tasks-review` | 必须 | 必须 | 必须 |
| `hf-test-review` | 可选 | 必须 | 必须 |
| `hf-code-review` | 不参与 | 必须 | 必须 |
| `hf-traceability-review` | 可选 | 必须 | 必须 |
| `hf-regression-gate` | 不参与 | 必须 | 必须 |
| `hf-doc-freshness-gate` | 不参与 | 可选 | 必须 |
| `hf-completion-gate` | 可选 | 必须 | 必须 |

### 4.4 评审执行流程

```
HF workflow 进入 review/gate 节点
    │
    ▼
Husky 产出工件 + Review Briefing
    │
    ▼
Husky 在 approval step 暂停，输出调人信号
    │
    ▼
真人切换到 Collie（opencode）
    读：Context Pack + Review Briefing + 被评审工件
    执行：角色锚定清单 + 强制反方立场 + 分歧发现量化
    写：review_<节点>_Collie_YYYYMMDD_HHMM.md
    │
    ▼
真人切换到 Teddy（claude）
    读：Context Pack + Review Briefing + 被评审工件 + Collie 评审文档
    执行：角色锚定清单 + 强制反方立场 + 分歧发现量化
    对 Collie 结论冲突点：填入分歧记录
    写：review_<节点>_Teddy_YYYYMMDD_HHMM.md
    │
    ▼
真人读取三方评审文档
    检查：锚定清单完整性 / 反方立场实质度 / 分歧记录质量
    对分歧点量化评分（采纳 opencode 1-10 分制）
    执行红线检查（采纳 opencode 红线定义）
    裁决：共识 → 通过 | 分歧 → 妥协或升级
    写：resolution_<节点>_YYYYMMDD_HHMM.md
    │
    ▼
Husky 按 Resolution 执行
    修改工件（如有 ❌ 项）→ 重新提交
    或继续推进（全部通过 / 有条件通过已处理）
    │
    ▼
hf-workflow-router 消费 Resolution 中的 HF Verdict
    按迁移表进入下一 canonical 节点
```

### 4.5 上下文注入机制（两层）

**第一层：固定 Review Context Pack**（源自 codex-draft § 6）

每次评审的固定最小上下文，所有评审者都读取：

```text
AGENTS.md
docs/principles/soul.md
docs/principles/methodology-coherence.md
docs/principles/sdd-artifact-layout.md
docs/principles/hf-sdd-tdd-skill-design.md
docs/principles/architectural-health-during-tdd.md
docs/principles/emergent-vs-upfront-patterns.md
docs/agent_configs/<Agent>_*.md
features/<active>/README.md
features/<active>/progress.md
features/<active>/spec.md
features/<active>/design.md
features/<active>/tasks.md
features/<active>/reviews/
docs/TECH_DEBT.md
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

- [ ] 已读取 Review Context Pack 中的项目核心文档
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
- [ ] 与 TECH_DEBT.md 中已有债务的交互风险是否评估？
```

**节点专属补充项**（存放在 `goguo-review-orchestrator/templates/` 下，按需加载）：

每个 HF review 节点可以有额外的专属检查项。初始阶段只创建核心节点的清单（spec-review、design-review），其他节点在首次评审前按需补充。

### 4.7 强制反方立场

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
4. 已落盘到 `docs/TECH_DEBT.md`，绑定后续任务或版本
5. 目标迭代不得超出 strategy-discovery 确定的 OPP 序列范围

#### 技术债记录格式

```markdown
| ID | 来源 | 描述 | 影响 | 触发修复条件 | 目标迭代 | 责任人 | 阻塞 |
|----|------|------|------|-------------|----------|--------|------|
| TD-001 | Teddy review | ... | ... | 用户报告数 > X | OPP-004 | Husky | 否 |
```

### 4.12 Resolution 工件（源自 codex-draft § 11）

Resolution 是 Multi-Agent Review Panel 给 HF router 的**唯一汇总输入**。

```markdown
# Resolution — <节点名>

## 输入工件
- Collie review: features/<active>/reviews/review_<节点>_Collie_YYYYMMDD_HHMM.md
- Teddy review: features/<active>/reviews/review_<节点>_Teddy_YYYYMMDD_HHMM.md
- Husky response: features/<active>/reviews/response_<节点>_Husky_YYYYMMDD_HHMM.md

## 三方一致点
-

## 分歧点（含量化评分）
| 分歧点 | Collie 评分 | Teddy 评分 | 红线检查 | 处理 |
|--------|-----------|-----------|----------|------|

## 已接受修订
-

## 已延期技术债
| Debt ID | 来源 finding | 延期理由 | 触发条件 | 目标迭代 |
|---------|-------------|----------|----------|----------|

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
  "record_path": "features/<active>/reviews/resolution_<节点>_YYYYMMDD_HHMM.md"
}
```
```

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

---

## 5. 落地文件清单

### 5.1 新增文件

| 文件路径 | 用途 | 优先级 |
|----------|------|--------|
| `docs/agent_configs/multi-agent-review-panel.md` | 多 Agent 评审协议主文档（合并 codex-draft 核心内容） | P0 |
| `docs/agent_configs/templates/review-briefing-template.md` | Review Briefing 模板 | P0 |
| `docs/agent_configs/templates/collie-review-template.md` | Collie 评审文档模板（含锚定清单 + 反方立场 + 分歧量化） | P0 |
| `docs/agent_configs/templates/teddy-review-template.md` | Teddy 评审文档模板（含锚定清单 + 反方立场 + 分歧量化 + 串行互读） | P0 |
| `docs/agent_configs/templates/husky-response-template.md` | Husky 可行性回应模板 | P0 |
| `docs/agent_configs/templates/resolution-template.md` | Resolution 模板 | P0 |
| `docs/agent_configs/templates/disagreement-record-template.md` | 分歧记录模板 | P0 |
| `docs/agent_configs/templates/tech-debt-template.md` | 技术债注册模板 | P0 |
| `docs/TECH_DEBT.md` | 项目技术债总表 | P1 |
| `docs/agent_configs/changelog.md` | Agent 配置变更记录 | P2 |
| `skills/goguo-review-orchestrator/SKILL.md` | 项目级评审编排 Skill（可选，P1 阶段创建） | P1 |

### 5.2 修改文件

| 文件路径 | 修改内容 | 优先级 |
|----------|----------|--------|
| `AGENTS.md` | § 6 后增加 § 7 多 Agent 协作架构声明 | P0 |
| `docs/agent_configs/Husky_codex_goguo.md` | (1) 参与矩阵 (2) 工件发现路径 (3) Briefing 产出规范 (4) 暂停信号行为 (5) 可行性回应模板引用 (6) 统一命名 | P0 |
| `docs/agent_configs/Collie_opencode_goguo.md` | (1) 参与矩阵 (2) 工件发现路径 (3) 锚定清单引用 (4) 反方立场必填要求 (5) 统一命名（补时分） | P0 |
| `docs/agent_configs/Teddy_claude_goguo.md` | (1) 参与矩阵 (2) 工件发现路径 (3) 锚定清单引用 (4) 反方立场必填要求 (5) 统一命名（R3→Teddy） | P0 |

### 5.3 不修改的文件

`skills/hf-*` 下所有文件、`docs/principles/` 下所有文件、`skills/using-hf-workflow/`、`skills/hf-workflow-router/` —— 均不做任何修改。

---

## 6. 实施步骤

### Step 1：创建项目级协议主文档（P0）

将本方案 § 4.1-4.14 的核心协议内容写入 `docs/agent_configs/multi-agent-review-panel.md`。

### Step 2：创建模板文件（P0）

创建 `docs/agent_configs/templates/` 下的 7 个模板文件。

### Step 3：修改三份 Agent 配置（P0）

按 § 5.2 修改 Husky / Collie / Teddy 配置。

### Step 4：更新 AGENTS.md（P0）

增加 § 7 多 Agent 协作架构声明。

### Step 5：创建技术债文件（P1）

创建 `docs/TECH_DEBT.md`，初始为空表头。

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

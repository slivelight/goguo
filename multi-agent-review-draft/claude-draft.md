# Goguo 多 Agent 评审可实施方案 — Claude 视角补充草案

> 本文档与 `codex-draft.md` 并行，基于 Claude 对多 Agent 评审机制的深度分析补充。
> codex-draft.md 已涵盖角色分工、适用节点、Resolution 工件、分歧处理等核心协议，
> 本文档重点补充：方案比较、评审上下文注入机制、角色锚定机制、反方立场协议，
> 以及完整的落地文件清单和实施路径。

---

## 1. 三方案比较与选择

### 1.1 方案定义

**方案一：修改 HF 框架以适应多 Agent 参与评审**

扩展 HF 框架本身的 review dispatch 机制，使 `hf-workflow-router` 原生支持多 Agent 并行/串行派发、结论聚合、共识检测和升级路由。涉及修改 `review-dispatch-protocol.md`、`profile-node-and-transition-map.md` 等 HF 核心文件，并新增 `hf-strategy-review` 等 canonical 节点。

**方案二：不修改 HF 框架，纯配置驱动**

HF 框架流程不变。Husky 在 codex 内完整运行 HF workflow。在 interactive 模式的 approval step 暂停点，真人手动调 Collie/Teddy 参与评审，真人汇总裁决后通知 Husky 继续。多 Agent 协作完全通过 agent 配置文件和项目级文档实现。

**方案三：项目级编排 Skill 包装（推荐）**

不修改任何 `hf-*` skill，但在项目中新建一个项目专属 skill（`skills/goguo-review-orchestrator/`），作为 HF review 节点之上的多 Agent 编排层。该 skill 不是 HF canonical 节点，不参与 router 迁移判断，而是由真人（或 Husky 在 approval step 暂停时）手动调用的便捷工具。同时继承方案二的所有 agent 配置工作。

### 1.2 多维度比较

#### 维度一：质量 — 拧毛巾张力效果

| 子维度 | 方案一（改 HF） | 方案二（纯配置） | 方案三（项目级 Skill） |
|--------|----------------|-----------------|---------------------|
| 多模型多样性 | 真正多平台 | 真正多平台 | 真正多平台 |
| 角色分离清晰度 | 框架强制分派 | 配置声明，靠真人执行 | Skill 模板化 prompt 确保角色清晰 |
| 交叉质疑深度 | 框架可编排串行互读 | 靠真人搬运信息，易跳过 | Skill 定义互读顺序模板 |
| 共识检测严谨性 | 框架自动检测 | 真人目视判断 | Skill 提供聚合检查清单 |
| 评审遗漏风险 | 低（框架不跳节点） | 中高（真人疲劳跳过） | 中低（Skill 有检查清单） |

#### 维度二：成本 — 协同效率

| 子维度 | 方案一（改 HF） | 方案二（纯配置） | 方案三（项目级 Skill） |
|--------|----------------|-----------------|---------------------|
| 实施成本 | 高：修改 5+ 个 HF 核心文件 | 低：只改 agent 配置 + 项目文档 | 中：新建 1 个项目级 skill + 配置 |
| 每次评审操作成本 | 低：Husky 自动派发 | 高：真人手动切换 3 个窗口 | 中：Skill 生成结构化 prompt |
| HF 框架升级维护 | 高：需合并冲突 | 低：直接覆盖 `skills/` | 低：覆盖 `hf-*` 不影响项目 skill |
| 新 Agent 加入成本 | 中：改 dispatch protocol | 低：新建 agent config | 低：新建 config + 更新 skill |
| 学习曲线 | 需理解 HF 扩展点 | 只需理解自己角色配置 | 需理解项目 skill + 角色配置 |

#### 维度三：范围 — 达成项目目标

| 子维度 | 方案一（改 HF） | 方案二（纯配置） | 方案三（项目级 Skill） |
|--------|----------------|-----------------|---------------------|
| HF 全节点覆盖 | 原生支持 | 理论全覆盖，依赖手动执行 | 理论全覆盖，Skill 按节点生成 prompt |
| 与 strategy-discovery 对齐 | 新增 strategy-review 节点 | 走现有 discovery-review + debate | 同方案二，不新增节点 |
| OPP-002→004→001 序列 | 框架级多特性循环 | HF 已支持多特性循环 | 同上 |
| 可扩展到未来项目 | 通用化改造可复用 | 配置模式可复制 | Skill 模式可复制 |

#### 维度四：额外考量

| 子维度 | 方案一（改 HF） | 方案二（纯配置） | 方案三（项目级 Skill） |
|--------|----------------|-----------------|---------------------|
| HF 框架纯净性 | 污染框架 | 框架保持原样 | 框架保持原样 |
| 容错性 | 需异常处理 | 真人灵活调度 | 真人灵活调度 |
| 可追溯性 | 框架自动记录 | 依赖文档结构化程度 | Skill 提供标准模板 |
| 迭代节奏匹配 | 重流程拖慢迭代 | 最轻 | 适中 |

### 1.3 推荐结论

**推荐方案三（项目级编排 Skill）+ 方案二核心配置**，理由：

1. **HF 框架升级兼容性是硬约束**。HF 框架源自参考项目，未来必然持续迭代。方案一把多 Agent 逻辑焊进框架核心文件，每次更新有合并冲突风险。
2. **拧毛巾的核心价值在于多模型多样性，不在于框架级自动化**。方案三通过 skill 提供结构化 prompt 模板和聚合检查清单，降低真人操作成本，又不需要改框架。
3. **方案二的操作成本是实际痛点**。full profile 有 7 个 review + 3 个 gate 节点，三个 OPP 特性走一遍就是 30 次手动协调。方案三的 skill 把重复性操作模板化。
4. **方案一投入产出不匹配**。当前还在 product-discovery 阶段，先重后轻风险大。如果拧毛巾机制需要调整，方案三只改一个 skill 文件。
5. **保留方案二的灵活性**。项目级 skill 是可选的便捷工具，真人仍可按方案二手动操作。

---

## 2. 核心问题：评审上下文与角色遵循

### 2.1 问题一：评审者的项目针对性

Collie 和 Teddy 没有参与上游节点（strategy-discovery → product-discovery → specify → design → tasks），直接空降到 review 节点。它们不知道：

- 为什么选了 OPP-002 作为第一个特性
- spec 中某条 NFR 是因为什么折衷才写成那个阈值
- 设计时排除了哪些方案、为什么排除
- 实现时遇到了什么意外约束

**风险**：评审意见变成"建议增加测试""考虑边界情况"——正确但无用，有共识但无张力。

### 2.2 问题二：角色遵循与真实辩论

LLM 天然有认同倾向（sycophancy bias）——倾向于同意而非挑战。三个 agent 如果都走"看起来不错，小建议几条"的路线，拧毛巾就失效了。同时，又不能让 agent 固执己见——项目需要推进，有些问题可以留到下一轮迭代。

### 2.3 解决框架：三层机制

| 层级 | 机制 | 目的 |
|------|------|------|
| 第一层 | Review Briefing + 决策追溯链 | 解决上下文缺失，确保评审有针对性 |
| 第二层 | 角色锚定清单 + 结构化分歧协议 | 确保角色遵循，保证真实辩论 |
| 第三层 | 强制反方立场 | 对抗 LLM 认同倾向 |

---

## 3. 第一层：Review Briefing + 决策追溯链

### 3.1 设计原则

不让 Collie/Teddy 自己去找上下文，而是为它们准备一份结构化的评审简报，包含刚好足够的定向上下文。

在每个 review 节点，Husky（作为工件作者）在产出评审工件的同时，额外产出一份 **Review Briefing** 文件，放入 `features/<NNN>/reviews/`。

Review Briefing 由 Husky 产出，因为 Husky 是上游所有节点的执行者，拥有完整的决策上下文。作为工件作者，它最清楚哪些决策有风险、哪些折衷需要被质疑。这也符合拧毛巾的精神：**作者自己暴露软肋，评审者负责攻击**。

### 3.2 Review Briefing 模板

```markdown
# Review Briefing — <节点名>

## 1. 当前工件

- 路径：features/<NNN-slug>/<artifact>.md
- 工件状态：稳定草稿 / 已完成
- HF 节点：hf-xxx-review
- 当前 Profile：full

## 2. 上游关键决策（本工件继承的约束）

| 决策 ID | 内容 | 原因 | 来源工件 |
|---------|------|------|----------|
| D1 | 选用系统代理而非 TUN 模式 | TUN 需要驱动签名，Win/Linux 差异大 | design.md § 3.2 |
| D2 | 网络评估仅检测代理/DNS/hosts | 注册表和防火墙不在 MVP 范围 | spec.md FR-012 |

> 评审者：如对某决策的折衷存疑，请直接引用决策 ID 提出质疑。

## 3. 本轮评审焦点（作者主动请求的关注点）

- [必答] D1 的折衷是否影响 AC03（10 秒恢复）？
- [必答] FR-012 的范围排除是否留下用户可感知的 gap？
- [可选] 代理降级策略的覆盖度是否充分？

## 4. 角色定向提示

### 给 Collie（价值视角）：
- 对照验收场景 AC01-AC04，检查本工件是否有价值承诺未被覆盖
- 检查是否有"技术正确但用户无感"的工作项
- 关注 soul.md 底线：是否有可能误伤直连网站

### 给 Teddy（质量视角）：
- 对照 spec.md § NFR，检查设计是否可测试
- 检查是否有质量门禁项在当前设计中无法被验证
- 关注 docs/TECH-DEBT.md 中已有债务与本轮设计的交互风险
```

### 3.3 Briefing 与 codex-draft § 6 的关系

codex-draft § 6 定义了 **Review Context Pack**——每次评审的固定最小上下文清单（AGENTS.md、soul.md、progress.md 等）。Review Briefing 是在此基础上新增的**本轮评审定向简报**，二者的关系：

```
评审者接收的信息层次：

1. 固定上下文（codex-draft § 6 Review Context Pack）
   → 项目级静态文档，每次评审都读取
   → 提供项目背景、原则、架构约定

2. 定向简报（本节 Review Briefing）
   → 每轮评审动态生成
   → 提供本轮特有的决策追溯、焦点问题、角色定向提示

3. 被评审工件本身
   → spec.md / design.md / tasks.md 等
```

三层叠加确保评审者既有项目全局视野，又有本轮评审的精准定向。

### 3.4 Briefing 产出行为规范（写入 Husky 配置）

在 Husky 配置中增加以下行为规范：

```markdown
## 评审简报产出规范

当 Husky 完成工件起草、HF workflow 进入 review 节点时，Husky 必须：

1. 读取当前工件内容，识别其中的关键决策点
2. 回溯上游工件（spec ← discovery ← strategy），提取影响当前工件的决策链
3. 生成 Review Briefing，至少包含：
   - § 2 上游关键决策 ≥ 2 条
   - § 3 本轮评审焦点 ≥ 2 条必答 + 1 条可选
   - § 4 角色定向提示（Collie + Teddy 各 ≥ 2 条）
4. 将 Briefing 写入 features/<active>/reviews/briefing-<节点名>-YYYYMMDD.md
5. 在 approval step 暂停时，输出提示：
   "当前处于 <节点名>，已生成 Review Briefing。
    建议调 Collie（价值视角）和 Teddy（质量视角）参与评审。
    Briefing 路径：features/<active>/reviews/briefing-<节点名>-YYYYMMDD.md"
```

---

## 4. 第二层：角色锚定清单 + 结构化分歧协议

### 4.1 角色锚定清单

不是给 agent 一个模糊的"你是价值捍卫者"角色描述，而是给一份可逐条检查的清单。每个 review 节点，agent 必须逐项检查并给出明确结论。

#### 4.1.1 清单设计原则

- **每项必须给出明确结论**（`✅` 通过 / `⚠️` 疑虑 / `❌` 不通过 + 说明），不允许模棱两可
- **"必须提出 ≥ 2 条质疑"是强制性的**，防止 agent 只打勾不思考
- 清单内容**按节点定制**（spec-review 的清单和 design-review 的清单不同）
- 清单分为**通用项**（每个节点都有）和**节点专属项**（按节点类型不同）

#### 4.1.2 Collie 通用锚定项（所有节点适用）

```markdown
### Collie 通用价值检查（每个 review/gate 节点必填）

- [ ] 本工件是否与当前轮 wedge（OPP-002 安装后网络评估与基线恢复）对齐？
- [ ] 是否存在偏离 strategy-discovery 确定的 OPP-002→004→001 序列的内容？
- [ ] 验收场景 AC01-AC04 是否仍可被当前工件覆盖？
- [ ] 是否有 soul.md 底线被触碰的风险？（误伤直连 / 增加配置负担）
```

#### 4.1.3 Collie 节点专属锚定项（示例）

**hf-spec-review 节点：**

```markdown
### Collie 价值覆盖检查 — hf-spec-review

- [ ] 每条 FR 是否可追溯到 strategy-discovery 中的用户痛点或 JTBD？
- [ ] 验收场景 AC01-AC04 是否被 spec 的 FR/NFR 完整覆盖？
- [ ] 是否存在"技术驱动但无用户价值"的 FR？
- [ ] MoSCoW 分级是否与 strategy-discovery 的 OPP 优先级一致？
- [ ] Must-have 项是否控制在不影响 MVP 范围内？

### 必须提出的质疑（≥ 2 条）
从 Husky 配置 § 6 质疑模板中选择至少 2 条，或提出新质疑。
质疑必须具体到工件中的章节/条目编号。
```

**hf-design-review 节点：**

```markdown
### Collie 价值覆盖检查 — hf-design-review

- [ ] 设计方案是否保持了 spec 中承诺的用户价值？
- [ ] 是否存在对用户透明但影响体验的架构折衷？（如延迟增加、资源占用）
- [ ] 零配置目标在设计中是否有落地路径？
- [ ] 代理降级场景对用户的可感知影响是否已评估？

### 必须提出的质疑（≥ 2 条）
聚焦：技术方案是否牺牲了用户价值换取实现便利。
```

#### 4.1.4 Teddy 通用锚定项（所有节点适用）

```markdown
### Teddy 通用质量检查（每个 review/gate 节点必填）

- [ ] 是否存在无 fresh evidence 的关键断言？
- [ ] 当前工件与 docs/TECH-DEBT.md 中已有债务是否存在交互风险？
- [ ] 是否存在安全/隐私红线被触碰的迹象？（数据外发 / 敏感操作无审计）
```

#### 4.1.5 Teddy 节点专属锚定项（示例）

**hf-spec-review 节点：**

```markdown
### Teddy 可测试性检查 — hf-spec-review

- [ ] 每条 FR 是否有可验证的验收标准（量化指标或 Given-When-Then）？
- [ ] NFR 是否可转化为可执行的测试用例？
- [ ] 是否存在无法在 CI 环境中验证的 FR？如有，是否标注了手动测试方案？
- [ ] spec 的 FR/NFR 是否能映射到分层测试策略（单元/集成/E2E）？
- [ ] 是否存在依赖外部服务/资源的 FR？其不可用时的降级策略是否定义？

### 必须提出的质疑（≥ 2 条）
从 Teddy 配置 § 6 质疑模板中选择至少 2 条，或提出新质疑。
质疑必须具体到工件中的章节/条目编号。
```

**hf-code-review 节点：**

```markdown
### Teddy 代码质量检查 — hf-code-review

- [ ] 实现是否与 design.md 中声明的架构一致？
- [ ] SUT Form 声明（naive / pattern:tactical / emergent）是否与实际匹配？
- [ ] 是否存在未覆盖的边界条件或异常路径？
- [ ] 性能敏感路径是否有基准测试结果？（AGENTS.md 性能敏感任务要求）
- [ ] 代码中是否存在 unsafe / 未审计依赖 / 硬编码配置？

### 必须提出的质疑（≥ 2 条）
聚焦：代码质量是否满足质量门禁表中定义的阈值。
```

### 4.2 结构化分歧协议

当 Collie 和 Teddy 对同一工件有不同意见时，按固定格式表达分歧，确保争议可追溯、可裁决。

#### 4.2.1 分歧记录模板

```markdown
## 分歧记录

### 分歧点：<描述>

- **Collie 立场**：[通过 / 有条件通过 / 阻塞] — 原因：
- **Teddy 立场**：[通过 / 有条件通过 / 阻塞] — 原因：
- **Husky 回应**：[接受 / 反驳 / 折衷方案] — 说明：

### 证据对比

| 维度 | Collie 依据 | Teddy 依据 |
|------|------------|------------|
| 用户影响 | ... | ... |
| 技术风险 | ... | ... |

### 妥协评估（由提出折衷方案的一方填写）

- **折衷内容**：
- **遗留影响**：
- **触发修复条件**：在什么情况下必须解决（用户报告数 > X / 性能低于 Y / 某场景失败率 > Z）
- **目标迭代**：OPP-004 或 v0.2
- **Tech Debt ID**：TD-NNN（注册到 docs/TECH-DEBT.md）
```

#### 4.2.2 妥协有效性条件

只有**同时满足以下条件**，妥协才成立：

1. 不触碰任何角色的不可妥协底线（codex-draft § 9）
2. 有明确影响范围和风险说明
3. 填写了触发修复条件（不允许"以后再说"这种无条件的延期）
4. 已落盘到 `docs/TECH-DEBT.md`，并绑定后续任务或版本
5. 目标迭代不得超出 strategy-discovery 确定的 OPP 序列范围

#### 4.2.3 与 codex-draft § 10 的关系

codex-draft § 10 定义了分歧处理矩阵和妥协允许条件。本节的分歧记录模板是其**结构化落地形式**——codex-draft 定义了"什么情况下怎么处理"，本节定义了"处理过程如何记录"。

---

## 5. 第三层：强制反方立场

### 5.1 设计原理

对抗 LLM 认同倾向的最直接方法：强制每个 agent 生成一份反方论证。这不是可选的，而是评审文档的**必填章节**。

### 5.2 反方立场模板（写入各 agent 配置的评审输出规范）

在每个 agent 的评审文档中增加必填章节：

```markdown
## 反方立场（必填）

假设你完全反对本工件的方案，给出最强有力的反对理由：

### 反对理由 1：<标题>
- **论点**：
- **依据**（引用工件章节/条目）：
- **强度评估**：高 / 中 / 低

### 反对理由 2：<标题>
- **论点**：
- **依据**：
- **强度评估**：高 / 中 / 低

### 反对理由 3：<标题>
- **论点**：
- **依据**：
- **强度评估**：高 / 中 / 低

### 角色立场回应

从你自己的角色（Collie 价值 / Teddy 质量 / Husky 可行性）出发，
逐一回应上述反对理由：

| 反对理由 | 回应 | 处理 |
|----------|------|------|
| 理由 1 | ... | 接受该反对 / 反驳（附依据） / 可折衷（附条件） |
| 理由 2 | ... | ... |
| 理由 3 | ... | ... |
```

### 5.3 反方立场的评判标准

真人在阅读评审文档时，对反方立场的质量做如下判断：

| 现象 | 含义 | 处理 |
|------|------|------|
| 3 条反对全部"高强度" + 全部"反驳" | Agent 缺乏自省，可能回避问题 | 真人需手动审查反对理由是否有理 |
| 3 条反对全部"高强度" + 全部"接受" | 工件确实存在严重问题 | 需回修 |
| 反对理由引用了具体工件章节 | 评审有针对性，质量高 | 采纳 |
| 反对理由是泛泛的（"考虑性能"无引用） | 评审流于形式 | 标记为低质量评审，要求补充 |
| 折衷方案有明确的触发条件和 Tech Debt ID | 妥协有原则 | 可接受 |
| 折衷方案无触发条件 | 妥协不成立 | 退回要求补全 |

---

## 6. 评审执行流程

### 6.1 完整流程图

```
Husky 产出工件
    │
    ▼
HF workflow 进入 review/gate 节点
    │
    ▼
Husky 产出 Review Briefing
（含 § 2 决策追溯 + § 3 评审焦点 + § 4 角色定向提示）
    │
    ▼
Husky 在 approval step 暂停，输出调人信号：
  "当前处于 <节点名>，已生成 Briefing。
   建议调 Collie（价值）和 Teddy（质量）参与评审。"
    │
    ▼
┌─ 真人切换到 Collie（opencode）─────────────────┐
│  Collie 读：                                    │
│    1. 固定上下文（Review Context Pack）          │
│    2. Review Briefing                           │
│    3. 被评审工件                                │
│  Collie 执行：                                  │
│    → 逐项完成角色锚定清单（第二层）              │
│    → 生成反方立场（第三层）                      │
│    → 写 review-<节点>-Collie-YYYYMMDD-HHMM.md   │
└─────────────────────────────────────────────────┘
    │
    ▼
┌─ 真人切换到 Teddy（claude）─────────────────────┐
│  Teddy 读：                                     │
│    1. 固定上下文                                │
│    2. Review Briefing                           │
│    3. 被评审工件                                │
│    4. Collie 评审文档（串行互读）               │
│  Teddy 执行：                                   │
│    → 逐项完成角色锚定清单（第二层）             │
│    → 生成反方立场（第三层）                     │
│    → 对 Collie 结论中冲突点填入分歧记录（第二层）│
│    → 写 review-<节点>-Teddy-YYYYMMDD-HHMM.md    │
└─────────────────────────────────────────────────┘
    │
    ▼
┌─ 真人读取三方评审文档 ─────────────────────────┐
│  检查：                                        │
│    → 三个 agent 是否都完成了锚定清单？          │
│    → 反方立场是否实质（引用了具体工件章节）？   │
│    → 分歧点是否有完整证据对比？                 │
│    → 妥协项是否有触发条件 + Tech Debt ID？      │
│                                                │
│  裁决：                                        │
│    → 三方共识 → 写 resolution → Husky 继续     │
│    → 分歧可调停 → 写 resolution + 妥协 → 继续  │
│    → 分歧不可调停 → 写 resolution 标记真人裁决  │
└────────────────────────────────────────────────┘
    │
    ▼
Husky 按 resolution 执行：
  - 如有 ❌ 项 → 回修工件 → 重新提交评审
  - 如全部 ✅/⚠️ 已处理 → resolution 的 HF Verdict 交回 router
    │
    ▼
hf-workflow-router 消费 resolution 中的 HF Verdict
按迁移表进入下一 canonical 节点
```

### 6.2 串行互读的设计选择

Teddy 在写评审前先读 Collie 的评审文档（串行模式）。选择串行而非并行的原因：

1. **拧毛巾效果更强**：Teddy 看到 Collie 的视角后，可以在自己的评审中直接回应 Collie 的观点，形成真正的碰撞
2. **分歧即时暴露**：Teddy 在写评审时就能识别与 Collie 的分歧点，直接填入分歧记录模板，不需要真人后续再手动对比
3. **成本可控**：Teddy 只多读一份文档（Collie 的评审），不会显著增加评审耗时

对于**不需要深度碰撞的节点**（如 `hf-doc-freshness-gate`），真人可选择跳过串行互读，让 Collie 和 Teddy 并行独立评审。这个选择权在真人，不强制。

---

## 7. 项目级编排 Skill：goguo-review-orchestrator

### 7.1 定位

`skills/goguo-review-orchestrator/` 是一个项目专属 skill，**不是 HF canonical 节点**。它的作用是为真人在 review 节点的多 Agent 协调提供结构化工具，而非替代 HF 的路由和评审机制。

### 7.2 Skill 功能

```
goguo-review-orchestrator
  ├── SKILL.md                          # Skill 入口
  ├── references/
  │   ├── review-context-guide.md       # 如何准备固定上下文
  │   ├── role-focus-matrix.md          # 节点×角色焦点矩阵
  │   └── debate-quality-rubric.md      # 反方立场评判标准
  └── templates/
      ├── review-briefing-template.md   # Review Briefing 模板
      ├── collie-checklist/             # Collie 按节点的锚定清单
      │   ├── spec-review.md
      │   ├── design-review.md
      │   ├── tasks-review.md
      │   └── ... (按需增加)
      ├── teddy-checklist/              # Teddy 按节点的锚定清单
      │   ├── spec-review.md
      │   ├── design-review.md
      │   ├── test-review.md
      │   ├── code-review.md
      │   └── ... (按需增加)
      ├── disagreement-record.md        # 分歧记录模板
      ├── resolution-record.md          # Resolution 模板
      └── tech-debt-registration.md     # 技术债注册模板
```

### 7.3 与 codex-draft 的对齐

| codex-draft 定义的 | 本 skill 提供的 |
|--------------------|-----------------|
| § 6 Review Context Pack | `references/review-context-guide.md` 指导如何准备 |
| § 7 强制角色模板 | `templates/` 下的锚定清单是角色模板的结构化细化 |
| § 10 分歧处理矩阵 | `templates/disagreement-record.md` 是分歧处理的结构化记录 |
| § 11 Resolution 工件 | `templates/resolution-record.md` 是 Resolution 的标准模板 |
| § 10.3 技术债格式 | `templates/tech-debt-registration.md` 是技术债的注册模板 |

### 7.4 使用方式

1. **真人手动调用**：在 review 节点暂停时，真人可以按 skill 提供的模板手动组织多 Agent 评审
2. **Husky 辅助调用**：Husky 在产出 Briefing 时，可以引用 skill 中的模板路径，方便真人直接传递给 Collie/Teddy
3. **Collie/Teddy 配置引用**：agent 配置中引用 skill 中的锚定清单路径，确保评审时加载正确的检查项

---

## 8. 完整落地文件清单

### 8.1 新增文件

| 文件路径 | 用途 | 优先级 |
|----------|------|--------|
| `skills/goguo-review-orchestrator/SKILL.md` | 项目级评审编排 Skill 入口 | P0 |
| `skills/goguo-review-orchestrator/references/review-context-guide.md` | 固定上下文准备指南 | P0 |
| `skills/goguo-review-orchestrator/references/role-focus-matrix.md` | 节点×角色焦点矩阵 | P0 |
| `skills/goguo-review-orchestrator/references/debate-quality-rubric.md` | 反方立场评判标准 | P1 |
| `skills/goguo-review-orchestrator/templates/review-briefing-template.md` | Review Briefing 模板 | P0 |
| `skills/goguo-review-orchestrator/templates/collie-checklist/spec-review.md` | Collie spec-review 锚定清单 | P1 |
| `skills/goguo-review-orchestrator/templates/collie-checklist/design-review.md` | Collie design-review 锚定清单 | P1 |
| `skills/goguo-review-orchestrator/templates/collie-checklist/tasks-review.md` | Collie tasks-review 锚定清单 | P1 |
| `skills/goguo-review-orchestrator/templates/teddy-checklist/spec-review.md` | Teddy spec-review 锚定清单 | P1 |
| `skills/goguo-review-orchestrator/templates/teddy-checklist/design-review.md` | Teddy design-review 锚定清单 | P1 |
| `skills/goguo-review-orchestrator/templates/teddy-checklist/test-review.md` | Teddy test-review 锚定清单 | P1 |
| `skills/goguo-review-orchestrator/templates/teddy-checklist/code-review.md` | Teddy code-review 锚定清单 | P1 |
| `skills/goguo-review-orchestrator/templates/disagreement-record.md` | 分歧记录模板 | P0 |
| `skills/goguo-review-orchestrator/templates/resolution-record.md` | Resolution 模板 | P0 |
| `skills/goguo-review-orchestrator/templates/tech-debt-registration.md` | 技术债注册模板 | P0 |
| `docs/agent-configs/changelog.md` | Agent 配置变更记录 | P2 |
| `docs/TECH-DEBT.md` | 项目技术债总表 | P1 |

### 8.2 修改文件

| 文件路径 | 修改内容 | 优先级 |
|----------|----------|--------|
| `AGENTS.md` | § 6 末尾增加 § 7 多 Agent 协作架构声明 | P0 |
| `docs/agent-configs/Husky-codex-goguo.md` | (1) 增加 HF 节点参与矩阵 (2) 增加工件发现路径声明 (3) 增加 Review Briefing 产出行为规范 (4) 增加 approval step 暂停信号行为 (5) 评审命名统一为 `review-<节点>-Husky-YYYYMMDD-HHMM.md` | P0 |
| `docs/agent-configs/Collie-opencode-goguo.md` | (1) 增加 HF 节点参与矩阵 (2) 增加工件发现路径声明 (3) 增加反方立场必填章节要求 (4) 增加角色锚定清单引用路径 (5) 评审命名统一为 `review-<节点>-Collie-YYYYMMDD-HHMM.md`（补时分） | P0 |
| `docs/agent-configs/Teddy-claude-goguo.md` | (1) 增加 HF 节点参与矩阵 (2) 增加工件发现路径声明 (3) 增加反方立场必填章节要求 (4) 增加角色锚定清单引用路径  | P0 |

### 8.3 不修改的文件（HF 框架）

以下文件**不做任何修改**：

- `skills/hf-*` 下所有文件
- `docs/principles/` 下所有文件
- `skills/using-hf-workflow/` 下所有文件
- `skills/hf-workflow-router/` 下所有文件

---

## 9. Agent 配置增加内容详述

### 9.1 三份配置共有的新增内容

#### 9.1.1 HF 评审节点参与矩阵

```markdown
## HF 评审节点参与矩阵

| HF 节点 | 本 Agent 角色 | 参与强度 |
|---------|-------------|---------|
| hf-discovery-review | <按角色填写> | 必须 / 可选 / 不参与 |
| hf-spec-review | ... | ... |
| hf-design-review | ... | ... |
| hf-ui-review | ... | ... |
| hf-tasks-review | ... | ... |
| hf-test-review | ... | ... |
| hf-code-review | ... | ... |
| hf-traceability-review | ... | ... |
| hf-regression-gate | ... | ... |
| hf-doc-freshness-gate | ... | ... |
| hf-completion-gate | ... | ... |

> "必须"：必须产出独立评审文档
> "可选"：由真人在 Briefing § 4 中决定是否邀请
> "不参与"：该节点不涉及本角色关注领域
```

建议的参与矩阵（需真人确认）：

| HF 节点 | Collie | Teddy | Husky |
|---------|--------|-------|-------|
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

#### 9.1.2 工件发现路径声明

```markdown
## 工件发现路径

本 Agent 通过以下路径发现当前工作流状态和评审工件：

1. **当前活跃 Feature**：读仓库根 `README.md` 中的 active feature 指针
2. **当前工作流阶段**：读 `features/<active>/progress.md` 的 `Current Stage` 字段
3. **被评审工件路径**：`progress.md` 中引用的 spec/design/tasks 等路径
4. **评审文档目录**：`features/<active>/reviews/`
5. **Review Briefing**：`features/<active>/reviews/briefing-<节点名>-YYYYMMDD.md`
6. **上游决策追溯**：从 Briefing § 2 中引用的源工件路径读取
```

### 9.2 Husky 配置特有新增

#### 9.2.1 Approval Step 暂停信号

```markdown
## Approval Step 暂停信号

当 HF workflow 进入 review 节点、approval step 暂停等待真人确认时，
Husky 必须输出以下格式的调人信号：

---
[Multi-Agent Review Panel] 当前节点：<节点名>
已生成 Review Briefing：features/<active>/reviews/briefing-<节点名>-YYYYMMDD.md

根据参与矩阵，本轮应邀请：
- Collie（价值视角）：必须 / 可选
- Teddy（质量视角）：必须 / 可选

请真人在以下路径传递评审上下文后，调 Collie 和/或 Teddy 参与评审。
评审完成后，请将 resolution 文件放入：
features/<active>/reviews/resolution-<节点名>-YYYYMMDD-HHMM.md
---
```

### 9.3 Collie 配置特有新增

#### 9.3.1 评审输出规范修订

在 Collie 配置 § 7 评审输出规范中增加：

```markdown
### 反方立场（必填章节）

评审文档必须包含"反方立场"章节。假设你完全反对本工件方案，
给出至少 3 条最强有力的反对理由，每条必须引用工件具体章节。
然后从价值视角逐一回应这些反对理由。

### 角色锚定清单

评审时必须逐项完成对应节点的锚定清单。
清单路径：skills/goguo-review-orchestrator/templates/collie-checklist/<节点名>.md
每项必须给出 ✅/⚠️/❌ + 说明。
"必须提出的质疑 ≥ 2 条"为硬性要求。
```

### 9.4 Teddy 配置特有新增

#### 9.4.1 评审输出规范修订

在 Teddy 配置 § 7 评审文档内容结构中增加：

```markdown
### 反方立场（必填章节）

评审文档必须包含"反方立场"章节。假设你完全反对本工件方案，
给出至少 3 条最强有力的反对理由，每条必须引用工件具体章节。
然后从质量视角逐一回应这些反对理由。

### 角色锚定清单

评审时必须逐项完成对应节点的锚定清单。
清单路径：skills/goguo-review-orchestrator/templates/teddy-checklist/<节点名>.md
每项必须给出 ✅/⚠️/❌ + 说明。
"必须提出的质疑 ≥ 2 条"为硬性要求。

### 串行互读

在写评审前，应先读取 Collie 的评审文档（如已产出）。
对与 Collie 结论冲突的点，必须在评审文档中填入分歧记录模板。
```

---

## 10. AGENTS.md 新增声明

在 `AGENTS.md` § 6 末尾增加：

```markdown
## 7. 多 Agent 协作架构

### 7.1 架构概述

本项目采用多 Agent + 真人协同的"拧毛巾"式评审机制。
HF 框架流程不做修改，多 Agent 评审叠加在 HF review/gate 节点的 approval step 暂停点之上。

- **主执行 Agent**：Husky（codex-agent-app），负责 HF workflow 全部节点的执行
- **评审 Agent**：Collie（opencode-agent-app）、Teddy（claude-agent-app），在 review/gate 节点参与评审

### 7.2 Agent 配置文件

| Agent | 平台 | 配置文件 |
|-------|------|----------|
| Husky | codex-agent-app | `docs/agent-configs/Husky-codex-goguo.md` |
| Collie | opencode-agent-app | `docs/agent-configs/Collie-opencode-goguo.md` |
| Teddy | claude-agent-app | `docs/agent-configs/Teddy-claude-goguo.md` |

### 7.3 通信约定

- Agent 间唯一通信通道 = 项目文件系统（结构化 Markdown 文档）
- 无需 API、消息队列或共享数据库
- 真人作为跨平台协调枢纽，负责将各 Agent 的评审产出汇聚、裁决、分发

### 7.4 评审协议

- 评审协议详见 `docs/agent-configs/multi-agent-review-panel.md`（codex-draft 主文档）
- 评审编排工具详见 `skills/goguo-review-orchestrator/`
- HF router 仅消费 `resolution` 工件中的 HF Verdict，不直接消费各 Agent 的原始评审

### 7.5 升级规则

- 三方无法共识 → 真人裁决 → Husky 按裁决执行
- 裁决触发条件：方向争议 / 价值 vs 可行性冲突 / 质量底线触碰
- 任何与 soul.md "方向、取舍、标准最终权在用户" 相关的问题，必须升级真人
```

---

## 11. 实施步骤

### Step 1：项目级协议与 Skill 骨架（P0）

1. 创建 `skills/goguo-review-orchestrator/SKILL.md`
2. 创建 `references/` 下的三个参考文档
3. 创建 `templates/` 下的核心模板（briefing、disagreement、resolution、tech-debt）

### Step 2：统一三方 Agent 配置（P0）

1. 三份配置增加 HF 节点参与矩阵（§ 9.1.1 建议矩阵，待真人确认）
2. 三份配置增加工件发现路径声明（§ 9.1.2）
3. 统一评审文档命名规范
4. Husky 配置增加 Briefing 产出规范和暂停信号行为
5. Collie/Teddy 配置增加反方立场和锚定清单要求

### Step 3：更新 AGENTS.md（P0）

1. 增加 § 7 多 Agent 协作架构声明（§ 10 完整内容）

### Step 4：创建技术债文件（P1）

1. 创建 `docs/TECH-DEBT.md`，初始为空表头

### Step 5：创建节点专属锚定清单（P1）

1. 根据当前项目阶段（OPP-002 product-discovery），优先创建：
   - `collie-checklist/spec-review.md`
   - `collie-checklist/design-review.md`
   - `teddy-checklist/spec-review.md`
   - `teddy-checklist/design-review.md`
2. 后续节点清单在首次评审前按需创建

### Step 6：试运行（P1）

选择 `hf-spec-review` 或 `hf-design-review` 作为首次试运行节点：

1. Husky 准备工件 + Briefing
2. 真人调 Collie → Collie 读 Briefing + 工件，完成锚定清单 + 反方立场，写评审
3. 真人调 Teddy → Teddy 读 Briefing + 工件 + Collie 评审，完成锚定清单 + 反方立场 + 分歧记录，写评审
4. 真人读取三方评审，写 resolution
5. Husky 按 resolution 执行

### Step 7：根据试运行微调（P2）

- 调整锚定清单的具体检查项
- 调整参与矩阵（如某些节点改为可选/不参与）
- 调整反方立场的条目数量
- 更新 `docs/agent-configs/changelog.md`

### Step 8：后续节点清单补全（按需）

- 随项目推进到新阶段，按需创建对应节点的锚定清单
- 每次创建后更新 changelog

---

## 12. 待真人确认事项

| 编号 | 事项 | 默认值 | 影响 |
|------|------|--------|------|
| Q1 | § 9.1.1 参与矩阵中 Collie/Teddy 各节点的参与强度 | 如表所示 | 决定每个 review 节点需要调哪些 Agent |
| Q2 | 反方立场是否固定 3 条，还是按节点调整（spec-review 3 条，gate 2 条） | 固定 3 条 | 影响评审文档长度和评审耗时 |
| Q3 | 串行互读是否默认开启，还是由真人在 Briefing § 3 中指定 | 默认开启 | Teddy 评审质量 vs 评审速度 |
| Q4 | `skills/goguo-review-orchestrator/` 的 skill 前缀是否用 `goguo-` | `goguo-` | 与 `hf-*` 命名空间区分 |
| Q5 | 首次试运行选择哪个 review 节点 | `hf-spec-review` | 首次验证效果 |

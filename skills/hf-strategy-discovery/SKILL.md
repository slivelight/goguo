---
name: hf-strategy-discovery
description: 适用于从零开始收集竞品、分析市场和技术、确定整体业务视图的场景。支持三档产品规模（small/medium/large）。不适用于已明确进入 hf-product-discovery（功能收敛）或 hf-specify（规格起草）阶段。
---

# HF 战略发现

从零开始进行市场洞察、竞品分析、技术可行性评估，收敛成一份可评审的战略洞察草稿，确定整体业务视图、战略控制点和进入 hf-product-discovery 的范围边界。本 skill 不写具体功能规格，不替代 hf-product-discovery。

支持三档产品规模，适配不同项目：
- **small**：MVP、单功能模块、内部工具（1-6人团队，1-3个月周期）
- **medium**：功能扩展、垂直应用、平台模块（10-50人团队，3-12个月周期）
- **large**：产品组合、企业级战略（50-500+人团队，1-3年周期）

## Methodology

本 skill 融合以下已验证方法。每条方法都有对应的 reference 与落地章节。

| 方法 | 核心原则 | 来源 | 落地 |
|------|----------|------|------|
| **华为5看方法论** | 看行业/趋势、看市场/客户、看竞争、看自己、看机会，形成战略洞察 | 华为DSTE体系、BLM模型 | 步骤 2；insight-tools-guide.md |
| **PESTEL分析** | 宏观环境六维度分析（政策/经济/社会/技术/环境/法律） | 管理学通用工具 | 步骤 2.1；insight-tools-guide.md |
| **$APPEALS模型** | 客户需求八要素分析 | IBM/华为优化 | 步骤 2.2；insight-tools-guide.md |
| **SPAN矩阵** | 战略定位分析（市场吸引力 × 竞争地位） | 麦肯锡/通用电气 | 步骤 2.5；insight-tools-guide.md |
| **7 Powers战略框架** | 识别战略控制点 | Hamilton Helmer | 步骤 3；strategy-decision-template.md |
| **GIST Planning** | Goals → Ideas → Steps → Tasks 策略路径 | Itamar Gilad | 步骤 3.3 |
| **OKR目标体系** | Objectives & Key Results 量化目标 | John Doerr | 步骤 3.2 |
| **JTBD / Jobs Stories** | 需求锚定到用户进展而非功能 | Christensen | 步骤 2.2；jtbd-framework.md |
| **Opportunity Solution Tree** | Outcome → Opportunity → Solution → Assumption | Teresa Torres | 步骤 2.5 |
| **Multi-Agent辩论策略** | 正方论证 → 反方质疑 → 评委综合，头脑风暴收敛 | RE 2025论文 | 步骤 4；multi-agent-debate-protocol.md |

## When to Use

适用：
- 用户还在问"这个方向是否值得投入""市场有多大""竞品情况如何"
- 用户还在收敛战略方向、技术路线、商业模式
- 用户只有零散的行业/市场信息，需要系统化整理
- 需要从市场/竞品/技术分析开始，确定整体业务视图
- 现有输入主要是行业报告、竞品分析、技术趋势资料
- 用户希望通过头脑风暴讨论，收敛战略方向
- **项目初始化阶段**：存在 `AGENTS.md` + `docs/` + `skills/` 目录，但档0必需文档（`README.md` / `CHANGELOG.md` / `docs/adr/0001-...`）不完整
- **full profile 激活**：仅在 full profile 下接入 workflow + route 流程

不适用：
- 已明确要写 formal spec / design / tasks → hf-specify / hf-design / hf-tasks
- 只需收敛单个功能方向 → hf-product-discovery
- 只需评审已有战略洞察 → hf-strategy-review（若存在）
- route/stage/证据冲突 → hf-workflow-router

Direct invoke 信号："先帮我把市场/竞品分析做清楚""先确定战略方向""还没到写功能规格，先做战略洞察""帮我头脑风暴一下这个方向是否可行"。

## Hard Gates

- 战略洞察草稿未通过评审前，不得把它当作正式规格输入
- 不得把猜测、口号或直觉伪装成已确认战略决策
- 不得顺手进入功能规格、设计或任务拆解
- 若请求已明显进入 hf-product-discovery 节点，不继续停留在本 skill
- 用户输入的行业/市场资料必须有来源，不允许无来源断言
- Multi-Agent辩论必须真实论证，不得沦为形式主义
- **档0必需文档补齐**：若项目初始化阶段，必须补齐 `README.md` + `CHANGELOG.md` + `docs/adr/0001-record-architecture-decisions.md` 后才能进入 hf-product-discovery
- **仅 full profile 激活**：standard / lightweight profile 不激活本 skill

## Workflow

### 0. 确定当前产品规模档位

根据用户输入的产品规模和团队规模，确定当前产品规模档位：

| 产品规模档位 | 产品类型 | 团队规模 | 规划周期 | 激活条件 |
|---------|---------|---------|---------|---------|
| small | MVP、单功能模块 | 1-6人 | 1-3个月 | 用户描述为"快速验证""MVP""小工具" |
| medium | 功能扩展、垂直应用 | 10-50人 | 3-12个月 | 用户描述为"产品迭代""B端产品""平台模块" |
| large | 产品组合、企业战略 | 50-500+人 | 1-3年 | 用户描述为"企业战略""产品矩阵""多产品线" |

**重要**：本 skill 仅在 HF 框架 **full profile** 下激活（与产品规模档位无关）。若当前 HF profile 为 lightweight / standard，需先升级到 full 或跳过本 skill。

默认：若无法判断产品规模档位，从 small 开始，用户可主动升级。

### 0A. 检查档0必需文档状态

若项目初始化阶段（存在 `AGENTS.md` + `docs/` + `skills/`），检查档0必需文档：

| 必需文档 | 路径 | 检测规则 |
|---------|------|----------|
| ADR-0001 | `docs/adr/0001-record-architecture-decisions.md` | 文件存在 + 内容非空 |
| README.md | 仓库根 `README.md` | 文件存在 + 包含系统定位 + ADR 索引链接 |
| CHANGELOG.md | 仓库根 `CHANGELOG.md` | 文件存在 + Keep a Changelog 格式 |

若档0必需文档不完整，本 skill 必须先补齐这些文档，再进行战略洞察分析。

**档0补齐时机**：
- 检查发生在 Step 0A
- 实际补齐发生在 Step 6A（战略洞察草稿形成后、Bridge to Product Discovery 输出前）
- 补齐内容来源：战略洞察分析的结果（业务方向、系统定位、ADR 初始决策）

**档0补齐职责**：
- 补齐动作由本 skill 执行
- 补齐后需用户评审确认才能进入 hf-product-discovery
- 补齐文件写入磁盘后，标记为"待评审"状态

### 1. 读取用户输入的最少必要上游材料

只读完成战略洞察所需的最少材料：用户请求、已有的行业报告/竞品分析/技术趋势资料、用户提供的业务方向描述。

先归纳：
- 当前想探索的业务方向
- 已有的行业/市场信息
- 已有的竞品/技术信息
- 用户的核心关注点
- 明显越界到功能规格的内容

### 2. 结构化执行洞察分析（按产品规模档位密度）

#### 2.1 看行业/趋势

| 产品规模档位 | 分析深度 | 工具 |
|---------|---------|------|
| small | 不激活 | - |
| medium | PESTEL 4维度（P/E/S/T） | insight-tools-guide.md |
| large | PESTEL 6维度 + 技术成熟度曲线 | insight-tools-guide.md |

输出：行业洞察报告（含机会窗口分析）

#### 2.2 看市场/客户

| 产品规模档位 | 分析深度 | 工具 |
|---------|---------|------|
| small | 目标用户 + 核心痛点 + 快速画像 | 用户访谈整理 |
| medium | $APPEALS 8要素 + JTBD Jobs Stories | insight-tools-guide.md |
| large | $APPEALS + JTBD + 时序知识图谱 + LLMREI访谈模拟 | insight-tools-guide.md |

输出：客户画像集 + 需求优先级矩阵

#### 2.3 看竞争

| 产品规模档位 | 分析深度 | 工具 |
|---------|---------|------|
| small | 不激活 | - |
| medium | 3-5竞品 + CPM矩阵 + 波特五力 | insight-tools-guide.md |
| large | 5+竞品 + 动态监控 + 博弈模拟 | insight-tools-guide.md |

输出：竞品分析报告 + 差异化策略建议

#### 2.4 看自己

| 产品规模档位 | 分析深度 | 工具 |
|---------|---------|------|
| small | 可行性评估（技术/资源/时间）+ 风险提示 | 快速评估 |
| medium | SWOT + 能力差距分析 | insight-tools-guide.md |
| large | SWOT + 企业能力图谱 + 跨业务线协同 | insight-tools-guide.md |

输出：能力画像 + 差距分析报告

#### 2.5 看机会

| 产品规模档位 | 分析深度 | 工具 |
|---------|---------|------|
| small | 不激活 | - |
| medium | SPAN矩阵 + Opportunity Solution Tree | insight-tools-guide.md |
| large | SPAN矩阵 + OST + 多维度机会评估 | insight-tools-guide.md |

输出：战略机会地图 + 机会优先级排序

#### 2.6 看技术预见（仅 large）

| 产品规模档位 | 分析深度 | 工具 |
|---------|---------|------|
| small | 不激活 | - |
| medium | 不激活 | - |
| large | 强制激活：Gartner曲线 + S曲线 + 专利趋势 + 学术前沿 | insight-tools-guide.md |

输出：技术预见报告 + 投资建议

### 3. 三定决策输出（按产品规模档位密度）

#### 3.1 定战略控制点（7 Powers框架）

| 产品规模档位 | 输出密度 |
|---------|---------|
| small | 1个战略控制点 + 简要描述 |
| medium | 1-3个战略控制点 + 7 Powers分析 |
| large | 1-3个战略控制点 + 7 Powers + 可持续性评估 |

#### 3.2 定目标（OKR体系）

| 产品规模档位 | 输出密度 |
|---------|---------|
| small | 1-2个成功阈值指标 |
| medium | 年度 OKR（3-5 Objective + 各3-5 KR） |
| large | 年度 + 季度 OKR + DSTE解码 |

#### 3.3 定策略（GIST Planning）

| 产品规模档位 | 输出密度 |
|---------|---------|
| small | 不激活（直接进入 hf-product-discovery） |
| medium | GIST 路径（Goals/Ideas/Steps/Tasks） |
| large | GIST + 资源规划 + 变革管理 |

### 4. Multi-Agent 头脑风暴辩论（按产品规模档位激活规则）

#### 激活条件

| 产品规模档位 | 激活规则 |
|---------|---------|
| small | 用户主动要求时激活 |
| medium | 用户提出疑虑或有争议决策时激活 |
| large | 所有争议决策强制激活 |

#### 辩论流程（详见 multi-agent-debate-protocol.md）

1. **Phase 1**：主 Agent 整理用户输入，形成战略方案 v1
2. **Phase 2**：用户交互反馈（收集疑虑、风险、假设）
3. **Phase 3**：proponent Agent 正方论证（市场机会 + 能力匹配 + 执行路径）
4. **Phase 4**：challenger Agent 反方质疑（市场风险 + 执行风险 + 竞争风险）
5. **Phase 5**：arbiter Agent 评委综合（论据强度 + 关键权衡 + 置信度）
6. **Phase 6**：主 Agent 方案修订，形成战略方案 v2

输出：辩论记录 + 最终决策建议（含置信度）

### 5. Bridge to Product Discovery

形成 Bridge to Product Discovery 小节，明确：
- **推荐带入 hf-product-discovery 的 Product Opportunities 清单**（支持 ≥1 个）
  - 每个 Opportunity 包含：编号、描述、Desired Outcome、Success Threshold、关键假设
  - 标注当前轮优先级（P0/P1/P2）
  - 标注是否为当前轮 wedge（优先处理的 Opportunity）
- 已确认的战略方向和战略控制点
- 已锁定的 Desired Outcome 和 Success Threshold
- 需要继续保留为 assumption 的内容
- 当前不进入功能收敛的候选项

**多 Opportunity 场景处理**：
- 若输出 ≥2 个 Product Opportunities，按优先级排序
- 每个 Opportunity 将独立走 hf-product-discovery → hf-discovery-review → hf-specify 链路
- 一个 Opportunity 完成规格批准后，回到 hf-product-discovery 处理下一个
- 所有 Opportunity 完成后，进入 hf-design

### 6. 补齐档0必需文档（若档0不完整）

若 Step 0A 检测到档0必需文档不完整，在此步骤补齐：

#### 6.1 补齐 `docs/adr/0001-record-architecture-decisions.md`

从 Step 3（三定决策）提取：
- **Status**：Accepted
- **Context**：项目初始化阶段，需要建立架构决策记录机制
- **Decision**：采用 ADR (Architecture Decision Records) 作为架构决策文档格式，编号从 0001 开始，永不复用
- **Consequences**：所有重大架构决策必须记录 ADR，引用格式为 `ADR-NNNN`

#### 6.2 补齐 `README.md`

从 Step 2（5看洞察）和 Step 3（三定决策）提取：
- **系统定位**：来自执行摘要中的"业务方向"
- **Active Feature 指针**：当前为"项目初始化阶段，无 active feature"
- **ADR 索引链接**：`docs/adr/0001-record-architecture-decisions.md`
- **简介**：一句话描述系统目标

#### 6.3 补齐 `CHANGELOG.md`

使用 Keep a Changelog 格式：
- **Unreleased**：项目初始化，档0必需文档补齐
- **Initial Release**：（待后续填写）

#### 6.4 补齐后标记状态

档0必需文档补齐后，在战略洞察草稿中标记：
- `档0补齐状态: 已完成（待用户评审确认）`
- 补齐文件路径清单

**用户评审确认要求**：
- 用户必须确认档0必需文档内容是否正确
- 确认后才能进入 hf-product-discovery
- 若用户要求修改，回到 Step 6 重新补齐

### 7. 形成战略洞察草稿

按 strategy-discovery-template.md 起草战略洞察文档（按产品规模档位密度）。

### 8. 评审前自检与 handoff

交给用户确认前：
- 洞察分析已有明确来源，不是凭空断言
- 三定决策有依据支撑
- 已区分已确认战略 vs 待验证假设
- Bridge to Product Discovery 已明确范围边界
- 未把功能规格细节提前写入正文
- **档0必需文档已补齐**（若项目初始化阶段）
- **档0补齐内容已标记待用户评审确认**

## Output Contract

完成时产出：
- 战略洞察草稿（默认路径：docs/insights/YYYY-MM-DD-<topic>-strategy-discovery.md）
- 文档中明确的 Bridge to Product Discovery 小节
- Multi-Agent辩论记录（若激活）
- 状态同步：文档状态（草稿）

**项目初始化时额外产出**：
- `docs/adr/0001-record-architecture-decisions.md`（档0必需）
- `README.md`（档0必需，包含：系统定位 + active feature 指针 + ADR 索引链接）
- `CHANGELOG.md`（档0必需，Keep a Changelog 格式）
- **档0补齐状态标记**：在战略洞察草稿中标记 `档0补齐状态: 已完成（待用户评审确认）`

**用户评审确认要求**：
- 用户必须评审并确认档0补齐内容 + Bridge to Product Discovery
- 确认后，更新档0补齐状态为 `已完成（用户评审已确认）`
- 确认时间写入战略洞察草稿

**hf-product-discovery 进入条件**：
- 档0必需文档已补齐 + 用户评审已确认 + Bridge to Product Discovery 存在

输出长度按产品规模档位：
- small：≤ 2000字
- medium：5000-10000字
- large：10000+字 + 辩论记录 + 档0必需文档

**hf-product-discovery 输入契约**：

本 skill 输出必须满足 `hf-product-discovery` 的最小输入要求：

| hf-product-discovery 输入 | hf-strategy-discovery 输出对应 |
|---------------------------|--------------------------------|
| 核心问题 / 问题陈述 | 战略洞察草稿 section 1（执行摘要） |
| 目标用户 / JTBD situation | 战略洞察草稿 section 3（看市场/客户 + JTBD） |
| wedge / 最小机会点 | 战略洞察草稿 section 6（看机会 + OST） |
| Desired Outcome + Success Threshold | Bridge to Product Discovery 中已锁定字段 |
| 关键假设 | Bridge to Product Discovery 中待验证假设清单 |

若输出不满足以上契约，需在本 skill 内继续补齐，不得直接进入 hf-product-discovery。

## 和其他 Skill 的区别

| Skill | 区别 |
|-------|------|
| `hf-product-discovery` | hf-strategy-discovery 回答"市场有多大、方向是否值得、竞品如何"；hf-product-discovery 回答"具体功能方向是什么、wedge是什么"。两者使用 JTBD 的目的不同：战略发现用 JTBD 识别市场机会和客户画像，产品发现用 JTBD 锚定具体需求问题和进展。 |
| `hf-specify` | hf-strategy-discovery 不写正式规格，只做战略层面洞察和三定决策输出 |
| `hf-workflow-router` | router 负责 runtime routing；本 skill 假设当前已明确在做战略洞察 |
| `hf-experiment` | hf-strategy-discovery 可能输出待验证假设（如 Blocking 假设）；hf-experiment 负责执行 Build-Measure-Learn 验证循环。战略发现的假设通常是宏观层面的（市场假设、竞争假设），而 hf-experiment 的假设更具体（功能假设、用户行为假设） |

## Red Flags

- 把零散信息直接当成已确认战略决策
- 用功能列表代替市场分析和竞品分析
- 战略洞察文档混入具体功能规格
- 没区分已确认事实和待验证假设
- PESTEL/$APPEALS 分析没有来源支撑
- Multi-Agent辩论沦为形式主义（论据空洞）
- Profile 选择与产品规模不匹配

## Reference Guide

| 主题 | Reference | 加载时机 |
|------|-----------|---------|
| 战略洞察模板 | references/strategy-discovery-template.md | 起草战略洞察草稿时 |
| 三定决策模板 | references/strategy-decision-template.md | 执行三定决策时 |
| 5看分析工具指南 | references/insight-tools-guide.md | 执行洞察分析时 |
| Multi-Agent辩论协议 | references/multi-agent-debate-protocol.md | 激活辩论流程时 |

## Verification

- [ ] 已根据产品规模确定正确的产品规模档位（small/medium/large）
- [ ] 已确认当前 HF profile 为 full（本 skill 仅在 full profile 下激活）
- [ ] 战略洞察草稿已保存到约定路径
- [ ] 洞察分析已有明确来源
- [ ] 三定决策有依据支撑
- [ ] Bridge to Product Discovery 已明确范围边界
- [ ] 战略洞察正文未混入功能规格细节
- [ ] Multi-Agent辩论（若激活）有真实论据
- [ ] 输出长度符合产品规模档位要求
- [ ] **档0必需文档已补齐**（项目初始化时）：ADR-0001 + README.md + CHANGELOG.md
- [ ] **hf-product-discovery 输入契约已满足**：问题陈述 + 目标用户 + wedge + Desired Outcome + Success Threshold
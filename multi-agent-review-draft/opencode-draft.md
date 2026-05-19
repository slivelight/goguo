多Agent参与HF评审完整实施方案
---
一、目录结构规划
goguo/
├── AGENTS.md                          # 增加Reviewer Pool配置段
├── docs/
│   ├── agent-configs/                  # 新增目录
│   │   ├── Husky-codex-goguo.md       # 主执行者配置
│   │   ├── Collie-opencode-goguo.md   # 技术评审者配置
│   │   ├── Teddy-claude-goguo.md      # UX评审者配置
│   │   └── review-conflict-resolution.md # 真人裁决流程指南
│   ├── principles/
│   │   └── soul.md                    # 不修改
│   │   └── methodology-coherence.md   # 不修改
│   └── adr/
│       └── 0001-record-architecture-decisions.md
├── skills/
│   ├── hf-workflow-router/
│   │   ├── SKILL.md                   # 不修改核心Workflow
│   │   └── references/
│   │       ├── review-dispatch-protocol.md      # 扩展Step 9A/9B
│   │       ├── reviewer-return-contract.md      # 扩展字段(向后兼容)
│   │       ├── multi-reviewer-merge-guide.md    # 新增：评审记录合并指南
│   │       └── conflict-resolution-trigger.md   # 新增：分歧仲裁触发条件
│   ├── hf-spec-review/
│   │   ├── SKILL.md                   # 扩展Step 1.5A/3.6
│   │   └── references/
│   │       ├── review-record-template.md        # 扩展模板
│   │       └── role-anchor-checklist.md         # 新增：角色定位检查清单
│   ├── hf-design-review/              # 同hf-spec-review扩展
│   ├── hf-ui-review/                  # 同hf-spec-review扩展
│   ├── hf-tasks-review/               # 同hf-spec-review扩展
│   ├── hf-test-review/                # 同hf-spec-review扩展
│   ├── hf-code-review/                # 同hf-spec-review扩展
│   └── hf-traceability-review/        # 同hf-spec-review扩展
└── features/
    └── <NNN-slug>/
        ├── reviews/
        │   ├── spec-review-task-001.md    # 支持多评审者意见合并
        │   └── design-review-task-001.md  # 支持多评审者意见合并
        └── approvals/
            ├── spec-approval-001.md       # 支持真人裁决记录
            └── multi-reviewer-arbitration-001.md # 新增：多评审者真人裁决记录
---
二、核心文件内容规划
2.1 AGENTS.md增加段(在现有§6后增加)
 7. Multi-Agent Reviewer Pool（多评审者池）
本项目启用多Agent评审机制，形成"拧毛巾式"监督，弥补单一厂家Agent的模型偏见。
 7.1 Reviewer Pool定义
| Agent | 平台 | 角色 | 评审侧重 | 激活节点 | 配置文件 |
|-------|------|------|----------|---------|---------|
| Husky | Codex | 主执行者 | — | 不自审(遵循HF分离原则) | `docs/agent-configs/Husky-codex-goguo.md` |
| Collie | Opencode | 技术评审者 | 架构/代码/测试/性能/安全 | spec-review / design-review / test-review / code-review / traceability-review | `docs/agent-configs/Collie-opencode-goguo.md` |
| Teddy | Claude | UX评审者 | 需求/交互/可用性/a11y | spec-review / design-review / ui-review | `docs/agent-configs/Teddy-claude-goguo.md` |
 7.2 评审激活模式
**单评审者模式**(默认)：
- Router派发单一reviewer subagent
- 适用节点：test-review / code-review / traceability-review / regression-gate / completion-gate
**多评审者模式**(需激活)：
- Router并行派发≥2个reviewer subagent
- 适用节点：spec-review / design-review / ui-review / tasks-review
- 激活条件：
  1. 当前profile为`full`
  2. 当前节点命中Reviewer Pool激活表
  3. Router读取本配置段，确认Reviewer Pool存在
**激活控制**(手动开关)：
- 默认启用多评审者模式
- 用户可通过命令`/hf-workflow-router --single-reviewer`强制单评审者模式
- Router在progress.md中记录当前激活模式
 7.3 项目上下文注入(强制)
所有评审者(含单/多模式)必须接收以下项目核心上下文：
| 上下文类别 | 来源路径 | 注入时机 |
|-----------|---------|---------|
| 战略控制点 | `docs/insights/*-strategy-discovery.md` → Section 3(三定决策) | 所有review节点 |
| 核心业务目标 | `docs/insights/*-strategy-discovery.md` → Bridge to Product Discovery | 所有review节点 |
| 当前feature核心目标 | `features/<active>/spec.md` → Section 1(问题陈述) | 所有review节点 |
| 当前feature关键约束 | `features/<active>/spec.md` → Section 12(假设与失效影响) | 所有review节点 |
| 项目质量标准 | `AGENTS.md` → §4 Coding/Testing/Architecture标准 | 所有review节点 |
| 关键架构决策 | `docs/adr/0001-*.md` → Summary | design-review / code-review |
**注入责任**：Router构造review request时强制注入，评审者读取失败 → 角色定位检查失败 → 评审blocked。
 7.4 评审纪律与角色定位
**独立性原则**：
- 各评审者独立评审，评审期间不交互意见(盲审模式)
- 各评审者独立写评审记录初稿，不读取其他评审者记录
- Router在收集所有评审者返回后，才合并评审记录
- 合并时显式标注意见差异点(分歧发现章节)
**角色定位检查**(强制执行)：
- 评审者必须读取角色定位配置文件(`docs/agent-configs/<Agent>_*.md`)
- 评审者必须执行角色定位检查清单(见配置文件§3)
- 检查未完成 → 评审失败 → Router拒绝接受评审结果
**不跨权评审**：
- Collie不评审Teddy的职责范围(UX/交互/可用性/a11y)，发现UX问题只在findings中标注"建议Teddy关注"
- Teddy不评审Collie的职责范围(架构/代码/性能/安全)，发现技术问题只在findings中标注"建议Collie关注"
- 跨权标注不计入正式findings，不计入verdict判断
 7.5 分歧发现与量化
**分歧发现强制任务**(多评审者模式下)：
- 每个评审者必须列出至少3个潜在分歧点
- 每个分歧点必须量化：
  - 分歧强度评分(1-10分)
  - 置信度(百分比)
  - 分歧理由(锚定到项目具体决策)
**分歧强度评分标准**：
| 分歧强度 | 评分范围 | 说明 |
|---------|---------|------|
| 轻微分歧 | 1-2 | 技术细节差异，不影响核心决策 |
| 中度分歧 | 3-4 | 角度差异，需成本收益分析权衡 |
| 重大分歧 | 5-7 | 角度差异显著，需真人裁决 |
| 关键分歧 | 8-10 | 核心决策冲突，必须真人裁决 |
**分歧发现不足判定**：
- 潜在分歧点数量 < 3 → 分歧发现不足 → Router在真人裁决时提示关注
- 最大分歧强度评分 < 3 → 分歧发现过于保守 → Router提示评审者可能遗漏风险
 7.6 有原则妥协规则
 7.6.1 妥协触发条件(量化)
| 分歧强度差距 | 妥协决策 | 妥协要求 |
|-------------|---------|---------|
| ≤ 2(轻微分歧) | 可自动妥协 | 记录技术债务DEBT-NNN，无需真人裁决 |
| 3-4(中度分歧) | 可自动妥协(需量化) | 记录技术债务DEBT-NNN + 成本收益分析 + 修复计划，无需真人裁决 |
| ≥ 5(重大/关键分歧) | 不妥协 | 必须真人裁决 |
**分歧强度差距计算**：
- 取两个评审者对同一分歧点的评分差值
- 例：Collie评分4，Teddy评分7 → 差距=3，属于中度分歧
 7.6.2 妥协记录要求(技术债务持久化)
当自动妥协时，评审记录必须增加技术债务段：
## 技术债务记录(自动妥协结果)
### DEBT-NNN: <债务描述>
**触发分歧**：Collie(技术评审者)认为需要性能优化，Teddy(UX评审者)认为UX体验可接受
**分歧强度**：Collie评分6，Teddy评分9，差距=3(中度分歧)
**妥协条件满足**：
- [x] 分歧强度差距 ≤ 4(中度分歧)
- [x] 不违反核心质量属性红线(性能p95=450ms < 红线500ms)
- [x] 可在下一轮迭代优化(有明确修复计划)
- [x] 有成本收益分析(当前轮优化成本高，下一轮成本低)
**妥协意见**：通过(采纳Teddy意见，Collie妥协)
**妥协理由**：
- 性能未触及红线(p95=450ms < 500ms红线)
- 当前轮优化成本高(需重构核心模块，预估3天)
- 下一轮优化成本低(增量优化，预估0.5天)
- UX体验提升收益显著(用户满意度预估提升15%)
**修复计划**：
- 修复时间：下一轮迭代(Feature NNN+1)
- 触发条件：p95性能超过400ms或用户满意度下降≥5%
- 责任人：Husky(主执行者) + Collie(技术评审者监督)
- 验证标准：p95性能降至350ms以下，用户满意度保持≥85%
**技术债务编号**：DEBT-0012
**债务记录路径**：`docs/insights/technical-debt-log.md`
7.6.3 不妥协红线(不可自动妥协)
以下情况不触发自动妥协，必须真人裁决：
*核心质量属性红线(不可妥协)*：
| 质量属性 | 红线定义 | 触发条件 |
|---------|---------|---------|
| 性能红线 | p95响应时间 > 500ms | 性能评审发现触及红线 → 必须当前轮修复，不可妥协 |
| 安全红线 | OWASP Top 10漏洞 | 安全评审发现漏洞 → 必须当前轮修复，不可妥协 |
| 架构红线 | 违反ADR-0001核心决策 | 架构评审发现违反ADR → 必须当前轮修复，不可妥协 |
| 可用性红线 | WCAG 2.2 AA不达标 | UX评审发现不达标 → 必须当前轮修复，不可妥协(Teddy角色) |
| 数据红线 | 用户隐私数据泄露风险 | 安全评审发现风险 → 必须当前轮修复，不可妥协 |
妥协条件不满足红线：
- 分歧强度差距 ≥ 5(重大/关键分歧)
- 妥协意见无法在下一轮迭代优化(需要当前轮立即修复)
- 妥协意见没有明确的成本收益分析
- 妥协意见没有明确的修复计划
7.6.4 自动妥协执行流程
Router在Step 9B中执行自动妥协判断：
 9B.2 自动妥协判断流程
1. 计算所有分歧点的分歧强度差距
2. 对每个分歧点执行红线检查：
   - 读取`AGENTS.md` §7.6.3 红线定义
   - 检查妥协意见是否触及红线
   - 若触及红线 → 不妥协，进入真人裁决
3. 对每个分歧点执行妥协条件检查：
   - 若分歧强度差距 ≤ 2 → 可自动妥协，记录DEBT-NNN
   - 若分歧强度差距 3-4 → 检查成本收益分析+修复计划，满足则妥协
   - 若分歧强度差距 ≥ 5 → 不妥协，进入真人裁决
4. 所有分歧点妥协处理完成后：
   - 若所有分歧点都已妥协 → 合并评审记录，进入下一步
   - 若有分歧点无法妥协 → 进入真人裁决
7.7 真人裁决节点
当分歧强度差距 ≥ 5或有分歧点触及红线时，Router进入真人裁决节点。
真人裁决输入：
- Router向用户展示所有评审者意见和分歧发现分析
- Router向用户展示分歧强度量化评分
- Router向用户展示红线检查结果
- Router向用户展示自动妥协建议(若分歧强度差距≤4)
真人裁决选项：
- 选择其中一个评审者意见
- 综合多个评审者意见(用户自定义综合方案)
- 新意见(用户提出新的裁决方案)
- 接受自动妥协建议(若有)
真人裁决记录：
- Router记录裁决结果到features/<active>/approvals/multi-reviewer-arbitration-NNN.md
- 裁决记录包含：
  - 所有评审者意见摘要
  - 分歧强度量化评分
  - 红线检查结果
  - 用户裁决意见
  - 裁决理由
  - 裁决时间
7.8 评审记录合并与持久化
合并路径：
- 多评审者评审记录写入同一路径：features/<active>/reviews/<review-type>-task-NNN.md
- 真人裁决记录写入：features/<active>/approvals/multi-reviewer-arbitration-NNN.md
- 技术债务记录写入：docs/insights/technical-debt-log.md(项目级持久化)
合并格式：
 Design Review Record - Task 001
 Review Metadata
- Review Type: design-review
- Review Date: YYYY-MM-DD HH:MM
- Multi-Reviewer Mode: true
- Active Feature: NNN-slug
- Record Path: features/NNN-slug/reviews/design-review-task-001.md
 Reviewer: Collie(opencode) - 技术评审者
 Conclusion: 需修改
 Role Anchor Check(已完成)
- [x] 已读取战略控制点: 7 Powers - 网络效应控制点
- [x] 已读取feature核心目标: 实现用户认证系统
- [x] 已读取项目质量标准: AGENTS.md §4
- [x] 每条finding已锚定项目决策: 引用ADR-0001/spec §3.2/design §4.1
- [x] 不跨权评审: UX问题标注"建议Teddy关注"
 Key Findings(severity | classification | rule_id)
1. **critical** | **USER-INPUT** | D3: 设计未明确性能预算，需要用户确认p95响应时间阈值
2. **important** | **LLM-FIXABLE** | D5: 接口边界说明不足，模块间依赖关系未显式写出
 Divergence Discovery Analysis(强制章节)
| 分歧点 | Collie意见 | 假设Teddy意见 | 分歧强度 | 置信度 | 理由 |
|--------|-----------|--------------|---------|--------|------|
| 错误提示机制 | 需修改(技术实现清晰但缺少错误码规范) | 可能通过(UX视角:用户语言表达可接受) | 4 | 80% | 技术实现正确但缺少统一错误码规范，UX表达可接受但可能影响开发者排查 |
| 加载状态处理 | 通过(有loading状态) | 可能需修改(UX视角:空白状态体验差) | 5 | 70% | 有loading但空白状态和错误状态UX细节缺失，可能影响用户体验 |
| 响应式布局 | 通过(性能可接受) | 可能需修改(UX视角:移动端体验不足) | 4 | 75% | 性能达标(p95=450ms)但移动端交互细节未在设计中明确 |
 Compromise Suggestion(若有)
- 是否妥协: false(分歧强度差距≥5，违反不妥协红线)
- 妥协理由: 加载状态处理分歧强度差距=5(重大分歧)，需真人裁决
- 技术债务编号: null
---
 Reviewer: Teddy(claude) - UX评审者
 Conclusion: 通过
 Role Anchor Check(已完成)
- [x] 已读取战略控制点: 7 Powers - 网络效应控制点
- [x] 已读取feature核心目标: 实现用户认证系统
- [x] 已读取项目质量标准: AGENTS.md §4
- [x] 每条finding已锚定项目决策: 引用spec §5.1/design §6.2
- [x] 不跨权评审: 性能问题标注"建议Collie关注"
 Key Findings(severity | classification | rule_id)
1. **important** | **LLM-FIXABLE** | U3: 加载状态UX细节缺失(空白状态/错误状态)，建议补充
 Divergence Discovery Analysis(强制章节)
| 分歧点 | Teddy意见 | 假设Collie意见 | 分歧强度 | 置信度 | 理由 |
|--------|-----------|--------------|---------|--------|------|
| 错误提示机制 | 通过(用户语言表达清晰) | 可能需修改(技术视角:缺少错误码规范) | 3 | 75% | UX表达可接受但技术层面可能缺少统一规范，影响开发者排查效率 |
| 加载状态处理 | 需修改(空白状态体验差) | 可能通过(技术视角:有loading状态) | 6 | 85% | 技术实现有loading但UX体验不完整(空白状态/错误状态)，显著影响用户感知 |
| 响应式布局 | 需修改(移动端体验不足) | 可能通过(技术视角:性能可接受) | 5 | 80% | 性能达标但移动端交互细节缺失，影响移动用户体验 |
 Compromise Suggestion(若有)
- 是否妥协: false(分歧强度差距≥5，违反不妥协红线)
- 妥协理由: 加载状态处理分歧强度差距=6(关键分歧)，需真人裁决
- 技术债务编号: null
---
 Divergence Comparison(Router合并)
 实际分歧点分析
| 分歧点 | Collie评分 | Teddy评分 | 分歧强度差距 | 红线检查 | 妥协决策 |
|--------|-----------|----------|-------------|---------|---------|
| 错误提示机制 | 4 | 3 | 1(轻微分歧) | 不触及红线 | 可自动妥协 |
| 加载状态处理 | 5 | 6 | 1(轻微分歧) | 不触及红线 | 可自动妥协 |
| 响应式布局 | 4 | 5 | 1(轻微分歧) | 不触及红线 | 可自动妥协 |
 Divergence Discovery Effectiveness
- Collie分歧发现准确度: 100%(3个潜在分歧点全部命中实际分歧)
- Teddy分歧发现准确度: 100%(3个潜在分歧点全部命中实际分歧)
- 分歧发现质量: 优秀(所有分歧点均已识别)
---
 Technical Debt Record(自动妥协结果)
 DEBT-0013: 错误提示机制缺少统一错误码规范
**触发分歧**：Collie认为需修改(缺少错误码规范)，Teddy认为通过(UX表达可接受)
**分歧强度**：Collie评分4，Teddy评分3，差距=1(轻微分歧)
**妥协条件满足**：
- [x] 分歧强度差距 ≤ 2(轻微分歧)
- [x] 不违反核心质量属性红线
- [x] 可在下一轮迭代优化
- [x] 有成本收益分析
**妥协意见**：通过(采纳Teddy意见，Collie妥协)
**妥协理由**：
- UX表达可接受，不影响用户感知
- 统一错误码规范可提升开发者排查效率，但不影响当前轮核心目标
- 当前轮添加错误码规范成本中等(预估1天)，下一轮成本低(增量完善)
**修复计划**：
- 修复时间：下一轮迭代(Feature NNN+1)
- 触发条件：开发者反馈排查效率低或错误数量增长≥20%
- 责任人：Husky(主执行者) + Collie(技术评审者监督)
- 验证标准：错误码覆盖率达到80%，开发者排查时间缩短30%
 DEBT-0014: 加载状态UX细节缺失
**触发分歧**：Collie认为通过(有loading状态)，Teddy认为需修改(空白状态体验差)
**分歧强度**：Collie评分5，Teddy评分6，差距=1(轻微分歧)
**妥协条件满足**：
- [x] 分歧强度差距 ≤ 2(轻微分歧)
- [x] 不违反核心质量属性红线
- [x] 可在下一轮迭代优化
- [x] 有成本收益分析
**妥协意见**：需修改(采纳Teddy意见，Collie妥协)
**妥协理由**：
- UX体验提升收益显著(用户满意度预估提升10%)
- 补充空白状态/错误状态UX细节成本低(预估0.5天)
- 不影响性能和技术实现稳定性
**修复计划**：
- 修复时间：当前轮(立即修复)
- 责任人：Husky(主执行者) + Teddy(UX评审者监督)
- 验证标准：空白状态/错误状态UX完整，用户满意度≥80%
 DEBT-0015: 响应式布局移动端体验不足
**触发分歧**：Collie认为通过(性能可接受)，Teddy认为需修改(移动端体验不足)
**分歧强度**：Collie评分4，Teddy评分5，差距=1(轻微分歧)
**妥协条件满足**：
- [x] 分歧强度差距 ≤ 2(轻微分歧)
- [x] 不违反核心质量属性红线(性能未触及红线)
- [x] 可在下一轮迭代优化
- [x] 有成本收益分析
**妥协意见**：通过(采纳Collie意见，Teddy妥协)
**妥协理由**：
- 性能达标(p95=450ms < 500ms红线)
- 当前轮补充移动端交互细节成本高(预估2天)
- 下一轮成本低(增量完善，预估0.5天)
- 移动用户占比低(预估20%)，优先级可延后
**修复计划**：
- 修复时间：下一轮迭代(Feature NNN+1)
- 触发条件：移动用户占比增长≥30%或移动端满意度下降≥10%
- 责任人：Husky(主执行者) + Teddy(UX评审者监督)
- 验证标准：移动端交互细节完整，移动用户满意度≥75%
---
 Final Verdict(自动妥协合并结果)
 Conclusion: 需修改
 下一步: hf-design(回修DEBT-0014加载状态UX细节，其他DEBT延后下一轮)
 需真人确认: false(所有分歧点已自动妥协，未触及红线)
7.9 技术债务持久化路径
技术债务记录持久化到项目级文件，便于跨feature追踪：
 docs/insights/technical-debt-log.md
 技术债务清单
 DEBT-0013: 错误提示机制缺少统一错误码规范
- 来源Feature: 001-user-auth
- 触发评审: design-review-task-001
- 妥协时间: YYYY-MM-DD
- 计划修复: Feature 002迭代
- 触发条件: 开发者反馈排查效率低或错误数量增长≥20%
- 责任人: Husky + Collie
- 状态: 待修复
 DEBT-0015: 响应式布局移动端体验不足
- 来源Feature: 001-user-auth
- 触发评审: design-review-task-001
- 妥协时间: YYYY-MM-DD
- 计划修复: Feature 002迭代
- 触发条件: 移动用户占比增长≥30%或移动端满意度下降≥10%
- 责任人: Husky + Teddy
- 状态: 待修复
 DEBT统计
- 总债务数: 2
- 待修复数: 2
- 已修复数: 0
- 平均债务年龄: 0天
---
三、Agent配置文件内容规划
3.1 Husky-codex-goguo.md(主执行者配置)
 Husky - 主执行者配置
 基本信息
- Agent名称: Husky
- 平台: Codex
- 角色: 主执行者
- 职责: 执行HF workflow所有节点(除review/gate节点外)
 核心原则
 不自审原则(遵循HF分离原则)
Husky作为主执行者，**不得**参与自己提交工件的评审：
- Husky完成spec/design/tasks/实现后，Router派发其他评审者(Collie/Teddy)
- Husky不参与spec-review / design-review / ui-review / tasks-review / test-review / code-review / traceability-review
- Husky可参与其他feature的评审(跨feature评审)，但需在Reviewer Pool中配置
 项目上下文掌握
Husky作为主执行者，必须掌握完整项目上下文：
- 战略洞察: `docs/insights/*-strategy-discovery.md`
- 当前feature: `features/<active>/` 所有工件
- 项目标准: `AGENTS.md` 所有约定
- 架构决策: `docs/adr/` 所有ADR
 执行纪律
- 遵循HF soul原则: 方向/取舍/标准最终权在用户
- 遵循HF证据驱动: 所有交付物可回读/可恢复
- 遵循HF质量优先: 质量优先于进度
- 遵循HF TDD纪律: RED-GREEN-REFACTOR + Fresh Evidence
 Reviewer Pool参与
Husky作为主执行者，**不激活**为评审者(不自审原则)。
若需Husky参与其他feature评审，需在`AGENTS.md` §7.1 Reviewer Pool中配置：
- 跨feature评审场景: Feature A完成后，Husky可参与Feature B的评审
- 配置方式: 在Reviewer Pool增加Husky评审角色(需显式声明不自审Feature A)
3.2 Collie-opencode-goguo.md(技术评审者配置)
 Collie - 技术评审者配置
 基本信息
- Agent名称: Collie
- 平台: Opencode
- 角色: 技术评审者
- 职责: 评审架构/代码/测试/性能/安全维度
- 激活节点: spec-review / design-review / test-review / code-review / traceability-review
 角色定位与评审侧重
 评审侧重维度
| 维度 | 关注点 | 红线定义 |
|------|--------|---------|
| 架构一致性 | 实现是否遵循已批准设计/ADR决策 | 违反ADR核心决策 → 必须修复 |
| 代码质量 | 正确性/防御性/可读性/架构健康 | 架构smell触发escalation → 必须修复 |
| 测试质量 | fail-first有效性/覆盖度/风险覆盖 | 测试覆盖核心行为缺失 → 必须补充 |
| 性能 | 性能预算是否满足/瓶颈识别 | p95 > 500ms → 必须优化 |
| 安全 | 威胁建模是否充分/防御措施 | OWASP Top 10漏洞 → 必须修复 |
 不评审维度(跨权标注)
Collie**不评审**以下维度(属Teddy职责范围)：
- UX/交互设计: IA / 交互流程 / 视觉设计 / 交互状态
- 可用性: Nielsen启发式 / 用户感知 / 用户语言表达
- 可访问性: WCAG 2.2 AA达标
- 移动端体验: 响应式细节 / 移动交互优化
**跨权标注规则**：
- 发现UX问题 → 在findings中标注"建议Teddy关注"，不计入正式findings
- 不跨权评审Teddy的职责范围，避免角色混权
 角色定位检查清单(强制执行)
 必须检查项(Checklist Anchor)
评审时强制执行以下检查，未完成 → 评审失败：
- [ ] **项目战略锚点**: 已读取`docs/insights/*-strategy-discovery.md`的战略控制点，评审是否偏离战略方向
  - 当前战略控制点: 7 Powers - 网络效应控制点(从strategy-discovery提取)
  - 评审锚定: 检查设计是否强化网络效应控制点
  
- [ ] **Feature核心目标锚点**: 已读取`features/<active>/spec.md`的核心目标，评审是否偏离feature约束
  - 当前feature核心目标: 实现用户认证系统(从spec §1提取)
  - 当前feature关键约束: 数据不上传Remote Server(从spec §12提取)
  - 评审锚定: 检查设计是否满足核心目标和关键约束
- [ ] **项目质量标准锚点**: 已读取`AGENTS.md` §4 Coding/Testing/Architecture标准，评审是否锚定项目约定而非通用标准
  - 项目质量标准: AGENTS.md §4 定义的性能红线(p95≤500ms) / 安全红线(OWASP Top 10)
  - 评审锚定: 检查设计是否满足项目定义的红线，而非通用行业标准
- [ ] **架构决策锚点**: 已读取`docs/adr/0001-*.md`关键ADR，评审是否遵循已批准架构决策
  - 关键ADR: ADR-0001(架构决策记录机制) / ADR-0005(Tauri跨平台架构)
  - 评审锚定: 检查设计是否遵循ADR决策，偏离是否有理由且可追溯
- [ ] **角色定位纪律**: 已读取本配置文件，不评审Teddy职责范围
  - 不跨权评审UX/交互/可用性/a11y
  - 发现UX问题 → 在findings中标注"建议Teddy关注"
- [ ] **分歧发现任务**: 已识别至少3个潜在分歧点(与Teddy视角差异)
  - 列出至少3个潜在分歧点
  - 每个分歧点量化分歧强度评分(1-10)和置信度(百分比)
  - 分歧发现不足 → 评审质量不足，Router提示关注
- [ ] **Finding锚定项目决策**: 每条finding已锚定到项目具体决策
  - Finding必须引用ADR编号 / spec章节 / design章节
  - 不允许泛议(如"代码质量不高"需改为"违反ADR-0005依赖方向原则，见design §4.1")
 检查未完成处理
- 检查项未完成 → 写blocked record，提示角色定位未锚定
- Router拒绝接受评审结果，要求重新执行检查清单
- 评审记录中必须包含"Role Anchor Check"章节，列出检查结果
 分歧发现任务(强制执行)
 任务目标
弥补单一厂家Agent模型偏见，主动暴露与Teddy(UX评审者)的意见分歧点。
 执行要求(多评审者模式下强制)
1. **假设Teddy视角**: 从UX视角思考可能给出的不同意见
2. **列出潜在分歧点**: 至少3个，即使你认为这些分歧不重要
3. **量化分歧强度**: 对每个分歧点给出评分(1-10)和置信度(百分比)
4. **输出格式**: 在评审记录中增加"Divergence Discovery Analysis"章节
 分歧强度评分标准
| 分歧强度 | 评分范围 | 评分依据 |
|---------|---------|---------|
| 轻微分歧 | 1-2 | 技术细节差异，不影响核心决策 |
| 中度分歧 | 3-4 | 角度差异，需成本收益分析权衡 |
| 重大分歧 | 5-7 | 角度差异显著，需真人裁决 |
| 关键分歧 | 8-10 | 核心决策冲突，必须真人裁决 |
 示例输出
## Divergence Discovery Analysis(强制章节)
| 分歧点 | Collie意见 | 假设Teddy意见 | 分歧强度 | 置信度 | 理由 |
|--------|-----------|--------------|---------|--------|------|
| 错误提示机制 | 需修改(缺少统一错误码规范) | 可能通过(UX表达可接受) | 4 | 80% | 技术实现正确但缺少统一规范，UX表达可接受但可能影响开发者排查效率 |
| 加载状态处理 | 通过(有loading状态) | 可能需修改(UX视角:空白状态体验差) | 5 | 70% | 技术实现有loading但UX体验不完整，可能影响用户感知 |
| 响应式布局 | 通过(性能可接受) | 可能需修改(UX视角:移动端体验不足) | 4 | 75% | 性能达标(p95=450ms)但移动端交互细节未明确，可能影响移动用户体验 |
分歧发现不足判定
Router在合并评审记录时，检查分歧发现质量：
- 潜在分歧点数量 < 3 → 分歧发现不足 → Router在真人裁决时提示关注
- 最大分歧强度评分 < 3 → 分歧发现过于保守 → Router提示评审者可能遗漏风险
- 分歧发现准确度 < 50% → 分歧发现失效 → Router提示评审者角色定位可能有问题
有原则妥协规则
妥协触发条件(量化)
| 分歧强度差距 | Collie妥协决策 | 妥协要求 |
|-------------|---------------|---------|
| ≤ 2(轻微分歧) | 可自动妥协 | 记录DEBT-NNN，无需真人裁决 |
| 3-4(中度分歧) | 可自动妥协(需量化) | 记录DEBT-NNN + 成本收益分析 + 修复计划，无需真人裁决 |
| ≥ 5(重大/关键分歧) | 不妥协 | 必须真人裁决 |
妥协记录要求
当Collie妥协时，必须在评审记录中增加技术债务段(见AGENTS.md §7.6.2示例)。
不妥协红线(不可自动妥协)
以下情况Collie不妥协，必须真人裁决：
- 分歧强度差距 ≥ 5(重大/关键分歧)
- 妥协意见违反核心质量属性红线(从AGENTS.md §7.6.3提取)
- 妥协意见无法在下一轮迭代优化(需要当前轮立即修复)
- 妥协意见没有明确的成本收益分析
红线检查(强制执行)
Collie在妥协前，强制执行红线检查：
- 性能红线: p95响应时间 > 500ms → 不妥协，必须当前轮修复
- 安全红线: OWASP Top 10漏洞 → 不妥协，必须当前轮修复
- 架构红线: 违反ADR核心决策 → 不妥协，必须当前轮修复
- 数据红线: 用户隐私数据泄露风险 → 不妥协，必须当前轮修复
妥协示例(中度分歧)
 Compromise Suggestion
 DEBT-0013: 错误提示机制缺少统一错误码规范
**触发分歧**：Collie认为需修改(缺少错误码规范)，Teddy认为通过(UX表达可接受)
**分歧强度**：Collie评分4，Teddy评分3，差距=1(轻微分歧)
**Collie妥协条件满足**：
- [x] 分歧强度差距 ≤ 2(轻微分歧)
- [x] 不违反核心质量属性红线(不影响安全/性能)
- [x] 可在下一轮迭代优化(有明确修复计划)
- [x] 有成本收益分析：
  - 当前轮添加错误码规范成本: 预估1天
  - 下一轮添加成本: 预估0.5天(增量完善)
  - 收益: 开发者排查效率提升30%
**Collie妥协意见**：通过(采纳Teddy意见，Collie妥协)
**Collie妥协理由**：
- UX表达可接受，不影响用户感知(Teddy视角成立)
- 统一错误码规范可提升开发者排查效率，但不影响当前轮核心目标(技术视角成立)
- 妥协后仍有技术收益，可下一轮优化(技术债务可持久化)
**修复计划**：
- 修复时间: 下一轮迭代(Feature NNN+1)
- 触发条件: 开发者反馈排查效率低或错误数量增长≥20%
- 责任人: Husky(主执行者) + Collie(技术评审者监督)
- 验证标准: 错误码覆盖率达到80%，开发者排查时间缩短30%
项目上下文注入(强制接收)
Collie在评审时，必须接收Router注入的项目核心上下文：
| 上下文类别 | 来源 | 用途 |
|-----------|------|------|
| 战略控制点 | docs/insights/*-strategy-discovery.md → Section 3 | 评审是否偏离战略方向 |
| 核心业务目标 | docs/insights/*-strategy-discovery.md → Bridge | 评审是否满足业务目标 |
| Feature核心目标 | features/<active>/spec.md → Section 1 | 评审是否偏离feature核心目标 |
| Feature关键约束 | features/<active>/spec.md → Section 12 | 评审是否满足关键约束 |
| 项目质量标准 | AGENTS.md → §4 | 评审是否锚定项目约定 |
| 关键架构决策 | docs/adr/0001-*.md → Summary | 评审是否遵循ADR决策 |
注入失败处理：
- Router未注入上下文 → Collie写blocked record，提示缺少项目上下文
- Collie未读取上下文 → Role Anchor Check失败 → 评审失败
### 3.3 Teddy-claude-goguo.md(UX评审者配置)
类似Collie配置，调整评审侧重维度为UX/交互/可用性/a11y。
---
## 四、Router扩展内容规划
### 4.1 review-dispatch-protocol.md扩展(在现有内容后增加)
```markdown
## 多评审者派发协议(新增)
### 前提条件
本协议仅在以下条件下激活：
- 当前profile为`full`
- `AGENTS.md` §7声明Reviewer Pool存在
- 当前review节点命中Reviewer Pool激活表
否则，遵循单评审者模式(现有流程)。
---
## Step 9A: 多评审者盲审派发(新增)
### 9A.1 激活判断
父会话读取`AGENTS.md` §7.1 Reviewer Pool定义：
- 若当前review节点命中激活表 → 进入多评审者模式
- 若用户强制指定`--single-reviewer` → 强制单评审者模式
- 否则 → 单评审者模式(现有流程)
**激活判断逻辑**：
```markdown
if (AGENTS.md §7存在 && profile==full && 当前节点命中激活表 && 用户未强制单评审者):
  -> 多评审者模式
else:
  -> 单评审者模式(现有流程)
9A.2 项目上下文注入
父会话构造review request时，强制注入项目核心上下文：
注入内容：
{
  项目核心上下文(强制注入): {
    strategy_context: {
      战略控制点: 从docs/insights/*-strategy-discovery.md Section 3提取,
      核心业务目标: 从Bridge to Product Discovery提取
    },
    feature_context: {
      当前feature编号: 从progress.md提取,
      当前feature核心目标: 从features/<active>/spec.md Section 1提取,
      当前feature关键约束: 从features/<active>/spec.md Section 12提取,
      当前feature风险清单: 从features/<active>/spec.md Section 4 Key Hypotheses提取
    },
    project_standards: {
      质量标准: 从AGENTS.md §4提取,
      红线定义: 从AGENTS.md §7.6.3提取
    },
    architecture_decisions: {
      关键ADR摘要: 从docs/adr/0001-*.md Summary提取
    }
  }
}
注入责任：
- 父会话负责读取上述文件并提取上下文
- 上下文注入失败 → 父会话写blocked record，提示缺少项目上下文
- 评审者读取上下文失败 → Role Anchor Check失败 → 评审失败
注入路径：
- strategy_context: 父会话读取docs/insights/最新strategy-discovery文件
- feature_context: 父会话读取features/<active>/spec.md
- project_standards: 父会话读取AGENTS.md
- architecture_decisions: 父会话读取docs/adr/关键ADR
9A.3 Reviewer Pool读取
父会话读取AGENTS.md §7.1 Reviewer Pool定义，识别当前节点的激活评审者：
| review节点 | 激活评审者 | 派发数量 |
|-----------|-----------|---------|
| spec-review | Collie + Teddy | 2 |
| design-review | Collie + Teddy | 2 |
| ui-review | Teddy | 1(ui-review仅UX评审者) |
| tasks-review | Collie + Teddy | 2 |
| test-review | Collie | 1(技术评审者) |
| code-review | Collie | 1(技术评审者) |
| traceability-review | Collie | 1(技术评审者) |
派发逻辑：
- 激活评审者≥2 → 多评审者模式
- 激活评审者=1 → 单评审者模式(即使Reviewer Pool存在)
9A.4 盲审Review Request构造
父会话为每个评审者构造独立的review request：
盲审规则：
- 各评审者独立评审，评审期间不交互意见
- 各review request不包含其他评审者的评审记录
- 各review request包含相同的项目上下文注入(公平注入)
*Review Request模板(多评审者模式)*：
{
  review_type: design-review,
  review_skill: hf-design-review,
  
  reviewer_identity: Collie|Teddy,
  multi_reviewer_mode: true,
  blind_review_mode: true,
  
  topic: 评审用户认证系统设计,
  
  artifact_paths: [
    features/001-user-auth/design.md
  ],
  
  supporting_context_paths: [
    docs/insights/2026-05-06-goguo-strategy-discovery.md,
    features/001-user-auth/spec.md,
    docs/adr/0001-record-architecture-decisions.md,
    docs/adr/0005-tauri-cross-platform-architecture.md,
    AGENTS.md
  ],
  
  项目核心上下文(强制注入): {
    strategy_context: {
      战略控制点: 7 Powers - 网络效应控制点,
      核心业务目标: 实现用户认证系统，强化网络效应
    },
    feature_context: {
      当前feature编号: 001-user-auth,
      当前feature核心目标: 实现用户认证系统，数据不上传Remote Server,
      当前feature关键约束: 用户数据存储在OS标准应用目录,
      当前feature风险清单: [HYP-001: 本地存储安全性, HYP-002: 跨平台兼容性]
    },
    project_standards: {
      质量标准: AGENTS.md §4: p95≤500ms, WCAG 2.2 AA, OWASP Top 10,
      红线定义: AGENTS.md §7.6.3: p95>500ms性能红线, OWASP Top 10安全红线, WCAG 2.2 AA可用性红线
    },
    architecture_decisions: {
      关键ADR摘要: {
        ADR-0001: 采用ADR作为架构决策记录机制,
        ADR-0005: 采用Tauri实现跨平台架构，前端Rust+Web技术
      }
    }
  },
  
  角色定位锚点(强制注入): {
    评审者角色: Collie(技术评审者) / Teddy(UX评审者),
    角色定位配置文件: docs/agent-configs/<Agent>_*.md,
    角色定位检查清单: 从配置文件§3提取,
    分歧发现强制任务: 列出至少3个潜在分歧点，量化分歧强度和置信度,
    不跨权评审纪律: 不评审其他评审者职责范围，发现跨权问题标注建议关注
  },
  
  workspace_isolation: worktree-active,
  worktree_path: /tmp/goguo-worktree-001,
  worktree_branch: feat/001-user-auth,
  expected_record_path: features/001-user-auth/reviews/design-review-task-001.md,
  current_profile: full
}
9A.5 并行派发Reviewer Subagent
父会话并行派发多个reviewer subagent：
派发方式：
- 父会话构造多个review request(每个评审者一个)
- 父会话使用Task tool并行派发多个reviewer subagent
- 每个reviewer subagent在fresh context中独立执行评审
派发示例：
父会话执行:
1. 读取AGENTS.md §7.1 Reviewer Pool，识别激活评审者: Collie + Teddy
2. 为Collie构造review request(reviewer_identity=Collie)
3. 为Teddy构造review request(reviewer_identity=Teddy)
4. 并行派发:
   - Task(description="Collie design review", prompt="执行hf-design-review，reviewer_identity=Collie, review_request=<Collie的review request>")
   - Task(description="Teddy design review", prompt="执行hf-design-review，reviewer_identity=Teddy, review_request=<Teddy的review request>")
派发约束：
- 每个reviewer subagent独立执行评审，不交互意见
- 每个reviewer subagent独立写评审记录初稿(同一文件路径，不同reviewer_identity段)
- 每个reviewer subagent强制执行Role Anchor Check和Divergence Discovery
9A.6 收集Reviewer返回摘要
父会话等待所有reviewer subagent完成，收集所有返回摘要：
收集内容：
- 每个reviewer返回的结构化摘要(见reviewer-return-contract.md扩展)
- 每个reviewer返回的评审记录路径
- 每个reviewer返回的分歧发现量化字段
收集约束：
- 等待所有激活的reviewer返回
- 若某个reviewer返回blocked/失败 → 父会话处理失败后重新派发或进入单评审者模式
- 收集完成后 → 进入Step 9B合并与仲裁
---
Step 9B: 分歧合并与仲裁(新增)
9B.1 读取所有评审记录
父会话读取所有评审者的评审记录(同一文件路径，不同reviewer_identity段)：
读取内容：
- 每个评审者的conclusion
- 每个评审者的key_findings
- 每个评审者的Role Anchor Check结果
- 母个评审者的Divergence Discovery Analysis
- 每个评审者的Compromise Suggestion
9B.2 分歧发现对比分析
父会话对比各评审者的分歧发现分析与实际分歧点：
对比内容：
- Collie分歧发现: 假设Teddy意见 vs Teddy实际意见
- Teddy分歧发现: 假设Collie意见 vs Collie实际意见
- 计算分歧发现准确度: 命中分歧点数量 / 潜在分歧点数量
对比输出：
 Divergence Comparison(父会话合并)
 实际分歧点分析
| 分歧点 | Collie评分 | Teddy评分 | 分歧强度差距 | 红线检查 | 妥协决策 |
|--------|-----------|----------|-------------|---------|---------|
| 错误提示机制 | 4 | 3 | 1(轻微分歧) | 不触及红线 | 可自动妥协 |
| 加载状态处理 | 5 | 6 | 1(轻微分歧) | 不触及红线 | 可自动妥协 |
| 响应式布局 | 4 | 5 | 1(轻微分歧) | 不触及红线 | 可自动妥协 |
 Divergence Discovery Effectiveness
- Collie分歧发现准确度: 100%(3个潜在分歧点全部命中实际分歧)
- Teddy分歧发现准确度: 100%(3个潜在分歧点全部命中实际分歧)
- 分歧发现质量: 优秀(所有分歧点均已识别)
9B.3 自动妥协判断
父会话对每个分歧点执行自动妥协判断：
自动妥协判断流程：
1. 计算分歧强度差距:
   - 取两个评审者对同一分歧点的评分差值
   - 例: Collie评分4, Teddy评分7 → 差距=3(中度分歧)
2. 执行红线检查:
   - 读取AGENTS.md §7.6.3红线定义
   - 检查妥协意见是否触及红线
   - 若触及红线 → 不妥协，进入真人裁决
3. 执行妥协条件检查:
   - 若分歧强度差距 ≤ 2 → 可自动妥协，记录DEBT-NNN
   - 若分歧强度差距 3-4 → 检查:
     - 是否有成本收益分析(评审者已提供)
     - 是否有修复计划(评审者已提供)
     - 满足 → 可自动妥协，记录DEBT-NNN
     - 不满足 → 不妥协，进入真人裁决
   - 若分歧强度差距 ≥ 5 → 不妥协，进入真人裁决
4. 所有分歧点处理完成后:
   - 若所有分歧点都已妥协 → 合并评审记录，进入下一步
   - 若有分歧点无法妥协 → 进入真人裁决
红线检查逻辑：
for (分歧点 in 所有分歧点):
  if (妥协意见触及AGENTS.md §7.6.3定义的红线):
    -> 不妥协，进入真人裁决
  else:
    -> 继续妥协条件检查
妥协条件检查逻辑：
for (分歧点 in 所有分歧点):
  差距 = abs(Collie评分 - Teddy评分)
  
  if (差距 <= 2):
    -> 可自动妥协，记录DEBT-NNN
  elif (差距 <= 4):
    if (有成本收益分析 && 有修复计划):
      -> 可自动妥协，记录DEBT-NNN
    else:
      -> 不妥协，进入真人裁决
  else:
    -> 不妥协，进入真人裁决
9B.4 技术债务记录(自动妥协时)
当自动妥协时，父会话在评审记录中增加技术债务段：
技术债务记录内容：
- DEBT编号: DEBT-NNN(项目级唯一编号，从technical-debt-log.md读取下一个编号)
- 触发分歧: Collie意见 vs Teddy意见
- 分歧强度: 评分差距
- 妥协条件满足清单
- 妥协意见: 采纳哪个评审者意见
- 妥协理由: 成本收益分析
- 修复计划: 修复时间/触发条件/责任人/验证标准
技术债务持久化：
- 父会话将技术债务记录到评审记录(features/<active>/reviews/<review-type>-task-NNN.md)
- 父会话将技术债务摘要追加到docs/insights/technical-debt-log.md(项目级持久化)
9B.5 真人裁决节点(分歧无法妥协时)
当分歧强度差距 ≥ 5或有分歧点触及红线时，父会话进入真人裁决节点。
真人裁决输入展示：
父会话向用户展示：
 多评审者分歧 - 需真人裁决
 评审者意见对比
| 评审者 | 角色 | Conclusion | Key Findings |
|--------|------|-----------|-------------|
| Collie | 技术评审者 | 需修改 | 设计缺少统一错误码规范(影响开发者排查效率) |
| Teddy | UX评审者 | 通过 | UX表达清晰，用户语言表达可接受 |
 分歧强度量化
| 分歧点 | Collie评分 | Teddy评分 | 分歧强度差距 | 红线检查 |
|--------|-----------|----------|-------------|---------|
| 错误提示机制 | 4 | 3 | 1(轻微分歧) | 不触及红线 |
| 加载状态处理 | 5 | 6 | 1(轻微分歧) | 不触及红线 |
| 响应式布局 | 4 | 5 | 1(轻微分歧) | 不触及红线 |
 分歧发现质量
- Collie分歧发现准确度: 100%
- Teddy分歧发现准确度: 100%
 自动妥协建议
所有分歧点分歧强度差距 ≤ 2，可自动妥协：
- DEBT-0013: 错误提示机制缺少统一错误码规范 → 采纳Teddy意见(通过)，下一轮修复
- DEBT-0014: 加载状态UX细节缺失 → 采纳Teddy意见(需修改)，当前轮修复
- DEBT-0015: 响应式布局移动端体验不足 → 采纳Collie意见(通过)，下一轮修复
**建议Final Verdict**: 需修改(回修DEBT-0014，其他延后)
 真人裁决选项
请选择裁决方案：
1. **采纳Collie意见**: 需修改，补充错误码规范和加载状态技术实现
2. **采纳Teddy意见**: 通过，延后技术优化
3. **综合意见**: 需修改，仅修复加载状态UX细节，错误码规范和响应式布局延后
4. **接受自动妥协建议**: 需修改，仅修复DEBT-0014，其他延后(推荐)
5. **新意见**: 您提出新的裁决方案
真人裁决选项处理：
父会话等待用户选择裁决方案：
- 用户选择1-4 → 父会话记录裁决结果，按裁决方案进入下一步
- 用户选择5 → 父会话等待用户输入新方案，记录后进入下一步
真人裁决记录：
父会话将裁决结果记录到features/<active>/approvals/multi-reviewer-arbitration-NNN.md：
 Multi-Reviewer Arbitration Record - 001
 Arbitration Metadata
- Arbitration Type: design-review分歧裁决
- Arbitration Date: YYYY-MM-DD HH:MM
- Active Feature: 001-user-auth
- Record Path: features/001-user-auth/approvals/multi-reviewer-arbitration-001.md
 评审者意见摘要
 Collie(技术评审者)
- Conclusion: 需修改
- Key Findings:
  1. 设计缺少统一错误码规范(影响开发者排查效率)
  2. 加载状态技术实现有loading但缺少错误状态处理
 Teddy(UX评审者)
- Conclusion: 通过
- Key Findings:
  1. 加载状态UX细节缺失(空白状态/错误状态)
  2. 响应式布局移动端体验不足
 分歧强度量化
| 分歧点 | Collie评分 | Teddy评分 | 分歧强度差距 |
|--------|-----------|----------|-------------|
| 错误提示机制 | 4 | 3 | 1(轻微分歧) |
| 加载状态处理 | 5 | 6 | 1(轻微分歧) |
| 响应式布局 | 4 | 5 | 1(轻微分歧) |
 真人裁决结果
- 用户裁决时间: YYYY-MM-DD HH:MM
- 用户裁决方案: 接受自动妥协建议(方案4)
- 用户裁决意见: 需修改，仅修复DEBT-0014(加载状态UX细节)，其他延后
- 用户裁决理由: 
  - 加载状态UX细节缺失影响用户体验，当前轮修复成本低(0.5天)
  - 错误码规范和响应式布局不影响当前轮核心目标，下一轮优化
  - 自动妥协建议充分权衡技术视角和UX视角，符合有原则妥协规则
 裁决后续行动
- Final Verdict: 需修改
- Next Action: hf-design(回修DEBT-0014)
- 技术债务持久化:
  - DEBT-0013: 延后下一轮修复
  - DEBT-0015: 延后下一轮修复
9B.6 Final Verdict与下一步
父会话根据裁决结果形成Final Verdict：
自动妥协时：
- Final Verdict = 妥协后的综合意见
- Next Action = 根据妥协意见决定(需修改 → hf-design回修，通过 → approval step)
真人裁决时：
- Final Verdict = 用户裁决意见
- Next Action = 根据裁决意见决定
Final Verdict输出：
 Final Verdict(父会话合并结果)
 Conclusion: 需修改
 下一步: hf-design(回修DEBT-0014加载状态UX细节，其他DEBT延后下一轮)
 需真人确认: false(所有分歧点已自动妥协，未触及红线)
 技术债务清单
- DEBT-0013: 错误提示机制缺少统一错误码规范(延后下一轮)
- DEBT-0014: 加载状态UX细节缺失(当前轮修复)
- DEBT-0015: 响应式布局移动端体验不足(延后下一轮)
9B.7 评审记录合并与持久化
父会话合并所有评审者的评审记录到同一文件：
合并方式：
- 同一文件路径: features/<active>/reviews/<review-type>-task-NNN.md
- 不同reviewer_identity段: 每个评审者独立段
- 合并段: Divergence Comparison / Technical Debt Record / Final Verdict
持久化路径：
- 评审记录: features/<active>/reviews/<review-type>-task-NNN.md
- 真人裁决记录: features/<active>/approvals/multi-reviewer-arbitration-NNN.md
- 技术债务摘要: docs/insights/technical-debt-log.md
---
父会话职责扩展(多评审者模式)
父会话在多评审者模式下承担额外职责：
增加职责
- 判断是否应进入多评审者模式(读取AGENTS.md §7)
- 注入项目核心上下文(读取strategy/feature/standards/ADR文件)
- 并行派发多个reviewer subagent
- 收集所有reviewer返回摘要
- 对比分歧发现与实际分歧点
- 执行自动妥协判断和红线检查
- 记录技术债务到评审记录和项目级文件
- 进入真人裁决节点(分歧无法妥协时)
- 合并评审记录
保持职责(与单评审者模式相同)
- 判断当前是否应进入review节点
- 选择正确的review skill
- 消费reviewer返回摘要
- 在需要时发起approval step
- 根据摘要继续推进或回流修订
不承担职责
- 在当前上下文直接执行review判断
- 代替reviewer写review记录
- 替代真人完成裁决(真人裁决节点由真人决策)
---
### 4.2 reviewer-return-contract.md扩展(在现有内容后增加)
```markdown
## 多评审者模式扩展(向后兼容)
### 新增字段(多评审者模式下必须)
```json
{
  "reviewer_identity": "Collie|Teddy|Husky",
  "multi_reviewer_mode": true,
  
  "分歧发现量化(多评审者模式下强制)": {
    "潜在分歧点数量": 3,
    "最大分歧强度": 5,
    "分歧发现准确度预估": "70%",
    "分歧点列表": [
      {
        "分歧点": "错误提示机制",
        "本评审者意见": "需修改(缺少错误码规范)",
        "假设他评审者意见": "可能通过(UX表达可接受)",
        "分歧强度": 4,
        "置信度": "80%",
        "理由": "技术实现正确但缺少统一规范，UX表达可接受但可能影响开发者排查效率"
      },
      {
        "分歧点": "加载状态处理",
        "本评审者意见": "通过(有loading状态)",
        "假设他评审者意见": "可能需修改(UX视角:空白状态体验差)",
        "分歧强度": 5,
        "置信度": "70%",
        "理由": "技术实现有loading但UX体验不完整，可能影响用户感知"
      },
      {
        "分歧点": "响应式布局",
        "本评审者意见": "通过(性能可接受)",
        "假设他评审者意见": "可能需修改(UX视角:移动端体验不足)",
        "分歧强度": 4,
        "置信度": "75%",
        "理由": "性能达标(p95=450ms)但移动端交互细节未明确，可能影响移动用户体验"
      }
    ]
  },
  
  "妥协建议(若有)": {
    "是否妥协": false,
    "妥协理由": "分歧强度差距≥5，违反不妥协红线",
    "技术债务编号": null,
    "妥协条件满足清单": null,
    "修复计划": null
  },
  
  "角色定位检查结果(多评审者模式下强制)": {
    "检查完成": true,
    "未完成项": [],
    "项目战略锚点": "已读取战略控制点: 7 Powers - 网络效应控制点",
    "feature核心目标锚点": "已读取feature核心目标: 实现用户认证系统",
    "项目质量标准锚点": "已读取AGENTS.md §4质量标准",
    "架构决策锚点": "已读取ADR-0001/ADR-0005",
    "角色定位纪律": "不跨权评审UX维度",
    "分歧发现任务": "已列出3个潜在分歧点"
  }
}
向后兼容规则
单评审者模式(默认):
- reviewer_identity: 可选，默认为"Husky"或null
- multi_reviewer_mode: false(默认值)
- 其他新增字段: 可选，不强制
多评审者模式(激活时):
- reviewer_identity: 必须，从review_request读取
- multi_reviewer_mode: true(必须)
- 分歧发现量化字段: 必须，强制执行分歧发现任务
- 妥协建议字段: 可选，仅在评审者建议妥协时填写
- 角色定位检查结果: 必须，强制执行Role Anchor Check
字段说明
| 字段 | 说明 | 单评审者模式 | 多评审者模式 |
|------|------|-------------|-------------|
| reviewer_identity | 评审者身份标识 | 可选 | 必须 |
| multi_reviewer_mode | 是否处于多评审者模式 | false(默认) | true(必须) |
| 分歧发现量化 | 分歧发现分析结果 | 可选 | 必须(强制任务) |
| 妥协建议 | 评审者的妥协建议 | 可选 | 可选(若建议妥协) |
| 角色定位检查结果 | Role Anchor Check结果 | 可选 | 必须(强制检查) |
父会话消费规则扩展(多评审者模式)
父会话收到多个reviewer摘要后，按以下顺序处理：
1. 收集所有摘要: 等待所有激活的reviewer返回
2. 读取所有评审记录: 从同一文件路径读取不同reviewer_identity段
3. 分歧发现对比: 对比各评审者的分歧发现分析与实际分歧点
4. 自动妥协判断: 执行自动妥协判断流程(见review-dispatch-protocol.md §9B.3)
5. 真人裁决: 若分歧无法妥协 → 进入真人裁决节点
6. Final Verdict: 根据裁决结果形成Final Verdict
7. 下一步: 根据Final Verdict进入下一步
Return Contract验证规则
父会话在消费reviewer摘要前，验证Return Contract完整性：
单评审者模式:
- 验证现有字段(conclusion/next_action_or_recommended_skill/record_path/key_findings)
- 新增字段不强制验证
多评审者模式:
- 验证现有字段(必须)
- 验证reviewer_identity(必须存在且合法)
- 验证multi_reviewer_mode=true(必须)
- 验证分歧发现量化字段(必须存在且潜在分歧点数量≥3)
- 验证角色定位检查结果字段(必须存在且检查完成=true)
- 验证失败 → 父会话拒绝接受评审结果，要求reviewer补充
---
## 五、Review Skill Workflow扩展(在各hf-*review skill中增加)
### 5.1 在各hf-*review skill的Workflow中增加步骤(保持现有步骤不变)
在各`hf-spec-review` / `hf-design-review` / `hf-ui-review` / `hf-tasks-review` / `hf-test-review` / `hf-code-review` / `hf-traceability-review` skill的Workflow中增加：
```markdown
### 1.5A: 角色定位检查(多评审者模式下强制执行)
**前提**：当前处于多评审者模式，`review_request.multi_reviewer_mode=true`
**任务**：
1. **读取角色定位配置文件**：
   - 读取`docs/agent-configs/<Agent>_*.md`(Agent名称从review_request.reviewer_identity读取)
   - 读取角色定位检查清单(见配置文件§3)
2. **执行角色定位检查清单**：
   - [ ] 已读取项目战略控制点(从review_request.项目核心上下文.strategy_context提取)
   - [ ] 已读取当前feature核心目标(从review_request.项目核心上下文.feature_context提取)
   - [ ] 已读取项目质量标准(从review_request.项目核心上下文.project_standards提取)
   - [ ] 已读取关键架构决策(从review_request.项目核心上下文.architecture_decisions提取)
   - [ ] 每条finding已锚定到项目具体决策(引用ADR编号/spec章节/design章节)
   - [ ] 不评审其他评审者职责范围(发现跨权问题在findings中标注"建议X关注")
3. **检查结果记录**：
   - 在评审记录中增加"Role Anchor Check"章节，列出检查结果
   - 在return contract中增加`角色定位检查结果`字段
4. **检查未完成处理**：
   - 检查项未完成 → 写blocked record，提示角色定位未锚定
   - Return contract中`角色定位检查结果.检查完成=false`
   - Return contract中列出未完成项
---
### 3.6: 分歧发现分析(多评审者模式下强制执行)
**前提**：当前处于多评审者模式，`review_request.multi_reviewer_mode=true`
**任务**：
1. **识别其他评审者角色**：
   - 读取`AGENTS.md` §7.1 Reviewer Pool定义
   - 识别当前节点的其他激活评审者(如: Collie评审者识别Teddy为其他评审者)
2. **假设其他评审者视角**：
   - 从其他评审者的角色视角思考可能给出的不同意见
   - 例: Collie(技术评审者)假设Teddy(UX评审者)从UX视角可能给出的意见
3. **列出潜在分歧点**(至少3个)：
   - 即使你认为这些分歧不重要，也必须列出
   - 每个分歧点必须量化：
     - 分歧强度评分(1-10分)
     - 置信度(百分比)
     - 分歧理由(锚定到项目具体决策)
4. **分歧强度评分标准**：
   | 分歧强度 | 评分范围 | 评分依据 |
   |---------|---------|---------|
   | 轻微分歧 | 1-2 | 技术细节差异，不影响核心决策 |
   | 中度分歧 | 3-4 | 角度差异，需成本收益分析权衡 |
   | 重大分歧 | 5-7 | 角度差异显著，需真人裁决 |
   | 关键分歧 | 8-10 | 核心决策冲突，必须真人裁决 |
5. **输出格式**：
   - 在评审记录中增加"Divergence Discovery Analysis"章节
   - 在return contract中增加`分歧发现量化`字段
6. **红线约束**：
   - 潜在分歧点数量 < 3 → 分歧发现任务未完成 → Router拒绝接受评审结果
   - 分歧强度评分模糊(如"分歧强度不明") → 分歧发现任务未完成 → Router拒绝接受
**示例输出**：
```markdown
## Divergence Discovery Analysis(强制章节)
### 潜在分歧点识别
| 分歧点 | Collie意见 | 假设Teddy意见 | 分歧强度 | 置信度 | 理由 |
|--------|-----------|--------------|---------|--------|------|
| 错误提示机制 | 需修改(缺少统一错误码规范，影响开发者排查效率) | 可能通过(UX视角:用户语言表达可接受) | 4 | 80% | 技术实现正确但缺少统一规范，UX表达可接受但可能影响开发者排查效率，引用design §4.2错误处理设计 |
| 加载状态处理 | 通过(技术实现有loading状态) | 可能需修改(UX视角:空白状态和错误状态UX细节缺失) | 5 | 70% | 技术实现有loading但UX体验不完整，可能影响用户感知，引用design §5.1状态管理设计 |
| 响应式布局 | 通过(性能p95=450ms达标) | 可能需修改(UX视角:移动端交互细节未明确) | 4 | 75% | 性能达标(p95=450ms < 500ms红线)但移动端交互细节未在设计中明确，可能影响移动用户体验，引用spec §3.2响应式需求 |
### 分歧发现不足判定
- 潜在分歧点数量: 3(满足要求)
- 最大分歧强度: 5(重大分歧，需真人裁决)
- 分歧发现准确度预估: 75%(基于角色定位置信度)
---
4A: 妥协建议(多评审者模式下可选)
前提：当前处于多评审者模式，review_request.multi_reviewer_mode=true
任务(可选)：
若评审者认为某些分歧点可以妥协，可提出妥协建议：
1. 妥协触发条件检查：
   - 分歧强度差距 ≤ 4(轻微/中度分歧)
   - 不违反核心质量属性红线(从review_request.项目核心上下文.project_standards.红线定义提取)
   - 可在下一轮迭代优化(有明确修复计划)
   - 有成本收益分析
2. 妥协记录：
   - 在评审记录中增加"Compromise Suggestion"章节
   - 在return contract中增加妥协建议字段
   - 提出技术债务编号(DEBT-NNN)
3. 不妥协红线检查：
   - 分歧强度差距 ≥ 5(重大/关键分歧) → 不妥协
   - 妥协意见违反核心质量属性红线 → 不妥协
   - 妥协意见无法在下一轮迭代优化 → 不妥协
示例输出：
 Compromise Suggestion(可选章节)
 是否妥协: true
 DEBT-0013: 错误提示机制缺少统一错误码规范
**触发分歧**: Collie认为需修改(缺少错误码规范)，Teddy认为通过(UX表达可接受)
**分歧强度**: Collie评分4，假设Teddy评分3，差距预估=1(轻微分歧)
**妥协条件满足**:
- [x] 分歧强度差距 ≤ 2(轻微分歧)
- [x] 不违反核心质量属性红线(不影响安全/性能)
- [x] 可在下一轮迭代优化(有明确修复计划)
- [x] 有成本收益分析：
  - 当前轮添加错误码规范成本: 预估1天
  - 下一轮添加成本: 预估0.5天(增量完善)
  - 收益: 开发者排查效率提升30%
**妥协意见**: 通过(假设采纳Teddy意见，Collie妥协)
**妥协理由**:
- UX表达可接受，不影响用户感知(Teddy视角成立)
- 统一错误码规范可提升开发者排查效率，但不影响当前轮核心目标(技术视角成立)
- 妥协后仍有技术收益，可下一轮优化(技术债务可持久化)
**修复计划**:
- 修复时间: 下一
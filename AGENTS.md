# AGENTS.md（项目级标准注入点）

## 1. Soul docs / Constitution（最高锚点）

这些文档构成本项目"宪法层"，HF 在所有判断中以这些文档为最高优先级：

- **价值锚点（不可省）**：`docs/principles/soul.md`
- **方法论协作 / 反替代规则 / Phase × profile 激活**：`docs/principles/methodology-coherence.md`
- **工件管理约定（路径单一信源）**：`docs/principles/sdd-artifact-layout.md`
- **SDD + TDD 设计原则**：`docs/principles/hf-sdd-tdd-skill-design.md`
- **编码期架构健康**：`docs/principles/architectural-health-during-tdd.md`
- **模式前置 vs 浮现**：`docs/principles/emergent-vs-upfront-patterns.md`

> 任何 skill 与 soul 冲突时以 soul 为准。

## 2. 工件路径（以 sdd-artifact-layout 为权威）

- 路径权威：`docs/principles/sdd-artifact-layout.md`
- 当前 `docs/` 档位：☐ 档 0（最小） ☑ 档 1（推荐起步） ☑ 档 2（按需启用）
- 已启用的档 2 子目录（按存在同步，未启用的不阻塞）：
  - ☑ `docs/architecture.md`（档 1）
  - ☐ `docs/diagrams/`
  - ☐ `docs/runbooks/`
  - ☐ `docs/slo/`
  - ☐ `docs/postmortems/`
  - ☐ `docs/release-notes/`
  - ☐ `docs/bug-patterns/catalog.md`
  - ☐ `docs/insights/`
  - ☐ `docs/index.md`
  - ☐ `docs/arc42/`

> Feature 内工件（spec / design / tasks / progress / reviews / approvals / verification / evidence / closeout）一律落在 `features/<NNN-slug>/`，由 `sdd-artifact-layout.md` § "features/ 下放什么" 单源定义。

## 3. Execution Policy（执行模式声明）

- 默认 Execution Mode：☑ `interactive`  ☐ `auto`
- **auto 模式边界声明**（与 `skills/hf-workflow-router/references/execution-semantics.md` 对齐）：
  - auto 模式下，approval record 仍**必须**落盘。
  - approval record 是**组织策略下的可追溯代理**，记录"在已声明 policy 下，何节点放行"。
  - 它**不**替代产品负责人对最终产物（spec / design / 上线候选物）的真人确认。
  - 任何节点遇到 policy 未覆盖、或与 soul 第 1 条（方向 / 取舍 / 标准最终权在用户）冲突的情形，必须 hard stop 并回到父会话。
- 严格合规场景：☑ 禁用 auto 模式（router 必须遵守）

## 4. Coding / Testing / Architecture 标准

- 代码风格：`docs/principles/coding-principles.md`
- 语言版本与依赖管理：`docs/principles/coding-principles.md`
- 测试纪律：fresh evidence / RED-GREEN / Two Hats（见 `docs/principles/architectural-health-during-tdd.md`）；三层测试方法论——FR 验收测试 / 契约测试 / 管道集成测试（见 `docs/principles/testing-principles.md`）
- 架构原则锚点：☐ `docs/principles/architecture-principles.md`（未启用时由 soul + sdd-artifact-layout 兜底）
- 设计原则锚点路径（hf-design 读取）：`docs/principles/`
- 测试设计强制规范：§7（所有新 Feature 必填"L1~L5 自动化测试设计"章节，详细条款见 `docs/principles/testing-principles.md` §"L1~L5 自动化测试设计强制规范"；等级矩阵见 `docs/test-level-matrix.md`）

## 5. Definition of Done 项目附加项

在 `hf-completion-gate` 内置 DoD 之外，本项目额外要求：

- ☐ <所有公共 API 改动必须更新 `docs/release-notes/` 草稿>
- ☑ <性能敏感任务必须附 baseline 对比证据>
- ☐ <填空>

## 6. 其它项目级声明

- Active feature 指针来源：☑ 仓库根 `README.md`（档 0/1）  ☐ `docs/index.md`（档 2）
- ADR 编号策略：仓库级单一 pool，顺序号，从 0001 起，永不复用（详见 `sdd-artifact-layout.md`）
- 分支与 PR 约定：
  - 主分支：`main`，受保护，仅通过 PR 合并
  - 功能分支：`feat/<adr编号>-<简短描述>`，例：`feat/0012-local-server-auth`
  - 修复分支：`fix/<issue编号>-<描述>`，例：`fix/45-memory-leak`
  - PR 要求：必须通过 CI（`cargo clippy`、`pnpm lint`、`cargo test`），至少 1 人 Review，Tauri 跨平台构建通过
  - 合并策略：Squash and Merge，提交信息遵循 Conventional Commits（`feat:`、`fix:`、`refactor:`）
- 隐私 / 安全 / 合规相关声明：
  - 所有用户数据默认存储在 OS 标准应用目录，不上传 Remote Server
  - 敏感操作：涉及本地文件写入、系统命令执行的 API 必须二次确认 + 审计日志

## 7. Feature 自动化测试设计强制规范

> 详细条款、HF 全流程检查点、显式豁免清单说明见 `docs/principles/testing-principles.md` §"L1~L5 自动化测试设计强制规范"。
> 立项日期：2026-06-18（F115 建立）；适用范围：所有新立项 Feature 与新启动的 fix Feature。

### 7.1 强制条款（摘要）

- **条款 1**：`hf-design` 必须输出 `§N L1~L5 自动化测试设计` 章节（模板：`docs/principles/test-design-section-template.md`），作为 design.md 必填章节。
- **条款 2**：该章节未通过 review **不得进入** `hf-tasks` 阶段。
- **条款 3**：章节必须同时覆盖 **UX 能力（L4/L5）** 与 **非 UI 能力（L1/L2/L3）**，每条 FR 给出测试用例 + 数据 + 脚本入口。
- **条款 4**：`hf-finalize` 必须验证该章节中所有声明的测试已实现且通过，否则不通过完成门。

### 7.2 引用关系

- 强制规范详细条款：`docs/principles/testing-principles.md` §"L1~L5 自动化测试设计强制规范"
- 章节模板：`docs/principles/test-design-section-template.md`
- 测试等级矩阵：`docs/test-level-matrix.md`
- 仓库根 `README.md` "当前活动特性"附近已加提示文案（FR-2.4.3-R3）

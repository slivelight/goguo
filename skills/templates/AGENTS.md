# AGENTS.md（项目级标准注入点）

> **如何使用**：把这个文件复制到仓库根，重命名为 `AGENTS.md`，按本项目实际情况填空。HF 的所有 skill 在执行时都会从仓库根读取 `AGENTS.md`，把里面的声明视为项目级权威配置（"立标准"由用户负责）。
>
> 这是**最小骨架**，不强制全部填写；未填写的字段视为"项目当前未启用此项标准"，HF 会按 `read-on-presence` 原则容错处理。

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
  - ☑ `docs/bug-patterns/catalog.md`
  - ☑ `docs/insights/`
  - ☑ `docs/index.md`
  - ☑ `docs/arc42/`

> Feature 内工件（spec / design / tasks / progress / reviews / approvals / verification / evidence / closeout）一律落在 `features/<NNN-slug>/`，由 `sdd-artifact-layout.md` § "features/ 下放什么" 单源定义。

## 3. Execution Policy（执行模式声明）

- 默认 Execution Mode：☐ `interactive`  ☑ `auto`
- **auto 模式边界声明**（与 `skills/hf-workflow-router/references/execution-semantics.md` 对齐）：
  - auto 模式下，approval record 仍**必须**落盘。
  - approval record 是**组织策略下的可追溯代理**，记录"在已声明 policy 下，何节点放行"。
  - 它**不**替代产品负责人对最终产物（spec / design / 上线候选物）的真人确认。
  - 任何节点遇到 policy 未覆盖、或与 soul 第 1 条（方向 / 取舍 / 标准最终权在用户）冲突的情形，必须 hard stop 并回到父会话。
- 严格合规场景：☑ 禁用 auto 模式（router 必须遵守）

## 4. Coding / Testing / Architecture 标准

- 代码风格：
  1. 严格遵循 `rustfmt.toml` 格式规范与 `Cargo.toml` 中的 `[lints.clippy]` 检查规则;
  2. 提交前强制执行：`cargo fmt --all` 与 `cargo clippy --all-targets --all-features -- -D warnings`
  3. 详细配置见项目根目录 `rustfmt.toml` 及 `Cargo.toml` 顶部
- 语言版本与依赖管理：
  1. 工具链版本锁定于 `rust-toolchain.toml`，禁止依赖全局 `rustup` 状态
  2. 依赖统一使用 `^x.y.z` 语义化版本，严格提交 `Cargo.lock`，禁用 `*` 通配符
  3. 详细策略见项目根目录 `rust-toolchain.toml` 与依赖管理规范(单独文件：`docs/dependency-management.md`)
- 测试纪律：fresh evidence / RED-GREEN / Two Hats（见 `docs/principles/architectural-health-during-tdd.md`）
- 架构原则锚点：☑ `docs/principles/architecture-principles.md`（未启用时由 soul + sdd-artifact-layout 兜底）
- 设计原则锚点路径（hf-design 读取）：`docs/principles/`

## 5. Definition of Done 项目附加项

在 `hf-completion-gate` 内置 DoD 之外，本项目额外要求：

- ☐ <所有公共 API 改动必须更新 `docs/release-notes/` 草稿>
- ☑ <性能敏感任务必须附 baseline 对比证据>
- ☐ <填空>

## 6. 其它项目级声明

- Active feature 指针来源：☐ 仓库根 `README.md`（档 0/1）  ☑ `docs/index.md`（档 2）
- ADR 编号策略：仓库级单一 pool，顺序号，从 0001 起，永不复用（详见 `sdd-artifact-layout.md`）
- 分支与 PR 约定：
  - 主分支：`main`，受保护，仅通过 PR 合并
  - 功能分支：`feat/<adr编号>-<简短描述>`，例：`feat/0012-local-server-auth`
  - 修复分支：`fix/<issue编号>-<描述>`，例：`fix/45-memory-leak`
  - PR 要求：必须通过 CI（`cargo clippy`、`pnpm lint`、`cargo test`），至少 1 人 Review，Tauri 跨平台构建通过
  - 合并策略：Squash and Merge，提交信息遵循 Conventional Commits（`feat:`、`fix:`、`refactor:`）
- 隐私 / 安全 / 合规相关声明（如适用）：
  - 所有用户数据默认存储在 OS 标准应用目录，不上传 Remote Server
  - 敏感操作：涉及本地文件写入、系统命令执行的 API 必须二次确认 + 审计日志

---

> 本文件可随项目演化扩展，但 § 1（soul docs）与 § 2（工件路径权威）**必须始终存在**——这是 HF 区分"工程团队执行"与"用户拍板的标准"的分界线。

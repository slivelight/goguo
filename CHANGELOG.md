# 变更日志

本文件记录本项目的重要变更。

格式参考 Keep a Changelog；本项目遵循 HarnessFlow 工件管理纪律。

## [未发布]

### 新增

- 新增 GoGuo 战略发现草稿：`docs/insights/2026-04-29-goguo-strategy-discovery.md`。
- 新增 `OPP-002` 安装后网络评估与基线恢复产品发现草稿：`docs/insights/2026-04-30-goguo-opp-002-baseline-restore-discovery.md`。
- 新增 ADR-0001，用于建立仓库级架构决策记录机制。
- 新增初始 `README.md`，包含系统定位、当前活动特性指针和 ADR 索引。

### 变更

- 根据 14 条内联评审意见修订 GoGuo 战略发现输出。
- 将 MVP 目标范围从仅 Windows 扩展为 PC 端 Windows + Linux/WSL 工作流。
- 将恢复目标从“系统原始状态”重述为“安装时评估并确认的网络基线”。
- 记录 GoGuo 战略发现审查通过，并确认 `OPP-002`、`OPP-004`、`OPP-001` 为当前阶段优先事项。
- 根据结构化人工评审标注修订 `OPP-002` 产品发现草稿，明确初始状态快照、用户确认 baseline、Windows 与 WSL/Linux 协同环境边界、异常续跑、Proxy Guard 覆盖范围和后续模拟用户访谈置信度要求。

### 新增（2026-05-10）

- 新增 `OPP-002` 产品发现审查记录：`docs/insights/2026-05-10-goguo-opp-002-discovery-review.md`（10/10 PASS，1 条观察项）。
- 新增 `OPP-002` 产品发现审批记录：`docs/insights/2026-05-10-goguo-opp-002-discovery-approval.md`（放行进入 `hf-specify`）。
- 新增 Feature 001 目录：`features/001-baseline-restore/`。
- 新增 Feature 001 需求规格草稿：`features/001-baseline-restore/spec.md`。

### 新增（2026-05-11）

- 根据 7 条人工评审标注修订 Feature 001 需求规格：状态项分类改为平台无关、补充缺失状态项、明确子进程、修订协同部署场景、评估性能指标、统一数据存储路径、修订 CON-1 为平台无关。
- 新增 Feature 001 规格审查记录：`features/001-baseline-restore/reviews/spec-review-2026-05-11.md`（3 条问题已修订确认，PASS）。
- 新增 Feature 001 规格审批记录：`features/001-baseline-restore/approvals/spec-approval-2026-05-11.md`（规格确认完成）。
- 新增 `OPP-004` PC 端 Linux/WSL 支持产品发现草稿：`docs/insights/2026-05-11-goguo-opp-004-wsl-support-discovery.md`。
- 新增 `OPP-004` 产品发现审查记录：`docs/insights/2026-05-11-goguo-opp-004-discovery-review.md`（10/10 PASS）。
- 新增 `OPP-004` 产品发现审批记录：`docs/insights/2026-05-11-goguo-opp-004-discovery-approval.md`（放行进入 `hf-specify`）。
- 新增 Feature 002 目录：`features/002-wsl-support/`。
- 新增 Feature 002 需求规格草稿：`features/002-wsl-support/spec.md`。
- 新增 Feature 002 规格审查记录：`features/002-wsl-support/reviews/spec-review-2026-05-11.md`（1 条低优先级问题已修订，PASS）。
- 新增 Feature 002 规格审批记录：`features/002-wsl-support/approvals/spec-approval-2026-05-11.md`（规格确认完成）。
- 新增 `OPP-001` 目标站点规则配置与可达性诊断产品发现草稿：`docs/insights/2026-05-11-goguo-opp-001-site-rules-discovery.md`。
- 新增 `OPP-001` 产品发现审查记录：`docs/insights/2026-05-11-goguo-opp-001-discovery-review.md`（10/10 PASS）。
- 新增 `OPP-001` 产品发现审批记录：`docs/insights/2026-05-11-goguo-opp-001-discovery-approval.md`（放行进入 `hf-specify`）。
- 新增 Feature 003 目录：`features/003-site-rules/`。
- 新增 Feature 003 需求规格草稿：`features/003-site-rules/spec.md`。
- 新增 Feature 003 规格审查记录：`features/003-site-rules/reviews/spec-review-2026-05-11.md`（7/7 PASS）。
- 新增 Feature 003 规格审批记录：`features/003-site-rules/approvals/spec-approval-2026-05-11.md`（规格确认完成）。

### 变更（2026-05-11 续）

- 根据 11 条人工评审标注修订 Feature 003 需求规格：SC-1 调整为两档性能目标（常规 10s/节点切换 30s）；FR-2.1.2-R3 重构为站点定义（Site Definition）数据模型；新增 FR-2.1.1-R6（自定义站点定义保存）；FR-2.2.2-R3 增加回退用户通知；新增 FR-2.3.2-R5/R6/R7（代理节点池管理）；NFR-3.1-3 收紧为 3s 并新增 NFR-3.1-4 并行探测；ASM-2 置信度下调；新增 OP-5（需补充 OPP-003 用户交互界面）。
- 根据 2 条审查问题修订：chatgpt/OpenAI 站点定义合并消除域名重叠；ASM-3 更新为展开后域名规模（200-500 条）。
- 新增 Feature 003 第二轮规格审查记录：`features/003-site-rules/reviews/spec-review-2026-05-11-r2.md`（7/7 PASS，2 条问题已修订，1 条观察项）。
- 更新 Feature 003 规格审批记录：两轮审查 + 11 条标注修订后确认完成。

### 新增（2026-05-11 续二）

- 新增 `OPP-003` 用户交互界面产品发现草稿：`docs/insights/2026-05-11-goguo-opp-003-user-interaction-discovery.md`（从 P1 升级为阻塞依赖，范围从"最小状态面板"扩展为"独立桌面应用"）。
- 根据 3 条人工评审标注修订 OPP-003 产品发现：技术选型从 Web UI 改为独立桌面应用、增加跨平台一致性要求、增加手工调整引导+重新采集流程。
- 新增 `OPP-003` 产品发现审查记录：`docs/insights/2026-05-11-goguo-opp-003-discovery-review.md`（10/10 PASS，1 条问题已修订，2 条观察项）。
- 新增 `OPP-003` 产品发现审批记录：`docs/insights/2026-05-11-goguo-opp-003-discovery-approval.md`（放行进入 `hf-specify`）。
- 新增 Feature 004 目录：`features/004-user-interaction/`。
- 新增 Feature 004 需求规格草稿：`features/004-user-interaction/spec.md`（7 模块 ~40 FR + 10 NFR + 5 CON + 5 SC）。
- 根据 6 条人工评审标注修订 Feature 004 需求规格：新增无登录/鉴权管理员运行约束、step by step 手工调整指引、部署模式仅影响运行时、批量预览按站点分组、节点池元数据展示、冷启动收紧为 3s。
- 新增 Feature 004 规格审查记录：`features/004-user-interaction/reviews/spec-review-2026-05-12.md`（7/7 PASS）。
- 新增 Feature 004 规格审批记录：`features/004-user-interaction/approvals/spec-approval-2026-05-12.md`（规格确认完成）。

## [初始版本]

- 待定。

### 新增（2026-05-12）

- 新增 ADR-0002：Desktop App Framework — Tauri。
- 新增 ADR-0003：mihomo 集成架构 — 托管子进程。
- 新增 ADR-0004：数据存储策略 — 安装根目录下文件式 JSON。
- 新增 ADR-0005：跨平台策略 — Platform Adapter 模式。
- 新增 ADR-0006：前端框架选型 — React + TypeScript。
- 更新 `docs/architecture.md`：填充统一架构概述（系统目标、约束、C4 组件视图、关键交互、非功能属性、风险、术语表、ADR 索引）。
- 新增 Feature 001 设计文档：`features/001-baseline-restore/design.md`。
- 新增 Feature 002 设计文档：`features/002-wsl-support/design.md`。
- 新增 Feature 003 设计文档：`features/003-site-rules/design.md`。
- 新增 Feature 004 设计文档：`features/004-user-interaction/design.md`。
- 新增 Feature 004 UI 设计文档：`features/004-user-interaction/ui-design.md`。
- 更新 Feature 001~004 的 `README.md` 状态为 `hf-design`。

### 变更（2026-05-14）

- 根据 7 条人工评审标注修订 Feature 001 设计文档。
- WSL/Linux 适配器拆分为 `WslAdapter` + `LinuxAdapter`，共享 `LinuxBaseAdapter` 公共逻辑（标注 id:01）。
- WslAdapter 新增 `wsl-wsl2-network-mode` 检测项（标注 id:02）。
- 补充 `MihomoConfig` 结构定义（标注 id:03）。
- 明确 ProxyGuard 自动恢复无需二次确认，但必须通知用户；与用户主动停止（需确认）区分（标注 id:04）。
- 审计查询新增分页和过滤机制（标注 id:05）。
- 续跑期间 UI 新增恢复蒙层组件（标注 id:06）。
- 恢复任务新增状态机设计（Pending/InProgress/Completed/Failed/UserAcknowledged）和闭环路径（标注 id:07）。
- 同步更新 ADR-0005、Feature 002/004 设计文档和 `docs/architecture.md`。

### 变更（2026-05-14 续）

- 根据 1 条人工评审标注修订 Feature 002 设计文档：WslAdapter 和 LinuxAdapter 补充 `shell-proxy` 和 `reachability` 两个可检测不可恢复项，与 Feature 001 定义对齐。
- 根据 7 条人工评审标注修订 Feature 003 设计文档。
- 确认 MVP 阶段所有目标站点共享单一 PROXY 代理组，未来可按站点分配不同代理组（标注 id:01）。
- 补充分层探测策略（Level 1 DNS+HEAD → Level 2 GET+状态码 → Level 3 TLS 诊断），ProbeMethod 新增 `TlsHandshake`（标注 id:02）。
- 确认不采用主备 mihomo 进程设计，NodePool 生命周期管理在 Rust 后端完成（标注 id:03）。
- 确认 5 种代理协议均为 mihomo 原生支持，不支持协议在订阅解析阶段过滤并审计记录（标注 id:04）。
- 非目标站点可达性验证采用"静态校验 + A/B 即时探测"方案，不修改 baseline 数据模型（标注 id:05）。
- 域名规则数量限制从"200-500 条硬限制"修订为分档性能目标：标准档 500 / 扩展档 1000 / 压力档 2000+（标注 id:06）。
- 测试策略新增 P99 恢复时间目标：常规 ≤ 20s / 节点切换 ≤ 60s（标注 id:07）。
- 同步更新 Feature 001 design.md（BaselineSnapshot 数据模型）、Feature 003 spec.md（CON-3、ASM-3）和 `docs/architecture.md`（风险表）。

### 变更（2026-05-15）

- 根据 30 条人工评审标注修订 Feature 004 UI 设计文档。
- 设计原则从 4 条扩展为 6 条：原"信息密度优先"重述为"默认友好，进阶可达"，新增"恢复信心可见"和"意图优先，技术隐藏"。补充渐进披露三层定义和四类敏感操作明确列举（标注 id:01~05）。
- 状态栏新增恢复任务状态展示（标注 id:07）。
- 仪表盘状态卡片从 4 个（服务/Baseline/Windows/WSL）合并为 3 个（服务/Baseline/部署模式），站点可达性改为语义化展示（标注 id:08~11）。
- 用户界面术语全面用户化：目标站点→需要访问的网站、诊断→网站状态、代理节点池→访问通道、节点→通道（标注 id:12,15,20,21,23）。
- 站点管理页面：输入框占位符改为意图导向文案，预设模板改为下拉预览，不可达站点改为内嵌诊断面板（标注 id:13,14,16）。
- 规则预览统计改为语义化："为 N 个网站配置了访问方式"（标注 id:18）。
- 设置页部署模式从 3 项改为 4 项，对齐 Feature 002（标注 id:17）。
- Wizard：Step 1 增加用户意图唤醒，Step 3 增加整体进度指示和"一键自动调整"推荐入口，Step 4 术语改为语义化（标注 id:24~27）。
- 确认对话框增加"当前值 → 恢复后"后果可视化（标注 id:28）。
- 通知面板增加"查看全部通知"历史归档入口（标注 id:29）。
- 交互状态清单新增恢复任务状态行（标注 id:30）。

### 变更（2026-05-15 续）

- 执行 Feature 001~004 跨文档设计评审，发现 11 项问题（5 HIGH / 4 MEDIUM / 2 LOW），修复 7 项高/中等问题。
- Feature 001：RecoveryStatus 枚举补全 `UserAcknowledged`；AppConfig 补充 `ProbeConfig` 和 `non_target_probe_sites` 字段。
- Feature 002：新增 Tauri Commands 节（5 个部署模式相关命令）；章节编号修正；C4 约束措辞精确化。
- Feature 003：补充 `ProbeHistory`/`ProbeMethod`/`NodeHealthChecker`/`NodeHealthResult` 数据模型定义；新增 `SubscriptionParser` 模块（订阅解析、协议过滤）；Tauri Commands 补充 `import_subscription`/`get_subscription_sources`。
- Feature 004 design.md：§4.2 仪表盘描述同步（部署模式卡片 + 语义化可达性）；§5.2 确认表补充"部署模式切换"。

### 新增（2026-05-15 续二）

- 新增 Feature 001 设计审查记录：`features/001-baseline-restore/reviews/design-review-2026-05-15.md`（7/7 PASS，7 条标注已修订，2 条跨文档问题已修复）。
- 新增 Feature 001 设计审批记录：`features/001-baseline-restore/approvals/design-approval-2026-05-15.md`（设计确认完成）。
- 新增 Feature 002 设计审查记录：`features/002-wsl-support/reviews/design-review-2026-05-15.md`（7/7 PASS，1 条标注已修订，1 条跨文档问题已修复）。
- 新增 Feature 002 设计审批记录：`features/002-wsl-support/approvals/design-approval-2026-05-15.md`（设计确认完成）。
- 新增 Feature 003 设计审查记录：`features/003-site-rules/reviews/design-review-2026-05-15.md`（7/7 PASS，7 条标注已修订，2 条跨文档问题已修复，1 条观察项）。
- 新增 Feature 003 设计审批记录：`features/003-site-rules/approvals/design-approval-2026-05-15.md`（设计确认完成）。
- 新增 Feature 004 设计审查记录：`features/004-user-interaction/reviews/design-review-2026-05-15.md`（7/7 PASS，30 条 UI 标注已修订，2 条跨文档问题已修复）。
- 新增 Feature 004 设计审批记录：`features/004-user-interaction/approvals/design-approval-2026-05-15.md`（设计确认完成）。

### 新增（2026-05-18）

- 新增 Feature 001 任务计划：`features/001-baseline-restore/tasks.md`（10 里程碑、17 个任务）。
- 新增 Feature 002 任务计划：`features/002-wsl-support/tasks.md`（7 里程碑、8 个任务）。
- 新增 Feature 003 任务计划：`features/003-site-rules/tasks.md`（9 里程碑、11 个任务）。
- 新增 Feature 004 任务计划：`features/004-user-interaction/tasks.md`（10 里程碑、13 个任务）。
- 新增 Feature 001~004 进度跟踪文件：`features/*/progress.md`。
- 更新 Feature 001~004 README 状态为 `hf-tasks`。

### 变更（2026-05-18 续）

- 新增 Feature 001 任务计划审查记录：`features/001-baseline-restore/reviews/tasks-review-2026-05-18.md`（通过，8.5/10，3 条 minor findings）。
- 新增 Feature 002 任务计划审查记录：`features/002-wsl-support/reviews/tasks-review-2026-05-18.md`（通过，8.7/10，2 条 minor findings + 1 观察项）。
- 新增 Feature 003 任务计划审查记录：`features/003-site-rules/reviews/tasks-review-2026-05-18.md`（通过，8.3/10，1 important + 3 minor findings）。
- 新增 Feature 004 任务计划审查记录：`features/004-user-interaction/reviews/tasks-review-2026-05-18.md`（通过，8.2/10，1 important + 4 minor findings）。

### 变更（2026-05-18 续二）

- 修复 F003 T6.1 B+C 验证测试种子：补充 4 个边界场景（空参考站点、全部超时、pre/post 都不可达、部分可达）。
- 修复 F004 T2.1 粒度问题：拆分为 T2.1a（核心 3 Store）+ T2.1b（扩展 4 Store），同步更新依赖链、队列投影和关键路径。
- 新增 Feature 001 任务计划审批记录：`features/001-baseline-restore/approvals/tasks-approval-2026-05-18.md`（放行进入 hf-test-driven-dev）。
- 新增 Feature 002 任务计划审批记录：`features/002-wsl-support/approvals/tasks-approval-2026-05-18.md`（放行）。
- 新增 Feature 003 任务计划审批记录：`features/003-site-rules/approvals/tasks-approval-2026-05-18.md`（放行，important finding 已修复）。
- 新增 Feature 004 任务计划审批记录：`features/004-user-interaction/approvals/tasks-approval-2026-05-18.md`（放行，important finding 已修复）。
- Feature 001~004 hf-tasks 阶段全部完成，统一进入 `hf-test-driven-dev`。

### 新增（2026-05-19）

- 完成 T1.1 项目脚手架：Tauri 2.x + React + TypeScript + pnpm 项目初始化。
- 新增 `Cargo.toml`（workspace）、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`、`src-tauri/src/lib.rs`、`src-tauri/src/main.rs`、`src-tauri/build.rs`。
- 新增 `package.json`、`vite.config.ts`、`tsconfig.json`、`tsconfig.node.json`、`index.html`。
- 新增 `src/App.tsx`、`src/main.tsx`、`src/vite-env.d.ts`。
- 修复 `rust-toolchain.toml` TOML 格式错误，更新为 stable 通道。
- 更新 `.gitignore`：移除对 `src-tauri/tauri.conf.json` 的忽略。
- 验证通过：`cargo build`（Rust 后端编译成功）、`pnpm build`（前端构建成功）、`cargo clippy`（零 error）。

### 新增（2026-05-20）

- 完成 Feature 001 全部 17 个任务（T1.1 项目脚手架 ~ T10.1 集成测试），包括 WindowsAdapter（T3.2，在 Windows 环境下 TDD 完成）。
- 实现 8 个核心模块：数据模型、存储层、PlatformAdapter trait、WindowsAdapter、BaselineManager、MihomoManager、AuditLogger、ConfigManager、ProxyGuard、RecoveryManager、Tauri Commands。
- 测试覆盖：144 个单元测试 + 11 个集成测试 = 155 个测试全绿，clippy 零警告。
- 翻转 ADR-0002/0003/0004/0005/0006 状态为 `accepted`（设计已落地）。
- Feature 001 workflow closeout 完成。
- 完成 Feature 002 全部 8 个任务（T2.1 LinuxBaseAdapter ~ T7.1 集成测试）。
- 实现 7 个核心模块：LinuxBaseAdapter、WslDetector、WslNetworkStrategy、WslAdapter、LinuxAdapter、DeploymentManager、5 个 Tauri Commands。
- 测试覆盖：125 个新增测试（Feature 002），项目总测试 280 个全绿，clippy 零警告。
- Feature 002 workflow closeout 完成。

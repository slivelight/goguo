# GoGuo

GoGuo（过得去）是面向普通办公用户与 PC 端开发者/知识工作者的本地网络可达性工具，目标是在国内网络环境下跨家庭、办公、差旅场景切换时，让指定目标网站与开发工具链可解释地恢复可用，同时不破坏原本可直连网站的访问。

## 系统定位

GoGuo 优先定位为“PC 端 Windows + Linux/WSL 网络可达性诊断、基线恢复与目标站点规则辅助”的工具。它不以完整复制通用代理客户端为第一目标，而是把现有 github-host 原型中的网络检测、策略决策、节点/IP/订阅质量追踪、监控恢复能力包装成普通用户与开发者可使用的产品闭环。

## 当前活动特性

> 所有 Feature design 必须含 "L1~L5 自动化测试设计" 章节（详见 [AGENTS.md §7](AGENTS.md#7-feature-自动化测试设计强制规范)）；等级矩阵见 [docs/test-level-matrix.md](docs/test-level-matrix.md)。

- **Feature 001: 安装后网络评估与基线恢复** — `closed`（2026-05-20 workflow closeout）
  - 17/17 任务完成，155 测试全绿，clippy 零警告
  - 详见 `features/001-baseline-restore/closeout.md`
- **Feature 002: WSL/Linux 支持** — `closed`（2026-05-20 workflow closeout）
  - 8/8 任务完成，125 测试全绿，clippy 零警告
  - 详见 `features/002-wsl-support/closeout.md`
- **Feature 003: 目标站点规则配置与可达性诊断** — `closed`（2026-05-20 workflow closeout）
  - 11/11 任务完成，206 测试全绿，clippy 零警告
  - 详见 `features/003-site-rules/closeout.md`
- **Feature 004: 用户交互界面** — `closed`（2026-05-21 workflow closeout）
  - 14/14 任务完成，146 前端测试全绿，P0~P8 业务审视缺陷全部修复
  - 详见 `features/004-user-interaction/closeout.md`

- **Feature 113: 三层测试重构** — `closed`（2026-06-13）
  - 12/12 任务完成，88 测试（60 passed + 28 ignored），24.75s，零生产代码变更
  - 新发现 1 项：`add_user_override` 审计 gap → F114
  - 详见 `features/113-test-restructure/progress.md`、`docs/test-trace-matrix.md`

- **Feature 109: Baseline 恢复语义修复** — `进行中`
  - 9 项 P0~P2 gap 修复（语义反转、WSL 分类、审计日志、ProxyGuard 扩展等）
  - 详见 `features/109-baseline-restore-semantic-fix/spec.md`
- **Feature 110: F001~004 设计-实现 Gap 闭环** — `spec/design 草稿`
  - 20 项剩余差距系统性闭环（3 P0 + 9 P1 + 11 P2）
  - 详见 `features/110-design-gap-closure/spec.md`

## 测试

仓库使用三层测试方法论（[详见 principles/testing-principles.md](docs/principles/testing-principles.md)）：单元（unit）/ FR 验收（fr-acceptance）/ 端到端（e2e）。每个 Feature 的 `design.md` 必须含"L1~L5 自动化测试设计"章节（详见 [AGENTS.md §7](AGENTS.md#7-feature-自动化测试设计强制规范)）。

**主入口**（含三层测试）：

```bash
pnpm test:all
```

**分层入口**：

| 命令 | 范围 | 适用场景 |
|------|------|---------|
| `pnpm test` | 主仓库前端单元 + fr-acceptance（vitest）| 迭代开发期快速反馈 |
| `pnpm test:e2e` | e2e 套件（WebDriverIO + tauri-driver）| 桌面端到端验证（首次配置见 [`e2e/README.md`](e2e/README.md) Step 0）|
| `pnpm test:all` | 上述两层全跑 | finalize / pre-commit 全量验证 |
| `pnpm test:feature -- f<NNN>` | 单 Feature 三层（cargo + vitest + e2e）| Feature finalize 时定向验证 |
| `pnpm test:e2e:feature -- f<NNN>-<slug>` | 单 Feature e2e 层 | e2e 调试 / 局部验证 |
| `cargo test --workspace` | 后端单元 + fr-acceptance + 集成测试 | 后端迭代 / pre-commit |

**等级矩阵**：[`docs/test-level-matrix.md`](docs/test-level-matrix.md)（每条 FR 的 L1~L5 责任行）。
**追溯矩阵**：[`docs/test-trace-matrix.md`](docs/test-trace-matrix.md)（FR ID → 测试函数名 1:1 映射）。

## ADR 索引

- [ADR-0001：记录架构决策](docs/adr/0001-record-architecture-decisions.md)
- [ADR-0002：Desktop App Framework — Tauri](docs/adr/0002-tauri-desktop-framework.md)
- [ADR-0003：mihomo 集成架构 — 托管子进程](docs/adr/0003-mihomo-subprocess-integration.md)
- [ADR-0004：数据存储策略 — 文件式 JSON](docs/adr/0004-file-based-json-storage.md)
- [ADR-0005：跨平台策略 — Platform Adapter 模式](docs/adr/0005-platform-adapter-pattern.md)
- [ADR-0006：前端框架选型 — React + TypeScript](docs/adr/0006-react-frontend-framework.md)

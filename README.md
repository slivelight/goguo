# GoGuo

GoGuo（过得去）是面向普通办公用户与 PC 端开发者/知识工作者的本地网络可达性工具，目标是在国内网络环境下跨家庭、办公、差旅场景切换时，让指定目标网站与开发工具链可解释地恢复可用，同时不破坏原本可直连网站的访问。

## 系统定位

GoGuo 优先定位为“PC 端 Windows + Linux/WSL 网络可达性诊断、基线恢复与目标站点规则辅助”的工具。它不以完整复制通用代理客户端为第一目标，而是把现有 github-host 原型中的网络检测、策略决策、节点/IP/订阅质量追踪、监控恢复能力包装成普通用户与开发者可使用的产品闭环。

## 当前活动特性

- **Feature 001: 安装后网络评估与基线恢复** — `closed`（2026-05-20 workflow closeout）
  - 17/17 任务完成，155 测试全绿，clippy 零警告
  - 详见 `features/001-baseline-restore/closeout.md`
- **Feature 002: WSL/Linux 支持** — `closed`（2026-05-20 workflow closeout）
  - 8/8 任务完成，125 测试全绿，clippy 零警告
  - 详见 `features/002-wsl-support/closeout.md`

下一步候选：

- Feature 003: 目标站点规则配置（`features/003-site-rules/`）
- Feature 004: 用户交互界面（`features/004-user-interaction/`）

## ADR 索引

- [ADR-0001：记录架构决策](docs/adr/0001-record-architecture-decisions.md)
- [ADR-0002：Desktop App Framework — Tauri](docs/adr/0002-tauri-desktop-framework.md)
- [ADR-0003：mihomo 集成架构 — 托管子进程](docs/adr/0003-mihomo-subprocess-integration.md)
- [ADR-0004：数据存储策略 — 文件式 JSON](docs/adr/0004-file-based-json-storage.md)
- [ADR-0005：跨平台策略 — Platform Adapter 模式](docs/adr/0005-platform-adapter-pattern.md)
- [ADR-0006：前端框架选型 — React + TypeScript](docs/adr/0006-react-frontend-framework.md)

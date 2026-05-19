# GoGuo

GoGuo（过得去）是面向普通办公用户与 PC 端开发者/知识工作者的本地网络可达性工具，目标是在国内网络环境下跨家庭、办公、差旅场景切换时，让指定目标网站与开发工具链可解释地恢复可用，同时不破坏原本可直连网站的访问。

## 系统定位

GoGuo 优先定位为“PC 端 Windows + Linux/WSL 网络可达性诊断、基线恢复与目标站点规则辅助”的工具。它不以完整复制通用代理客户端为第一目标，而是把现有 github-host 原型中的网络检测、策略决策、节点/IP/订阅质量追踪、监控恢复能力包装成普通用户与开发者可使用的产品闭环。

## 当前活动特性

当前处于产品规划阶段，尚无 active feature。

下一步候选入口：

- `docs/insights/2026-04-29-goguo-strategy-discovery.md`
- 当前推荐 wedge：`OPP-002` 安装后网络评估与基线恢复
- 目标环境前置：`OPP-004` PC 端 Linux/WSL 支持
- 产品价值 wedge：`OPP-001` 目标站点规则配置与可达性诊断

## ADR 索引

- [ADR-0001：记录架构决策](docs/adr/0001-record-architecture-decisions.md)

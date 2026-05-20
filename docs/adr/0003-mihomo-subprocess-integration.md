# ADR-0003: mihomo 集成架构 — 托管子进程

- **Status**: accepted
- **Date**: 2026-05-12
- **Deciders**: 用户
- **Affected Features**: 001, 003

## Context

GoGuo 的核心代理能力由 mihomo（Clash Meta 内核）提供。mihomo 是 Go 语言编写的独立代理程序，接受 YAML 配置文件和 RESTful API 控制。GoGuo 需要：

1. 管理 mihomo 进程生命周期（启动/停止/崩溃恢复）（Feature 001 FR-2.5）
2. 基于站点定义自动生成 mihomo 规则配置（Feature 003 FR-2.2）
3. 通过 mihomo API 执行配置热重载（Feature 003 NFR-3.1-2: 5s 内）
4. 监控 mihomo 进程健康状态（Feature 001 FR-2.5.1）

## Decision

**mihomo 作为托管子进程运行，GoGuo 通过配置文件生成 + RESTful API 控制集成。**

集成模式：
- **配置驱动**：GoGuo 生成 mihomo YAML 配置文件，通过 API 触发热重载
- **进程托管**：GoGuo 负责启动/停止/监控 mihomo 进程
- **API 通信**：通过 mihomo 的 RESTful API（默认 `127.0.0.1:9090`）获取状态和触发操作
- **Proxy Guard 覆盖**：Proxy Guard 监控 mihomo 进程存活、端口监听和 API 响应

## Alternatives Considered

| 方案 | 优势 | 劣势 | 结论 |
|------|------|------|------|
| **托管子进程 + 配置 + API** | 不修改 mihomo 源码、升级灵活、隔离清晰 | 进程管理复杂度、IPC 延迟 | **选定** |
| 嵌入 mihomo（cgo/FFI） | 无进程管理、调用直接 | Go/Rust FFI 复杂、升级困难、mihomo 非库设计 | 排除 |
| 纯 API（用户自行启动 mihomo） | GoGuo 职责最简 | 用户体验差、进程不可控、Proxy Guard 无法保障 | 排除：违反 Feature 001 FR-2.5 |
| 替换为 Rust 原生代理核心 | 单进程、无 IPC | 开发量巨大、协议覆盖不足 | 排除：远期假设 |

## Consequences

- **正面**: mihomo 版本独立升级；配置文件可人工审查；API 接口标准化；与现有 MonitorServer 模式一致。
- **负面**: 需处理子进程管理（僵尸进程、异常退出、端口冲突）；配置生成到生效有 ~1s 延迟（含 API 调用）；mihomo 二进制需随 GoGuo 分发。
- **关键接口**:
  - 配置文件路径：`<install-root>/data/mihomo/config.yaml`
  - API 地址：`127.0.0.1:<configured-port>`（默认 9090）
  - 热重载 API：`PUT /configs?force=true` + 配置文件路径
  - 健康 API：`GET /version`、`GET /proxies`

## References

- `features/001-baseline-restore/spec.md`（FR-2.5 Proxy Guard）
- `features/003-site-rules/spec.md`（FR-2.2 规则生成、NFR-3.1-2）
- `docs/insights/2026-04-29-goguo-strategy-discovery.md`（现有 MonitorServer REST API）

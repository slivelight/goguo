# ADR-0005: 跨平台策略 — Platform Adapter 模式

- **Status**: proposed
- **Date**: 2026-05-12
- **Deciders**: 用户
- **Affected Features**: 001, 002

## Context

GoGuo 需要在 Windows、WSL 和独立 Linux 三种环境下执行网络状态采集、配置和恢复操作。三种环境的状态项不同：

- **Windows**: hosts 文件、注册表（系统代理/PAC）、WinHTTP、DNS 缓存、TUN 等
- **WSL**: 环境变量、Git 配置、`/etc/resolv.conf`、`/etc/environment`、WSL2 网络模式等
- **Linux**: 环境变量、Git 配置、`/etc/resolv.conf`、`/etc/environment` 等（与 WSL 共享大部分逻辑，但无 WSL2 特有检测项）

Feature 001/002 要求：
- 共享核心恢复流程逻辑，新增平台只需实现适配层（Feature 002 NFR-3.5-1）
- 平台无关的状态项分类（可恢复/可检测不可恢复/暂不覆盖）
- 运行时自动检测部署组合（Feature 001 FR-2.9.1）
- 支持独立 Linux 环境部署 GoGuo

## Decision

**采用 Platform Adapter trait 模式：定义跨平台抽象接口，按平台实现具体适配器，运行时根据检测到的环境选择适配器组合。**

核心设计：

```rust
/// 跨平台状态项采集接口
trait PlatformAdapter: Send + Sync {
    /// 采集所有该平台的状态项
    fn collect_state_items(&self) -> Result<Vec<StateItem>>;

    /// 读取单个状态项的当前值
    fn read_state(&self, item_id: &str) -> Result<StateValue>;

    /// 写入单个状态项（仅可恢复项）
    fn write_state(&self, item_id: &str, value: &StateValue) -> Result<()>;

    /// 返回该平台支持的状态项定义列表
    fn state_definitions(&self) -> Vec<StateItemDef>;
}

/// Windows 平台适配器
struct WindowsAdapter { /* registry API, hosts path, etc. */ }

/// WSL 平台适配器（包含 WSL2 网络模式检测）
struct WslAdapter { /* shell exec, file paths, wsl detector */ }

/// 独立 Linux 平台适配器
struct LinuxAdapter { /* shell exec, file paths */ }
```

`WslAdapter` 和 `LinuxAdapter` 共享 `LinuxBaseAdapter` 公共逻辑（环境变量读写、Git 配置、`/etc/resolv.conf`、`/etc/environment`）。

部署组合映射：

| 部署组合 | 活跃适配器 |
|----------|-----------|
| 仅 Windows | `WindowsAdapter` |
| 仅 WSL | `WslAdapter` |
| 仅 Linux | `LinuxAdapter` |
| Windows + WSL 协同 | `WindowsAdapter` + `WslAdapter` |

## Alternatives Considered

| 方案 | 优势 | 劣势 | 结论 |
|------|------|------|------|
| **Platform Adapter trait（三适配器）** | 核心逻辑共享、WSL/Linux 独立演进、支持独立 Linux 部署 | 公共逻辑需抽象到 base | **选定** |
| 合并 WSL/Linux 为单一适配器 | 实现量少 | 无法区分 WSL 独有特性（WSL2 网络模式）、独立 Linux 部署时携带无用 WSL 逻辑 | 排除 |
| 条件编译 (`#[cfg(target_os)]`) | 零运行时开销 | 编译时绑定、不可动态切换、测试需交叉编译 | 排除：协同部署需同时支持两侧 |
| 独立二进制（每平台一个） | 完全解耦 | 多代码库、UI 不一致、违反 Feature 004 CON-2 | 排除 |

## Consequences

- **正面**: 新增平台只需实现 `PlatformAdapter` trait；核心恢复逻辑（BaselineManager）完全不感知平台差异；WSL 和 Linux 各自独立演进；单元测试可用 mock adapter 覆盖。
- **负面**: 公共逻辑提取到 `LinuxBaseAdapter` 增加一层抽象；协同部署下两个适配器同时运行增加复杂度。
- **测试策略**: 为每个适配器编写集成测试，在真实平台环境中执行读写矩阵验证（Feature 001 ASM-1, Feature 002 ASM-1）。

## References

- `features/001-baseline-restore/spec.md`（FR-2.1.1, FR-2.9.1, NFR-3.4-1）
- `features/002-wsl-support/spec.md`（NFR-3.5-1, FR-2.2.2-R1）

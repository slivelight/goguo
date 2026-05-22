# ADR-0007: 协同模式 RemoteAdapter 模式

- **Status**: accepted
- **Date**: 2026-05-22
- **Deciders**: 用户
- **Affected Features**: 101 (fix for 002)

## Context

ADR-0005 承诺：Platform Adapter trait 模式，运行时动态选择适配器，协同模式（Coordinated）下同时创建双侧适配器。v0.1.0 实现违反了此决策，使用 `#[cfg(target_os)]` 条件编译导致：

1. 协同模式只创建 1 个适配器（宿主平台），无法操作对侧
2. `wsl-proxy-env` write_state 是空操作（`Ok(())`）
3. ADR-0005 明确排除的 `#[cfg]` 方案被采用（40+ 处）

需要在兑现 ADR-0005 承诺的同时，不破坏 484 个现有测试。

## Decision

**新增 RemoteAdapter 模式：不修改现有适配器，为每个远程方向创建独立的 RemoteAdapter，通过桥接执行器跨平台操作。**

### 架构

```
方向 A: Windows 单实例 → 管理 Win + WSL
  WindowsAdapter (本地, cfg(windows))
  WslRemoteAdapter<E: CommandExecutor> (远程, 通过 wsl -e)

方向 B: WSL 单实例 → 管理 Win + WSL
  WslAdapter (本地, cfg(linux))
  WindowsRemoteAdapter<E: CommandExecutor> (远程, 通过 powershell.exe)
```

### 核心抽象

```rust
trait CommandExecutor: Send + Sync {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String, String>;
    fn env_var(&self, key: &str) -> Option<String>;
    fn home_dir(&self) -> Option<PathBuf>;
    fn read_file(&self, path: &str) -> Result<String, String>;
    fn write_file(&self, path: &str, content: &str) -> Result<(), String>;
}
```

4 种实现：
- `SystemCommandExecutor` — 本地 `std::process::Command`
- `WslBridgeExecutor` — Windows→WSL，前缀 `wsl [-d distro] -e`
- `PowershellBridgeExecutor` — WSL→Windows，包装 `powershell.exe -Command`
- `MockCommandExecutor` — 测试替身

### 为什么保留 cfg gate

`windows.rs` 依赖 `winreg` crate（仅 Windows 编译），无法在 Linux 编译。同理 `wsl.rs` 使用 Linux API。RemoteAdapter 模式不要求移除这些 cfg gate，而是在运行时通过桥接跨平台操作。

**cfg 使用合规性**：cfg 仅用于本地适配器的编译隔离（技术约束），不再用于部署模式的逻辑选择（架构约束）。后者通过 DeploymentManager 运行时动态完成。

### DeploymentManager 修改

```rust
Coordinated => {
    // Windows: 2 adapters
    #[cfg(target_os = "windows")]
    { WindowsAdapter(本地) + WslRemoteAdapter(远程) }
    // WSL: 2 adapters
    #[cfg(target_os = "linux")]
    { WslAdapter(本地) + WindowsRemoteAdapter(远程) }
}
```

## Alternatives Considered

### 1. 移除所有 cfg gate（被排除）

将 `windows.rs` 的 `winreg` 依赖替换为 PowerShell 命令执行，使所有适配器在所有平台编译。

排除原因：
- 重写 484 个现有测试的风险
- `winreg` 提供类型安全的注册表操作，比 PowerShell 字符串解析更可靠
- 现有适配器经过充分验证，不应为架构纯粹性重写

### 2. 条件编译 + 特性门控（被排除）

使用 Cargo feature 而非 `#[cfg(target_os)]`，如 `#[cfg(feature = "windows-adapter")]`。

排除原因：
- 仍为编译时绑定，不满足运行时动态选择要求
- 增加 CI 构建矩阵复杂度
- 不解决协同模式双侧同时操作的核心问题

### 3. RemoteAdapter 模式（采纳）

新增远程适配器 + 桥接执行器，保留现有适配器不变。

采纳原因：
- 零回归风险（现有 484 测试不变）
- 远程适配器可独立测试（MockCommandExecutor）
- 桥接执行器封装跨平台差异
- 纯解析函数提取到 `windows_base.rs` / `linux_base.rs`，跨模块复用

## Consequences

- **正面**：协同模式兑现 ADR-0005 承诺，双侧同时管理
- **正面**：WslRemoteAdapter 的 `write_proxy_env()` 修复了 F102（proxy-env 空操作）
- **正面**：`windows_base.rs` 和 `linux_base.rs` 的纯解析函数可跨平台复用
- **风险**：远程操作依赖 `wsl`/`powershell.exe` 可用性，运行时可能失败
- **风险**：WindowsOnly 模式在 WSL 上的默认行为变化（0→1 adapter）

## References

- ADR-0005: `docs/adr/0005-platform-adapter-pattern.md`
- F002 spec: `features/002-wsl-support/spec.md` §FR-2.2.1-R4, SC-2
- 审计报告: `docs/insights/2026-05-21-v010-spec-design-impl-drift-audit.md`

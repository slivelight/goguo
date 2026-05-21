# F002 Coordinated Mode Gap Analysis

- **Date**: 2026-05-21
- **Type**: hf-hotfix 输入（规格-设计-实现契约断裂）
- **Severity**: BLOCKER（协同模式完全不可用）

## 问题描述

F002 spec、design、ADR-0005 三层文档一致承诺：协同模式（Coordinated）下 WindowsAdapter + WslAdapter **同时创建、双侧管理、状态一致**。

实际代码使用 `#[cfg(target_os)]` 条件编译，导致同一进程只能创建**一个**适配器，协同模式退化为平台自动检测。

## 三层承诺 vs 实现

| 层 | 文档 | 承诺 | 实际 |
|----|------|------|------|
| ADR-0005 | 跨平台策略 | 排除条件编译方案；协同=两个适配器 | 使用了被排除的 `#[cfg]` 方案 |
| F002 spec | FR-2.2.1-R4, SC-2 | 同时管理两侧，代理状态一致 | 无法操作对侧 |
| F002 design | §2.5 | `Coordinated → WindowsAdapter + WslAdapter` | `Coordinated → cfg 二选一` |

## 具体代码断层

### 断层 1：DeploymentManager 条件编译（`deployment_manager.rs:128-144`）

```rust
// 当前：条件编译导致只创建一个适配器
DeploymentMode::Coordinated => {
    #[cfg(target_os = "windows")]
    { adapters.push(WindowsAdapter::new()); }   // Windows 只得 Win 适配器
    #[cfg(target_os = "linux")]
    { adapters.push(WslAdapter::new()); }        // WSL 只得 WSL 适配器
}
```

**应该**：运行时动态选择，双侧同时创建。

### 断层 2：WslAdapter 无法在 Windows 编译

`wsl.rs` 和 `linux_base.rs` 被 `#[cfg(target_os = "linux")]` 保护，Windows 编译时完全排除。

内部使用 Linux 路径（`/etc/resolv.conf`、`/proc/version`），Windows 上不可用。

### 断层 3：WindowsAdapter 无法操作 WSL 资源

WindowsAdapter 管理的 9 个状态项全部是 Windows 资源（注册表、hosts、WinHTTP），无 WSL 操作能力。

### 断层 4：wsl-proxy-env write_state 是空操作（`wsl.rs`）

```rust
ID_PROXY_ENV => {
    // "Proxy env vars are typically set in shell session;
    // for now we report success as they will be applied via shell RC"
    Ok(())  // ← 什么都没做
}
```

### 断层 5：mihomo 绑定地址不考虑 WSL 网络模式

`config.rs` 默认 `api_address: "127.0.0.1"`，NAT 模式下 WSL 无法访问 Windows mihomo。

## 影响范围

- 所有选择 "Coordinated" 部署模式的用户
- F004 设置页中的"Coordinated"选项（实际上不生效）
- SC-2 验证标准无法通过

## 修复方向

### 方案：Windows 单实例 + WslRemoteAdapter

基于业务分析，**Windows 是 mihomo 和系统代理的自然宿主**。修复方向为：

1. **Windows 上创建 WslRemoteAdapter**：通过 `wsl -e` 和 `\\wsl$\` 路径桥接 WSL 操作
2. **移除条件编译**：改为运行时动态选择（兑现 ADR-0005）
3. **实现 wsl-proxy-env 写入**：通过 `wsl -e bash -c "export ..."` 设置 WSL 代理
4. **网络模式适配**：NAT 时将代理地址设为网关 IP，mirrored 时共享 localhost

### 涉及文件

| 文件 | 修改类型 |
|------|---------|
| `src-tauri/src/adapters/mod.rs` | 新增 `WslRemoteAdapter` 或修改 cfg 策略 |
| `src-tauri/src/adapters/wsl_remote.rs` | 新建（Windows→WSL 桥接适配器） |
| `src-tauri/src/managers/deployment_manager.rs` | 移除 cfg，改为运行时创建双侧适配器 |
| `src-tauri/src/adapters/wsl.rs` | 修复 wsl-proxy-env 空操作 |
| `src-tauri/src/models/config.rs` | mihomo 绑定地址按网络模式适配 |
| `src-tauri/src/lib.rs` | 协同模式初始化逻辑 |

### 不在范围内

- WSL 单实例方案（独立 hotfix，优先级低于 Windows 单实例）
- shell RC 文件自动修改（spec CON-1 明确排除）
- 包管理器代理配置（spec 明确排除）

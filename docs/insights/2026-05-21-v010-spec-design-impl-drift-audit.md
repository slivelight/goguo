# v0.1.0 Spec-Design-Implementation 漂移全量审计

- **Date**: 2026-05-21
- **Type**: v0.1.0 全量审计（规格-设计-实现对齐检查）
- **方法**: 逐一对照 F001~F004 的 spec.md → design.md → tasks.md → progress.md → closeout.md → 实际代码，同时审计 6 个 ADR 合规性

## 审计汇总

| Feature | BLOCKER | HIGH | MEDIUM | LOW | 审计结论 |
|---------|---------|------|--------|-----|---------|
| F001 基线恢复 | 0 | 2 | 1 | 1 | 有缺口，需修复 |
| F002 WSL 支持 | 2 | 0 | 1 | 0 | 严重断裂 |
| F003 站点规则 | 0 | 0 | 0 | 0 | **无漂移** |
| F004 用户交互 | 0 | 0 | 0 | 0 | **无漂移** |

### ADR 合规

| ADR | 标题 | 状态 |
|-----|------|------|
| ADR-0001 | 记录架构决策 | ✅ 合规 |
| ADR-0002 | Tauri 框架 | ✅ 合规 |
| ADR-0003 | mihomo 托管子进程 | ✅ 合规 |
| ADR-0004 | 文件式 JSON 存储 | ✅ 合规 |
| ADR-0005 | 跨平台 Platform Adapter | ❌ 严重违规（cfg 条件编译） |
| ADR-0006 | React + TypeScript | ✅ 合规 |

---

# F002 WSL 支持（原有分析）

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

## 协同模式的两种部署方向

协同模式（Coordinated）的核心承诺是：**单实例运行 GoGuo，同时管理 Windows 和 WSL 两侧网络配置，修改后双侧同步生效**。

这要求支持两种部署方向：

| 部署方向 | 宿主 | 远程侧 | mihomo 运行位置 |
|----------|------|--------|----------------|
| **Win 单实例** | Windows | WSL（通过 `wsl -e` 桥接） | Windows |
| **WSL 单实例** | WSL | Windows（通过 `cmd.exe /c` 或 PowerShell 桥接） | WSL |

两种方向都必须实现双侧适配器（WindowsAdapter + WslAdapter），区别仅在于本地适配器直接操作、远程适配器通过桥接操作。

## 修复方向

### 方向 A：Windows 单实例 + WslRemoteAdapter

**Windows 是 mihomo 和系统代理的自然宿主**：

1. **Windows 上创建 WslRemoteAdapter**：通过 `wsl -e` 和 `\\wsl$\` 路径桥接 WSL 操作
2. **移除条件编译**：改为运行时动态选择（兑现 ADR-0005）
3. **实现 wsl-proxy-env 写入**：通过 `wsl -e bash -c "export ..."` 设置 WSL 代理
4. **网络模式适配**：NAT 时将代理地址设为网关 IP，mirrored 时共享 localhost

### 方向 B：WSL 单实例 + WindowsRemoteAdapter

**WSL 上运行 GoGuo，桥接管理 Windows 侧**：

1. **WSL 上创建 WindowsRemoteAdapter**：通过 `cmd.exe /c` 和 `powershell.exe -Command` 桥接 Windows 操作（注册表、WinHTTP、hosts 文件）
2. **移除条件编译**：同方向 A，运行时动态选择
3. **Windows 系统代理写入**：通过 `powershell.exe -Command "Set-ItemProperty ..."` 修改 Windows 注册表
4. **mihomo 绑定地址适配**：NAT 模式下绑定 `0.0.0.0`，使 Windows 可通过 WSL 网关 IP 访问代理
5. **Windows hosts 文件写入**：通过 `cmd.exe /c "type ... > C:\Windows\System32\drivers\etc\hosts"` 或 PowerShell 操作

### 两种方向的共性改动

| 文件 | 修改类型 | 说明 |
|------|---------|------|
| `src-tauri/src/adapters/mod.rs` | 修改 cfg 策略 | 移除 `#[cfg(target_os)]`，改为运行时动态创建双侧适配器 |
| `src-tauri/src/managers/deployment_manager.rs` | 重构 | 移除 cfg 条件编译，Coordinated 模式根据宿主平台创建本地+远程适配器 |
| `src-tauri/src/models/config.rs` | 修改 | mihomo 绑定地址按部署方向和网络模式适配 |

### 方向 A 特有改动

| 文件 | 修改类型 |
|------|---------|
| `src-tauri/src/adapters/wsl_remote.rs` | 新建（Windows→WSL 桥接适配器，通过 `wsl -e` 操作） |
| `src-tauri/src/adapters/wsl.rs` | 修复 wsl-proxy-env 空操作 |

### 方向 B 特有改动

| 文件 | 修改类型 |
|------|---------|
| `src-tauri/src/adapters/windows_remote.rs` | 新建（WSL→Windows 桥接适配器，通过 `cmd.exe` / `powershell.exe` 操作） |
| `src-tauri/src/adapters/windows.rs` | 适配：将直接 WinAPI 调用改为可从 WSL 桥接调用的形式 |

### 不在范围内

- shell RC 文件自动修改（spec CON-1 明确排除）
- 包管理器代理配置（spec 明确排除）

---

# F001 基线恢复

## H1-1: 非目标站点可达性验证未实现 (HIGH)

- **承诺**: spec FR-2.4.3-R1 "停止恢复后，必须验证非目标直连站点的可达性不低于 baseline"；SC-5 "非目标直连站点访问成功率不低于 baseline"
- **实际**: 未找到非目标站点可达性验证代码。设计阶段被 F003 的 B+C 探测方案替代，但 spec 中 SC-5 未标注
- **影响**: SC-5 无对应测试证据
- **修复方向**: 在 closeout 中标注 SC-5 被 F003 B+C 方案覆盖

## H1-2: 续跑期间禁止新操作缺 UI 机制 (HIGH)

- **承诺**: spec FR-2.6.2-R2 "续跑过程中不允许启动新的网络修改操作"
- **实际**: 后端续跑逻辑存在，但 F004 未实现恢复蒙层/UI 阻塞机制
- **影响**: 续跑期间用户可能在 UI 上切换模式，导致状态竞争
- **修复方向**: 后端续跑期间加状态锁，或前端加蒙层

## M1-1: ProxyGuard 自动恢复依赖未实现的后台任务 (MEDIUM)

- **承诺**: spec FR-2.5.2-R4 "检测到系统代理指向不可达服务时，必须立即恢复到 baseline 值"
- **实际**: ProxyGuard 逻辑存在，但缺少定期监控的后台任务调度机制
- **修复方向**: 实现后端定时任务，或标注依赖前端触发

## L1-1: Tauri Events 未完整发射 (LOW)

- **承诺**: design §4 定义了 recovery:*、service:started、proxy-guard:* 等事件
- **实际**: 6 个事件定义存在但需后台任务基础设施才能发射（closeout 已记录）
- **修复方向**: 待后台任务基础设施就绪后补齐

---

# F003 站点规则

**无漂移发现。** spec/design/impl 三层对齐良好。已知限制（Mock 替身、订阅仅 base64）均在 closeout 中正确记录为 future work，不构成 spec 违背。

---

# F004 用户交互

**无漂移发现。** 7 个 Store、6 个页面实现完整。已知限制（6 个事件待后台任务、无 start_service）在 closeout 中正确记录。

---

# ADR-0005 严重违规详情

- **决策**: 采用 Platform Adapter trait 模式，运行时动态选择适配器
- **排除方案**: `#[cfg(target_os)]` 条件编译 — 理由 "编译时绑定、不可动态切换、协同部署需同时支持两侧"
- **实际**: 代码中使用 40+ 处条件编译，导致协同模式不可能实现
- **影响文件**: `adapters/mod.rs`(5)、`deployment_manager.rs`(10)、`baseline_manager.rs`(2)、`commands/baseline.rs`(10)、`services/`(4)、`lib.rs`(2)
- **根因**: 实现阶段未对照 ADR 验证技术选型，审查链也未捕获（HF 框架三道防线缺失）

---

# 修复 Feature 分配

| Feature | 级别 | 问题 | Authority Source |
|---------|------|------|-----------------|
| F101 | BLOCKER | 协同模式不可用（双侧适配器） | F002 spec §FR-2.2.1-R4, ADR-0005 |
| F102 | BLOCKER | proxy-env 空操作 | F002 spec §FR-2.1.1, F002 design §2.5 |
| F103 | HIGH | SC-5 非目标站点验证 | F001 spec §FR-2.4.3-R1, SC-5 |
| F104 | HIGH | 续跑期间 UI 阻塞 | F001 spec §FR-2.6.2-R2 |
| F105 | MEDIUM | ProxyGuard 后台任务 | F001 spec §FR-2.5.2-R4 |
| F106 | LOW | Tauri Events 未发射 | F001 design §4 |

> F102 将合并到 F101 中实施（WslRemoteAdapter 的 write_state 自然包含 proxy-env 写入）。

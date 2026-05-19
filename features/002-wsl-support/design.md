# Feature 002: PC 端 Linux/WSL 支持 — 设计

- **Feature**: 002-wsl-support
- **阶段**: `hf-design`
- **状态**: 草稿
- **日期**: 2026-05-12
- **上游输入**: `features/002-wsl-support/spec.md`
- **关联 ADR**: ADR-0004, ADR-0005

## 1. 设计概述

Feature 002 将 Feature 001 的 baseline/restore 能力从 Windows 侧扩展到 WSL 和独立 Linux 侧。核心设计决策是复用 Feature 001 的 PlatformAdapter 抽象——WSL/Linux 侧 4 个可恢复项通过 `WslAdapter` / `LinuxAdapter` 实现，无需修改核心恢复流程。

## 2. 模块设计

### 2.1 WslAdapter

**职责**：实现 `PlatformAdapter` trait，覆盖 WSL 侧的状态项读写。包含 WSL 独有的 WSL2 网络模式检测。

```rust
struct WslAdapter {
    wsl_detector: WslDetector,
    base: LinuxBaseAdapter,  // 共享 Linux 读写逻辑
}

/// 共享的 Linux 读写逻辑基类
struct LinuxBaseAdapter {
    shell_executor: ShellExecutor,
}
```

**实现策略**：

| 状态项 | 读取方式 | 写入方式 | 权限需求 |
|--------|----------|----------|----------|
| `wsl-proxy-env` | 读 `$http_proxy` 等环境变量 | `export` 当前会话（`child process`） | 用户级 |
| `wsl-git-proxy` | `git config --global --get` | `git config --global --set` | 用户级 |
| `wsl-resolv-conf` | 读 `/etc/resolv.conf` | 写临时文件 + rename 覆盖 | root |
| `wsl-etc-environment` | 读 `/etc/environment` | 写临时文件 + rename 覆盖 | root |
| `wsl-shell-proxy` | 扫描 `.bashrc/.zshrc` | —（可检测不可恢复项） | 用户级 |
| `wsl-reachability` | `curl` 测试目标站点 | —（可检测不可恢复项） | 用户级 |
| `wsl-wsl2-network-mode` | 读 `/proc/version` + `.wslconfig` | —（只读检测） | 用户级 |

**root 权限处理**：
- 写入 `/etc/resolv.conf` 和 `/etc/environment` 需要 root 权限
- 策略：检测权限，若不足则：
  1. 在审计中记录权限不足
  2. 生成 `sudo` 前缀的命令供用户手动执行
  3. 标记该状态项为"恢复失败——需手动处理"

### 2.2 LinuxAdapter

**职责**：实现 `PlatformAdapter` trait，覆盖独立 Linux 环境的状态项读写。与 WslAdapter 共享 `LinuxBaseAdapter` 公共逻辑，但无 WSL 独有检测项。

```rust
struct LinuxAdapter {
    base: LinuxBaseAdapter,
}
```

**实现策略**：

| 状态项 | 读取方式 | 写入方式 | 权限需求 |
|--------|----------|----------|----------|
| `linux-proxy-env` | 读环境变量 | `export` 当前会话 | 用户级 |
| `linux-git-proxy` | `git config --global` | `git config --global` | 用户级 |
| `linux-resolv-conf` | 读 `/etc/resolv.conf` | 写文件（需 root） | root |
| `linux-etc-environment` | 读 `/etc/environment` | 写文件（需 root） | root |
| `linux-shell-proxy` | 扫描 `.bashrc/.zshrc` | —（可检测不可恢复项） | 用户级 |
| `linux-reachability` | `curl` 测试目标站点 | —（可检测不可恢复项） | 用户级 |
<!-- [TODO] id:01;status:close;date:2026-05-14T16:19  与features/001-baseline-restore/design.md中”2.2 PlatformAdapter”部分linux平台的采集项不一致，需要与其保持一致；任务处理结果：WslAdapter 和 LinuxAdapter 各补充 `shell-proxy`（扫描 `.bashrc/.zshrc`）和 `reachability`（`curl` 测试）两个可检测不可恢复项，与 Feature 001 定义完全对齐。-->
### 2.3 WslDetector

**职责**：检测 WSL 环境状态和网络模式。

```rust
struct WslDetector;

impl WslDetector {
    /// 检测 WSL 是否安装
    fn is_wsl_installed() -> bool;

    /// 检测当前是否运行在 WSL 内
    fn is_running_in_wsl() -> bool;

    /// 检测 WSL2 网络模式（NAT / 镜像 / 未安装）
    fn detect_network_mode() -> WslNetworkMode;

    /// 获取 WSL 发行版信息
    fn get_distro_info() -> Option<DistroInfo>;
}

enum WslNetworkMode {
    Nat,       // WSL2 默认
    Mirrored,  // Windows 11 22H2+ .wslconfig networkingMode=mirrored
    NotInstalled,
}
```

**网络模式检测逻辑**：
1. 检查 `%USERPROFILE%\.wslconfig` 中 `networkingMode` 配置
2. 若未配置或值为 `nat` → NAT 模式
3. 若值为 `mirrored` → 镜像模式
4. 记录检测结果到 baseline 快照（Feature 002 FR-2.3.1-R2）

### 2.4 WslNetworkStrategy

**职责**：根据 WSL2 网络模式选择配置策略。

```rust
enum WslConfigStrategy {
    /// NAT 模式：显式配置代理环境变量、Git 代理和 DNS
    ExplicitConfig,
    /// 镜像模式 + 已满足可达性：跳过代理配置
    SkipConfig,
    /// 镜像模式 + 未满足可达性：降级为显式配置
    FallbackToExplicit,
}
```

**策略选择逻辑**：
1. NAT 模式 → `ExplicitConfig`
2. 镜像模式 → 先验证 WSL 侧目标站点可达性
   - 可达 → `SkipConfig`（仅记录当前状态到 baseline）
   - 不可达 → `FallbackToExplicit`
3. 策略和理由记入审计（Feature 002 FR-2.3.2-R3）

### 2.5 DeploymentManager

**职责**：管理部署组合的识别和切换。

```rust
struct DeploymentManager {
    wsl_detector: WslDetector,
    config_manager: Arc<ConfigManager>,
}

impl DeploymentManager {
    /// 自动识别当前部署组合
    fn detect_deployment_mode() -> DeploymentMode;

    /// 用户切换部署模式（仅影响运行时行为）
    fn set_deployment_mode(mode: DeploymentMode) -> Result<()>;

    /// 获取当前活跃的 PlatformAdapter 列表
    fn get_active_adapters(&self) -> Vec<&dyn PlatformAdapter>;
}
```

**部署模式 → 适配器映射**：

| 模式 | 活跃适配器 | 配置范围 |
|------|-----------|----------|
| `WindowsOnly` | `WindowsAdapter` | 仅 Windows 侧状态项 |
| `WslOnly` | `WslAdapter` | 仅 WSL 侧状态项 |
| `LinuxOnly` | `LinuxAdapter` | 仅 Linux 侧状态项 |
| `Coordinated` | `WindowsAdapter` + `WslAdapter` | Windows + WSL 两侧状态项 |

## 3. 与 Feature 001 的集成

### 3.1 BaselineManager 扩展

Feature 002 不修改 BaselineManager 的核心逻辑。集成方式：

1. **BaselineManager** 在初始化时通过 `DeploymentManager` 获取活跃适配器列表
2. `collect_initial_snapshot()` 遍历所有活跃适配器
3. WSL/Linux 侧状态项追加到统一 baseline 中（`platform` 字段标记为 `Wsl` 或 `Linux`）
4. 恢复流程同样遍历所有活跃适配器

### 3.2 审计复用

WSL/Linux 侧操作使用与 Feature 001 完全一致的 `AuditRecord` 格式，写入同一审计日志文件。

### 3.3 恢复任务复用

WSL/Linux 侧恢复项纳入统一的 `RecoveryTask`，确保续跑逻辑跨平台一致。

## 4. Tauri Commands

| Command | 说明 | 对应需求 |
|---------|------|----------|
| `detect_deployment_mode` | 自动检测当前部署模式 | Feature 002 FR-2.2.1 |
| `get_deployment_mode` | 获取当前部署模式 | Feature 004 ServiceStore |
| `set_deployment_mode` | 切换部署模式（需二次确认） | Feature 004 设置页 |
| `get_wsl_status` | 获取 WSL 安装/运行状态 | Feature 002 FR-2.3.1 |
| `get_network_mode` | 获取 WSL2 网络模式（NAT/镜像） | Feature 002 FR-2.3.1-R2 |

**说明**：Feature 002 的 BaselineManager/RecoveryTask 操作复用 Feature 001 的 Tauri Commands（`start_initial_assessment`、`stop_service` 等），通过 DeploymentManager 控制活跃适配器范围。Feature 002 仅新增部署模式相关命令。

## 5. 关键流程

### 5.1 WSL/Linux 侧配置执行

1. BaselineManager 触发恢复/配置操作
2. 对应适配器（`WslAdapter` 或 `LinuxAdapter`）的 `write_state()` 执行具体写入
3. 逐项执行，每项执行后立即验证
4. 单项失败不阻塞后续项
5. 全部完成后端到端验证（`curl` 测试目标站点可达性）

### 5.2 WSL2 网络模式切换检测

1. DeploymentManager 定期检测 WSL 状态变化
2. 检测到 WSL 被安装/卸载 → 重新识别部署组合
3. 更新活跃适配器列表
4. 通知 UI 刷新状态展示

## 6. 约束与不变量

- **C1**: 不修改 shell 配置文件（`.bashrc/.zshrc/.profile`）（Feature 002 CON-3）
- **C2**: 不配置包管理器代理和 Docker 代理（Feature 002 CON-4）
- **C3**: WSL/Linux 侧恢复操作使用 root 权限时，必须通过用户手动确认或 sudo 命令
- **C4**: 仅处理 Feature 001 已分类为"可恢复项"的 4 个状态项的自动恢复；可检测不可恢复项仅输出提示（Feature 002 CON-1）

## 7. 风险与缓解

| 风险 | 缓解 |
|------|------|
| `/etc/resolv.conf` 写入需 root | 检测权限 → 降级为只读 + 提供可执行 sudo 命令 |
| 非 Ubuntu/Debian 发行版兼容性 | 最佳努力支持 + 失败时降级为只读评估（NFR-3.4-2） |
| WSL 未运行时无法执行 WSL 侧操作 | 检测 WSL 状态 → 标记不可用 → 提示用户启动 WSL |
| 会话级 export 不影响已运行进程 | 同时写入 `/etc/environment` 确保新进程继承 |
| WSL 与 Windows 网络模式不一致 | 差异报告 + 部署组合状态展示 |

## 8. 测试策略

| 测试 | 方式 |
|------|------|
| WslAdapter / LinuxAdapter 单元测试 | Mock ShellExecutor，验证命令生成 |
| 读写矩阵验证 | P0 probe：在 Ubuntu/Debian 上逐项验证读写 |
| WSL2 模式检测 | 在 NAT 和镜像模式下分别验证 |
| 协同部署配置/恢复 | Windows + WSL 同时运行完整流程 |
| 权限不足降级 | 模拟非 root 环境，验证降级行为 |
| 仅 WSL 部署 | 在 WSL 内独立运行 GoGuo |

## 9. 开放问题处理

| OP ID | 设计阶段处理 |
|-------|-------------|
| OP-1 | WSL/Linux 可恢复项写入路径设计已明确，P0 probe 阶段验证 |
| OP-2 | 仅 WSL 部署下 GoGuo 运行方式：GoGuo 作为 Tauri 应用在 WSL 内运行，需要 WebKitGTK；若不可用，降级为 headless 模式（无 UI） |
| OP-3 | 初始支持 Ubuntu/Debian，其他发行版根据 P0 probe 结果评估 |

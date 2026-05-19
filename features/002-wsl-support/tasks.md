# Feature 002: PC 端 Linux/WSL 支持 — 任务计划

- 状态: 草稿
- 主题: wsl-support
- 阶段: `hf-tasks`
- 上游输入: `features/002-wsl-support/spec.md`、`features/002-wsl-support/design.md`
- **前置依赖**: Feature 001 M1~M4 完成（PlatformAdapter trait + BaselineManager + 项目脚手架）

## 1. 概述

本任务计划将 Feature 002 设计转化为可执行任务。Feature 002 扩展 Feature 001 的 baseline/restore 能力到 WSL 和独立 Linux 环境，核心是实现 WslAdapter、LinuxAdapter（共享 LinuxBaseAdapter）和 DeploymentManager。

**实现范围**：LinuxBaseAdapter、WslAdapter、LinuxAdapter、WslDetector、WslNetworkStrategy、DeploymentManager、5 个 Tauri Commands。

## 2. 里程碑

| 里程碑 | 目标 | 退出标准 | 对应设计章节 |
|--------|------|----------|-------------|
| **M1: Linux 基础层** | LinuxBaseAdapter 共享读写逻辑 | 单元测试通过：环境变量/Git/resolv/hosts 读写 | design §2.1 |
| **M2: WSL 检测** | WslDetector + WslNetworkStrategy | 单元测试通过：NAT/镜像/未安装检测 | design §2.3, §2.4 |
| **M3: WslAdapter** | WSL 侧 7 个状态项完整实现 | 单元测试通过：7 项读写矩阵 | design §2.1 |
| **M4: LinuxAdapter** | 独立 Linux 侧 6 个状态项完整实现 | 单元测试通过：6 项读写矩阵 | design §2.2 |
| **M5: DeploymentManager** | 4 种部署模式识别与切换 | 单元测试通过：模式检测+适配器映射 | design §2.5 |
| **M6: Tauri Commands** | 5 个部署模式相关命令 | 命令可调用 | design §4 |
| **M7: 集成验证** | WSL/Linux 侧完整流程测试 | 协同部署配置/恢复通过 | design §8 |

## 3. 文件 / 工件影响图

```
src-tauri/src/
  adapters/
    linux_base.rs         # LinuxBaseAdapter（共享逻辑）
    wsl.rs                # WslAdapter
    linux.rs              # LinuxAdapter
    mod.rs                # 新增导出
  services/
    wsl_detector.rs       # WslDetector + WslNetworkMode
    deployment_manager.rs # DeploymentManager
  commands/
    deployment.rs         # F002 Tauri Commands（5 个）
  models/
    config.rs             # DeploymentMode 枚举（已有，扩展）
```

## 4. 需求与设计追溯

| 需求 ID | 设计章节 | 任务覆盖 |
|---------|---------|---------|
| FR-2.2.1 (部署模式识别) | §2.5 detect_deployment_mode | T5.1 |
| FR-2.3.1 (WSL 状态检测) | §2.3 WslDetector | T2.1 |
| FR-2.3.1-R2 (WSL2 网络模式) | §2.3 detect_network_mode | T2.1 |
| FR-2.3.2-R3 (WSL 配置策略) | §2.4 WslConfigStrategy | T2.2 |
| FR-2.4.1~R4 (WSL/Linux 状态项) | §2.1, §2.2 适配器 | T3.1, T4.1 |
| NFR-3.4-1 (跨平台一致) | §6 约束 C1~C4 | M7 验证 |
| NFR-3.4-2 (发行版兼容) | §7 风险 | M7 验证 |

## 5. 任务拆解

### T2.1: LinuxBaseAdapter 共享逻辑

- **目标**: 实现 LinuxBaseAdapter——环境变量、Git 配置、/etc/resolv.conf、/etc/environment 的通用读写逻辑
- **Acceptance**: 4 类状态项读写方法可工作；ShellExecutor 抽象可 Mock
- **依赖**: Feature 001 T3.1（PlatformAdapter trait）
- **Ready When**: F001 PlatformAdapter trait 完成
- **初始队列状态**: pending
- **Selection Priority**: 1
- **Files / 触碰工件**: `src-tauri/src/adapters/linux_base.rs`
- **测试设计种子**: Mock ShellExecutor → 验证命令生成；环境变量读/写测试；/etc/resolv.conf 读写测试（临时文件）；/etc/environment 读写测试（临时文件）
- **Verify**: `cargo test --lib adapters::linux_base`
- **预期证据**: 4 类共享逻辑测试全通过
- **完成条件**: 共享读写逻辑正确，可被 WslAdapter/LinuxAdapter 复用

### T2.2: WslDetector + WslNetworkMode

- **目标**: 实现 WSL 环境检测（is_wsl_installed / is_running_in_wsl / detect_network_mode / get_distro_info）
- **Acceptance**: 正确识别 NAT/镜像/未安装三种状态；发行版信息可获取
- **依赖**: Feature 001 T2.1（数据模型）
- **Ready When**: F001 数据模型完成
- **初始队列状态**: pending
- **Selection Priority**: 1
- **Files / 触碰工件**: `src-tauri/src/services/wsl_detector.rs`
- **测试设计种子**: Mock .wslconfig → NAT/镜像检测；/proc/version 解析测试；未安装场景测试
- **Verify**: `cargo test --lib services::wsl_detector`
- **预期证据**: WSL 检测逻辑测试通过
- **完成条件**: 三种网络模式正确识别

### T2.3: WslNetworkStrategy

- **目标**: 实现 WSL 配置策略选择（ExplicitConfig / SkipConfig / FallbackToExplicit）
- **Acceptance**: NAT→ExplicitConfig；镜像+可达→SkipConfig；镜像+不可达→FallbackToExplicit
- **依赖**: T2.2
- **Ready When**: WslDetector 完成
- **初始队列状态**: pending
- **Selection Priority**: 2
- **Files / 触碰工件**: `src-tauri/src/services/wsl_detector.rs`（策略逻辑）
- **测试设计种子**: 3 种策略选择测试；策略选择记入审计验证
- **Verify**: `cargo test --lib services::wsl_detector -- strategy`
- **预期证据**: 策略选择测试通过
- **完成条件**: 3 种策略选择逻辑正确

### T3.1: WslAdapter 实现

- **目标**: 实现 WslAdapter（7 个状态项：4 可恢复 + 3 可检测不可恢复），复用 LinuxBaseAdapter
- **Acceptance**: 7 个状态项全部可读取；4 个可恢复项写入后读回一致；wsl-wsl2-network-mode 正确检测
- **依赖**: T2.1, T2.2
- **Ready When**: LinuxBaseAdapter + WslDetector 完成
- **初始队列状态**: pending
- **Selection Priority**: 3
- **Files / 触碰工件**: `src-tauri/src/adapters/wsl.rs`
- **测试设计种子**: 7 项读写矩阵测试；wsl-proxy-env 写入（export）→读回验证；root 权限不足降级测试；wsl-shell-proxy 只读扫描测试
- **Verify**: `cargo test --lib adapters::wsl`
- **预期证据**: WSL 适配器全部测试通过
- **完成条件**: 7 个状态项完整实现，读写+降级逻辑正确

### T4.1: LinuxAdapter 实现

- **目标**: 实现 LinuxAdapter（6 个状态项：4 可恢复 + 2 可检测不可恢复），复用 LinuxBaseAdapter
- **Acceptance**: 6 个状态项全部可读取；4 个可恢复项写入后读回一致
- **依赖**: T2.1
- **Ready When**: LinuxBaseAdapter 完成
- **初始队列状态**: pending
- **Selection Priority**: 3
- **Files / 触碰工件**: `src-tauri/src/adapters/linux.rs`
- **测试设计种子**: 6 项读写矩阵测试；与 WslAdapter 共享逻辑一致性验证；root 权限不足降级测试
- **Verify**: `cargo test --lib adapters::linux`
- **预期证据**: Linux 适配器全部测试通过
- **完成条件**: 6 个状态项完整实现，与 WslAdapter 共享逻辑无重复

### T5.1: DeploymentManager

- **目标**: 实现 4 种部署模式的自动检测、切换和适配器映射
- **Acceptance**: 自动检测返回正确模式；切换后适配器列表更新；4 种模式的适配器映射正确
- **依赖**: T3.1, T4.1, T2.2, Feature 001 T6.2（ConfigManager）
- **Ready When**: 全部适配器 + WslDetector + ConfigManager 完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src-tauri/src/services/deployment_manager.rs`
- **测试设计种子**: 仅 Windows→WindowsOnly；Windows+WSL→Coordinated；仅 WSL→WslOnly；仅 Linux→LinuxOnly；模式切换→适配器列表更新
- **Verify**: `cargo test --lib services::deployment_manager`
- **预期证据**: 部署模式管理测试通过
- **完成条件**: 4 种模式检测+切换+映射正确

### T6.1: Feature 002 Tauri Commands

- **目标**: 实现 5 个 Tauri Commands：detect_deployment_mode, get_deployment_mode, set_deployment_mode, get_wsl_status, get_network_mode
- **Acceptance**: 每个命令可被前端 invoke() 调用；set_deployment_mode 需二次确认标记
- **依赖**: T5.1
- **Ready When**: DeploymentManager 完成
- **初始队列状态**: pending
- **Selection Priority**: 5
- **Files / 触碰工件**: `src-tauri/src/commands/deployment.rs`, `src-tauri/src/commands/mod.rs`
- **测试设计种子**: 每个命令参数/返回值测试；set 需确认标记验证
- **Verify**: `cargo test --lib commands::deployment`
- **预期证据**: 5 个命令测试通过
- **完成条件**: 5 个 Tauri Commands 实现，类型正确

### T7.1: 集成测试

- **目标**: WSL/Linux 侧完整配置+恢复流程测试；协同部署端到端验证
- **Acceptance**: WSL 侧 4 个可恢复项恢复正确；Linux 侧 4 个可恢复项恢复正确；协同部署两侧同时恢复
- **依赖**: T6.1
- **Ready When**: Tauri Commands 完成
- **初始队列状态**: pending
- **Selection Priority**: 6
- **Files / 触碰工件**: `src-tauri/tests/integration_wsl.rs`
- **测试设计种子**: WslAdapter 真实读写（WSL 环境）；LinuxAdapter 真实读写（Linux 环境）；协同模式 Windows+WSL 同时恢复；权限不足降级场景
- **Verify**: `cargo test --test integration_wsl`
- **预期证据**: 集成测试通过
- **完成条件**: 跨平台恢复流程正确

## 6. 依赖与关键路径

```
F001 T3.1 ─→ T2.1 ─→ T3.1(WslAdapter)
F001 T2.1 ─→ T2.2 ─→ T2.3
T2.1 ─→ T4.1(LinuxAdapter)
T3.1 + T4.1 + T2.2 + F001 T6.2 ─→ T5.1 ─→ T6.1 ─→ T7.1
```

**关键路径**：F001 T3.1 → T2.1 → T3.1 → T5.1 → T6.1 → T7.1

**可并行任务组**：
- T2.1 + T2.2（分别依赖 F001 不同任务，可同时开始）
- T3.1 + T4.1（均依赖 T2.1，可同时开始）

## 7. 完成定义与验证策略

| 里程碑 | DoD | 验证方式 |
|--------|-----|---------|
| M1 | Linux 共享逻辑可工作 | `cargo test --lib adapters::linux_base` |
| M2 | WSL 检测正确 | `cargo test --lib services::wsl_detector` |
| M3 | WSL 7 项读写矩阵 | `cargo test --lib adapters::wsl` |
| M4 | Linux 6 项读写矩阵 | `cargo test --lib adapters::linux` |
| M5 | 4 种部署模式正确 | `cargo test --lib services::deployment_manager` |
| M6 | 前端可调用 | `cargo test --lib commands::deployment` |
| M7 | 跨平台恢复正确 | `cargo test --test integration_wsl` |

## 8. 当前活跃任务选择规则

1. Feature 002 所有任务均依赖 Feature 001 部分任务完成
2. 在 F001 T3.1（PlatformAdapter trait）完成后，T2.1 可开始
3. 在 F001 T2.1（数据模型）完成后，T2.2 可开始
4. **Current Active Task**: T2.1（等待 F001 T3.1 完成后启动）

## 9. 任务队列投影视图

| 阶段 | 任务 | 状态 |
|------|------|------|
| Phase 1 | T2.1 LinuxBaseAdapter · T2.2 WslDetector | ⬜ pending（等待 F001） |
| Phase 2 | T2.3 WslNetworkStrategy · T3.1 WslAdapter · T4.1 LinuxAdapter | ⬜ pending |
| Phase 3 | T5.1 DeploymentManager | ⬜ pending |
| Phase 4 | T6.1 Tauri Commands | ⬜ pending |
| Phase 5 | T7.1 集成测试 | ⬜ pending |

## 10. 风险与顺序说明

| 风险 | 影响 | 缓解 |
|------|------|------|
| WSL 环境测试可用性 | T3.1/T7.1 需要 WSL 环境 | CI 配置 WSL2 runner；无 WSL 时跳过 |
| /etc/resolv.conf 写入需 root | T2.1 权限问题 | 设计已包含权限检测+降级策略 |
| 非 Ubuntu/Debian 兼容性 | T4.1/T7.1 | 最佳努力+降级为只读评估 |
| WebKitGTK 可用性 | 仅 WSL 部署时 UI 不可用 | 降级为 headless 模式（OP-2） |

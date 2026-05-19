# Feature 001: 安装后网络评估与基线恢复 — 任务计划

- 状态: 草稿
- 主题: baseline-restore
- 阶段: `hf-tasks`
- 上游输入: `features/001-baseline-restore/spec.md`、`features/001-baseline-restore/design.md`

## 1. 概述

本任务计划将 Feature 001 设计转化为可执行、可验证的任务单元。Feature 001 是 GoGuo 的安全前置层，提供 baseline 快照生命周期管理、ProxyGuard 异常监控和审计日志能力。

**实现范围**：BaselineManager、PlatformAdapter trait + WindowsAdapter、ProxyGuard、MihomoManager、AuditLogger、ConfigManager、RecoveryTask 状态机、10 个 Tauri Commands。

**技术栈**：Rust（后端，Tauri Commands）、Tauri 2.x、JSON/JSONL 存储（ADR-0004）。

## 2. 里程碑

| 里程碑 | 目标 | 退出标准 | 对应设计章节 |
|--------|------|----------|-------------|
| **M1: 项目脚手架** | Tauri + React 项目初始化，共享类型定义 | `cargo build` + `pnpm dev` 可启动空应用 | — |
| **M2: 数据模型与存储** | 全部核心数据模型 + JSON/JSONL 存储 | 单元测试通过：序列化/反序列化 + 文件读写 | design §3, §2.5, §2.6 |
| **M3: PlatformAdapter 层** | trait 定义 + WindowsAdapter 完整实现 | 单元测试通过：9 个 Windows 状态项读写矩阵 | design §2.2 |
| **M4: BaselineManager 核心** | 快照采集/形成/确认/对比/恢复 | 单元测试通过（Mock Adapter） | design §2.1 |
| **M5: MihomoManager** | 子进程生命周期管理 + API 交互 | 集成测试通过：启动/停止/热重载 | design §2.4 |
| **M6: AuditLogger + ConfigManager** | 结构化审计 + 应用配置管理 | 单元测试通过：审计记录写入 + 配置读写 | design §2.5, §2.6 |
| **M7: ProxyGuard** | mihomo 进程/端口/API 监控 + 自动恢复 | 集成测试通过：崩溃注入 → 自动重启 | design §2.3 |
| **M8: RecoveryTask 状态机** | 5 态状态机 + 续跑逻辑 | 单元测试通过：全部状态转换路径 | design §5.3 |
| **M9: Tauri Commands** | 10 个 IPC 命令对接前端 | 手动验证：每个命令可被前端 invoke() 调用 | design §4 |
| **M10: 集成验证** | 完整 baseline 生命周期端到端测试 | 端到端计时通过（P95 ≤ 10s） | design §8 |

## 3. 文件 / 工件影响图

```
src-tauri/
  src/
    models/
      baseline.rs          # BaselineSnapshot, StateItem, EnvironmentInfo
      recovery.rs          # RecoveryTask, RecoveryItem, RecoveryStatus
      audit.rs             # AuditRecord, AuditAction, AuditResult, FailureExplanation
      config.rs            # AppConfig, MihomoConfig, ProxyGuardConfig, ProbeConfig
    adapters/
      mod.rs               # PlatformAdapter trait
      windows.rs           # WindowsAdapter
    managers/
      baseline_manager.rs  # BaselineManager
      mihomo_manager.rs    # MihomoManager
      config_manager.rs    # ConfigManager
    services/
      proxy_guard.rs       # ProxyGuard
      audit_logger.rs      # AuditLogger
    commands/
      baseline.rs          # F001 Tauri Commands
    lib.rs
    main.rs
  tauri.conf.json
  Cargo.toml
  data/
    baseline/              # initial-snapshot.json, baseline-v1.json
    state/                 # recovery-task.json
    audit/                 # audit-YYYY-MM-DD.jsonl
    config/                # settings.json
    mihomo/                # config.yaml (由 F003 生成)
```

## 4. 需求与设计追溯

| 需求 ID | 设计章节 | 任务覆盖 |
|---------|---------|---------|
| FR-2.1.1 (首次评估) | §2.1 collect_initial_snapshot | T4.1 |
| FR-2.2.1 (baseline 形成) | §2.1 form_baseline/confirm_baseline | T4.2 |
| FR-2.2.2 (状态摘要) | §2.1 get_state_summary | T4.1, T9.1 |
| FR-2.3.2 (偏离对比) | §2.1 compare_with_baseline | T4.3 |
| FR-2.4.1 (停止服务) | §5.1 恢复流程 | T4.4 |
| FR-2.4.2 (恢复进度) | §5.3 状态机 | T8.1 |
| FR-2.5 (服务状态) | §2.3 ProxyGuard | T7.1 |
| FR-2.6.1/2 (续跑) | §5.2 续跑流程 | T8.2 |
| FR-2.7 (审计) | §2.5 AuditLogger | T6.1 |
| NFR-3.1-1 (冷启动 ≤3s) | §8 性能设计 | M10 验证 |
| SC-1 (P95 恢复时间) | §8 测试策略 | T10.1 |

## 5. 任务拆解

### T1.1: Tauri + React 项目脚手架

- **目标**: 初始化 Tauri 2.x + React + TypeScript + pnpm 项目结构
- **Acceptance**: `cargo build` 成功；`pnpm dev` 启动空 Tauri 窗口
- **依赖**: 无
- **Ready When**: 项目开始
- **初始队列状态**: pending
- **Selection Priority**: 1（最先执行）
- **Files / 触碰工件**: `Cargo.toml`, `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `vite.config.ts`, `tsconfig.json`
- **测试设计种子**: 构建产物验证——`cargo build` 无错误，`pnpm build` 无错误
- **Verify**: `cargo build && pnpm build`
- **预期证据**: 构建成功输出，空窗口可启动
- **完成条件**: Rust 后端编译通过，前端 dev server 启动，Tauri 窗口可显示

### T2.1: 核心数据模型定义

- **目标**: 定义 BaselineSnapshot、StateItem、RecoveryTask、RecoveryItem、RecoveryStatus、AuditRecord、AppConfig 等核心 Rust struct/enum
- **Acceptance**: 所有 struct 可序列化/反序列化（serde）；单元测试通过
- **依赖**: T1.1
- **Ready When**: 项目脚手架完成
- **初始队列状态**: pending
- **Selection Priority**: 2
- **Files / 触碰工件**: `src-tauri/src/models/baseline.rs`, `src-tauri/src/models/recovery.rs`, `src-tauri/src/models/audit.rs`, `src-tauri/src/models/config.rs`, `src-tauri/src/models/mod.rs`
- **测试设计种子**: 每个模型各一个序列化/反序列化往返测试；RecoveryStatus 5 态枚举覆盖；AppConfig 字段完整性断言
- **Verify**: `cargo test --lib models`
- **预期证据**: 所有模型测试绿色
- **完成条件**: 所有 struct 编译通过，serde 往返测试全通过

### T2.2: JSON/JSONL 存储层

- **目标**: 实现 BaselineStorage（JSON 读写）和审计日志存储（JSONL 追加+按日滚动+分页查询）
- **Acceptance**: 文件读写正确；JSONL 追加不破坏已有内容；分页查询返回正确切片；并发安全
- **依赖**: T2.1
- **Ready When**: 数据模型定义完成
- **初始队列状态**: pending
- **Selection Priority**: 2
- **Files / 触碰工件**: `src-tauri/src/storage/mod.rs`, `src-tauri/src/storage/baseline_storage.rs`, `src-tauri/src/storage/audit_storage.rs`
- **测试设计种子**: 写入 → 读取 → 断言内容一致；JSONL 跨日文件滚动测试；分页 offset/limit 边界测试；并发写入无数据丢失
- **Verify**: `cargo test --lib storage`
- **预期证据**: 存储层全部测试绿色
- **完成条件**: BaselineStorage 和 AuditStorage 单元测试全通过

### T3.1: PlatformAdapter trait 定义

- **目标**: 定义 PlatformAdapter trait（read_state_items / write_state / get_platform / get_state_item_definitions）
- **Acceptance**: trait 编译通过，接口与设计文档 §2.2 一致
- **依赖**: T2.1
- **Ready When**: 数据模型定义完成
- **初始队列状态**: pending
- **Selection Priority**: 3
- **Files / 触碰工件**: `src-tauri/src/adapters/mod.rs`
- **测试设计种子**: 创建 MockAdapter 实现 trait，验证接口调用
- **Verify**: `cargo test --lib adapters::tests`
- **预期证据**: MockAdapter 可通过 trait 调用全部方法
- **完成条件**: trait 接口编译通过，Mock 实现测试通过

### T3.2: WindowsAdapter 实现

- **目标**: 实现 WindowsAdapter，覆盖 9 个 Windows 状态项的读写（4 可恢复 + 5 可检测不可恢复）
- **Acceptance**: 每个状态项的读取可返回值；4 个可恢复项写入后读回一致
- **依赖**: T3.1
- **Ready When**: PlatformAdapter trait 定义完成
- **初始队列状态**: pending
- **Selection Priority**: 3
- **Files / 触碰工件**: `src-tauri/src/adapters/windows.rs`
- **测试设计种子**: 每个状态项独立读写测试（Mock shell/registry）；win-hosts 文件读写测试；win-system-proxy 注册表读写测试
- **Verify**: `cargo test --lib adapters::windows`
- **预期证据**: Windows 状态项读写矩阵测试全通过
- **完成条件**: 9 个状态项全部实现，单元测试通过

### T4.1: BaselineManager 采集与摘要

- **目标**: 实现 collect_initial_snapshot() 和 get_state_summary()
- **Acceptance**: 采集返回所有活跃适配器的状态项；摘要按分类展示
- **依赖**: T3.1, T2.2
- **Ready When**: PlatformAdapter trait + 存储层完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src-tauri/src/managers/baseline_manager.rs`
- **测试设计种子**: Mock 2 个适配器，验证采集遍历全部适配器；验证快照持久化到文件
- **Verify**: `cargo test --lib managers::baseline_manager -- collect`
- **预期证据**: 采集测试通过，快照文件正确生成
- **完成条件**: 采集+摘要逻辑正确，快照存储到 initial-snapshot.json

### T4.2: BaselineManager 确认与对比

- **目标**: 实现 form_baseline()、confirm_baseline()、compare_with_baseline()
- **Acceptance**: 确认后 baseline 文件生成；对比返回逐项匹配结果；偏离项正确标记
- **依赖**: T4.1
- **Ready When**: 采集逻辑完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src-tauri/src/managers/baseline_manager.rs`
- **测试设计种子**: 确认后验证 baseline-v1.json 生成；修改值后对比返回偏离项；未偏离项标记 match
- **Verify**: `cargo test --lib managers::baseline_manager -- confirm`
- **预期证据**: 确认/对比逻辑测试通过
- **完成条件**: baseline 确认→存储→对比→偏离检测全部正确

### T4.3: BaselineManager 恢复逻辑

- **目标**: 实现 restore_to_baseline()——按适配器分组、风险从低到高、逐项恢复+即时验证
- **Acceptance**: 可恢复项全部恢复到 baseline 值；单项失败不阻塞后续项；结果记录到 RecoveryTask
- **依赖**: T4.2, T8.1
- **Ready When**: 确认/对比逻辑 + RecoveryTask 模型完成
- **初始队列状态**: pending
- **Selection Priority**: 5
- **Files / 触碰工件**: `src-tauri/src/managers/baseline_manager.rs`
- **测试设计种子**: Mock 适配器 4 项恢复——全部成功；1 项失败不阻塞；恢复后读回验证
- **Verify**: `cargo test --lib managers::baseline_manager -- restore`
- **预期证据**: 恢复逻辑测试通过（含部分失败场景）
- **完成条件**: 恢复逻辑正确处理成功/部分失败/全部失败场景

### T5.1: MihomoManager 进程管理

- **目标**: 实现 mihomo 子进程的 start/stop/health_check/is_running
- **Acceptance**: start 启动子进程并等待 API 就绪；stop 优雅停止（SIGTERM→SIGKILL）；health_check 检测进程+API+端口
- **依赖**: T2.1
- **Ready When**: 数据模型完成（MihomoConfig）
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src-tauri/src/managers/mihomo_manager.rs`
- **测试设计种子**: 启动→检查 running→停止→检查 not running；API 未就绪超时处理；进程残留清理
- **Verify**: `cargo test --lib managers::mihomo_manager`
- **预期证据**: 进程管理测试通过（需 mihomo 二进制）
- **完成条件**: 子进程生命周期管理正确，API 就绪检测有效

### T5.2: MihomoManager 配置与热重载

- **目标**: 实现 reload_config()——写入配置文件 + PUT /configs API 热重载
- **Acceptance**: 配置写入正确；热重载后 mihomo 应用新配置
- **依赖**: T5.1
- **Ready When**: 进程管理完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src-tauri/src/managers/mihomo_manager.rs`
- **测试设计种子**: 写入测试配置→热重载→验证新端口生效；重载失败→错误返回
- **Verify**: `cargo test --lib managers::mihomo_manager -- reload`
- **预期证据**: 热重载测试通过
- **完成条件**: 配置生成+热重载逻辑正确

### T6.1: AuditLogger 实现

- **目标**: 实现 JSONL 审计日志写入、按日滚动、分页查询（含过滤）
- **Acceptance**: 审计记录追加正确；按日期/类型过滤正确；五要素提示生成正确
- **依赖**: T2.2
- **Ready When**: 存储层完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src-tauri/src/services/audit_logger.rs`
- **测试设计种子**: 写入 100 条→查询 offset=0/limit=50→验证分页；过滤 action_type→验证结果；五要素提示字段完整性
- **Verify**: `cargo test --lib services::audit_logger`
- **预期证据**: 审计日志测试通过
- **完成条件**: 审计写入+查询+过滤+五要素提示全部正确

### T6.2: ConfigManager 实现

- **目标**: 实现 AppConfig 读写，包含 DeploymentMode 切换
- **Acceptance**: 配置文件不存在时使用默认值；读写一致；部署模式切换后持久化
- **依赖**: T2.1
- **Ready When**: 数据模型完成
- **初始队列状态**: pending
- **Selection Priority**: 3
- **Files / 触碰工件**: `src-tauri/src/managers/config_manager.rs`
- **测试设计种子**: 默认配置→写入→读取→断言一致；切换 DeploymentMode→验证持久化
- **Verify**: `cargo test --lib managers::config_manager`
- **预期证据**: 配置管理测试通过
- **完成条件**: 配置读写+默认值+部署模式切换正确

### T7.1: ProxyGuard 监控与恢复

- **目标**: 实现 mihomo 进程/端口/API 定期监控 + 异常自动恢复（重启≤3次→恢复baseline）
- **Acceptance**: 进程崩溃后自动重启；重启超限后自动恢复 baseline 并通知用户（不需确认）
- **依赖**: T5.1, T4.3, T6.1
- **Ready When**: MihomoManager + BaselineManager + AuditLogger 完成
- **初始队列状态**: pending
- **Selection Priority**: 6
- **Files / 触碰工件**: `src-tauri/src/services/proxy_guard.rs`
- **测试设计种子**: 模拟进程崩溃→验证自动重启（≤3 次）；重启超限→验证恢复 baseline 触发；系统代理不一致→验证自动恢复
- **Verify**: `cargo test --lib services::proxy_guard`
- **预期证据**: ProxyGuard 监控+恢复测试通过
- **完成条件**: 监控逻辑正确，异常恢复路径（重启/恢复 baseline）正确

### T8.1: RecoveryTask 状态机

- **目标**: 实现 5 态状态机（Pending/InProgress/Completed/Failed/UserAcknowledged）+ 单任务存储
- **Acceptance**: 状态转换全部合法；终态文件正确清理；非法转换被拒绝
- **依赖**: T2.1, T2.2
- **Ready When**: 数据模型 + 存储层完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src-tauri/src/services/recovery.rs`
- **测试设计种子**: 5 态全部转换路径测试；单任务覆盖测试；终态文件删除/保留测试；非法转换拒绝测试
- **Verify**: `cargo test --lib services::recovery`
- **预期证据**: 状态机全部转换测试通过
- **完成条件**: 状态机逻辑正确，存储格式符合设计

### T8.2: 续跑逻辑

- **目标**: 启动时检查 recovery-task.json，若存在 Pending/InProgress 任务则续跑
- **Acceptance**: 启动后续跑未完成任务；续跑期间禁止新网络修改操作；续跑完成后验证
- **依赖**: T8.1, T4.3
- **Ready When**: 状态机 + 恢复逻辑完成
- **初始队列状态**: pending
- **Selection Priority**: 7
- **Files / 触碰工件**: `src-tauri/src/services/recovery.rs`, `src-tauri/src/main.rs`
- **测试设计种子**: 写入 Pending 任务→启动→验证续跑执行；InProgress 任务→验证续跑从断点继续；Completed 任务→验证不触发续跑
- **Verify**: `cargo test --lib services::recovery -- resume`
- **预期证据**: 续跑逻辑测试通过
- **完成条件**: 启动后续跑逻辑正确，续跑完成后状态正确

### T9.1: Feature 001 Tauri Commands

- **目标**: 实现 10 个 Tauri Commands：start_initial_assessment, get_state_summary, trigger_readjustment, confirm_baseline, get_baseline_status, stop_service, get_service_status, get_recovery_progress, get_audit_log, 以及 Tauri Events 定义
- **Acceptance**: 每个命令可被前端 invoke() 调用；Events 正确推送到前端
- **依赖**: T4.1, T4.2, T4.3, T5.1, T6.1, T7.1, T8.1
- **Ready When**: 所有核心模块完成
- **初始队列状态**: pending
- **Selection Priority**: 8
- **Files / 触碰工件**: `src-tauri/src/commands/baseline.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`
- **测试设计种子**: 每个命令的参数/返回值类型测试；get_audit_log 分页参数验证；Tauri Event payload 序列化测试
- **Verify**: `cargo test --lib commands::baseline`
- **预期证据**: 全部命令测试通过
- **完成条件**: 10 个 Commands + 9 个 Events 全部实现，类型正确

### T10.1: 端到端集成测试

- **目标**: 完整 baseline 生命周期测试（评估→确认→修改→恢复），计时验证 P95 ≤ 10s
- **Acceptance**: 完整流程无错误；审计记录完整；恢复时间达标
- **依赖**: T9.1
- **Ready When**: Tauri Commands 全部完成
- **初始队列状态**: pending
- **Selection Priority**: 9
- **Files / 触碰工件**: `src-tauri/tests/integration_baseline.rs`
- **测试设计种子**: 完整流程：评估→确认→启动 mihomo→停止→恢复→验证；注入 mihomo 崩溃→ProxyGuard 自动恢复；续跑场景：恢复中断→重启→续跑完成
- **Verify**: `cargo test --test integration_baseline`
- **预期证据**: 集成测试全通过，性能计时达标
- **完成条件**: 端到端流程正确，性能指标满足

## 6. 依赖与关键路径

```
T1.1 ─→ T2.1 ─→ T2.2 ─→ T6.1
  │        │                  ↑
  │        ├─→ T3.1 ─→ T3.2  │
  │        │        ↓         │
  │        ├─→ T6.2           │
  │        │                  │
  │        ├─→ T5.1 ─→ T5.2  │
  │        │                  │
  │        ├─→ T8.1 ─→ T8.2  │
  │        │                  │
  │        └─→ T4.1 ─→ T4.2 ─┤─→ T4.3 ─→ T7.1
  │                          │            ↓
  │                          └─────→ T9.1 ─→ T10.1
```

**关键路径**：T1.1 → T2.1 → T4.1 → T4.2 → T4.3 → T7.1 → T9.1 → T10.1

**可并行任务组**：
- 组 A（数据基础）: T2.2 + T3.1 + T5.1 + T6.2 + T8.1（均仅依赖 T2.1）
- 组 B（模块实现）: T3.2 + T5.2 + T6.1（分别依赖各自前置）

## 7. 完成定义与验证策略

| 里程碑 | DoD | 验证方式 |
|--------|-----|---------|
| M1 | 空应用可编译启动 | `cargo build && pnpm dev` |
| M2 | 全部模型可序列化 | `cargo test --lib models` |
| M3 | Windows 状态项读写矩阵 | `cargo test --lib adapters` |
| M4 | Baseline 全生命周期 | `cargo test --lib managers::baseline_manager` |
| M5 | mihomo 进程可管理 | `cargo test --lib managers::mihomo_manager` |
| M6 | 审计+配置可工作 | `cargo test --lib services::audit_logger && ...config_manager` |
| M7 | 崩溃自动恢复 | `cargo test --lib services::proxy_guard` |
| M8 | 状态机+续跑 | `cargo test --lib services::recovery` |
| M9 | 前端可调用 | `cargo test --lib commands` |
| M10 | 端到端达标 | `cargo test --test integration_baseline` |

## 8. 当前活跃任务选择规则

1. 选取所有状态为 `pending` 且全部依赖任务已完成的任务
2. 若有多个候选，按 **Selection Priority** 数值从小到大选取
3. 若优先级相同，按任务 ID（T1.1 < T2.1 < ...）顺序选取
4. **Current Active Task**: T1.1（项目脚手架）

## 9. 任务队列投影视图

| 阶段 | 任务 | 状态 |
|------|------|------|
| Phase 1 | T1.1 项目脚手架 | ⬜ pending |
| Phase 2 | T2.1 数据模型 | ⬜ pending |
| Phase 3 | T2.2 存储层 · T3.1 Adapter trait · T5.1 Mihomo 进程 · T6.2 ConfigManager · T8.1 状态机 | ⬜ pending |
| Phase 4 | T3.2 WindowsAdapter · T4.1 采集 · T5.2 热重载 · T6.1 AuditLogger | ⬜ pending |
| Phase 5 | T4.2 确认对比 · T8.2 续跑 | ⬜ pending |
| Phase 6 | T4.3 恢复 · T7.1 ProxyGuard | ⬜ pending |
| Phase 7 | T9.1 Tauri Commands | ⬜ pending |
| Phase 8 | T10.1 集成测试 | ⬜ pending |

## 10. 风险与顺序说明

| 风险 | 影响 | 缓解 |
|------|------|------|
| Windows 注册表 API 需要 admin 权限 | T3.2 可能需要调整权限模型 | 设计已包含权限检测+降级策略 |
| mihomo 二进制依赖 | T5.1/T10.1 需要真实 mihomo | CI 中预置 mihomo 二进制 |
| 续跑逻辑复杂性 | T8.2 可能引入状态不一致 | 状态机严格验证，单任务存储 |
| Tauri Commands 测试环境 | T9.1 需要 Tauri 运行时 | 使用 tauri::test 工具 |

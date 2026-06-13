# Feature 109: Baseline 恢复语义修复 — 需求规格

- **Feature**: 109-baseline-restore-semantic-fix
- **阶段**: `hf-specify`
- **状态**: 草稿
- **日期**: 2026-06-08
- **上游输入**:
  - `features/001-baseline-restore/spec.md` — 原始 F001 需求
  - `features/001-baseline-restore/design.md` — 原始 F001 设计
  - OPP-002 Gap 分析报告（2026-06-07 会话产出）

## 1. 概述

### 1.1 目的

本 feature 修复 F001（baseline-restore）已实现代码与设计规格之间的语义错位和功能缺口。核心问题是：UI 上"立即恢复"按钮的实际代码行为是**重新启动代理服务**（`triggerReadjustment` = `start_initial_assessment` + `apply_proxy_env` + `activate_system_proxy`），而非**恢复到 baseline 直连状态**。这导致用户点击"恢复"后，系统代理仍指向 mihomo、代理站点仍可见，与"恢复"的语义预期完全相反。

### 1.2 范围

本规格覆盖以下修复项：

1. **P0-109-1**: "立即恢复"按钮语义反转 — 调用 `triggerReadjustment`（重启代理）而非 `restore_to_baseline`（恢复 baseline）
2. **P0-109-2**: `wsl-proxy-env` 分类不一致 — 本地 WSL 适配器标为 Excluded，远程 WSL 适配器标为 Restorable
3. **P1-109-3**: 恢复流程不写审计日志 — `restore_to_baseline()` / `stop_service()` 不调用 AuditLogger
4. **P1-109-4**: ProxyGuard 仅检查进程存活，不覆盖设计要求的 6 类监控对象
5. **P1-109-5**: 非目标站点验证使用硬编码 URL 而非 AppConfig 配置
6. **P2-109-6**: `determine_proxy_address()` 仅 Linux 编译，Windows 端无法感知 WSL NAT/Mirrored
7. **P2-109-7**: 恢复后无 DNS 缓存刷新
8. **P2-109-8**: ProxyStrategy 与恢复流程未集成
9. **P2-109-9**: 恢复前无偏差确认 — 不先对比再恢复，直接写所有 Restorable 项
<!-- ? id:01;status:open;date:2026-06-09T15:00 codebase验证：compare_with_baseline()方法已存在(baseline_manager.rs:169-212)可返回Match/Deviated/MissingInBaseline，但restore_to_baseline()未调用它。gap描述应更精确为"偏差确认基础设施已存在但未集成到恢复流程"，而非"完全缺失" -->

### 1.3 术语

| 术语 | 定义 |
|------|------|
| 语义反转 | UI 文案与代码实际行为的含义完全相反（如"恢复"实际为"重启服务"） |
| 立即恢复 | 仪表盘快捷操作按钮，语义应为"恢复到 baseline 直连状态" |
| triggerReadjustment | 当前"立即恢复"按钮调用的 Tauri command，语义为"重新评估+重启代理" |
| 偏差确认 | 恢复前先对比当前值与 baseline 值，仅恢复有差异的项 |

### 1.4 目标用户

| 用户 | 本规格关注点 |
|------|-------------|
| 普通办公用户 | "恢复"按钮的行为必须与文案一致；恢复结果必须可见 |
| PC 端开发者/知识工作者 | WSL 协同场景下恢复后代理地址正确、系统代理不残留 |
| 进阶用户/运维调试用户 | 审计日志可追溯恢复过程；恢复结果可验证 |

### 1.5 成功标准

| # | 标准 | 验证方式 |
|---|------|----------|
| SC-1 | "立即恢复"按钮执行后，所有 Restorable 项回到 baseline 值 | 点击"立即恢复"后逐项对比当前状态与 baseline |
| SC-2 | "立即恢复"按钮执行后，mihomo 进程已停止，系统代理已清除 | `ps aux \| grep mihomo` 无进程；Windows 注册表 ProxyEnable=0 |
| SC-3 | WSL 场景下 proxy-env 分类一致（本地与远程均为 Excluded） | 验证两个适配器的 `state_item_definitions()` 中 proxy-env category |
| SC-4 | 恢复流程每个恢复动作记入审计日志 | 恢复后查询 `get_audit_log`，确认包含每个 state_item 的恢复记录 |
| SC-5 | 恢复前先对比偏差，仅恢复有差异的项 | 恢复 baseline 一致场景时，恢复报告显示"0 项恢复，N 项匹配" |
| SC-6 | 非目标站点验证使用 AppConfig 配置而非硬编码 | 修改 `AppConfig.non_target_probe_sites` 后验证生效 |
| SC-7 | 恢复完成后 DNS 缓存已刷新 | 恢复后 `ipconfig /displaydns` 或 `resolvectl statistics` 显示缓存已清 |
| SC-8 | 恢复结果向用户展示（区分"已恢复"/"需手动处理"） | UI 展示恢复结果摘要，包含成功/失败/跳过项数 |
<!-- ? id:05;status:open;date:2026-06-09T15:00 SC-8验证方式"UI展示恢复结果摘要"缺乏可量化标准。建议补充：恢复结果需在UI持续展示至少N秒或用户主动关闭，且失败项必须包含具体state_item名称和失败原因 -->

## 2. 功能需求

### 2.1 P0: "立即恢复"语义修复

#### FR-2.1.1 恢复按钮行为修正

**要求**:
- FR-2.1.1-R1: 仪表盘"立即恢复"按钮必须调用 `restore_to_baseline` 路径（停止 mihomo + 恢复所有 Restorable 项 + 清除 proxy-env + 清除系统代理），而非 `triggerReadjustment`
- FR-2.1.1-R2: 恢复完成后必须向用户展示恢复结果摘要（成功项数/失败项数/跳过项数）
- FR-2.1.1-R3: 恢复过程中必须显示 RecoveryOverlay 蒙层（与 `stop_service` 一致）

#### FR-2.1.2 `triggerReadjustment` 语义澄清

**要求**:
- FR-2.1.2-R1: `triggerReadjustment` 从仪表盘快捷操作区移除；仅在 baseline 确认流程中保留（其语义为"重新评估"，属于 F001 §2.2 确认前操作）
- FR-2.1.2-R2: 若仪表盘需要"恢复后重新启动"的语义，需设计新命令 `restore_and_restart`，但**不在本 feature 范围内**（待用户决策后另开 feature）

#### FR-2.1.3 恢复后站点列表状态

**要求**:
- FR-2.1.3-R1: 恢复到 baseline 后，代理站点列表不应消失（站点配置是持久数据，不属于 baseline 状态项），但 mihomo 已停止，站点不再走代理
<!-- ? id:04;status:open;date:2026-06-09T15:00 "站点配置是持久数据"——当前active_sites为纯内存状态（F003 spec范畴，本spec §4.1也排除），app重启后站点列表会丢失。FR-2.1.3-R2要求展示"站点列表保留但未激活"，但若用户恢复后重启app则列表丢失，UX不一致。建议在FR中明确限定"当前会话内" -->
- FR-2.1.3-R2: 恢复后仪表盘必须明确展示"服务已停止，站点列表保留但未激活"的状态

### 2.2 P0: WSL proxy-env 分类统一

#### FR-2.2.1 分类一致性

**要求**:
- FR-2.2.1-R1: `wsl_remote.rs` 中 `wsl-proxy-env` 的 category 必须改为 Excluded（与 `wsl.rs` 一致）
- FR-2.2.1-R2: proxy-env 作为服务生命周期 overlay（F108 结论），不应参与 `restore_to_baseline()` 的 Restorable 项遍历
- FR-2.2.1-R3: 停止服务时仍需 `clear_proxy_env()` + `deactivate_system_proxy()`（服务生命周期清理），恢复到 baseline 时不需要恢复 proxy-env

### 2.3 P1: 恢复流程审计日志

#### FR-2.3.1 恢复审计记录

**要求**:
- FR-2.3.1-R1: `restore_to_baseline()` 中每个 Restorable 项的恢复动作（成功/失败/跳过）必须记入 AuditLogger
- FR-2.3.1-R2: `stop_service()` 调用 `restore_to_baseline()` 后的汇总结果必须记入审计
- FR-2.3.1-R3: 审计记录必须包含：state_item_id、目标值、实际结果、失败原因（若有）

### 2.4 P1: ProxyGuard 监控范围扩展

#### FR-2.4.1 系统代理残留检查

**要求**:
- FR-2.4.1-R1: ProxyGuard 检测到 mihomo 进程不可达时，必须同时检查系统代理是否仍指向 mihomo 地址（ProxyEnable=1 + ProxyServer 包含 mihomo 端口）
<!-- ? id:03;status:open;date:2026-06-09T15:00 ProxyEnable/ProxyServer为Windows注册表概念，Linux端（含WSL本地）无此机制。Linux端系统代理残留如何定义？是否需补充Linux端的代理残留检测（如检查http_proxy环境变量是否仍指向mihomo地址）？ -->
- FR-2.4.1-R2: 检测到系统代理残留时，必须立即将系统代理恢复到 baseline 值（FR-2.5.2-R4）
- FR-2.4.1-R3: 系统代理残留检查结果必须记入审计

> 注：本 feature 仅覆盖"系统代理残留"这一最关键的监控缺口。端口占用、客户端异常、子进程残留等监控扩展不在本 feature 范围内，待后续 feature 按优先级增量覆盖。

### 2.5 P1: 非目标站点验证配置化

#### FR-2.5.1 验证站点来源

**要求**:
- FR-2.5.1-R1: `verify_non_target_sites()` 必须使用 `AppConfig.non_target_probe_sites` 而非硬编码 URL 列表
- FR-2.5.1-R2: AppConfig 默认值仍为 baidu + bing，但用户可通过配置文件自定义

### 2.6 P2: Windows 端 proxy 地址感知

#### FR-2.6.1 代理地址动态选择

**要求**:
- FR-2.6.1-R1: `determine_proxy_address()` 在 Windows 端（Coordinated 模式）也应能感知 WSL NAT/Mirrored 模式，通过 WslRemoteAdapter 获取 WSL 状态
- FR-2.6.1-R2: Windows 端调用 `activate_system_proxy()` 时，NAT 模式下使用 WSL eth0 IP，Mirrored 模式下使用 127.0.0.1

### 2.7 P2: 恢复后 DNS 缓存刷新

#### FR-2.7.1 DNS 缓存清理

**要求**:
- FR-2.7.1-R1: 恢复完成后必须执行 DNS 缓存刷新（Windows: `ipconfig /flushdns`；Linux: `resolvectl flush-caches` 或等效）
- FR-2.7.1-R2: DNS 刷新失败不阻塞恢复流程，但必须记入审计日志

### 2.8 P2: ProxyStrategy 与恢复流程集成

#### FR-2.8.1 Mirrored 模式恢复优化

**要求**:
- FR-2.8.1-R1: Mirrored 模式下 ProxyStrategy 为 SkipConfig 时，恢复流程应跳过 proxy-env 相关恢复步骤（因为 Mirrored 模式不设置系统代理）
<!-- ? id:06;status:open;date:2026-06-09T15:00 "Mirrored模式不设置系统代理"这一假设需验证——WSL Mirrored模式下mihomo仍然可以设置http_proxy环境变量（apply_proxy_env在F108中已实现）。跳过proxy-env恢复的前提是Mirrored模式下确未写入proxy-env，需确认activate_system_proxy在Mirrored模式下的实际行为 -->
- FR-2.8.1-R2: ProxyStrategy 信息必须传入 `restore_to_baseline()` 或在恢复流程中查询

### 2.9 P2: 恢复前偏差确认

#### FR-2.9.1 先对比再恢复

**要求**:
- FR-2.9.1-R1: `restore_to_baseline()` 必须先采集当前值并与 baseline 对比，仅恢复有差异的项
<!-- TODO id:08;status:open;date:2026-06-09T15:00 偏差确认需read_state()逐项读取当前值。当前PlatformAdapter trait无read_state(item_id)方法（仅有read_state_items()返回全量），design建议新增read_state()但未评估对现有适配器（Windows/Linux/WSL/WslRemote共4个）的实现工作量 -->
- FR-2.9.1-R2: 一致项标记为"跳过"（Skipped），并在恢复结果中体现
- FR-2.9.1-R3: 偏差确认结果记入审计日志

## 3. 非功能需求

### 3.1 性能

- NFR-3.1-1: 恢复流程总耗时 ≤ 20s（与 F001 NFR-3.1-3 一致）
<!-- ? id:02;status:open;date:2026-06-09T15:00 20s是否包含DNS刷新(P2-109-7)和非目标站点验证(P1-109-5)？DNS刷新在Linux下可能需等待resolvectl超时，非目标验证涉及网络探测，两者叠加可能突破20s。建议明确NFR计算范围或调整阈值 -->

### 3.2 安全

- NFR-3.2-1: 修复后不得引入新的安全缺口（恢复流程不可被绕过）

### 3.3 可追溯性

- NFR-3.3-1: 所有恢复操作必须有审计记录可查

## 4. 约束与边界

### 4.1 不在本 feature 范围内

- 启动续跑（P0 级，但涉及 App 启动流程改动，需单独 feature）
<!-- TODO id:07;status:open;date:2026-06-09T15:00 启动续跑标记为P0但排除在本feature外。若用户在恢复过程中app崩溃（is_restoring=true但未完成），重启后无续跑机制将导致系统代理残留+用户无法操作。建议明确：本feature是否需至少保证恢复过程的原子性（全部完成或全部回滚） -->
- active_sites 持久化（属于 F003 站点规则引擎范畴）
- config.yaml rules 段硬编码与动态站点管理脱节（属于 F003 范畴）
- externally_managed mihomo 停止不彻底（属于 F107 范畴）
- 端口占用/子进程残留/客户端异常等 ProxyGuard 扩展监控
- "恢复后重新启动代理"新命令设计

### 4.2 必须遵守的约束

- 不得破坏 F001 已通过的设计评审结论（除非本 feature 的 spec 评审明确推翻）
- 不得改变 baseline 数据格式（backward compatible）
- proxy-env 作为 Excluded 的结论（F108）不可回退
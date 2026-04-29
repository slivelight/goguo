# GitHub Hosts Manager — 业务架构：用户交互规格说明书

> **版本**: v2.0 | **日期**: 2026-04-25
> **范围**: arch.md 中 BA 架构的细化，定义完整的用户交互规格
> **源参考**: `docs/superpowers-v2/specs/bat-menu-redesign-spec.md` + BAT codebase 实际实现
> **关联**: arch.md §1 (BA 概览)

---

## 1. 设计目标

| 目标 | 说明 | 优先级 |
|------|------|--------|
| 一键可达 | 用户按 `1` 即获得最佳 GitHub 访问体验 | P0 |
| 状态可见 | 每次菜单刷新显示当前访问状态 | P0 |
| 渐进披露 | 常用操作 1 键直达，高级功能通过子菜单暴露 | P1 |
| 操作闭环 | 每个操作有明确输入→执行→结果展示→确认流程 | P1 |
| 网络切换零感知 | 用户换网络后系统自动调整策略 | P0 |
| 故障可诊断 | 自动恢复失败时给出问题定位和操作建议 | P1 |

---

## 2. 用户画像

| 角色 | 占比 | 典型行为 | 核心诉求 |
|------|------|----------|----------|
| **普通用户** | 70% | 只用 `1` 号一键访问，偶尔看状态 | "GitHub 能打开就行" |
| **进阶用户** | 25% | 切换模式、选节点、管理规则 | "我要控制流量走向" |
| **运维用户** | 5% | 查日志、修 IP 池、调试网络 | "出了问题能快速定位" |

---

## 3. 主菜单结构

```
===========================================================
           GitHub Hosts Manager
===========================================================

 ┌─ Status Panel ────────────────────────────────────────┐
 │ (由 Render-StatusPanel.ps1 渲染，7 行数据面板)        │
 └───────────────────────────────────────────────────────┘

  1. One-Click Access (Auto-detect best method)
  2. Quick Update Hosts
  3. Access Mode Management
  4. Proxy Node Management
  5. Rule Management
  ─────────────────────────────────────────
  6. Network Diagnostics
  7. Logs and Statistics
  8. System Settings
  9. Clear Cache
  ─────────────────────────────────────────
  0. Exit

  Select:
```

**菜单实现**: `GitHub-Hosts-Manager.bat`，单实例锁定（`GHM_NO_SINGLETON`），UTF-8 编码（`chcp 65001`）

---

## 4. 用例模型

### 4.1 UC1: 一键智能访问

| 项目 | 规格 |
|------|------|
| **入口** | 主菜单 `1` / 子菜单 3.1 |
| **调用** | `scripts\Invoke-IntelligentMode.ps1 -Action start` |
| **前置** | 无（任何状态都可触发） |
| **后置** | GitHub 可访问 + 后台监控启动 + 状态面板更新 |

**执行流程**:

```
用户按 1
  → 检测网络环境 (Get-NetworkEnvironment)
  → 查询策略表 (network-strategies.json)
  → 三路并行探测 (Direct/Hosts/Proxy)
  → 评分算法选最优方案 (Resolve-ConnectionPlan)
  → 执行方案 (Invoke-HostsMethod / Invoke-ProxyMethod / 无操作)
  → 验证 GitHub 可达性
  → 启动后台监控 (MonitorServer)
  → 显示结果面板
```

**结果面板**:

```
  Method:    Hosts
  Reason:    Direct blocked, Hosts fastest
  Confidence: 95%

  Execution: SUCCESS
  Latency:   142ms
```

### 4.2 UC2: 快速更新 Hosts

| 项目 | 规格 |
|------|------|
| **入口** | 主菜单 `2` |
| **调用** | `scripts\Update-GitHubHosts.ps1 -QuickMode -Verbose` |
| **前置** | 管理员权限（`net session` 验证） |
| **后置** | hosts 文件更新 + DNS 缓存清理 + IP 探测 |

**执行流程**:
```
检查管理员权限 → 失败则提示提权
→ Update-GitHubHosts.ps1 -QuickMode
→ 清理缓存
→ Invoke-GitHubProbeLite -Method Hosts
→ 显示结果
```

### 4.3 UC3: 访问模式管理

| 项目 | 规格 |
|------|------|
| **入口** | 主菜单 `3` |
| **子菜单** | 8 个选项 + Monitor 控制面板 |

**子菜单结构**:

```
  Access Mode Management
  ────────────────────────────
  当前网络/模式: (从 proxy-state.json 读取)

  --- Quick Switch ---
  1. Intelligent Switch     → :action-1 (复用 UC1)
  2. Force Hosts Mode       → 停代理 + 更新 Hosts + 探测
  3. Force Proxy Mode       → 启动代理核心 + 探测
  4. Switch to Direct Mode  → 停代理 + 停智能模式 + 探测
  5. Stop All Modes+Monitor → 停所有服务 + 探测(None)

  --- Network Strategy ---
  6. View Network Strategy  → Show-NetworkStrategy.ps1
  7. Modify Network Strategy → Edit-NetworkStrategy
  8. Monitor Control Panel  → 见 §4.3.1

  0. Return
```

**UC3.2 Force Hosts**:
```
Invoke-ProxyMode.ps1 -Action stop   # 先停代理
→ Update-GitHubHosts.ps1 -QuickMode # 写 hosts
→ Invoke-GitHubProbeLite -Method Hosts # 验证
```

**UC3.3 Force Proxy**:
```
Invoke-ProxyMode.ps1 -Action start  # 启动 mihomo
→ Invoke-GitHubProbeLite -Method Proxy # 验证
```

**UC3.8 Monitor 控制面板**:

```
  Monitor Control Panel

  MonitorServer: RUNNING (PID: 12345, Uptime: 7200s)   ← CMD type monitor-status.txt
  Method: Hosts | Network: Home (192.168.1.1) | GitHub: ok (45ms)

  1. Start  2. Stop  3. Restart  4. Status Detail  5. Auto-start Config  0. Return
```

| 选项 | 实现 | 说明 |
|------|------|------|
| 1. Start | `Control-MonitorServer.ps1 -Action Start` | 含两阶段健康检查 |
| 2. Stop | `Control-MonitorServer.ps1 -Action Stop` | API stop + 端口清理 |
| 3. Restart | `Control-MonitorServer.ps1 -Action Restart` | 单进程 Stop+Start |
| 4. Detail | `GET http://127.0.0.1:9091/api/status` | 完整 JSON |
| 5. Auto-start | 读写 `proxy-settings.json` 的 `autoStart` 字段 | 开关切换 |

### 4.4 UC4: 代理节点管理

| 项目 | 规格 |
|------|------|
| **入口** | 主菜单 `4` |
| **子菜单** | 10 个选项 |

```
  Proxy Node Management
  ────────────────────────────
  订阅数/核心状态: (从 proxy-state.json + subscriptions.json 读取)

  1. View Node List and Latency    → Show-NodeList.ps1
  2. Full Node Speed Test          → Invoke-SpeedTest.ps1
  3. Update Subscription Nodes     → SubscriptionParser + ProxyConfigGenerator
  4. Configure Subscription URL    → Invoke-ProxyMode.ps1 -Action config
  ──────────────────────────────
  5. Pool Dashboard                → GET /api/pool
  6. Subscription Sources          → GET/POST/DELETE /api/sources (子菜单)
  7. Manual Pool Scan              → POST /api/pool/scan
  8. HA Status                     → GET /api/ha (planning)
  ──────────────────────────────
  9. Whitelist Management          → config/sites/*.json (子菜单)
  T. Transport Mode                → TUN/System Proxy 切换 (子菜单)

  0. Return
```

**UC4.6 Subscription Sources 子菜单**:
```
  列出所有源: [ON/OFF] [type] id
  1. Add Source    → POST /api/sources + 用户输入 URL
  2. Verify Source → POST /api/sources/{id}/verify
  3. Remove Source → DELETE /api/sources/{id}
  0. Return
```

**UC4.9 Whitelist Management 子菜单**:
```
  列出所有站点: [ON/OFF] name (id) - N domains
  1. Toggle Site       → 读写 config/sites/{id}.json 的 enabled 字段
  2. View Site Domains → 按分类显示域名
  3. IP Quality Summary → 从 node-pool.json 统计各站点的节点质量
  0. Return
```

**UC4.T Transport Mode 子菜单**:
```
  当前模式: TUN / System Proxy
  1. TUN Mode (WinTun, all traffic)    → PUT /api/config {enableTun: true}
  2. System Proxy Mode (HTTP/SOCKS)    → PUT /api/config {enableTun: false}
  0. Return
```

### 4.5 UC5: 规则管理

| 项目 | 规格 |
|------|------|
| **入口** | 主菜单 `5` |
| **子菜单** | 4 个选项 |

```
  Rule Management
  ────────────────────────────
  规则统计: (扫描 config/ruleset/*.yaml 行数)

  1. Update GitHub Rule Sets → New-GitHubDomainRuleSet + New-GitHubIPRuleSet -FetchLatest
  2. Add Custom Rule         → 用户输入 pattern/type/action → Add-CustomRule
  3. View All Rules          → 读取全部 yaml 文件内容
  4. Delete Custom Rule      → 列出规则按编号选择 (partially implemented)

  0. Return
```

**自定义规则输入交互**:
```
  Pattern (e.g. mycompany.com): <用户输入>
  Type (DOMAIN-SUFFIX):         <默认 DOMAIN-SUFFIX>
  Action (DIRECT/PROXY/BLOCK):  <默认 DIRECT>
```

### 4.6 UC6: 网络诊断

| 项目 | 规格 |
|------|------|
| **入口** | 主菜单 `6` |
| **子菜单** | 5 个选项 |

```
  Network Diagnostics
  ────────────────────────────
  当前网络: (从 proxy-state.json 读取)

  1. GitHub Connectivity Test (3-way) → Show-GitHubConnectivity.ps1
  2. Auto-Recovery Diagnostics       → proxy-state.json.history 最近 10 条
  3. Test IP Connectivity             → Test-IPModule.ps1
  4. View IP Pool State               → View-IPPoolState.ps1
  5. View Network Environment         → View-NetworkProfiles.ps1

  0. Return
```

**UC6.1 三路连通性测试输出**:
```
  Direct:
    TCP 连接: ✓ 成功 (230ms) / ✗ 失败
    HTTPS:    ✓ 200 OK / ✗ 超时

  Hosts (最优 IP: x.x.x.x):
    TCP 连接: ✓ 成功 (85ms) / ✗ 失败
    HTTPS:    ✓ 200 OK / ✗ 超时

  Proxy (节点: JP-Hy2):
    API 延迟: ✓ 45ms / ✗ 超时
    核心状态: ■ 运行中 / ○ 未启动
```

**UC6.2 恢复诊断输出**:
```
  Recent Recovery Events:
  2026-04-24T08:54:11  Proxy    142ms  OK
  2026-04-24T07:30:00  Hosts    85ms   OK
  ...
```

### 4.7 UC7: 日志与统计

| 项目 | 规格 |
|------|------|
| **入口** | 主菜单 `7` |
| **子菜单** | 3 个选项 |

```
  Logs and Statistics
  ────────────────────────────
  1. View Operation Logs (last 50) → Get-Content ... -Tail 50
  2. Access History Statistics     → Show-AccessHistory.ps1
  3. Decision History              → Show-DecisionHistory (dot-sourced)

  0. Return
```

### 4.8 UC8: 系统设置

| 项目 | 规格 |
|------|------|
| **入口** | 主菜单 `8` |
| **子菜单** | 7 个选项 |

```
  System Settings
  ────────────────────────────
  1. Install Scheduled Task       → Install-GitHubHostsUpdate.ps1 (需管理员)
  2. Uninstall Scheduled Task     → Uninstall-GitHubHostsUpdate.ps1 (需管理员)
  3. View Task Status             → Get-ScheduledTask GitHubHostsUpdate
  4. Configure Monitoring Params  → 显示 network-strategies.json 参数
  5. Configure Proxy Core Path    → 显示 core/mihomo.exe 路径
  6. Clear Cache and DNS          → 复用 :clear-cache-silent
  7. Node Pool Settings           → 显示池参数

  0. Return
```

### 4.9 UC9: 清理缓存

| 项目 | 规格 |
|------|------|
| **入口** | 主菜单 `9` / 子菜单 8.6 |
| **调用** | `:clear-cache-silent` 子程序 |
| **操作** | 清理 `%APPDATA%\GitHubHosts` + `data\cache\*.tmp` |

---

## 5. 后台用例

| ID | 场景 | 触发 | 周期 | 实现位置 |
|----|------|------|------|----------|
| UC-B1 | 网络变更检测 | WMI 事件 + 30s 轮询 | 30s | MonitorServer Watchdog |
| UC-B2 | GitHub 探测 | TCP + HTTPS 双层 | 15s | `Invoke-GitHubProbe` |
| UC-B3 | 节点健康检查 | mihomo API 延迟 | 300s | MonitorServer |
| UC-B4 | IP 质量检查 | TCP 验证 hosts | 60s | MonitorServer |
| UC-B5 | 自动恢复 | 探测失败/网络变更 | 事件 | `Invoke-AutoRecovery` (5阶段) |
| UC-B6 | 节点池扫描 | 定时 | 30min/10min | `NodePoolScanner` |
| UC-B7 | 主备切换 | 主崩溃 | 事件 | planning |
| UC-B8 | 动态源发现 | 降级阶段 | 事件 | planning |

---

## 6. 状态面板

### 6.1 主菜单状态面板

**实现**: `scripts\Render-StatusPanel.ps1`，PowerShell 渲染 7 行面板，延迟 1-2s

| 行 | 字段 | 数据来源 | 可选值 |
|----|------|----------|--------|
| 1 | 访问方式 | `proxy-state.json` → `currentPlan.method` | Direct / Hosts / Proxy / 未连接 |
| 2 | GitHub 可达 | 实时 TCP `github.com:443` | ● 可达 (Nms) / ○ 不可达 |
| 3 | 代理核心 | `Get-ProxyCoreStatus` | ■ 运行中 (PID) / ○ 未启动 |
| 4 | 节点数 | `proxy-nodes-cache.json` | N 个可用 |
| 5 | 网络环境 | `Get-NetworkEnvironment` | 家庭/企业/移动/未知 + ISP |
| 6 | 当前策略 | 网络策略映射表 | Hosts → Direct → Proxy |
| 7 | 监控状态 | `monitor-status.txt` 存在性 | ● 运行中 / ○ 已停止 |

### 6.2 Monitor 控制面板状态

**实现**: CMD `type monitor-status.txt`，延迟 <50ms

```
  MonitorServer: RUNNING (PID: 4320, Uptime: 185s)
  Method: Hosts | Network: Home (192.168.1.1) | GitHub: ok (45ms)
  IP: 20.205.243.166 (github.com 45ms, score 85) | Hosts: 12
  Recovery: Proxy->Hosts (success, 30s ago) | SpeedTest: 5/24
```

**更新频率**:

| 触发 | 频率 | 更新内容 |
|------|------|----------|
| 网络检查 | 30s | Network, Gateway |
| GitHub 探测 | 15s | GitHub 可达性 |
| IP 质量检查 | 60s | IP 评分、Hosts 条目数 |
| 心跳兜底 | 30s | Uptime, 全量刷新 |
| 恢复完成 | 即时 | Recovery, Method |
| 服务停止 | 即时 | 删除文件 |

---

## 7. 显示规范

### 7.1 色块标识

| 方式 | 色块 | 颜色 |
|------|------|------|
| 直连 | `■` | Green |
| Hosts | `■` | Blue |
| 代理 | `■` | Orange |
| 未连接 | `■` | Red |

### 7.2 状态符号

| 符号 | 含义 |
|------|------|
| `●` | 在线/可用/运行中 |
| `○` | 离线/不可用/未启动 |
| `■` | 当前选中/激活 |
| `✓` | 操作成功 |
| `✗` | 操作失败 |

### 7.3 颜色规范

| 元素 | 颜色 | 用途 |
|------|------|------|
| 标题线 | Cyan | 分隔线和标题 |
| 成功/可用 | Green | 状态正常 |
| 警告 | Yellow | 非致命问题 |
| 错误/失败 | Red | 错误和失败 |
| 信息 | White | 一般文本 |
| 分隔符 | DarkGray | 视觉分隔 |

---

## 8. 沙盒模式

**启用**: 设置环境变量 `GHM_SANDBOX=1`

所有状态变更操作在沙盒模式下输出 `[SANDBOX] xxx simulated`，不执行实际操作。覆盖范围：
- 主菜单 1/2/9
- 子菜单 3 全部选项
- 子菜单 4 的 speedtest / update-sub / config-sub / sources / scan / whitelist / transport
- 子菜单 5 的 add / delete
- 子菜单 8 的 install / uninstall

---

## 9. 菜单导航规则

| 规则 | 说明 |
|------|------|
| 逐级回退 | 每个子菜单 `0` 返回上一级，`goto menu` 仅限主菜单 |
| 禁止跨级 | 不允许从子菜单直接 `goto` 到另一子菜单 |
| 操作后返回 | 每个操作执行后 `pause` + `goto` 本菜单标签 |
| 复用标签 | UC3.1 (智能切换) 复用 `:action-1`，UC8.6 (清理) 复用 `:action-9` |

---

## 10. 客户端-服务端架构

```
┌──────────────────────┐           ┌──────────────────────┐
│   客户端 (BAT 菜单)  │           │   服务端 MonitorServer │
│                      │           │                      │
│  CMD type 读状态文件  │◄──────────│  三层监控引擎         │
│  PS 脚本控制操作     │──────────►│  HTTP REST API :9091 │
│  用户交互收集        │  HTTP API │  自动恢复/级联降级    │
│  结果展示            │           │  Write-StatusTextFile │
└──────────────────────┘           └──────────────────────┘
```

**关键特性**:
- BAT 窗口关闭不影响 MonitorServer（独立进程）
- `type monitor-status.txt` (<50ms) 替代 PowerShell REST API (1-4s) 用于高频状态读取
- `Control-MonitorServer.ps1` 单进程统一控制 Start/Stop/Restart

---

## 11. 网络策略映射

| 网络类型 | 网关特征 | 首选 | 备选 | 保底 |
|----------|----------|------|------|------|
| Enterprise | `10.0.0.0/8` `172.16.0.0/12` | Proxy (VLESS) | Hosts | Direct |
| Home | `192.168.0.0/16` | Hosts | Direct | Proxy |
| Mobile | `100.64.0.0/10` (CGNAT) | Proxy (Hy2) | Hosts | Direct |
| Public | `172.16-31.x.x` | Proxy (强制) | — | — |
| Unknown | 其他 | 智能探测 | Proxy | Hosts |

---

## 12. 自动恢复机制

### 三层监控体系

```
┌────────────────────────────────────────────────────┐
│ L1: 网络变更 (WMI事件 + 30s轮询) → 策略重评估      │
├────────────────────────────────────────────────────┤
│ L2: GitHub 探测 (15s, TCP+HTTPS) → 级联降级        │
├────────────────────────────────────────────────────┤
│ L3: 节点健康 (300s, mihomo API) → 节点切换         │
└────────────────────────────────────────────────────┘
```

### 5 阶段恢复升级链

```
Phase 0: HA Failover (主备切换)
Phase 1: Get-TopCandidates (获取候选节点)
Phase 2: 缓存恢复循环 (TCP 验证缓存中的节点/IP)
Phase 3: 外部探测 + DecisionEngine 重评估
Phase 4: 退避 (30s → 60s → 120s → 300s → 600s)
Phase 5: 告警阈值 (连续 3 次失败 → 面板提示)
```

---

## 附录: 菜单编号与脚本映射完整表

| 菜单路径 | BAT 标签 | 脚本/命令 | 需管理员 | 沙盒保护 |
|----------|----------|-----------|----------|----------|
| 主 1 | `:action-1` | `Invoke-IntelligentMode.ps1 -Action start` | — | ✓ |
| 主 2 | `:action-2` | `Update-GitHubHosts.ps1 -QuickMode -Verbose` | ✓ | ✓ |
| 主 9 | `:action-9` | `:clear-cache-silent` | — | ✓ |
| 3.1 | `→ :action-1` | (复用 UC1) | — | ✓ |
| 3.2 | `:sub3-force-hosts` | `Invoke-ProxyMode stop` + `Update-GitHubHosts` | ✓ | ✓ |
| 3.3 | `:sub3-force-proxy` | `Invoke-ProxyMode start` | — | ✓ |
| 3.4 | `:sub3-direct` | `Invoke-ProxyMode stop` + `Invoke-IntelligentMode stop` | — | ✓ |
| 3.5 | `:sub3-stop-all` | `Invoke-IntelligentMode stop` + `Invoke-ProxyMode stop` | — | ✓ |
| 3.6 | `:sub3-view-strategy` | `Show-NetworkStrategy.ps1` | — | — |
| 3.7 | `:sub3-edit-strategy` | `Edit-NetworkStrategy` (dot-sourced) | — | — |
| 3.8 | `:sub3-monitor` | CMD `type monitor-status.txt` | — | — |
| 3.8.1 | `:mon-start` | `Control-MonitorServer.ps1 -Action Start` | — | ✓ |
| 3.8.2 | `:mon-stop` | `Control-MonitorServer.ps1 -Action Stop` | — | ✓ |
| 3.8.3 | `:mon-restart` | `Control-MonitorServer.ps1 -Action Restart` | — | ✓ |
| 3.8.4 | `:mon-status` | `GET /api/status` | — | — |
| 3.8.5 | `:mon-autostart` | 读写 `proxy-settings.json` | — | ✓ |
| 4.1 | `:sub4-nodes` | `Show-NodeList.ps1` | — | — |
| 4.2 | `:sub4-speedtest` | `Invoke-SpeedTest.ps1` | — | ✓ |
| 4.3 | `:sub4-update-sub` | `SubscriptionParser` + `ProxyConfigGenerator` | — | ✓ |
| 4.4 | `:sub4-config-sub` | `Invoke-ProxyMode.ps1 -Action config` | — | ✓ |
| 4.5 | `:sub4-pool-dashboard` | `GET /api/pool` | — | — |
| 4.6 | `:sub4-sources` | `GET/POST/DELETE /api/sources` | — | ✓ |
| 4.7 | `:sub4-manual-scan` | `POST /api/pool/scan` | — | — |
| 4.8 | `:sub4-ha-status` | `GET /api/ha` | — | — |
| 4.9 | `:sub4-whitelist` | `config/sites/*.json` 读写 | — | ✓ |
| 4.T | `:sub4-transport` | `PUT /api/config` + `POST /api/restart` | — | ✓ |
| 5.1 | `:sub5-update` | `GitHubRuleSet.psm1` | — | — |
| 5.2 | `:sub5-add` | `Add-CustomRule` | — | ✓ |
| 5.3 | `:sub5-view` | 读取 yaml 文件 | — | — |
| 5.4 | `:sub5-delete` | (partially implemented) | — | ✓ |
| 6.1 | `:sub6-github-test` | `Show-GitHubConnectivity.ps1` | — | — |
| 6.2 | `:sub6-recovery` | `proxy-state.json.history` 读取 | — | — |
| 6.3 | `:sub6-ip-test` | `Test-IPModule.ps1` | — | — |
| 6.4 | `:sub6-ip-pool` | `View-IPPoolState.ps1` | — | — |
| 6.5 | `:sub6-network` | `View-NetworkProfiles.ps1` | — | — |
| 7.1 | `:sub7-logs` | `Get-Content -Tail 50` | — | — |
| 7.2 | `:sub7-history` | `Show-AccessHistory.ps1` | — | — |
| 7.3 | `:sub7-decisions` | `Show-DecisionHistory` (dot-sourced) | — | — |
| 8.1 | `:sub8-install` | `Install-GitHubHostsUpdate.ps1` | ✓ | ✓ |
| 8.2 | `:sub8-uninstall` | `Uninstall-GitHubHostsUpdate.ps1` | ✓ | ✓ |
| 8.3 | `:sub8-task-status` | `Get-ScheduledTask` | — | — |
| 8.4 | `:sub8-monitor-config` | 显示 `network-strategies.json` | — | — |
| 8.5 | `:sub8-core-path` | 显示 `core/mihomo.exe` 路径 | — | — |
| 8.6 | `→ :action-9` | (复用 UC9) | — | ✓ |
| 8.7 | `:sub8-pool-settings` | 显示池参数 | — | — |

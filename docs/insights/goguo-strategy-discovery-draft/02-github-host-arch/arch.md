# GitHub Hosts Manager — 4A 架构蓝图

> **版本**: v2.0 | **日期**: 2026-04-25
> **范围**: 项目全景架构梳理，基于 codebase 实际状态
> **读者**: 架构师、开发工程师、产品负责人
> **源文档**: 本文档为纲要，详细视图见 arch-BA.md / arch-AA.md / arch-DA.md / arch-TA.md

---

## 架构概览

```
┌──────────────────────────────────────────────────────────────────────┐
│  用户界面层 (BAT Menu + Status Panel)                    [BA: UC1-9] │
├──────────────────────────────────────────────────────────────────────┤
│  编排层   DecisionEngine → 策略路由 → Hosts / Proxy / Direct        │
│           MonitorServer (:9091) ← REST API + Watchdog               │
├───────────────────────┬──────────────────────────────────────────────┤
│  Hosts 引擎           │  Proxy 引擎                                │
│  IPFetcher → Scanner  │  ProxyCoreManager (mihomo)                  │
│  → Selector → Hosts   │  → SubscriptionParser → ConfigGen           │
│  → IPPoolMaintainer   │  → NodePoolScanner → GitHubRuleSet          │
│  → CommunitySource    │  → per-site rule-provider + proxy-group     │
├───────────────────────┴──────────────────────────────────────────────┤
│  数据层   CacheManager (4层缓存) + 原子 JSON 持久化                  │
├──────────────────────────────────────────────────────────────────────┤
│  基础设施  Logger · StateManager · NetworkMonitor                    │
│            config/*.json + config/sites/*.json → 外部配置驱动        │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 1. BA — 业务架构

> 详见 [arch-BA.md](arch-BA.md)

### 1.1 业务目标

| 目标 | 度量 | 当前基线 |
|------|------|----------|
| GitHub 可用性 | 连通率 % | 依赖网络环境，目标 >99% |
| 全站点覆盖 | 白名单站点可达率 | 仅 Proxy 模式覆盖 5 站点 153 域名 |
| 故障恢复时间 | 秒 | 自动检测 15s 周期 + 5 阶段升级恢复 |
| 用户操作成本 | 点击次数 | 一键访问 (UC1) = 1 次选择 |

### 1.2 用户角色

| 角色 | 占比 | 场景 | 核心操作 |
|------|------|------|----------|
| 普通用户 | 70% | 企业/家庭网络 GitHub 被墙 | UC1 一键访问 |
| 进阶用户 | 25% | 切换模式、选节点、管理规则 | UC3 模式管理 + UC4 节点管理 |
| 运维用户 | 5% | 批量部署、调试网络、查看日志 | UC6 诊断 + UC8 设置 |

### 1.3 核心用例概览

| ID | 用例 | 入口 | 状态 |
|----|------|------|------|
| UC1 | 一键智能访问 | 主菜单 1 | ready |
| UC2 | 快速更新 Hosts | 主菜单 2 | ready |
| UC3 | 访问模式管理 (8 子项) | 主菜单 3 | ready |
| UC4 | 代理节点管理 (10 子项) | 主菜单 4 | partial |
| UC5 | 规则管理 (4 子项) | 主菜单 5 | ready |
| UC6 | 网络诊断 (5 子项) | 主菜单 6 | ready |
| UC7 | 日志与统计 (3 子项) | 主菜单 7 | ready |
| UC8 | 系统设置 (7 子项) | 主菜单 8 | ready |
| UC9 | 清理缓存 | 主菜单 9 | ready |

### 1.4 后台场景

| ID | 场景 | 触发条件 | 周期 | 状态 |
|----|------|----------|------|------|
| UC-B1 | 网络变更检测 | 网关变化 | 30s | ready |
| UC-B2 | GitHub 健康探测 | TCP+HTTPS 双层 | 15s | ready |
| UC-B3 | 代理节点健康检查 | mihomo API 延迟 | 300s | ready |
| UC-B4 | IP 质量检查 | TCP 验证 hosts 条目 | 60s | ready |
| UC-B5 | 自动恢复 (5阶段) | 探测失败/网络变更 | 事件驱动 | ready |
| UC-B6 | 节点池扫描 | 定时 | 30min/10min | partial |
| UC-B7 | 主备切换 | 主实例崩溃 | 事件驱动 | planning |
| UC-B8 | 动态源发现 | 降级/紧急阶段 | 事件驱动 | planning |

### 1.5 三模式能力矩阵

| 目标站点 | HOST 模式 | DIRECT 模式 | PROXY 模式 | 配置来源 |
|----------|-----------|-------------|------------|----------|
| GitHub (47 域名) | ✅ hosts 文件 | ✅ 直连/可能被墙 | ✅ mihomo 代理 | `config/sites/github.json` |
| ChatGPT (22 域名) | ❌ 无 IP + SNI 阻断 | ⚠ 直连/被墙 | ✅ mihomo 代理 | `config/sites/chatgpt.json` |
| Claude (15 域名) | ❌ CDN 动态 IP | ⚠ 直连/被墙 | ✅ mihomo 代理 | `config/sites/claude.json` |
| Google (22 域名) | ❌ CDN 动态 IP | ⚠ 直连/被墙 | ✅ mihomo 代理 | `config/sites/google.json` |
| npmjs (3 域名) | ❌ 无 IP | ✅ 直连/可能被墙 | ✅ mihomo 代理 | `config/sites/npmjs.json` |

**关键结论**: 仅 Proxy 模式通过 mihomo 的 per-site proxy-group + rule-provider 能力覆盖全部 5 站点 109 域名。

---

## 2. AA — 应用架构

> 详见 [arch-AA.md](arch-AA.md)

### 2.1 模块地图

```
                         ┌─────────────────────┐
                         │  BAT Menu (9+0)      │
                         │  GitHub-Hosts-Manager │
                         └──────────┬───────────┘
                                    │
              ┌─────────────────────┼───────────────────────┐
              ▼                     ▼                       ▼
    ┌─────────────────┐  ┌──────────────────┐  ┌───────────────────┐
    │ 智能决策引擎     │  │ 系统监控服务     │  │ 用户交互脚本      │
    │ DecisionEngine  │  │ MonitorServer    │  │ scripts/*.ps1     │
    │   (28KB)        │  │   (72KB)         │  │   (23 文件)       │
    └────────┬────────┘  └────────┬─────────┘  └───────────────────┘
             │                    │
    ┌────────┼────────┐    ┌──────┼──────────┐
    ▼        ▼        ▼    ▼      ▼          ▼
┌────────┐┌────────┐┌────┐┌────┐┌────────┐┌──────────┐
│Hosts   ││Proxy   ││Dir ││Rest││Watchdog││Cache     │
│Engine  ││Engine  ││ect ││API ││        ││Manager   │
└────────┘└────────┘└────┘└────┘└────────┘└──────────┘
```

### 2.2 模块清单

| 模块 | 代码载体 | 大小 | 职责 | 状态 |
|------|----------|------|------|------|
| DecisionEngine | `modules/DecisionEngine.psm1` | 933 行 / 17 函数 | 网络检测、三路探测、评分算法、策略路由、模式执行 | ready |
| MonitorServer | `modules/MonitorServer.psm1` | 3193 行 / 94 函数 | HttpListener REST API + Watchdog 定时任务 + 自动恢复 | ready |
| CacheManager | `modules/CacheManager.psm1` | 951 行 / 17 函数 | 4 层缓存 (节点/IP/订阅/策略) + 原子写入 | ready |
| ProxyCoreManager | `modules/ProxyCoreManager.psm1` | 1001 行 / 27 函数 | mihomo 进程生命周期 + 系统代理注册表 + 配置热重载 | ready |
| ProxyConfigGenerator | `modules/ProxyConfigGenerator.psm1` | 669 行 / 11 函数 | mihomo YAML 生成 (DNS/代理组/规则/路由) | ready |
| SubscriptionParser | `modules/SubscriptionParser.psm1` | 638 行 / 15 函数 | 订阅拉取 + YAML/URI/Base64 解析 + mihomo 组测速 | ready |
| GitHubRuleSet | `modules/GitHubRuleSet.psm1` | 424 行 / 11 函数 | 域名管理 + IP 范围 + 规则文件生成 | ready |
| NodePoolScanner | `modules/NodePoolScanner.psm1` | 1339 行 / 29 函数 | 源管理 + TCP 快筛 + mihomo 精测 + 5 级优先级 | partial |
| IPSelector | `modules/IPSelector.psm1` | 1619 行 / 13 函数 | IP 评分排序 + 紧急池 + 社区源 + 退化策略 | ready |
| IPScanner | `modules/IPScanner.psm1` | 573 行 / 6 函数 | TCP/HTTP 并行扫描 + 延迟测量 | ready |
| IPFetcher | `modules/IPFetcher.psm1` | 1416 行 / 14 函数 | 多源 IP 获取 (API + 内置池 + 社区) | ready |
| IPPoolMaintainer | `modules/IPPoolMaintainer.psm1` | 2174 行 / 25 函数 | IP 池生命周期管理 + 定时刷新 + 质量追踪 | ready |
| StateManager | `modules/StateManager.psm1` | 1019 行 / 20 函数 | 网络指纹 + DPAPI 加密 + proxy-state.json 读写 | ready |
| NetworkMonitor | `modules/NetworkMonitor.psm1` | 257 行 / 6 函数 | WMI 网络适配器事件监听 | ready |
| Logger | `modules/Logger.psm1` | 293 行 / 10 函数 | 分级日志 + 文件轮转 | ready |
| EmergencyPool | `modules/EmergencyPool.psm1` | 596 行 / 10 函数 | 硬编码备用 IP 池 (最后手段) | ready |
| CommunitySource | `modules/CommunitySource.psm1` | 619 行 / 8 函数 | 社区维护的 IP 列表获取 | ready |
| IPRangeStats | `modules/IPRangeStats.psm1` | 471 行 / 8 函数 | /24 段质量追踪 + 优先扫描 | ready |
| **合计** | **18 模块** | **18,185 行 / 321 函数** | — | — |

### 2.3 脚本清单

| 脚本 | 行数 | BAT 入口 | 职责 | 状态 |
|------|------|----------|------|------|
| `Invoke-IntelligentMode.ps1` | 196 | 主菜单 1 / 子菜单 3.1 | UC1 一键访问入口 | ready |
| `Update-GitHubHosts.ps1` | 1009 | 主菜单 2 / 子菜单 3.2 | UC2 Hosts 更新完整流程 | ready |
| `Invoke-ProxyMode.ps1` | 420 | 子菜单 3.3 / 4.4 | 代理模式控制 (start/stop/config) | ready |
| `Render-StatusPanel.ps1` | 476 | 各菜单 | 统一面板渲染 (7 行数据面板) | ready |
| `Control-MonitorServer.ps1` | 281 | 子菜单 3.8 | 监控服务 start/stop/restart | ready |
| `Invoke-GitHubProbeLite.ps1` | 167 | 各脚本 | 轻量 GitHub 连通性探测 | ready |
| `Invoke-SpeedTest.ps1` | 31 | 子菜单 4.2 | 全节点测速 (mihomo API) | ready |
| `Show-NodeList.ps1` | 214 | 子菜单 4.1 | 节点列表展示 | ready |
| `Show-NetworkStrategy.ps1` | 376 | 子菜单 3.6/3.7 | 网络策略展示与编辑 | ready |
| `Show-GitHubConnectivity.ps1` | 346 | 子菜单 6.1 | 三路连通性测试报告 | ready |
| `Show-AccessHistory.ps1` | 341 | 子菜单 7.2/7.3 | 访问历史统计 | ready |
| `Install-GitHubHostsUpdate.ps1` | 352 | 子菜单 8.1 | Windows 计划任务安装 | ready |
| `Uninstall-GitHubHostsUpdate.ps1` | 129 | 子菜单 8.2 | 计划任务卸载 | ready |
| `Start-MonitorServer.ps1` | 94 | 内部 | MonitorServer 进程启动 | ready |
| `Stop-MonitorServer.ps1` | 50 | 内部 | MonitorServer 进程停止 | ready |
| `Maintain-IPPool.ps1` | 431 | 子菜单 6.3 | IP 池维护 | ready |
| `Start-NetworkMonitor.ps1` | 126 | 内部 | 网络监控独立启动 | ready |
| `Generate-ConfigFromLocal.ps1` | 60 | 内部 | 本地配置生成 | ready |
| `Run-UpdateWithMonitor.ps1` | 98 | 内部 | 带监控的更新 | ready |
| `Install-IPPoolMaintenanceTask.ps1` | 76 | 内部 | IP 池维护计划任务 | ready |
| `View-IPPoolState.ps1` | 47 | 子菜单 6.4 | IP 池状态查看 | ready |
| `View-NetworkProfiles.ps1` | 41 | 子菜单 7.4 | 网络配置查看 | ready |
| `Test-IPModule.ps1` | 48 | 内部 | IP 模块测试 | ready |
| **合计** | **23 脚本** | **5,409 行** | — | — |

### 2.4 REST API (:9091)

| 端点 | 方法 | 用途 | 状态 |
|------|------|------|------|
| `/api/health` | GET | 健康检查 | ready |
| `/api/status` | GET | 完整状态 | ready |
| `/api/status/simple` | GET | 精简状态 | ready |
| `/api/start` | POST | 启动监控 | ready |
| `/api/stop` | POST | 停止服务 | ready |
| `/api/switch` | POST | 切换模式 | ready |
| `/api/restart` | POST | 重启监控 | ready |
| `/api/speedtest` | POST/GET | 测速 | ready |
| `/api/speedtest/cancel` | POST | 取消测速 | ready |
| `/api/cache/clear` | POST | 清缓存 | ready |
| `/api/cache/stats` | GET | 缓存统计 | ready |
| `/api/strategy` | GET/PUT | 网络策略 | ready |
| `/api/config` | GET/PUT | 监控配置 | ready |
| `/api/pool` | GET | 节点池概览 | partial |
| `/api/pool/nodes` | GET | 池节点列表 | partial |
| `/api/pool/scan` | POST/GET | 触发/查询扫描 | partial |
| `/api/sources` | GET/POST/DELETE | 订阅源管理 | partial |
| `/api/sources/{id}/verify` | POST | 验证源 | partial |
| `/api/ha` | GET/POST | HA 状态/切换 | planning |
| `/api/ha/history` | GET | HA 切换历史 | planning |

---

## 3. DA — 数据架构

> 详见 [arch-DA.md](arch-DA.md)

### 3.1 概念数据模型

```
┌──────────────┐     ┌───────────────────┐     ┌──────────────────┐
│  网络环境     │     │  访问策略          │     │  代理节点         │
│  NetworkEnv  │────▶│  AccessStrategy   │────▶│  ProxyNode       │
│              │     │                   │     │                  │
│ type         │     │ method            │     │ name             │
│ gateway      │     │ primary/secondary │     │ protocol (ss/v2) │
│ dns[]        │     │ fallback          │     │ server:port      │
│ isp          │     │ monitorIntervals  │     │ priority (P0-P4) │
│ networkId    │     │                   │     │ score            │
└──────────────┘     └───────────────────┘     │ lastSuccess      │
                                                │ consecutiveFails │
┌──────────────┐     ┌───────────────────┐     └──────────────────┘
│  目标站点     │     │  订阅源           │
│  Site        │────▶│  SubscriptionSrc  │────▶┌──────────────────┐
│              │     │                   │     │  运行时状态       │
│ domains[]    │     │ url               │     │  RuntimeState    │
│ ipRanges[]   │     │ mirrorUrl         │     │                  │
│ healthCheck  │     │ format            │     │ currentPlan      │
│ nodeFilter   │     │ stats             │     │ networkProfile   │
└──────────────┘     └───────────────────┘     │ history[]        │
                                                │ poolSummary      │
                                                │ haState          │
                                                └──────────────────┘
```

### 3.2 文件映射

#### 配置数据 (只读或低频写)

| 数据实体 | 文件路径 | 管理者 |
|----------|----------|--------|
| 代理栈配置 | `config/proxy-settings.json` | 人工编辑 |
| 站点定义 (5站点) | `config/sites/{id}.json` | `GitHubRuleSet` 更新 |
| 订阅源 (DPAPI 加密) | `data/subscriptions.json` | `SubscriptionParser` |
| 订阅源注册表 | `config/subscription-sources.json` | `NodePoolScanner` |
| 网络策略表 | `data/network-strategies.json` | 人工编辑 / UC3g |
| 静态 IP 池 | `data/builtin_ip_pool.json` | 随版本发布 |
| 社区源配置 | `data/community-sources.json` | 随版本发布 |
| mihomo 运行配置 | `config/proxy-config.yaml` | `ProxyConfigGenerator` 生成 |
| 路由规则集 | `config/ruleset/*.yaml` | `GitHubRuleSet` 生成 |

#### 运行时状态 (高频写)

| 数据实体 | 文件路径 | 写入者 |
|----------|----------|--------|
| 代理核心状态 | `data/proxy-state.json` | `ProxyCoreManager` / `DecisionEngine` |
| 监控服务状态 | `data/monitor-status.json` | `MonitorServer` |
| 纯文本状态 | `data/monitor-status.txt` | `MonitorServer` (CMD type 读取) |
| 进程 PID | `data/proxy-core.pid` | `ProxyCoreManager` |
| 代理激活标记 | `data/.proxy-active` | `ProxyCoreManager` |

#### 缓存数据

| 数据实体 | 文件路径 | TTL |
|----------|----------|-----|
| 节点健康评分 | `data/cache/node-health.json` | 6h |
| IP 质量评分 | `data/cache/ip-quality.json` | 6h |
| 策略历史 | `data/cache/strategy-cache.json` | ∞ (仅记录) |
| 订阅缓存 | `data/cache/subscription-cache.json` | 可配 |
| 节点池 | `data/cache/node-pool.json` | 动态 |

---

## 4. TA — 技术架构

> 详见 [arch-TA.md](arch-TA.md)

### 4.1 技术栈

| 层次 | 技术选型 | 版本约束 | 原因 |
|------|----------|----------|------|
| 用户界面 | BAT 脚本 (CMD) | Windows CMD | 零依赖，双击即用 |
| 业务逻辑 | PowerShell | 5.1 (Windows 内置) | 无需安装运行时 |
| 代理核心 | mihomo (Clash.Meta) | 可执行文件 | 开源代理引擎，支持多协议 |
| 进程通信 | HttpListener REST | .NET Framework | PowerShell 原生 HTTP |
| 数据存储 | JSON 文件 | UTF-8 | 人可读，无需数据库 |
| 配置格式 | YAML | mihomo 原生 | 代理引擎标准格式 |
| 测试框架 | Pester 5.x | — | PowerShell 标准 |
| GUI 测试 | bat-gui-test (MCP) | Node.js 桥接 | BAT 界面自动化 |

### 4.2 运行时进程模型

```
┌─────────────────────────────────────────────────┐
│  Windows OS                                      │
│                                                  │
│  ┌─────────────────┐     ┌────────────────────┐ │
│  │ CMD.exe          │     │ powershell.exe      │ │
│  │ (BAT 进程)       │     │ (MonitorServer)     │ │
│  │                  │     │  :9091 HttpListener │ │
│  │  GitHub-Hosts-   │────▶│  Watchdog Loop      │ │
│  │  Manager.bat     │HTTP │  Background Jobs    │ │
│  └─────────────────┘     └────────┬───────────┘ │
│                                   │               │
│                          ┌────────▼───────────┐  │
│                          │ mihomo.exe          │  │
│                          │ (Primary)           │  │
│                          │  :7890 mixed proxy  │  │
│                          │  :9090 REST API     │  │
│                          └────────────────────┘  │
│                                                  │
│                          ┌────────────────────┐  │
│                          │ mihomo.exe          │  │
│                          │ (Standby) ★规划中   │  │
│                          │  :7892 mixed proxy  │  │
│                          │  :9092 REST API     │  │
│                          └────────────────────┘  │
│                                                  │
│  ┌─────────────────┐                             │
│  │ Task Scheduler   │                             │
│  │ (可选定时任务)    │                             │
│  └─────────────────┘                             │
└─────────────────────────────────────────────────┘
```

| 进程 | 端口 | 启动方式 | 生命周期 | 状态 |
|------|------|----------|----------|------|
| MonitorServer (PS) | :9091 | `Start-MonitorServer.ps1` | 常驻，heartbeat 30s | ready |
| mihomo Primary | :7890, :9090 | `ProxyCoreManager` | 按需启停 | ready |
| mihomo Standby | :7892, :9092 | `NodePoolScanner` | 扫描时启动 | planning |

### 4.3 安全机制

| 机制 | 实施位置 | 说明 |
|------|----------|------|
| DPAPI 加密 | `StateManager` | 订阅 URL 加密存储 |
| 管理员提权 | `Update-GitHubHosts.ps1` | hosts 写入需管理员权限 |
| 沙盒模式 | BAT `GHM_SANDBOX` 环境变量 | 测试时阻断状态变更 |
| 原子写入 | `CacheManager.Write-JsonAtomic` | temp + rename 防数据损坏 |
| 进程互斥 | `.proxy-active` + `proxy-core.pid` | 防止多实例冲突 |
| 端口空闲检查 | `ProxyCoreManager` | 启动前 `netstat` 检查 |

### 4.4 测试体系

| 测试类型 | 框架 | 用例数 | 状态 |
|----------|------|--------|------|
| 单元测试 | Pester 5.x | ~346 | ready |
| 集成测试 | Pester 5.x | — | ready |
| GUI 自动化 | bat-gui-test + MCP | 31 | ready |

---

## 5. 目录结构

```
D:\software\github-host\
├── GitHub-Hosts-Manager.bat          # 入口 (9 主菜单)
├── modules/                          # 17 个 PowerShell 模块
│   ├── MonitorServer.psm1            #   监控服务 + REST API + Watchdog
│   ├── DecisionEngine.psm1           #   智能决策引擎
│   ├── CacheManager.psm1             #   4 层缓存管理
│   ├── ProxyCoreManager.psm1         #   mihomo 进程管理
│   ├── ProxyConfigGenerator.psm1     #   mihomo 配置生成
│   ├── SubscriptionParser.psm1       #   订阅解析
│   ├── NodePoolScanner.psm1          #   节点池扫描
│   ├── GitHubRuleSet.psm1            #   站点域名/规则
│   ├── IPSelector.psm1               #   IP 评分选择
│   ├── IPScanner.psm1                #   IP 连通性扫描
│   ├── IPFetcher.psm1                #   IP 多源获取
│   ├── IPPoolMaintainer.psm1         #   IP 池生命周期
│   ├── StateManager.psm1             #   状态持久化 + 网络指纹
│   ├── NetworkMonitor.psm1           #   WMI 网络事件监听
│   ├── CommunitySource.psm1          #   社区 IP 源
│   ├── EmergencyPool.psm1            #   紧急 IP 池
│   └── Logger.psm1                   #   日志
├── scripts/                          # 23 个操作脚本
├── config/                           # 外部配置
│   ├── proxy-settings.json           #   代理栈参数 (DNS/tuning/transport)
│   ├── subscription-sources.json     #   订阅源注册表
│   ├── sites/                        #   站点定义 (5 站点)
│   │   ├── github.json               #     GitHub (73 域名, 有 IP 范围)
│   │   ├── chatgpt.json              #     ChatGPT (31 域名, nodeFilter)
│   │   ├── claude.json               #     Claude (18 域名, nodeFilter)
│   │   ├── google.json               #     Google (28 域名, nodeFilter)
│   │   └── npmjs.json               #     npmjs (3 域名, nodeFilter)
│   ├── proxy-config.yaml             #   mihomo 运行配置 (动态生成)
│   └── ruleset/                      #   路由规则文件
│       ├── github.yaml               #     全站点域名规则
│       ├── github-ip.yaml            #     GitHub IP 规则
│       ├── custom-direct.yaml        #     自定义直连规则
│       ├── custom-proxy.yaml         #     自定义代理规则
│       └── custom-block.yaml         #     自定义屏蔽规则
├── data/                             # 运行时数据
│   ├── proxy-state.json              #   代理核心状态 (v2.2)
│   ├── monitor-status.json           #   监控服务状态
│   ├── monitor-status.txt            #   纯文本状态 (CMD type 读取)
│   ├── network-strategies.json       #   网络策略表
│   ├── builtin_ip_pool.json          #   内置 IP 池
│   ├── community-sources.json        #   社区源配置
│   ├── cache/                        #   缓存目录
│   │   ├── node-health.json          #     节点健康评分
│   │   ├── node-pool.json            #     节点池数据
│   │   ├── ip-quality.json           #     IP 质量评分
│   │   ├── strategy-cache.json       #     策略历史
│   │   └── subscription-cache.json   #     订阅缓存
│   └── log/                          #   日志目录
├── core/                             #   mihomo 可执行文件
├── tests/                            #   测试
│   ├── Unit/                         #     单元测试 (Pester)
│   ├── Integration/                  #     集成测试
│   └── GUI/                          #     GUI 自动化测试
└── docs/                             #   文档
```

---

## 词汇表

| 术语 | 含义 |
|------|------|
| mihomo | Clash.Meta 代理核心，本项目的代理引擎 |
| Watchdog | MonitorServer 内的定时任务调度器 |
| DecisionEngine | 网络环境感知 + 三路探测 + 评分决策引擎 |
| CacheManager | 4 层缓存管理器 (节点/IP/订阅/策略) |
| Hosts 模式 | 修改系统 hosts 文件，将 GitHub 域名指向最优 IP |
| Proxy 模式 | 通过 mihomo 代理访问，per-site proxy-group 覆盖全站点 |
| Direct 模式 | 不做任何处理，直连目标站点 |
| fake-ip | mihomo DNS 模式，返回假 IP，真实解析在远端 |
| rule-provider | mihomo 规则提供者机制，从外部 YAML 文件加载规则 |
| 节点池 | 从多个订阅源收集的代理节点集合，按优先级管理 |
| P0-P4 | 节点优先级，P0 最高 (低延迟+高可用)，P4 最低 (已标记死亡) |
| DPAPI | Windows Data Protection API，用于敏感数据加密 |
| TUN | 透明代理模式，捕获所有 TCP/UDP 流量 (当前默认关闭) |

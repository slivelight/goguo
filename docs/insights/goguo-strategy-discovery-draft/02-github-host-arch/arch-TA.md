# GitHub Hosts Manager — 技术架构：部署模型、时序图与状态机

> **版本**: v2.0 | **日期**: 2026-04-25
> **范围**: arch.md §4 (TA) 的细化，运行时技术视图
> **关联**: arch.md §4, arch-AA.md (模块依赖), arch-DA.md (数据模型)

---

## 1. 部署清单

### 1.1 运行时组件

| 组件 | 二进制/脚本 | 端口 | 进程类型 | 生命周期 |
|------|------------|------|----------|----------|
| mihomo Primary | `core/mihomo.exe` | :7890(mixed) :7891(SOCKS) :9090(API) | 独立进程 | ProxyCoreManager 管理 |
| mihomo Standby | `core/mihomo.exe` | :7892(mixed) :9092(API) | 独立进程 | HA 模式，热备 |
| MonitorServer | PowerShell + HttpListener | :9091(API) | 后台 PowerShell 进程 | Control-MonitorServer.ps1 管理 |
| BAT 主进程 | CMD (`GitHub-Hosts-Manager.bat`) | 无 | 前台交互进程 | 用户手动启动 |

### 1.2 配置文件清单

| 文件 | 类型 | 写入者 | 说明 |
|------|------|--------|------|
| `config/proxy-settings.json` | 静态配置 | BAT 4.T / 3.8.5 | DNS/TUN/协议权重/传输 |
| `config/proxy-config.yaml` | 生成配置 | ProxyConfigGenerator | mihomo 运行时配置 |
| `config/sites/{id}.json` | 静态配置 | GitHubRuleSet / BAT 4.9 | 5 站点域名+健康检查+过滤 |
| `config/ruleset/{site}.yaml` | 生成配置 | GitHubRuleSet | mihomo rule-provider |
| `config/ruleset/{site}-ip.yaml` | 生成配置 | GitHubRuleSet | IP-CIDR 规则 (仅 GitHub) |
| `data/network-strategies.json` | 静态配置 | UC3g | 网络策略+监控参数 |
| `data/subscriptions.json` | 静态配置 (遗留) | BAT 4.2 | 订阅源 URL + DPAPI 加密 (被 SubscriptionParser 使用) |
| `config/subscription-sources.json` | 静态配置 (新) | BAT 4.2 | 订阅源注册表 (被 NodePoolScanner 使用) |

### 1.3 状态文件清单

| 文件 | 写入频率 | 写入者 | 说明 |
|------|----------|--------|------|
| `data/proxy-state.json` | 高 (模式切换) | ProxyCoreManager / MonitorServer | PID/模式/网络/历史 |
| `data/cache/node-pool.json` | 中 (扫描周期) | NodePoolScanner / MonitorServer | 节点池+优先级+评分 |
| `data/monitor-status.json` | 高 (5s 心跳) | MonitorServer | 运行状态+Watchdog 结果 |
| `data/monitor-status.txt` | 高 (状态变更) | MonitorServer | 纯文本状态面板 (CMD type 读取) |
| `data/monitor-server.pid` | 低 (启停) | MonitorServer | 进程 PID |
| `data/proxy-core.pid` | 低 (启停) | ProxyCoreManager | mihomo PID |
| `%TEMP%/ip_pool_maintenance_state.json` | 中 | IPPoolMaintainer | 状态机持久化 |

### 1.4 系统资源依赖

| 资源 | 路径/命令 | 权限要求 | 用途 |
|------|----------|----------|------|
| hosts 文件 | `%SystemRoot%\System32\drivers\etc\hosts` | 管理员 | Hosts 模式 IP 映射 |
| 系统代理注册表 | `HKCU:\...\Internet Settings` | 用户级 | ProxyEnable/ProxyServer |
| 计划任务 | `schtasks` | 管理员 | 定时更新 hosts |
| 防火墙规则 | `netsh advfirewall` | 管理员 | mihomo 端口放行 |
| DNS 刷新 | `ipconfig /flushdns` | 用户级 | hosts/代理切换后生效 |

---

## 2. 功能模块在进程中的部署

### 2.1 进程-模块部署图

```mermaid
graph TB
    subgraph BAT_Process["BAT 主进程 (CMD)"]
        direction TB
        UI["F01 BAT 菜单<br/>交互式选择"]
    end

    subgraph PS_Scripts["PowerShell 子进程 (按需)"]
        direction TB
        S1["Invoke-IntelligentMode.ps1"]
        S2["Update-GitHubHosts.ps1"]
        S3["Invoke-ProxyMode.ps1"]
        S4["Control-MonitorServer.ps1"]
    end

    subgraph Monitor_Process["MonitorServer 进程 (PowerShell)"]
        direction TB
        MS["MonitorServer.psm1<br/>HttpListener :9091"]
        WD["Watchdog 循环<br/>Network/GitHub/Node/IP"]
        RC["AutoRecovery<br/>5阶段升级链"]
        DE["DecisionEngine<br/>(模块内实例)"]
        CM["CacheManager<br/>4层缓存"]
        NPS["NodePoolScanner<br/>节点池扫描"]
        PCG["ProxyConfigGenerator<br/>配置生成"]
    end

    subgraph Mihomo_Process["mihomo 进程 (独立)"]
        direction TB
        CORE["Clash Meta 内核"]
        API["REST API :9090"]
        PROXY["Mixed Proxy :7890"]
    end

    subgraph Mihomo_Standby["mihomo Standby (HA)"]
        direction TB
        S_API["API :9092"]
        S_PROXY["Proxy :7892"]
    end

    UI -->|Start-Process powershell| S1
    UI -->|Start-Process powershell| S2
    UI -->|Start-Process powershell| S3
    UI -->|Start-Process powershell| S4
    S1 -->|Import-Module| DE
    S2 -->|Import-Module| DE
    S3 -->|Import-Module| DE
    S4 -->|Start-Process -Hidden| Monitor_Process
    Monitor_Process -->|Start-Process mihomo| Mihomo_Process
    Monitor_Process -.->|HA: Start-Process| Mihomo_Standby
    MS --> WD
    WD --> RC
    RC --> DE
```

### 2.2 模块在进程中的分布矩阵

| 模块 | BAT 进程 | PS 脚本子进程 | MonitorServer 进程 | mihomo 进程 |
|------|----------|--------------|-------------------|-------------|
| DecisionEngine | — | Import | Import | — |
| ProxyCoreManager | — | Import | Import | — |
| CacheManager | — | — | Import | — |
| NodePoolScanner | — | — | Import | — |
| ProxyConfigGenerator | — | Import | Import | — |
| SubscriptionParser | — | Import | Import | — |
| GitHubRuleSet | — | Import | Import | — |
| IPFetcher | — | Import | Import | — |
| IPSelector | — | Import | — | — |
| IPScanner | — | Import | — | — |
| IPPoolMaintainer | — | Import | — | — |
| StateManager | — | Import | Import | — |
| NetworkMonitor | — | Import | Import | — |
| Logger | — | Import | Import | — |
| MonitorServer | — | — | **宿主** | — |

**关键说明**:
- MonitorServer 是模块聚合度最高的进程，Import 了 14 个模块
- PS 脚本子进程按需启动，执行完毕退出，无持久状态
- mihomo 是独立二进制进程，仅通过 REST API (`:9090`) 被外部操控
- 同一模块在不同进程中各自 Import，内存状态独立（通过 JSON 文件共享状态）

---

## 3. 数据模型在配置/存储中的映射

### 3.1 数据流向图

```mermaid
graph LR
    subgraph Static_Config["静态配置 (人工维护)"]
        PS["proxy-settings.json<br/>DNS/TUN/协议权重"]
        SC["sites/*.json<br/>5站点域名+过滤"]
        NS["network-strategies.json<br/>网络策略表"]
        SUB["subscriptions.json<br/>订阅源+DPAPI"]
    end

    subgraph Generated_Config["生成配置 (代码产出)"]
        PCY["proxy-config.yaml<br/>mihomo运行时配置"]
        RS["ruleset/*.yaml<br/>per-site规则"]
    end

    subgraph Runtime_State["运行时状态 (代码读写)"]
        PST["proxy-state.json<br/>PID/模式/历史"]
        NP["cache/node-pool.json<br/>节点池+优先级"]
        MS["monitor-status.json<br/>心跳+Watchdog"]
        MT["monitor-status.txt<br/>纯文本面板"]
    end

    subgraph System["系统资源"]
        HOSTS["hosts 文件"]
        REG["注册表 (系统代理)"]
    end

    PS -->|ProxyConfigGenerator读取| PCY
    SC -->|GitHubRuleSet读取| RS
    SC -->|ProxyConfigGenerator读取| PCY
    NS -->|DecisionEngine读取| PST
    SUB -->|SubscriptionParser读取| NP

    PCY -->|mihomo -f 加载| M[("mihomo 进程")]
    RS -->|rule-provider 加载| M

    PST -->|ProxyCoreManager写入| M
    NP -->|NodePoolScanner写入| PCY
    MS -->|MonitorServer写入| MT
```

### 3.2 配置到运行时映射表

| 概念模型实体 | 配置文件 | 运行时载体 | 读取者 | 写入者 |
|-------------|---------|-----------|--------|--------|
| NetworkEnvironment | — | proxy-state.json.networkProfile | DecisionEngine | MonitorServer |
| AccessPlan | network-strategies.json | proxy-state.json.currentPlan | MonitorServer | DecisionEngine |
| AccessHistory | — | proxy-state.json.history[] | DecisionEngine | MonitorServer |
| ProxyNode | subscriptions.json → 解析 | cache/node-pool.json.nodes[] | NodePoolScanner | NodePoolScanner |
| SiteConfig | config/sites/{id}.json | 内存对象 | ProxyConfigGenerator | GitHubRuleSet / BAT 4.9 |
| SubscriptionSource | subscriptions.json | 内存对象 | SubscriptionParser | BAT 4.2 |
| IPCacheEntry | — | cache/ip-*.json | IPSelector | IPFetcher / IPScanner |
| StrategyCacheEntry | — | cache/strategy-*.json | DecisionEngine | MonitorServer |
| NetworkStrategy | network-strategies.json | 内存对象 | DecisionEngine | UC3g |
| ProxySettings | proxy-settings.json | 内存对象 | ProxyConfigGenerator | BAT 4.T |

---

## 4. 主要场景运行时序图

### 4.1 UC1 智能访问（正常流程）

```mermaid
sequenceDiagram
    actor User
    participant BAT as BAT 主进程
    participant PS as PowerShell 子进程
    participant DE as DecisionEngine
    participant SM as StateManager
    participant PCM as ProxyCoreManager
    participant MI as mihomo API

    User->>BAT: 按键 "1"
    BAT->>PS: Start-Process powershell<br/>-File Invoke-IntelligentMode.ps1
    PS->>DE: Start-IntelligentAccess

    Note over DE: Step 1: 网络检测
    DE->>SM: Get-NetworkFingerprint
    SM-->>DE: {type:Home, gateway, isp}

    Note over DE: Step 2: 三路并行探测
    par Direct 探测
        DE->>DE: Test-DirectTcpConnection<br/>github.com:443
    and Hosts 探测
        DE->>SM: Get-TopIPs(domain, count=5)
        SM-->>DE: [IP列表]
        DE->>DE: Test-DirectTcpConnection<br/>每个 IP
    and Proxy 探测
        DE->>PCM: Get-ProxyCoreStatus
        PCM-->>DE: {running:true, pid:1234}
        DE->>MI: GET /proxies/GitHub/delay
        MI-->>DE: {delay:245}
    end

    Note over DE: Step 3: 评分决策
    DE->>DE: Resolve-ConnectionPlan
    Note over DE: Direct: +10 +100(可达) +30(&lt;200ms)<br/>Hosts: +5 +100 +50(&lt;100ms)<br/>Proxy: +10(Home加成) +100 +10

    alt 选中 Proxy 模式
        DE->>PCM: Remove-GitHubHostsEntries
        DE->>PCM: Invoke-ProxyMethod
        PCM->>PCM: Set-SystemProxy(:7890)
        PCM->>DE: {success:true}
    else 选中 Hosts 模式
        DE->>DE: Invoke-HostsMethod
        DE->>SM: Update-GitHubHosts(QuickMode)
        SM-->>DE: {success:true}
    else 选中 Direct 模式
        DE->>PCM: Restore-SystemProxy
    end

    Note over DE: Step 4: 状态持久化
    DE->>SM: Save-DecisionState
    SM->>SM: Write proxy-state.json

    PS-->>BAT: Exit code
    BAT->>User: 显示结果面板
```

### 4.2 UC2 Hosts 更新（正常流程）

```mermaid
sequenceDiagram
    actor User
    participant BAT as BAT 主进程
    participant PS as Update-GitHubHosts.ps1
    participant IF as IPFetcher
    participant IS as IPSelector
    participant SM as StateManager
    participant SYS as 系统资源

    User->>BAT: 按键 "2"
    BAT->>BAT: 管理员权限检查
    BAT->>PS: Start-Process (管理员)

    PS->>SM: Initialize-IPPool
    PS->>IF: Get-AllGitHubIPs
    IF->>IF: 内置池 + 社区源 + DNS
    IF-->>PS: {domain: [IP列表]}

    Note over PS: 逐域名选最优 IP
    loop 每个 GitHub 域名
        PS->>IS: Select-BestIPsQuick
        IS->>SM: Get-TopIPs(domain, count=5)
        SM-->>IS: [候选 IP]

        loop 每个 候选 IP
            IS->>IS: TCP 443 连接测试
            IS->>IS: TLS 握手 + 证书验证
            IS->>SM: Update-IPPoolEntry
        end

        IS-->>PS: optimalIP
    end

    Note over PS: 写入-验证-重试循环 (最多 3 次)
    loop 尝试 1..3
        PS->>SYS: Backup-HostsFile
        PS->>SYS: Update-HostsFile (写入 hosts)
        PS->>SYS: ipconfig /flushdns
        PS->>PS: Test-GitHubAccessible<br/>HTTP HEAD github.com
        alt 验证成功
            PS-->>BAT: Exit 0
        else 验证失败
            PS->>PS: 收集坏 IP，排除后重试
        end
    end

    BAT->>User: 显示更新结果
```

### 4.3 UC4.1 启动代理模式（正常流程）

```mermaid
sequenceDiagram
    actor User
    participant BAT as BAT 主进程
    participant PS as Invoke-ProxyMode.ps1
    participant PCM as ProxyCoreManager
    participant SP as SubscriptionParser
    participant PCG as ProxyConfigGenerator
    participant GRS as GitHubRuleSet
    participant MI as mihomo 进程

    User->>BAT: 按键 "4" → "1"
    BAT->>PS: Start-Process powershell

    PS->>PCM: Remove-GitHubHostsEntries
    PS->>PCM: Get-ProxyCoreStatus
    alt 已运行
        PS->>PCM: Stop-ProxyCore
    end

    Note over PS: 配置生成
    PS->>SP: Get-SubscriptionConfig
    SP->>SP: DPAPI 解密 URL
    SP-->>PS: [{url, format}]

    loop 每个订阅源
        PS->>SP: Get-ProxyNodesFromSubscription
        SP-->>PS: [ProxyNode 列表]
    end

    PS->>PCG: New-ClashMetaConfig(allNodes)
    PCG->>GRS: Get-AllEnabledSites
    GRS-->>PCG: [5个 SiteConfig]
    PCG->>PCG: 生成 proxies + proxy-groups<br/>+ dns + rules + tun
    PCG-->>PS: config YAML 路径

    Note over PS: 配置验证
    PS->>MI: mihomo -t -f config.yaml
    MI-->>PS: 验证结果

    Note over PS: 启动核心
    PS->>PCM: Start-ProxyCore(config, core)
    PCM->>MI: Start-Process mihomo.exe<br/>-f config -d coreDir<br/>-WindowStyle Hidden
    MI-->>PCM: PID
    PCM->>PCM: 写入 proxy-core.pid<br/>防火墙规则

    Note over PS: 就绪探测 (≤10s)
    loop 每 1s × 10
        PS->>MI: GET http://127.0.0.1:9090/version
        MI-->>PS: {meta, version}
    end

    PS->>PCM: Set-SystemProxy(:7890)
    PCM->>PCM: 注册表 ProxyEnable=1<br/>ProxyServer=127.0.0.1:7890
    PS->>PS: ipconfig /flushdns
    PS->>PS: 写入 proxy-state.json

    PS-->>BAT: Exit
    BAT->>User: 显示代理状态
```

### 4.4 UC3 MonitorServer 启动与运行

```mermaid
sequenceDiagram
    actor User
    participant BAT as BAT 主进程
    participant CTL as Control-MonitorServer.ps1
    participant START as Start-MonitorServer.ps1
    participant MS as MonitorServer.psm1
    participant HL as HttpListener :9091
    participant WD as Watchdog 循环

    User->>BAT: 按键 "3" → "8"
    BAT->>CTL: Start-ServerProcess
    CTL->>CTL: 检查 /api/health (是否已运行)
    CTL->>CTL: 清理旧 PID + 端口占用
    CTL->>START: Start-Process powershell<br/>-WindowStyle Hidden
    START->>START: Import 14 个模块
    START->>MS: Start-MonitorServer

    Note over MS: 初始化阶段
    MS->>MS: Write-PidFile
    MS->>MS: Initialize-Cache (4层缓存)
    MS->>MS: Load-CurrentPlan
    MS->>MS: Get-NetworkEnvironment
    MS->>HL: HttpListener.Start()<br/>绑定 127.0.0.1:9091
    MS->>MS: Register-NetworkEvent (WMI)
    MS->>MS: Write-StatusTextFile

    Note over MS: 主循环
    loop while Running
        MS->>HL: GetContextAsync().Wait(100ms)
        alt 有 HTTP 请求
            MS->>MS: Invoke-ApiHandler(context)
        end

        opt 首次检查
            MS->>WD: Invoke-NetworkCheck
        end

        opt 每 30s 网络检查
            MS->>WD: Invoke-NetworkCheck
            MS->>MS: Write-StatusTextFile
        end

        opt 每 15s GitHub 探测
            MS->>WD: Invoke-GitHubProbe
            MS->>MS: Write-StatusTextFile
        end

        opt 每 300s 节点健康 (Proxy 模式)
            MS->>WD: Invoke-NodeHealthCheck
        end

        opt 每 60s IP 质量 (Hosts 模式)
            MS->>WD: Invoke-IPQualityCheck
        end

        opt 每 30s 心跳
            MS->>MS: Write-Heartbeat
            MS->>MS: Save-CacheToDisk
        end

        MS->>MS: Sleep 100ms
    end

    CTL->>HL: GET /api/health (轮询就绪)
    HL-->>CTL: 200 OK
    CTL-->>BAT: 显示 "服务已启动"
```

### 4.5 自动恢复（异常流程 — GitHub 不可达）

```mermaid
sequenceDiagram
    participant WD as Watchdog
    participant RC as AutoRecovery
    participant HA as HAFailover
    participant DE as DecisionEngine
    participant PCM as ProxyCoreManager
    participant LOG as RecoveryLog

    Note over WD: GitHub 探测连续失败 ≥ 3 次
    WD->>RC: Invoke-AutoRecovery(Trigger)

    rect rgb(255, 230, 230)
        Note over RC: Phase 0: HA 故障切换
        RC->>HA: Invoke-HAFailover
        HA->>HA: 检查 Standby 健康状态
        alt Standby 健康
            HA->>PCM: Switch-ActiveCore
            PCM-->>HA: 切换成功
            HA->>LOG: Write-RecoveryLog(HA Failover)
            RC-->>WD: 恢复成功，退出
        else 无 HA 或 Standby 不健康
            Note over RC: 进入 Phase 1
        end
    end

    rect rgb(255, 245, 230)
        Note over RC: Phase 1: 本地评估
        RC->>RC: Get-NetworkType
        RC->>RC: Get-TopCandidates(count=3)
        Note over RC: Phase 2: 缓存恢复
        loop 遍历缓存候选
            RC->>RC: Test-TcpConnection(candidate)
            alt 候选可达
                RC->>RC: Apply-RecoveryCandidate
                RC->>LOG: Write-RecoveryLog(Cache)
                RC-->>WD: 恢复成功，退出
            end
        end
    end

    rect rgb(255, 255, 230)
        Note over RC: Phase 3: 全量决策
        RC->>RC: Test-HttpsReachable(github.com)
        alt HTTPS 失败
            RC->>LOG: 记录 "疑似 SNI 阻断"
        end
        RC->>DE: Get-NetworkEnvironment
        RC->>DE: Test-ConnectionAvailability
        RC->>DE: Resolve-ConnectionPlan
        DE-->>RC: {method, confidence}

        alt 方案可行
            RC->>PCM: 执行选定模式
            RC->>LOG: Write-RecoveryLog(Full)
            RC-->>WD: 恢复成功，退出
        end
    end

    rect rgb(230, 230, 255)
        Note over RC: Phase 4: 退避
        RC->>RC: BackoffState.Count++
        RC->>RC: 计算退避间隔<br/>[30, 60, 120, 300, 600]s
        RC->>RC: 延迟下次探测

        opt 连续失败 ≥ 3 次
            Note over RC: Phase 5: 告警
            RC->>LOG: Write-AlertState
            LOG->>LOG: 记录 "ALERT: Recovery exhausted"
        end
    end
```

### 4.6 网络变更异常流程

```mermaid
sequenceDiagram
    participant NET as Windows 网络
    participant WMI as WMI 事件
    participant NM as NetworkMonitor
    participant MS as MonitorServer
    participant DE as DecisionEngine
    participant SM as StateManager

    NET->>WMI: 网关/适配器变更
    WMI->>NM: 触发 Register-WmiEvent Action
    NM->>NM: 去抖动检查 (5s 内不重复)
    NM->>MS: OnChangeHandler 回调

    MS->>SM: Get-DefaultGateway
    SM-->>MS: 新网关 IP
    MS->>DE: Get-NetworkType(newGateway)
    DE-->>MS: {type: "Mobile"} (vs 旧 "Home")

    MS->>MS: 更新 networkProfile

    Note over MS: 策略重评估
    MS->>DE: Test-ConnectionAvailability
    DE-->>MS: {direct:unreachable, hosts:unreachable, proxy:ok}

    MS->>DE: Resolve-ConnectionPlan(newEnv, newAvail)
    DE-->>MS: {method:"Proxy", confidence:85}

    MS->>MS: 切换到 Proxy 模式
    MS->>MS: Write-StatusTextFile
```

---

## 5. 主要状态机

### 5.1 IPPoolMaintainer 状态机 (F04-04)

```mermaid
stateDiagram-v2
    [*] --> Healthy : 初始化 (≥5 TierA, ≥15 总)

    Healthy --> Warning : TierA < 5
    Warning --> Healthy : TierA ≥ 5 && 总数 ≥ 15<br/>(需连续 2 次验证)
    Warning --> Emergency : TierA = 0 && 总数 < 3
    Emergency --> Recovery : 获得 ≥1 个新 IP
    Recovery --> Healthy : TierA ≥ 5 && 总数 ≥ 15<br/>(需连续 2 次成功)
    Recovery --> Emergency : 持续 >10min<br/>且 TierA = 0<br/>(需连续 2 次失败)
    Emergency --> [*] : 服务停止
```

**检查间隔**:

| 状态 | 间隔 | 说明 |
|------|------|------|
| Healthy | 300s (5min) | 日常维护，轻量扫描 |
| Warning | 120s (2min) | 增强扫描，补充 IP |
| Emergency | 60s (1min) | 紧急获取，启用社区源+紧急池 |
| Recovery | 30s | 高频验证，确保恢复稳定 |

**状态持久化**: `%TEMP%\ip_pool_maintenance_state.json`
- TTL: 24 小时（过期从 Healthy 重新开始）
- 字段: CurrentState, StateEnterTime, FailureCount, SuccessCount, LastStateData

**代码定位**: `modules/IPPoolMaintainer.psm1:232-284` (状态转换逻辑)

### 5.2 NodePool 扫描阶段状态机 (F05-05)

```mermaid
stateDiagram-v2
    [*] --> bootstrap : MonitorServer 启动

    bootstrap --> steady : P0+P1 ≥ 5
    bootstrap --> emergency : 全部节点死亡

    steady --> degraded : P0+P1 < 3
    degraded --> steady : P0+P1 ≥ 5
    degraded --> emergency : totalAlive = 0

    emergency --> bootstrap : P0+P1 ≥ 1
```

**扫描策略**:

| 阶段 | 全量扫描间隔 | 快检间隔 | 说明 |
|------|-------------|---------|------|
| bootstrap | 首次立即 | — | 启动时一次性扫描 |
| steady | 1800s (30min) | 600s (10min) | 定期全量 + 周期快检 |
| degraded | 600s (10min) | 300s (5min) | 加速扫描频率 |
| emergency | 60s | 30s | 最高频率，紧急发现节点 |

**代码定位**: `modules/NodePoolScanner.psm1:773-801` (阶段判断逻辑)

### 5.3 Proxy 模式状态机 (F05-01)

```mermaid
stateDiagram-v2
    [*] --> stopped : 初始状态

    stopped --> starting : Start-ProxyCore
    starting --> running : mihomo PID 有效<br/>API :9090 响应
    starting --> stopped : 启动失败/超时

    running --> reloading : Invoke-ProxyConfigReload
    reloading --> running : PUT /configs 成功
    reloading --> running : 失败但仍存活

    running --> stopping : Stop-ProxyCore
    stopping --> stopped : 进程退出

    running --> stopped : mihomo 崩溃 (非正常退出)
```

**状态判断逻辑** (`Get-ProxyCoreStatus`):

| 检查项 | 方法 | 说明 |
|--------|------|------|
| PID 存活 | `Get-Process -Id $pid -EA SilentlyContinue` | 进程级检查 |
| API 响应 | `GET http://127.0.0.1:9090/version` | 应用级检查 |
| 配置路径 | proxy-state.json.configPath | 配置文件有效性 |
| 一致性 | pid > 0 ↔ running = true | 交叉验证 |

**代码定位**: `modules/ProxyCoreManager.psm1:69-95` (停止), `:137-204` (启动)

### 5.4 自动恢复状态机 (F03-04)

```mermaid
stateDiagram-v2
    [*] --> Monitoring : Watchdog 正常

    Monitoring --> ProbeFailing : 探测失败
    ProbeFailing --> Monitoring : 探测恢复 (≤2次失败)
    ProbeFailing --> Recovering : 连续失败 ≥ 3次

    state Recovering {
        [*] --> Phase0_HA
        Phase0_HA --> Phase1_Evaluate : 无 HA 或失败
        Phase0_HA --> Recovered : HA 切换成功

        Phase1_Evaluate --> Phase2_Cache : 有缓存候选
        Phase2_Cache --> Phase3_Full : 缓存全部失败
        Phase2_Cache --> Recovered : 缓存候选可用

        Phase3_Full --> Phase4_Backoff : 全量决策也失败
        Phase3_Full --> Recovered : 决策成功执行

        Phase4_Backoff --> Monitoring : 退避等待后重试
    }

    Recovered --> Monitoring : 确认可达
    Phase4_Backoff --> Alert : 连续退避 ≥ 3次
    Alert --> Monitoring : 用户手动干预
```

**退避间隔表**:

| 尝试次数 | 退避间隔 | 累计时间 |
|----------|---------|---------|
| 1 | 30s | 30s |
| 2 | 60s | 90s |
| 3 | 120s | 210s |
| 4 | 300s | 510s |
| 5+ | 600s | 600s/次 |

**代码定位**: `modules/MonitorServer.psm1:1968-2056` (Invoke-AutoRecovery)

---

## 6. 进程通信机制

### 6.1 通信矩阵

| 发起方 | 接收方 | 协议 | 端点 | 用途 |
|--------|--------|------|------|------|
| MonitorServer | mihomo | HTTP | `:9090/proxies` | 节点延迟测试 |
| MonitorServer | mihomo | HTTP | `:9090/configs` | 配置热重载 |
| MonitorServer | mihomo | HTTP | `:9090/providers` | 规则/代理提供者管理 |
| Control-MonitorServer | MonitorServer | HTTP | `:9091/api/health` | 健康检查 |
| BAT / scripts | MonitorServer | HTTP | `:9091/api/*` | 17+ REST API |
| MonitorServer | mihomo Standby | HTTP | `:9092/*` | HA 管理 |
| scripts | 系统注册表 | Win32 API | `HKCU:\...\Internet Settings` | 系统代理设置 |
| scripts | hosts 文件 | 文件 I/O | `%SystemRoot%\...\hosts` | IP 映射 |
| MonitorServer | 文件系统 | 文件 I/O | `data/*.json` | 状态持久化 |
| NetworkMonitor | WMI | 事件订阅 | `Win32_NetworkAdapter` | 网络变更检测 |

### 6.2 同步 vs 异步

| 操作 | 模式 | 超时 | 说明 |
|------|------|------|------|
| BAT → PS 脚本 | 同步 (Start-Process) | 无 (用户等待) | 脚本执行完 BAT 才继续 |
| MonitorServer → mihomo API | 同步 (Invoke-WebRequest) | 5-10s | 短超时，避免阻塞主循环 |
| MonitorServer HttpListener | 异步 (GetContextAsync) | 100ms | 非阻塞等待，配合主循环 |
| TCP 连接测试 | 异步 (BeginConnect) | 3-5s | APM 模式，并行探测 |
| WMI 网络事件 | 异步 (Register-WmiEvent) | 持续 | 事件驱动 + 5s 去抖 |
| Watchdog 定时检查 | 伪异步 (主循环计时) | 100ms 轮询 | 单线程内时间片轮转 |

---

## 7. 错误处理与容错机制

### 7.1 原子写入保障

```
写入流程:
1. $tempPath = "$targetPath.tmp"
2. ConvertTo-Json -Depth 10 | Out-File $tempPath
3. Move-Item $tempPath $targetPath -Force
```

**保证**: 即使写入过程中断电，目标文件要么是旧的完整版本，要么是新的完整版本，不会出现半写状态。

### 7.2 进程崩溃恢复

| 场景 | 检测机制 | 恢复策略 |
|------|---------|---------|
| mihomo 崩溃 | NodeHealthCheck (300s) 探测 API 失败 | AutoRecovery Phase 0-3 |
| MonitorServer 崩溃 | BAT 菜单检查 /api/health | 用户手动重启 (BAT 3.8) |
| PowerShell 脚本异常 | try/catch + $ErrorActionPreference | 日志记录 + 退出码 |
| hosts 文件损坏 | Test-GitHubAccessiblePostUpdate | 回滚 Backup-HostsFile |
| JSON 解析失败 | ConvertFrom-Json try/catch | Warning + 内嵌默认值 |

### 7.3 降级策略

```
完整功能链:
  TUN(全协议) → 系统代理(HTTP/S) → Hosts(仅 GitHub 域名) → Direct(直连)

降级触发条件:
  - 订阅源失效 → Proxy 模式不可用 → 降级到 Hosts
  - IP 池耗尽 → Hosts 模式不可用 → 降级到 Direct
  - 网络本身可达 → Direct 模式直接工作
  - 网络被墙 → Direct 失败 → 告警用户
```

---

## 8. 性能约束

| 约束 | 值 | 来源 |
|------|-----|------|
| 节点池上限 | 200 节点 | NodePoolScanner.maxPoolSize |
| 历史记录上限 | 100 条 | proxy-state.json history[] |
| 主循环最小间隔 | 100ms | MonitorServer Sleep |
| hosts 备份 | 1 份 (hosts.backup) | Update-GitHubHosts |
| 缓存文件数 | ≤ 10 (各类 cache/*.json) | CacheManager |
| TCP 探测超时 | 3-5s (可配) | IPScanner / DecisionEngine |
| mihomo 启动就绪超时 | 10s | Invoke-ProxyMode |
| MonitorServer 启动就绪超时 | 15s | Control-MonitorServer |
| 退避上限 | 600s | AutoRecovery |

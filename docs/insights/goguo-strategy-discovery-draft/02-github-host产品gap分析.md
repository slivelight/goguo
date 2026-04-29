# 能力 Gap 分析

## 背景

基于三方面输入进行能力差距分析：
1. **知识库文章**（科学上网完全指南，22章+附录）—— 行业标准技术栈与最佳实践
2. **Clash Verge Rev**（GitHub 113k stars）—— 2025年代理客户端标杆，Tauri 2 + Rust
3. **项目实际 codebase**（18 模块，约 17,800 行 PowerShell 5.1）—— 当前实现状态

**对比定位**：本项目不是通用代理客户端，而是**中国大陆 GitHub 访问专用的自动化网络可达性工具**。因此分析聚焦于"代理客户端核心能力"和"项目独特定位"两个维度。

---

## 汇总差距矩阵

| # | 维度 | 项目能力 | 行业基准 | 差距等级 | 说明 |
|---|------|---------|---------|---------|------|
| 1 | 架构与技术栈 | Basic | Excellent | **Critical** | PS5.1 脚本 vs Rust+Tauri 原生 |
| 2 | UI/UX | None | Excellent | **Critical** | BAT 控制台 vs 现代 Web GUI |
| 3 | 协议支持 | Good | Excellent | Low | 已覆盖 5 大协议，缺 TUIC/WireGuard |
| 4 | 配置管理 | Basic | Excellent | Medium | JSON 外部配置 vs Merge/Script/WebDav |
| 5 | 规则系统 | Good | Excellent | Medium | DOMAIN-SUFFIX 单类型 vs 全规则类型+Provider |
| 6 | 订阅管理 | **Strong** | Good | — | **项目领先**：多源+质量追踪+DPAPI+镜像 |
| 7 | 节点管理 | **Strong** | Good | — | **项目领先**：评分/绑定/HA双实例/淘汰 |
| 8 | 系统集成 | Good | Excellent | Medium | 缺 Proxy Guard，TUN 已有 |
| 9 | 监控诊断 | Good | Good | Low | MonitorServer REST API 丰富，缺实时流量可视化 |
| 10 | 跨平台 | None | Excellent | **Critical*** | Windows Only（*仅 Phase 5 目标时为 Critical） |
| 11 | 安全隐私 | Good | Good | Low | DNS 污染检测是中国网络特有优势 |
| 12 | 自动更新维护 | Good | Basic | Medium | IP 池/缓存维护领先，缺自更新 |

**结论**：项目在**代理编排智能层**（DecisionEngine、IP 管理、节点池生命周期、HA）显著领先；在**用户体验交付层**（GUI、性能、跨平台）存在 Critical 差距。

---

## 分维度详细分析

### 1. 架构与技术栈 [Critical]

| 属性 | 本项目 | Clash Verge Rev |
|------|--------|----------------|
| 语言 | PowerShell 5.1 (~17,800行) | Rust (33%) + TypeScript (59%) |
| 平台 | Windows Only | Win/Linux/macOS |
| 运行时 | CMD→powershell.exe 子进程 | 原生二进制 + WebView |
| 部署 | 文件夹复制 | 安装包 + 自更新 |
| 内核 | mihomo 嵌入 | mihomo + Alpha 切换 |

**根因**：BAT 菜单 1 执行需 40-70 秒，源于每次操作重新加载 17,800 行模块。PowerShell 5.1 意味着：仅 Windows、子进程启动开销、无法提供真正的 GUI、所有模块冷启动延迟。

**改进方向**：MonitorServer 已持久运行（`:9091`），菜单 1 可改为轻量 API 调用而非完整模块重导入。长期需架构解耦（Phase 5 已设计）。

### 2. UI/UX [Critical]

| 属性 | 本项目 | Clash Verge Rev |
|------|--------|----------------|
| 界面 | 9 菜单 BAT 控制台 (1,006行) | Tauri + React/Vue Web GUI |
| 状态显示 | Write-Host 文本面板 | 实时仪表盘/图表/动画 |
| 主题 | 无（CMD 默认） | Dark/Light + CSS Injection |
| 交互 | 阻塞式 `set /p` | 响应式 Web UI |
| 国际化 | 中英混合 | 多语言 |

**改进方向**：MonitorServer 已有约 25 个 REST API 端点（status/health/speedtest/pool/HA/sources/strategy/config），可直接支撑 Web 前端。嵌入式 HTTP 静态文件服务是务实的第一步。

### 3. 协议支持 [Low]

| 协议 | 本项目 | CVR | 备注 |
|------|--------|-----|------|
| SS | 解析+生成 | mihomo | 已覆盖 |
| VMess | 解析+生成 | mihomo | 已覆盖 |
| VLESS+Reality | 解析(pbk/sid)+生成 | mihomo | 已覆盖 |
| Trojan | 解析+生成 | mihomo | 已覆盖 |
| Hysteria2 | 解析+生成 | mihomo | 已覆盖 |
| TUIC | 未实现 | mihomo | 缺失 |
| WireGuard | 未实现 | mihomo | 缺失 |

**评估**：知识库指出 2025 推荐 VLESS+Reality + Hysteria2，项目已覆盖。TUIC/WireGuard 为次要差距。

### 4. 配置管理 [Medium]

| 属性 | 本项目 | CVR |
|------|--------|-----|
| 配置格式 | JSON 外部配置 | YAML 原生 |
| Merge/Script | 无 | 支持 |
| 可视化编辑 | 无 | 内联编辑器 |
| 备份/同步 | 无 | WebDav |
| 配置验证 | `Test-ClashMetaConfig` | mihomo -t + Alpha |

**亮点**：项目已实现三维配置解耦（sites × subscriptions × proxy-settings），对目标用户比原始 YAML 更可维护。

**改进方向**：添加配置导入/导出、Merge/Script 增强（允许 JS 覆盖层修改生成的 YAML）。

### 5. 规则系统 [Medium]

| 属性 | 本项目 | CVR |
|------|--------|-----|
| 规则生成 | 从 sites/*.json 自动生成 | 用户可编辑 |
| 规则类型 | DOMAIN-SUFFIX 单一 | DOMAIN/IP-CIDR/GEOIP/RULE-SET 全类型 |
| Rule Provider | 无 | Loyalsoldier 等社区规则集 |
| 自定义规则 | custom-direct/proxy/block.yaml | 可视化编辑器 |

**评估**：知识库强调分流规则是核心功能（DOMAIN-SUFFIX > IP-CIDR > GEOIP > RULE-SET），当前仅支持 DOMAIN-SUFFIX。sites/*.json 中的 `ipRanges` 字段已定义但未用于规则生成。

**改进方向**：从 ipRanges 生成 IP-CIDR 规则、添加 rule-provider 支持（接入 Loyalsoldier/ACL4SSR 社区规则集）。

### 6. 订阅管理 [项目领先]

| 属性 | 本项目 | CVR |
|------|--------|-----|
| 多源管理 | 有（来源统计/质量追踪） | 基本列表 |
| 镜像URL | 有（mirrorUrl 字段） | 无 |
| DPAPI 加密 | 有 | 引用存储 |
| 来源质量追踪 | 有（成功/失败计数） | 无 |
| 订阅转换 | 无 | 内置/外部服务 |

**唯一差距**：缺 subconverter 订阅转换服务集成。

### 7. 节点管理 [项目领先]

| 属性 | 本项目 | CVR |
|------|--------|-----|
| 节点评分 | 加权（延迟+协议+可靠性） | 基本延迟排序 |
| 协议权重 | 有（hysteria2:0.70 ~ ss:1.00） | 无 |
| 站点-节点绑定 | 有（高质量节点→AI站点） | 无 |
| HA 双实例 | 有（主/备 mihomo） | 无 |
| 节点池生命周期 | 完整（扫描/评分/淘汰） | 基本列表 |
| 负载均衡 | url-test only | select/url-test/fallback/load-balance |

**改进方向**：在 ProxyConfigGenerator 中添加 mihomo `load-balance` 组类型。

### 8. 系统集成 [Medium]

| 属性 | 本项目 | CVR |
|------|--------|-----|
| 系统代理 | 注册表 HKCU | 系统代理 + Proxy Guard |
| TUN 模式 | 有（RFC-009 WinTun） | 有（gvisor/mixed） |
| Proxy Guard | **无** | 有（防止代理泄漏） |
| 端口防护 | 有（R5 netstat 检查） | 有 |

**关键差距**：无 Proxy Guard。mihomo 崩溃时系统代理指向死服务，用户失去网络。

### 9. 监控诊断 [Low]

MonitorServer (`:9091`, 2,905行) 提供约 25 个 REST API 端点，3-way probing（Direct/Hosts/Proxy）为独特优势。差距仅在实时流量可视化（需前端）。

### 10. 跨平台 [Critical*]

50+ 硬编码 GitHub 域名 + 大量 Windows API（WMI、DPAPI、Registry、schtasks、netstat、ipconfig）。

**仅在 Phase 5（开源通用化）目标下为 Critical**。若保持 Windows 专用 GitHub 工具定位，可接受。

### 11. 安全隐私 [Low]

| 属性 | 本项目 | 行业标准 |
|------|--------|---------|
| DNS 污染检测 | **有**（13 CIDR GFW 注入模式 + 交叉组验证） | 无（通用客户端不做） |
| DNS 防泄漏 | fake-ip + DoH 回退 | fake-ip |
| 凭证加密 | DPAPI | 存储配置 |
| IPv6 泄漏 | 配置禁用 | 可配置 |

---

## 项目独有优势（无竞品对标）

以下能力在 Clash Verge Rev 及任何通用代理客户端中**没有同类产品**：

| 功能 | 模块 | 核心价值 |
|------|------|---------|
| DecisionEngine | `DecisionEngine.psm1` (933行) | 3-way 网络探测 + 评分策略选择 |
| Hosts+Proxy 混合 | 全系统 | 零依赖 Hosts 作为 Proxy 备用方案 |
| IP 池生命周期 | `IPPoolMaintainer.psm1` (2,174行) | 状态机驱动的 IP 池维护 |
| IP 扫描流水线 | `IPScanner`+`IPSelector` (2,192行) | CIDR 范围扫描 + TLS/SNI 验证 |
| 多源 IP 获取 | `IPFetcher.psm1` (1,416行) | DNS(UDP+DoH)+社区+内置池+应急池 |
| 应急 IP 池 | `EmergencyPool.psm1` (596行) | 硬编码验证 IP 作为最后手段 |
| DNS 污染检测 | `IPFetcher.psm1` | 13 CIDR GFW 注入模式识别 |
| 网络指纹 | `StateManager.psm1` (1,019行) | WMI 网关检测的网络画像 |
| 协议抗封锁权重 | `proxy-settings.json` | hysteria2:0.70 ~ ss:1.00 |
| 站点-节点绑定 | `NodePoolScanner` | 高质量节点分配给高价值站点 |
| HA 双实例 | `ProxyCoreManager.psm1` | 主/备 mihomo 毫秒级切换 |

---

## 优先级排序建议

### P0 — 必须修复（直接影响用户成功）

1. **修复菜单 1 延迟（40-70+秒）**
   - 根因：每次 BAT 操作重加载 17,800 行模块
   - 方案：MonitorServer 已持久运行，菜单 1 改为轻量 API 调用
   - 关键文件：`GitHub-Hosts-Manager.bat`, `MonitorServer.psm1`

2. **修复"执行后无法访问 GitHub"**
   - 工具核心价值主张失效
   - v4 已设计 14 项 Hosts 模式修复（IR-019~IR-026），均未实现
   - 关键文件：`DecisionEngine.psm1`, v4 tracking

3. **Proxy 模式下 GitHub 延迟检测**
   - 状态面板显示"未检测"
   - DecisionEngine 应通过 mihomo API 探测 GitHub

### P1 — 高影响力改进

4. **在 :9091 构建最小 Web 前端**
   - 利用现有约 25 个 REST API 端点
   - 直接解决 UI/UX Critical 差距，无需架构移植

5. **添加自更新机制**
   - 版本检查（GitHub Releases API）+ 下载 + 原子替换

6. **实现 Proxy Guard**
   - 后台计时器验证 mihomo 存活 + 系统代理状态一致性

7. **配置导入/导出 + 备份**

### P2 — 战略改进

8. 添加 TUIC/WireGuard 协议支持
9. 添加 rule-provider 支持（接入社区规则集）
10. 订阅转换服务集成
11. 开始架构解耦（Phase 5：提取 Windows API 调用为平台抽象层）

---

## 关键文件索引

| 文件 | 行数 | 职责 | Gap 修复关联 |
|------|------|------|-------------|
| `modules/MonitorServer.psm1` | 2,905 | REST API + Watchdog | Web 前端基础、菜单延迟修复 |
| `modules/DecisionEngine.psm1` | 933 | 策略路由核心 | 菜单 1 延迟、GitHub 访问修复 |
| `GitHub-Hosts-Manager.bat` | 1,006 | BAT UI 入口 | 模块重加载延迟根因 |
| `modules/ProxyCoreManager.psm1` | 911 | mihomo 生命周期 | Proxy Guard、自更新 |
| `modules/IPFetcher.psm1` | 1,416 | 多源 IP 获取 | DNS 污染检测、IP 池维护 |
| `modules/IPPoolMaintainer.psm1` | 2,174 | IP 池状态机 | Hosts 模式高可用 |
| `docs/superpowers-v04/tracking.md` | — | v4 14 项设计 | Hosts 模式修复路线图 |
| `docs/superpowers-v05/tracking.md` | — | v5 27 项设计 | 架构通用化路线图 |

---

## 附录：Clash Verge Rev 4A 架构分析

> 作为竞品参照，梳理 CVR 的四层架构（BA|AA|DA|TA），帮助识别本项目架构演进方向。

### BA — 业务架构（Business Architecture）

**定位**：跨平台通用代理客户端 GUI，面向全球需要网络代理访问的用户群体。

| 业务能力域 | 子能力 | 说明 |
|-----------|--------|------|
| 代理核心管理 | mihomo 生命周期管理 | 进程启动/停止/重启，Alpha 版切换 |
| | 内核配置热更新 | 通过 RESTful API 实时推送配置变更 |
| 配置文件管理 | 多 Profile 管理 | 订阅导入、本地创建、Merge/Script 增强 |
| | 配置语法增强 | boa_engine 执行 JS 脚本动态修改生成的 YAML |
| | 云端备份同步 | WebDav 协议配置备份与跨设备同步 |
| 代理规则管理 | 可视化规则编辑 | 内联 Monaco Editor 编辑规则 |
| | 规则集订阅 | Loyalsoldier/ACL4SSR 等社区 rule-provider |
| 系统集成 | 系统代理设置 | 注册表/系统偏好设置 + Proxy Guard 守护 |
| | TUN 虚拟网卡 | gvisor/mixed 模式透明代理 |
| 运维监控 | 连接/流量统计 | mihomo API 实时拉取流量数据 |
| | 日志查看 | 实时日志流式展示 |
| 平台体验 | 多语言 i18n | clash-verge-i18n crate 独立管理 |
| | 主题定制 | Dark/Light/CSS Injection |
| | 自动更新 | GitHub Releases 检查 + 增量更新 |

**关键业务差异**（vs 本项目）：
- CVR 是**通用代理客户端**（面向所有代理用户），本项目是**GitHub 访问专用工具**（面向中国开发者）
- CVR 的智能层完全依赖 mihomo 内置策略（select/url-test/fallback/load-balance），无外部决策引擎
- CVR 无 Hosts 模式、无 IP 池管理、无网络环境检测——这些是本项目独有的业务能力

### AA — 应用架构（Application Architecture）

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri 2 Desktop App                   │
├─────────────────────────────┬───────────────────────────┤
│     Frontend (WebView)      │     Backend (Rust)        │
│                             │                           │
│  pages/                     │  cmd/    ← Tauri Commands │
│   ├─ ClashPage              │  config/                  │
│   ├─ ProfilePage            │   ├─ clash.rs  (mihomo)   │
│   ├─ ProxyPage              │   ├─ verge.rs (app)       │
│   ├─ RulesPage              │   ├─ runtime.rs (状态)     │
│   └─ SettingsPage           │   └─ profiles/ (订阅管理)  │
│                             │  core/   ← 核心调度        │
│  components/                │  enhance/ ← Merge/Script  │
│   ├─ ProxyNode              │  feat/    ← 功能编排       │
│   ├─ ProfileItem            │  process/ ← mihomo 进程   │
│   └─ SettingItem            │  module/  ← 辅助模块       │
│                             │  utils/   ← 工具函数       │
│  services/                  │                           │
│   ├─ api.ts (mihomo REST)   │  crates/ (workspace)      │
│   └─ cmds.ts (Tauri IPC)    │   ├─ clash-verge-draft    │
│                             │   ├─ clash-verge-i18n     │
│  hooks/ (SWR 数据获取)       │   ├─ clash-verge-limiter  │
│  providers/ (Context 状态)   │   ├─ clash-verge-logging  │
│                             │   ├─ clash-verge-signal    │
│                             │   └─ tauri-plugin-sysinfo  │
├─────────────────────────────┴───────────────────────────┤
│                    External Processes                    │
│  mihomo (:7890 mixed, :9090 API)  ← 代理核心            │
└─────────────────────────────────────────────────────────┘
```

**关键交互流**：

| 流程 | 路径 |
|------|------|
| 用户切换代理节点 | Frontend → `cmds.ts` → Tauri IPC → `cmd/` → mihomo API `PUT /proxies/:group` |
| 更新订阅 | Frontend → Tauri IPC → `feat/` → HTTP fetch → `enhance/`(Merge+Script) → 生成 YAML → 写入 mihomo config → 重载 |
| 修改规则 | Frontend → Monaco Editor → Tauri IPC → `config/` → 更新 YAML → mihomo API reload |
| 系统代理开关 | Frontend → Tauri IPC → `feat/` → `sysproxy` crate → OS 代理设置 + Proxy Guard |
| 流量监控 | Frontend ← SWR polling ← mihomo API `GET /traffic` + `GET /connections` |

**vs 本项目架构差异**：

| 维度 | CVR | 本项目 |
|------|-----|--------|
| 前后端通信 | Tauri IPC（进程内，零网络开销） | REST API（HTTP :9091，MonitorServer 独立进程） |
| 代理核心 | mihomo 作为子进程 + RESTful API 控制 | mihomo 作为子进程 + RESTful API 控制（相同） |
| 配置生成 | `enhance/` 模块（Merge + JS Script） | `ProxyConfigGenerator.psm1`（模板化生成） |
| 决策智能 | 无（用户手动选择 / mihomo 内置策略） | `DecisionEngine`（3-way 探测 + 自动策略选择） |
| 进程管理 | `process/` 模块 | `ProxyCoreManager.psm1` + `MonitorServer.psm1` Watchdog |

### DA — 数据架构（Data Architecture）

**配置层次模型**：

```
┌─────────────────────────────────────────┐
│ Layer 4: Runtime (内存)                  │
│   IRuntime { config: Mapping,           │
│     exists_keys: HashSet,               │
│     chain_logs: HashMap }               │
├─────────────────────────────────────────┤
│ Layer 3: Enhanced (增强后)               │
│   Merge 层 (YAML merge key)             │
│   + Script 层 (boa_engine JS 执行结果)  │
├─────────────────────────────────────────┤
│ Layer 2: File (持久化)                   │
│   profiles/{uid}.yaml  (订阅内容)       │
│   verge.yaml           (应用设置)       │
│   clash.yaml           (mihomo配置)     │
├─────────────────────────────────────────┤
│ Layer 1: Template (代码内嵌默认值)       │
│   IClashTemp::template()                │
│   IVergeTemp::template()                │
└─────────────────────────────────────────┘
```

**核心数据实体**：

| 实体 | 格式 | 存储位置 | 说明 |
|------|------|---------|------|
| VergeConfig | YAML | `~/.config/clash-verge/verge.yaml` | 应用全局设置（主题/语言/代理端口） |
| ClashConfig | YAML | `~/.config/clash-verge/clash.yaml` | mihomo 运行配置（代理/DNS/规则） |
| Profile | YAML | `~/.config/clash-verge/profiles/{uid}.yaml` | 订阅内容或本地配置 |
| ProfileItem | YAML 内嵌 | verge.yaml 中 `profiles.items[]` | 订阅元数据（URL/名称/更新时间） |
| RuntimeState | 内存 | IRuntime struct | 运行时配置快照 + 日志 |
| WebDav Backup | YAML | 远程 WebDav | 跨设备配置同步 |

**数据流**：

```
订阅 URL
  │
  ▼
HTTP Fetch (feat/)
  │
  ▼
原始 YAML (profiles/{uid}.yaml)
  │
  ▼
Merge 增强 ← profiles Merge 配置
  │
  ▼
Script 增强 ← boa_engine JS 脚本
  │
  ▼
最终配置 (Runtime Mapping)
  │
  ▼
mihomo API PUT /configs → 热重载
```

**vs 本项目数据架构差异**：

| 维度 | CVR | 本项目 |
|------|-----|--------|
| 主格式 | YAML（原生 mihomo 格式） | JSON（自定义结构）+ 生成 YAML |
| 配置耦合 | 直接操作 mihomo YAML | 三维解耦（sites × subscriptions × proxy-settings） |
| 增强机制 | Merge + JS Script（boa_engine） | 模板化生成（无运行时脚本） |
| 备份 | WebDav 云端同步 | 无（本地文件系统） |
| 加密 | aes-gcm（订阅 URL） | DPAPI（Windows 专用） |
| 验证 | mihomo -t 配置测试 | Test-ClashMetaConfig |

### TA — 技术架构（Technology Architecture）

**技术栈全景**：

```
┌─────────────────────────────────────────────────────┐
│                     Build System                     │
│  Vite 7 (前端) + Cargo (后端, LTO + strict clippy)  │
├────────────────────────┬────────────────────────────┤
│   Frontend (WebView)   │    Backend (Native)        │
│                        │                            │
│  React 19              │  Rust (Tauri 2)            │
│  MUI 7 (组件库)        │  tokio (异步运行时)         │
│  Monaco Editor (编辑器)│  serde_yaml_ng (序列化)     │
│  SWR (数据请求/缓存)    │  warp (HTTP server)        │
│  React Router 7 (路由) │  boa_engine (JS 运行时)     │
│  i18next (国际化)       │  sysproxy (系统代理+Guard)  │
│  Vite 7 (构建)         │  reqwest_dav (WebDav)       │
│                        │  aes-gcm (加密)             │
│                        │  nanoid (ID 生成)           │
│                        │  anyhow (错误处理)          │
├────────────────────────┴────────────────────────────┤
│                   Tauri 2 Runtime                    │
│  IPC Bridge | WebView Host | Plugin System           │
├─────────────────────────────────────────────────────┤
│                   OS Integration                     │
│  Windows: sysproxy, winreg, deelevate, runas         │
│  Linux:   sysproxy, dbus                             │
│  macOS:   sysproxy, launchd                          │
├─────────────────────────────────────────────────────┤
│                   External Process                   │
│  mihomo (Clash Meta) - 代理核心                      │
│  :7890 (mixed proxy)  :9090 (external controller)    │
└─────────────────────────────────────────────────────┘
```

**Cargo Workspace 结构**：

| Crate | 职责 |
|-------|------|
| `clash-verge-draft` | 主应用 crate |
| `clash-verge-i18n` | 国际化资源独立管理 |
| `clash-verge-limiter` | 速率限制 |
| `clash-verge-logging` | 日志基础设施 |
| `clash-verge-signal` | 进程信号处理 |
| `tauri-plugin-clash-verge-sysinfo` | 系统信息 Tauri 插件 |

**构建优化**：
- Release profile 启用 LTO（Link-Time Optimization）
- Strict Clippy lints（代码质量门禁）
- Vite 7 前端打包优化

**vs 本项目技术架构差异**：

| 维度 | CVR | 本项目 |
|------|-----|--------|
| 语言 | Rust + TypeScript | PowerShell 5.1 |
| UI 框架 | Tauri 2 + React 19 | BAT + Write-Host |
| 运行时 | 原生二进制 + WebView | CMD → powershell.exe 子进程 |
| 异步模型 | tokio (async/await) | 同步阻塞（PowerShell pipeline） |
| 包管理 | Cargo workspace + npm | 无（手动 Import-Module） |
| 构建系统 | Cargo + Vite | 无（零编译） |
| 跨平台 | Win/Linux/macOS | Windows Only |
| 冷启动 | <2 秒 | 40-70 秒（模块重加载） |

### 4A 架构对标总结

| 4A 维度 | CVR 优势 | 本项目差异化价值 |
|---------|---------|-----------------|
| BA | 通用定位覆盖面广，用户基数大 | 专用定位——GitHub 访问自动化，零配置即用 |
| AA | Tauri IPC 零网络开销，前后端分离 | DecisionEngine 智能决策层是 CVR 完全缺失的架构层 |
| DA | Merge+Script 增强灵活，WebDav 云同步 | 三维配置解耦对目标用户更可维护；IP 池状态机无竞品 |
| TA | Rust 原生性能 + 跨平台 + 现代工具链 | PS5.1 零依赖部署适合目标用户群（开发者本地工具） |

**核心洞察**：CVR 在 TA 层（技术架构）和 AA 层（应用架构的 UI 部分）显著领先；本项目在 BA 层（智能决策自动化）和 DA 层（IP 池状态管理）具有不可替代的差异化价值。两者的差距本质是**通用 GUI 客户端** vs **专用自动化工具**的定位差异，而非绝对能力差距。

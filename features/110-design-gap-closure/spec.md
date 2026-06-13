# Feature 110: F001~004 设计-实现 Gap 闭环 — 需求规格

- **Feature**: 110-design-gap-closure
- **阶段**: `hf-specify`
- **状态**: 草稿
- **日期**: 2026-06-09
- **上游输入**:
  - `features/001-baseline-restore/spec.md` + `design.md`
  - `features/002-wsl-support/spec.md` + `design.md`
  - `features/003-site-rules/spec.md` + `design.md`
  - `features/004-user-interaction/spec.md` + `design.md` + `ui-design.md`
  - F101~F109 closeout 记录（已覆盖的 gap 不再列入）

## 1. 概述

### 1.1 目的

本 feature 对 F001~F004 设计规格与当前 codebase 实现之间的剩余差距做系统性闭环。F101~F109 已修复了 9 批具体缺陷（协同模式、proxy-env 写入、非目标验证、续跑 UI 阻塞、ProxyGuard 后台巡检、Tauri 事件全集、mihomo 端口纳管、服务暂停/代理生命周期、baseline 恢复语义），本 feature 覆盖**其余未修复的 24 项差距**（P0:3 + P1:10 + P2:11）。

### 1.2 Gap 来源方法论

差距清单的产出方式：
1. 逐一比对 F001~F004 spec/design 中的每个 FR/NFR/SC 与 codebase 实际实现
2. 比对 F004 `ui-design.md` 30 条标注修订与实际前端渲染
3. 排除 F101~F109 已覆盖的项（详见 §1.3 排除表）
4. 按阻塞发布/核心交互缺失/渐进改善三级分级

### 1.3 已由 F101~F109 覆盖的项（不再列入）

| 已覆盖 gap | 覆盖 feature | 状态 |
|------------|-------------|------|
| 协同部署双侧适配器 | F101 | closed |
| WSL/Linux proxy-env 写入空操作 | F102 | closed |
| 恢复后非目标站点验证缺失 | F103 | closed |
| 续跑/恢复期间 UI 无阻塞保护 | F104 | closed |
| ProxyGuard 无后台定时巡检 | F105 | closed |
| Tauri 事件仅发射 3/6 | F106 | closed |
| mihomo 端口冲突与僵尸进程 | F107 | closed |
| 服务暂停后 ProxyGuard 自动拉起 + proxy-env 生命周期 | F108 | closed |
| "立即恢复"语义反转 + WSL proxy-env 分类 + 恢复审计 + ProxyGuard 代理残留 + 非目标配置化 + DNS 缓存刷新 + ProxyStrategy 集成 + 恢复偏差确认 | F109 | 进行中 |

## 2. Gap 清单与优先级分级

### 2.1 🔴 P0 — 阻塞发布 / 关键安全风险

| Gap ID | 差距描述 | 设计锚点 | 影响评估 |
|--------|----------|----------|----------|
| **G110-1** | **ProbeService 使用 MockProbeClient** — 生产环境需真实 HTTP/TLS 探测实现，当前 MockProbeClient 仅用于开发测试 | F003 §2.4 ProbeService, SC-1 P95恢复, SC-5 五要素诊断 | 无真实探测 = 可达性诊断完全失效 = 产品核心价值归零 |
| **G110-2** | **NodePool ↔ SubscriptionParser 管道未闭环** — `import_subscription` 仅解析订阅不将节点 add_node 到 NodePool | F003 §2.5 NodePool, §2.6 SubscriptionParser | 无代理节点导入 = 无 PROXY 策略节点 = 站点不可达 |
| **G110-3** | **Wizard Step 3 手工调整引导完全缺失** — 设计要求 step-by-step 引导（一键自动调整、命令行复制、系统设置指引、整体进度条），实现仅为简单占位 | F004 §4.7 Step 3, F001 FR-2.2.1-R3 | 普通办公用户无法完成 baseline 形成 = 产品首体验证阻塞 |

### 2.2 🟡 P1 — 核心交互完整性缺失

| Gap ID | 差距描述 | 设计锚点 | 影响评估 |
|--------|----------|----------|----------|
| **G110-4** | **7 个核心组件无独立文件** — SiteCard/RuleTable/DiagPanel/NodePoolTable/AuditLogTable/Wizard/WizardStep 全部内嵌在页面中 | F004 §5.1, §2 分层架构(Page→Shared→Store) | 组件复用性为零，页面膨胀，违反分层架构 |
| **G110-5** | **五要素诊断提示不完整** — DiagnosticsPage 仅展示"可能原因"+"建议操作"，缺少已尝试动作、尝试次数、是否需手动处理 | F001 FR-2.8.1-R1, F004 FR-2.6.2-R1 | SC-3 失败可触发手动处理无法验证 |
| **G110-6** | **规则预览功能简化** — 缺用户覆盖标记、待应用变更区块、MATCH,DIRECT 兜底规则展示 | F003 FR-2.4.2-R2, F004 FR-2.5.1-R2 | 进阶用户无法识别自定义规则，无法确认直连兜底生效 |
<!-- TODO id:02;status:open;date:2026-06-09T14:30 design §3.3假设preview_rules返回的规则含source字段(auto/user_override)，需验证后端get_rules/preview_rules的实际返回结构是否包含此字段 -->
| **G110-7** | **通知类型非语义化** — 实现用通用 info/success/warning/error 替代 rule-rollback/recovery/audit-change/node-pool | F004 §6.1 AppNotification type | 可追溯性丢失，用户无法按类别筛选通知 |
| **G110-8** | **StatusBar 无动态数据绑定** — 仅硬编码"状态:运行中+版本0.1.0"，缺 baseline/恢复/部署模式实时状态 | F004 §4.1 StatusBar | NFR-3.1-2 UI刷新≤2s 无法验证 |
| **G110-9** | **Header 缺状态指示灯+通知铃铛** — 仅显示"GoGuo"+"立即恢复"按钮 | F004 §4.1 Header | 主路径≤2步验证受阻 |
| **G110-10** | **仪表盘缺部署模式卡片** — 部署模式仅在设置页展示，仪表盘无此信息 | F004 §4.2, FR-2.3.2-R3 | 用户无法一眼看到当前部署模式 |
<!-- ? id:03;status:open;date:2026-06-09T14:30 codebase验证：DashboardPage已存在"环境信息"卡片展示运行环境/网络模式/代理策略（仅WSL时显示），gap描述"完全缺失"不准确，应修正为"缺两侧配置差异提示且仅WSL环境下可见" -->
| **G110-11** | **不可达站点缺内嵌诊断面板入口** — 用户需额外跳转到诊断页 | F004 ui-design id:16, FR-2.6.2-R1 | 主路径≤2步被破坏 |
| **G110-12** | **NotifBar 缺"查看全部通知"入口** — 仅文字提示"还有N条通知..." | F004 FR-2.7.1-R1 | 通知归档不可达 |

### 2.3 🟢 P2 — 渐进改善 / 可维护性

| Gap ID | 差距描述 | 设计锚点 | 影响评估 |
|--------|----------|----------|----------|
| **G110-13** | **IpScanner 仅 GitHub 候选 IP** — 其他站点无 IpDirect 策略的 candidate IPs | F003 IpScanner | IpDirect 仅对 GitHub 有效 |
| **G110-14** | **MihomoManager reload_config 无响应体解析** — 仅检查 200/204，无错误码映射 | F001 §2.4 MihomoManager | mihomo 重载失败时用户看不到原因 |
| **G110-15** | **审计日志缺日期/类型过滤 UI** — 仅"加载更多"分页 | F001 FR-2.7.2-R3, F004 §5.1 AuditLogTable | 审计可追溯性受限 |
| **G110-16** | **NodePoolTable 元数据不全** — 缺入池时间、检测时间、检测方式 | F004 FR-2.6.3-R3 | 进阶用户无法评估节点可靠性 |
| **G110-17** | **Settings 缺探测间隔+通知偏好配置** | F004 §4.6 | NFR-3.1 探测间隔可配置未实现 |
| **G110-18** | **CodeBlock 缺语法高亮** | F004 §5.1, FR-2.2.1-R3 | 命令复制缺乏视觉区分 |
| **G110-19** | **UI 术语未对齐设计标注** — "站点管理"vs"需要访问的网站"、"诊断"vs"网站状态"、"代理节点池"vs"访问通道" | F004 ui-design id:12/20/23 | 普通用户理解门槛高 |
| **G110-20** | **RecoveryAckDialog 按钮文案偏离** — "重试/确认" vs 设计 "确认已修复/重新恢复" | F004 §5.2 RecoveryAckDialog | FR-2.6.2-R4 用户无法理解闭环选项 |
| **G110-21** | **Wizard 步骤顺序偏离设计** — 部署模式在 Step 2 而非 Step 5 | F004 §4.7 | 部署模式选择应在 baseline 确认后 |
<!-- TODO id:04;status:open;date:2026-06-09T14:30 codebase已验证确认：WizardPage.tsx步骤顺序为['welcome','deployment-mode','initial-assessment',...]，deployment-mode在Step 2，F004 §4.7要求Step 5，gap成立 -->
| **G110-22** | **apply_rules 命令未单独暴露** — 内嵌在 add_site 流程中 | F003 §4 Tauri Commands | UI 无法独立触发规则应用（覆盖后无重新应用入口） |
| **G110-23** | **shadcn/ui 未引入** — ADR-0006 明确选定 shadcn/ui，实现全用原生 HTML+手写 CSS | ADR-0006, F004 §10 | 跨平台一致性无法保障，设计令牌体系缺失 |
| **G110-24** | **IpDirect 策略无运行时健康维护** — IP 缓存 24h TTL 期间无刷新机制；探测失败不触发 IP 重扫；无 fallback（IP 全不可达时仍强制 DIRECT）；休眠唤醒后不刷新 IP 缓存 | F003 SiteRuleEngine IpDirect, IpScanner, IpCache | 国内直连 GitHub IP 被干扰后持续不可达，`gh auth login` 等工具报 unexpected EOF；影响范围 = 所有 IpDirect 站点（当前仅 GitHub） |
<!-- ? id:08;status:open;date:2026-06-09T14:30 shadcn/ui引入涉及全局样式体系变更，单项工作量可能超过P0三项之和，与"补齐型feature"定位矛盾，建议拆为独立feature以控制回归风险和review范围 -->

## 3. 功能需求

### 3.1 P0: 真实探测实现（G110-1）

#### FR-3.1.1 ProbeClient 生产实现

**要求**:
- FR-3.1.1-R1: 必须实现真实的 `ProbeClient` trait 替代 `MockProbeClient`，支持 3 级探测（DNS+HEAD、GET+状态码、TLS+响应体）
- FR-3.1.1-R2: Level 1 (HTTP HEAD) 必须通过 mihomo 代理执行（反映真实用户体验）
- FR-3.1.1-R3: Level 2 (HTTP GET) 必须验证状态码 2xx/3xx = 可达
- FR-3.1.1-R4: Level 3 (TLS) 仅在不可达时触发，用于定位具体失败环节
- FR-3.1.1-R5: 并行探测所有目标站点（NFR-3.1-4: 总耗时≤单站点2倍）
- FR-3.1.1-R6: 探测超时默认 3s（NFR-3.1-3）
- FR-3.1.1-R7: 不可达站点自动切换降级探测频率
<!-- TODO id:07;status:open;date:2026-06-09T14:30 "降级探测频率"缺少量化规格——降级间隔多少秒？连续成功几次恢复默认频率？建议在design阶段补充状态机定义 -->

### 3.2 P0: 节点导入闭环（G110-2）

#### FR-3.2.1 SubscriptionParser → NodePool 管道

**要求**:
- FR-3.2.1-R1: `import_subscription` 命令必须将解析成功的支持协议节点通过 `NodePool::add_node()` 导入到节点池
- FR-3.2.1-R2: 不支持的协议节点必须被过滤并在 UI 中标记（FR-2.3.2 过滤行为）
- FR-3.2.1-R3: 导入结果（成功数/过滤数/失败数）必须记入审计
- FR-3.2.1-R4: 导入后必须触发 mihomo 配置重生成（NodePool → MihomoConfigManager → hot reload）
- FR-3.2.1-R5: 导入后必须触发代理节点健康检查（NodeHealthChecker）

### 3.3 P0: Wizard 手工调整引导（G110-3）

#### FR-3.3.1 Step-by-step 手工调整引导

**要求**:
- FR-3.3.1-R1: Wizard Step 3 必须展示当前网络状态中不理想的项，逐项给出调整建议
- FR-3.3.1-R2: 每项建议必须包含可执行的命令行命令（支持一键复制，CodeBlock 组件）
- FR-3.3.1-R3: 每项建议必须包含系统页面操作指引（如"打开 设置→网络和Internet→代理"）
- FR-3.3.1-R4: 必须提供一键自动调整按钮（针对 Restorable 项，调用 `triggerReadjustment`）
- FR-3.3.1-R5: 必须展示整体进度条（已完成/总数/当前项）
- FR-3.3.1-R6: 手工调整后，用户可触发重新采集→重新评估→重新确认的完整流程
- FR-3.3.1-R7: 确认前文案必须明确说明 baseline 是"用户确认的可用状态"而非"系统出厂状态"（FR-2.2.2-R2）

### 3.4 P1: 组件独立化（G110-4）

#### FR-3.4.1 核心组件拆分

**要求**:
- FR-3.4.1-R1: 以下组件必须从页面内嵌逻辑拆分为独立文件：SiteCard、RuleTable、DiagPanel、NodePoolTable、AuditLogTable
- FR-3.4.1-R2: Wizard 必须拆分为 Wizard + WizardStep 独立组件
- FR-3.4.1-R3: 拆分后页面文件必须仅做组件组合和数据绑定，不包含渲染逻辑
- FR-3.4.1-R4: 拆分不得破坏现有页面功能（行为等价验证）

### 3.5 P1: 五要素诊断完整性（G110-5）

#### FR-3.5.1 五要素诊断提示

**要求**:
- FR-3.5.1-R1: 不可达站点诊断提示必须完整包含五要素：原因、已尝试动作、尝试次数、建议动作、是否需要手动处理
- FR-3.5.1-R2: 五要素数据来源必须为后端 `FiveElementPrompt` 结构（已有 Rust 实现）
- FR-3.5.1-R3: 建议动作必须是用户可执行的（如"请在WSL中手动执行 `export http_proxy=...`"）
- FR-3.5.1-R4: 是否需要手动处理必须明确显示（Yes/No + 文案引导）

### 3.6 P1: 规则预览增强（G110-6）

#### FR-3.6.1 规则预览完整性

**要求**:
- FR-3.6.1-R1: 规则预览必须展示用户自定义覆盖规则（标记来源为"自定义"）
- FR-3.6.1-R2: 规则预览必须展示 MATCH,DIRECT 兜底规则
- FR-3.6.1-R3: 规则预览按站点分组展示时，每组必须标注站点名称和策略（PROXY/DIRECT）
- FR-3.6.1-R4: 存在待应用变更时，必须展示"待应用变更"区块

### 3.7 P1: 通知语义化（G110-7）

#### FR-3.7.1 通知类型语义化

**要求**:
- FR-3.7.1-R1: 通知类型必须从 info/success/warning/error 改为 rule-rollback/recovery/audit-change/node-pool
<!-- TODO id:09;status:open;date:2026-06-09T14:30 通知类型变更为breaking change——需确认NotifStore中已有通知实例的迁移策略，以及前端其他引用NotificationType的组件（如NotifBar）的兼容性 -->
- FR-3.7.1-R2: 通知必须包含 actions 字段（{label, command}），提供可操作入口
- FR-3.7.1-R3: 系统通知（规则回退、恢复失败）必须通过 Tauri notification API 推送（替代当前 Web Notification API）
- FR-3.7.1-R4: 系统通知不可用时应用内通知降级，不得丢失通知内容

### 3.8 P1: StatusBar 动态绑定（G110-8）

#### FR-3.8.1 StatusBar 实时状态

**要求**:
- FR-3.8.1-R1: StatusBar 必须动态展示：服务状态（运行/停止/异常）、Baseline 状态（确认/偏离/未确认）、部署模式（仅Windows/仅WSL/协同/仅Linux）
- FR-3.8.1-R2: 数据必须来源于 Tauri Event 推送（事件驱动，不轮询）
- FR-3.8.1-R3: 状态变更必须在 2s 内反映到 UI（NFR-3.1-2）

### 3.9 P1: Header 状态指示灯+通知铃铛（G110-9）

#### FR-3.9.1 Header 完整性

**要求**:
- FR-3.9.1-R1: Header 必须包含状态指示灯（绿色=运行、红色=异常、灰色=停止）
- FR-3.9.1-R2: Header 必须包含通知铃铛图标（点击展开 NotifBar）
- FR-3.9.1-R3: 状态指示灯颜色与 ServiceStore 状态实时同步

### 3.10 P1: 仪表盘部署模式卡片（G110-10）

#### FR-3.10.1 部署模式展示

**要求**:
- FR-3.10.1-R1: 仪表盘必须包含部署模式卡片，展示当前 DeploymentMode
- FR-3.10.1-R2: 卡片必须展示两侧（Windows/WSL）的配置差异提示（若存在差异）

### 3.11 P1: 不可达站点内嵌诊断（G110-11）

#### FR-3.11.1 站点卡片诊断入口

**要求**:
- FR-3.11.1-R1: 不可达站点卡片点击后必须展开内嵌 DiagPanel，展示五要素诊断提示
- FR-3.11.1-R2: DiagPanel 展开不触发页面跳转（主路径≤2步保障）

### 3.12 P1: 通知全部查看入口（G110-12）

#### FR-3.12.1 NotifBar 完整通知

**要求**:
- FR-3.12.1-R1: NotifBar 必须提供"查看全部通知"入口，跳转到通知历史页面或展开完整列表
- FR-3.12.1-R2: 通知历史必须支持按类型和时间筛选

### 3.13 P2: IpScanner 候选 IP 扩展（G110-13）

#### FR-3.13.1 多站点候选 IP

**要求**:
- FR-3.13.1-R1: IpScanner 必须为 SiteDefinition 中所有标记 AccessStrategy=IpDirect 的站点提供候选 IP
- FR-3.13.1-R2: 候选 IP 来源可通过 DNS 解析或配置文件指定

### 3.14 P2: Mihomo reload 响应解析（G110-14）

#### FR-3.14.1 重载错误信息提取

**要求**:
- FR-3.14.1-R1: `reload_config()` 必须解析 mihomo API 响应体，提取错误信息
- FR-3.14.1-R2: 重载失败时必须向 UI 提供具体的 mihomo 错误信息

### 3.15 P2: 审计日志过滤 UI（G110-15）

#### FR-3.15.1 AuditLogTable 过滤

**要求**:
- FR-3.15.1-R1: AuditLogTable 必须支持按日期范围和操作类型筛选
- FR-3.15.1-R2: 筛选参数通过 `get_audit_log` command 的 filter 参数传递

### 3.16 P2: NodePoolTable 元数据补齐（G110-16）

#### FR-3.16.1 节点元数据展示

**要求**:
- FR-3.16.1-R1: NodePoolTable 必须展示：节点名称、入池时间、当前可用状态、最近检测时间、检测方式
- FR-3.16.1-R2: 数据来源为 `get_node_pool_status` command 返回的 NodeInfo 结构

### 3.17 P2: Settings 探测+通知配置（G110-17）

#### FR-3.17.1 配置面板

**要求**:
- FR-3.17.1-R1: Settings 必须提供探测间隔配置输入（默认 30s）
- FR-3.17.1-R2: Settings 必须提供通知偏好勾选（应用内通知/系统通知开关）
- FR-3.17.1-R3: 配置修改后持久化到 AppConfig

### 3.18 P2: CodeBlock 语法高亮（G110-18）

#### FR-3.18.1 命令展示

**要求**:
- FR-3.18.1-R1: CodeBlock 必须支持语法高亮（shell 命令类型）
- FR-3.18.1-R2: 一键复制功能保留

### 3.19 P2: UI 术语对齐（G110-19）

#### FR-3.19.1 语义化文案

**要求**:
- FR-3.19.1-R1: 页面标题和关键术语必须对齐设计标注：
  - 站点管理 → "需要访问的网站"
  - 规则预览 → "代理规则"
  - 诊断 → "网站状态"
  - 代理节点池 → "访问通道"
- FR-3.19.1-R2: 非技术用户可见的所有文案优先使用意图导向术语

### 3.20 P2: RecoveryAckDialog 文案对齐（G110-20）

#### FR-3.20.1 确认按钮文案

**要求**:
- FR-3.20.1-R1: RecoveryAckDialog 按钮文案改为 "确认已修复" / "重新恢复"（与 F001 §5.3 恢复任务状态机对齐）

### 3.21 P2: Wizard 步骤顺序修正（G110-21）

#### FR-3.21.1 步骤顺序

**要求**:
- FR-3.21.1-R1: Wizard 步骤顺序必须对齐 F004 §4.7：评估→状态展示→手工调整→确认baseline→部署模式→站点选择→完成

### 3.22 P2: apply_rules 独立命令（G110-22）

#### FR-3.22.1 规则应用命令

**要求**:
- FR-3.22.1-R1: 必须新增独立 `apply_rules` Tauri command，支持规则覆盖后重新应用
- FR-3.22.1-R2: 该命令必须执行完整的规则生成→静态校验→A/B探测→应用/回退流程

### 3.23 P2: shadcn/ui 引入（G110-23）

#### FR-3.23.1 组件库迁移

**要求**:
- FR-3.23.1-R1: 必须引入 shadcn/ui 组件库（ADR-0006 决策）
- FR-3.23.1-R2: 核心交互组件（ConfirmDialog、StatusBadge、NotifBar、CodeBlock、RecoveryOverlay）必须迁移为 shadcn/ui 组件
- FR-3.23.1-R3: 迁移后全局样式通过 shadcn/ui 的 CSS 变量体系管理
- FR-3.23.1-R4: 迁移不得破坏现有页面功能（行为等价验证）

### 3.24 P1: IpDirect 运行时健康维护（G110-24）

#### FR-3.24.1 IpDirect 策略运行时健康维护

**要求**:
- FR-3.24.1-R1: IP 缓存 TTL 期间必须有定期刷新机制——在 TTL 过半时后台触发 IP 重扫，确保缓存始终包含有效 IP
- FR-3.24.1-R2: 探测失败必须触发 IP 重扫——当 ProbeService 报告 IpDirect 站点不可达时，立即触发 `IpScanner::scan()` 更新缓存
- FR-3.24.1-R3: 必须实现 DIRECT→PROXY fallback——当 IP 缓存中所有 IP 均不可用时，将该站点的 mihomo 规则从 `DOMAIN-SUFFIX,*,DIRECT` 临时切换为 `DOMAIN-SUFFIX,*,PROXY`，使用代理节点恢复可达性
- FR-3.24.1-R4: 休眠唤醒后必须刷新 IP 缓存——监听系统唤醒事件，唤醒后立即触发 IpDirect 站点的 IP 重扫
- FR-3.24.1-R5: IP 缓存刷新后必须热更新 mihomo 配置（`MihomoConfigManager::generate_config()` + `MihomoManager::reload_config()`），确保 hosts 映射和规则同步生效
- FR-3.24.1-R6: 所有 IP 缓存刷新和 fallback 切换操作必须记入审计日志

## 4. 非功能需求

### 4.1 性能

| ID | 需求 | 验证方式 |
|----|------|----------|
| NFR-4.1-1 | 单站点真实探测 ≤ 3s | 计时验证 |
| NFR-4.1-2 | 多站点并行探测总耗时 ≤ 单站点 2 倍 | 多站点计时 |
| NFR-4.1-3 | StatusBar 状态变更 ≤ 2s 反映到 UI | 状态变更计时 |
| NFR-4.1-4 | 节点导入+配置重载 ≤ 15s | 计时验证 |

### 4.2 可靠性

| ID | 需求 | 验证方式 |
|----|------|----------|
| NFR-4.2-1 | ProbeClient 真实探测失败不得产生用户可见副作用 | 探测失败注入 |
| NFR-4.2-2 | 节点导入失败不得阻塞后续站点规则功能 | 导入失败注入 |
| NFR-4.2-3 | shadcn/ui 迁移后所有页面功能行为等价 | 功能对比测试 |

### 4.3 安全

| ID | 需求 | 验证方式 |
|----|------|----------|
| NFR-4.3-1 | shadcn/ui 组件不得向远程发起请求 | 网络抓包 |

## 5. 约束

| ID | 约束 | 原因 |
|----|------|------|
| CON-1 | 不修改 F001~F004 设计规格本身——本 feature 是实现补齐，不是设计变更 | soul §1: 方向最终权在用户 |
| CON-2 | 不修改 baseline 数据格式 | backward compatible |
| CON-3 | shadcn/ui 迁移为渐进式：先替换核心交互组件，其余页面逐步迁移 | 避免一次性大重构风险 |
| CON-4 | ProbeClient 实现不得引入 reqwest 等重依赖（沿用 std::net::TcpStream + native-tls） | 与现有依赖策略一致 |
<!-- ? id:06;status:open;date:2026-06-09T14:30 禁用reqwest约束下，手写HTTP/1.1需自行处理chunked encoding、redirect（3xx跳转）、keep-alive等协议细节。结合WSL网络完全依赖mihomo的环境特性，3s超时目标（ASM-1）置信度"中"需下调为"低"，建议design阶段提供HTTP协议边界case清单 -->
| CON-5 | 数据存储沿用 F001 CON-5（统一安装根目录） | 一致性 |

## 6. 不在范围内

| 排除项 | 原因 |
|--------|------|
| F101~F109 已覆盖的所有 gap | 已有独立 feature 处理 |
| 启动续跑完整流程（App 启动时检查 recovery-task.json） | F109 范围 |
| active_sites 持久化 | F003 范畴 |
| config.yaml rules 段硬编码与动态站点管理脱节 | F003 范畴 |
| 完整节点池管理 UI（节点搜索/评估/排序） | OPP-005 |
| 完整设置面板（高级配置/日志浏览器） | F004 "不在范围内" |

## 7. 假设

| ID | 假设 | 置信度 | 验证方式 |
|----|------|--------|----------|
| ASM-1 | 真实 HTTP/TLS 探测可通过 std::net + native-tls 在 3s 内完成 | 中 | P0 probe 验证 |
| ASM-2 | NodePool::add_node 与 SubscriptionParser 的类型系统可直接对接 | 高 | 类型检查 |
| ASM-3 | shadcn/ui 与当前 React 19 + Zustand 5 + Vite 组合兼容 | 中 | 安装验证 |
| ASM-4 | 组件拆分不引入渲染行为差异 | 高 | 快照对比测试 |
| ASM-5 | Wizard 手工调整引导所需的后端数据（不理想项列表+调整建议）可通过现有 API 获取 | 中 | API 覆盖度评估 |

## 8. 开放问题

| ID | 问题 | 阻塞性 | 建议处理时机 |
|----|------|--------|-------------|
| OP-1 | ProbeClient 是否需要 async runtime（当前 Rust 后端全同步） | 非阻塞 | hf-design 阶段决策 |
| OP-2 | shadcn/ui 迁移是否需要同步更新暗色主题令牌体系 | 非阻塞 | hf-design 阶段 |
| OP-3 | Wizard Step 3 "一键自动调整"按钮的范围——仅 Restorable 项还是也包含可检测不可恢复项的建议 | 阻塞 | hf-specify 评审决定 |
<!-- ? id:05;status:open;date:2026-06-09T14:30 OP-3标记为阻塞但未在spec内解决，建议本次评审明确决策：一键自动调整仅限Restorable项（可自动恢复），detectable-not-restorable项仅展示建议文案+命令复制，不自动执行 -->
| OP-4 | 节点导入后是否需要用户确认才写入 mihomo 配置 | 非阻塞 | hf-design 阶段 |

## 9. 成功标准

| # | 标准 | 验证方式 |
|---|------|----------|
| SC-1 | 真实探测可返回目标站点可达性+响应时间 | 真实站点探测计时 |
| SC-2 | 订阅导入后节点池非空，mihomo 配置含导入节点 | 导入→检查节点池→检查配置 |
| SC-3 | Wizard Step 3 展示不理想项+调整建议+一键复制命令 | 首次引导走查 |
| SC-4 | 五要素诊断提示完整（5/5要素） | 注入不可达场景检查提示 |
| SC-5 | StatusBar 展示服务+baseline+部署模式实时状态 | 状态变更后检查 UI |
| SC-6 | 通知类型按业务语义分类 | 触发各类通知检查类型标签 |
| SC-7 | shadcn/ui 核心组件已替换，全局样式通过 CSS 变量管理 | 组件审查+样式审查 |
| SC-8 | IpDirect 站点 IP 失效后可在 60s 内自动恢复可达（重扫或 fallback） | 模拟 IP 失效→检查站点可达性恢复 |
<!-- ? id:10;status:open;date:2026-06-09T14:30 SC-7"核心组件已替换"可测试性不足——"核心组件"范围需明确列举（ConfirmDialog/StatusBadge/NotifBar/CodeBlock/RecoveryOverlay共5个？）。验证方式"组件审查"不满足NFR可量化要求，建议补充：所有P1/P2组件在shadcn迁移后功能测试100%通过 -->

## 10. 需求追溯矩阵

| 成功标准 | 功能需求 | 非功能需求 |
|----------|----------|-----------|
| SC-1 真实探测 | FR-3.1.1 | NFR-4.1-1, NFR-4.1-2 |
| SC-2 节点导入闭环 | FR-3.2.1 | NFR-4.1-4 |
| SC-3 Wizard 引导 | FR-3.3.1 | — |
| SC-4 五要素完整 | FR-3.5.1 | — |
| SC-5 StatusBar 动态 | FR-3.8.1, FR-3.9.1 | NFR-4.1-3 |
| SC-6 通知语义化 | FR-3.7.1 | — |
| SC-7 shadcn/ui | FR-3.23.1 | NFR-4.2-3, NFR-4.3-1 |
| SC-8 IpDirect 健康恢复 | FR-3.24.1 | NFR-4.2-1 |
# Feature 110: F001~004 设计-实现 Gap 闭环 — 设计文档

- **Feature**: 110-design-gap-closure
- **阶段**: `hf-design`
- **状态**: 草稿
- **日期**: 2026-06-09
- **上游规格**: `features/110-design-gap-closure/spec.md`
- **设计锚点**: `docs/principles/` 下 soul、sdd-artifact-layout、hf-sdd-tdd-skill-design
- **关联 ADR**: ADR-0002 (Tauri)、ADR-0006 (React+TS+shadcn/ui)

## 1. 设计概述

本设计对 F001~F004 设计规格与当前 codebase 之间的 24 项剩余差距提供系统性修复方案。按优先级分三批交付：

- **Batch A (P0)**: 真实探测、节点导入闭环、Wizard 手工调整引导 — 阻塞产品核心价值
- **Batch B (P1)**: 组件拆分、五要素完整、规则预览、通知语义化、StatusBar/Header、仪表盘部署模式、诊断入口、通知入口、**IpDirect 运行时健康维护** — 核心交互完整性
- **Batch C (P2)**: IpScanner 扩展、reload 解析、审计过滤、节点元数据、Settings 配置、CodeBlock、UI 术语、按钮文案、Wizard 顺序、apply_rules、shadcn/ui — 渐进改善

## 2. Batch A — P0 修改

### 2.1 D-110-1: ProbeClient 生产实现（G110-1）

#### 2.1.1 当前问题

`ProbeService` 在 `SiteRulesState::new()` 中使用 `MockProbeClient::new()`，无真实 HTTP/TLS 探测能力。`ProbeClient` trait 已定义但仅有 Mock 实现。

#### 2.1.2 修复方案

新增 `RealProbeClient` struct，实现 `ProbeClient` trait，沿用现有依赖（std::net::TcpStream + native-tls），不引入 reqwest：

```rust
struct RealProbeClient {
    mihomo_address: SocketAddr,
    timeout: Duration,
    tls_connector: native_tls::TlsConnector,
}

impl ProbeClient for RealProbeClient {
    fn probe_head(&self, url: &str, via_proxy: bool) -> ProbeResult;
    fn probe_get(&self, url: &str, via_proxy: bool) -> ProbeResult;
    fn probe_tls(&self, host: &str, port: u16) -> ProbeResult;
    fn probe_dns(&self, domain: &str) -> ProbeResult;
}
```

**实现策略**：
- Level 1 (HEAD): 通过 mihomo 代理（CONNECT 方法）发送 HTTP HEAD 请求，解析状态码
- Level 2 (GET): 同上，使用 HTTP GET + 响应码 2xx/3xx 判断可达
- Level 3 (TLS): 直接 TLS 握手验证（native_tls::TlsConnector），定位 DNS/TCP/TLS 具体失败环节
- DNS 解析: `std::net::ToSocketAddrs` 解析域名

**代理探测实现**（核心难点）：

```rust
fn probe_via_proxy(&self, url: &str, method: HttpMethod) -> ProbeResult {
    // 1. TCP connect to mihomo_address (127.0.0.1:7890)
    // 2. Send HTTP CONNECT request for target host
    // 3. If proxy responds 200, tunnel is established
    // 4. Send actual HTTP request through tunnel
    // 5. Parse response status code + timing
}
```

沿用 `MihomoManager::reload_config()` 中的原始 TCP HTTP 请求模式（已有实现，无外部依赖）。

<!-- TODO id:13;status:open;date:2026-06-09T14:30 HTTP CONNECT隧道实现需处理的边界case清单：(1)proxy需认证(407响应) (2)HTTPS目标的双TLS握手 (3)chunked transfer encoding解析 (4)connection:close后的资源释放 (5)mihomo未运行时的连接拒绝。建议在实现前补充HTTP/1.1 CONNECT协议边界case测试矩阵 -->

**SiteRulesState 初始化修改**：

```rust
// 当前
SiteRulesState::new(..., MockProbeClient::new())

// 修改后
SiteRulesState::new(..., RealProbeClient::new(
    config.mihomo.api_address,
    config.probe.timeout,
    native_tls::TlsConnector::new().unwrap(),
))
```

#### 2.1.3 并行探测

`ProbeService::probe_all()` 已使用 `std::thread::scope` 并行，仅需替换 client。`max_concurrent` 限制通过 semaphore 模式（AtomicU32 计数器）实现。

### 2.2 D-110-2: NodePool ↔ SubscriptionParser 管道闭环（G110-2）

#### 2.2.1 当前问题

`import_subscription` command 解析订阅后返回 `SubscriptionResponse`，但未调用 `NodePool::add_node()`。NodePool 与 SubscriptionParser 之间无数据管道。

#### 2.2.2 修复方案

在 `import_subscription` command 中，解析完成后遍历 `ParseResult.supported_nodes`，逐个调用 `NodePool::add_node()`：

```rust
// commands/site_rules.rs — import_subscription 修改
pub fn import_subscription(state: State<'_, SiteRulesState>, url: String) -> Result<SubscriptionResponse, String> {
    let parse_result = parser.import_from_url(&url)?;

    // NEW: Import supported nodes to NodePool
    let mut imported_count = 0;
    let mut filtered_count = 0;
    for parsed_node in &parse_result.supported_nodes {
        let proxy_node = ProxyNode::from_parsed(parsed_node);
        match state.node_pool.add_node(proxy_node) {
            Ok(()) => imported_count += 1,
            Err(_) => filtered_count += 1,
        }
    }
    filtered_count += parse_result.unsupported_nodes.len();

    // NEW: Regenerate mihomo config with new nodes
    let config_mgr = state.mihomo_config_manager.lock().expect("lock");
    config_mgr.regenerate(&state.site_rule_engine, &state.node_pool)?;

    // NEW: Reload mihomo
    let mut mihomo = state.mihomo_manager.lock().expect("lock");
    mihomo.reload_config(&config_path)?;

    // NEW: Trigger initial health check
    let checker = NodeHealthCheckerImpl::new(state.mihomo_manager.clone());
    checker.check_all(&state.node_pool)?;

    // Audit
    state.audit_logger.log_success("import_subscription", "node-pool",
        format!("imported={}, filtered={}", imported_count, filtered_count));
<!-- TODO id:11;status:open;date:2026-06-09T14:30 代码示例中多处使用.lock().expect("lock")，生产环境Mutex poison会导致panic，应改为.lock().map_err(|e| format!("lock failed: {}", e))? -->

    Ok(SubscriptionResponse {
        imported_count,
        filtered_count,
        unsupported_protocols: parse_result.unsupported_nodes.iter()
            .map(|n| n.protocol_name.clone()).collect(),
        ..parse_result.into()
    })
}
<!-- ? id:12;status:open;date:2026-06-09T14:30 导入流程缺少部分失败策略：若add_node成功N个但后续regenerate或reload失败，已添加到NodePool的节点不会回滚。建议补充事务性语义：先写入临时缓冲区，regenerate+reload成功后才commit到NodePool，失败则丢弃 -->
```

#### 2.2.3 NodeHealthChecker 实现

当前 `NodeHealthChecker` 仅为 trait + Mock 实现。新增 `NodeHealthCheckerImpl`：

```rust
struct NodeHealthCheckerImpl {
    mihomo_manager: Arc<Mutex<MihomoManager>>,
    timeout: Duration,
    failure_threshold: u32,
}

impl NodeHealthChecker for NodeHealthCheckerImpl {
    fn check_node(&self, node: &ProxyNode) -> NodeHealthResult {
        // 优先：mihomo REST API GET /proxies/:name/delay
        let api_result = self.check_via_mihomo_api(&node.name);
        if api_result.is_ok() {
            return api_result.unwrap();
        }
        // 降级：TCP connect 测试
        self.check_via_tcp(node.address)
    }
}
```

mihomo API 调用沿用 `MihomoManager` 中已有的原始 TCP HTTP 请求模式。

### 2.3 D-110-3: Wizard Step 3 手工调整引导（G110-3）

#### 2.3.1 当前问题

`WizardPage.tsx` 的 `initial-assessment` 步骤仅含一个简单 "点击评估" 按钮，缺少设计要求的 rich step-by-step 引导。

#### 2.3.2 修复方案

新增 `ManualAdjustGuide` 组件，作为 Wizard Step 3 的核心内容：

```tsx
// components/wizard/ManualAdjustGuide.tsx
interface AdjustmentItem {
  stateItemId: string;
  description: string;
  currentValue: string;
  idealValue: string;
  category: 'restorable' | 'detectable';
  suggestionType: 'command' | 'system-settings' | 'auto-adjust';
  suggestion: string;
  systemSettingsPath?: string;
}

interface Props {
  items: AdjustmentItem[];
  onAutoAdjust: () => void;
  onRecollect: () => void;
  onItemAdjusted: (itemId: string) => void;
  progress: { completed: number; total: number };
}
```

**组件结构**：

```
┌─ ManualAdjustGuide ─────────────────────────────────┐
│  [整体进度条] 3/8 项已调整                              │
│                                                        │
│  ┌─ AdjustmentItem ──────────────────────────┐        │
│  │ 系统代理设置 (Restorable)                   │        │
│  │ 当前: ProxyEnable=1, ProxyServer=...       │        │
│  │ 建议: 清除系统代理                           │        │
│  │ [一键复制命令] netsh winhttp reset proxy    │        │
│  │ [系统设置指引] 设置→网络→代理              │        │
│  │ [一键自动调整] ← 仅 Restorable 项可用       │        │
│  └─────────────────────────────────────────┘        │
│                                                        │
│  [一键自动调整所有 Restorable 项]                       │
│  [重新采集]                                           │
└────────────────────────────────────────────────────┘
```

**后端数据来源**：

新增 Tauri command `get_adjustment_suggestions`：

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn get_adjustment_suggestions(
    state: State<'_, AppState>,
) -> Result<Vec<AdjustmentSuggestion>, String> {
    let baseline_mgr = state.baseline_manager.lock().expect("lock");
    let snapshot = baseline_mgr.get_current_snapshot()?;

    // 对每个状态项生成调整建议
    let suggestions: Vec<AdjustmentSuggestion> = snapshot.items.iter()
        .filter(|item| is_suboptimal(item))
        .map(|item| generate_suggestion(item))
        .collect();

    Ok(suggestions)
}
```

```rust
struct AdjustmentSuggestion {
    state_item_id: String,
    description: String,
    current_value: String,
    ideal_value: String,
    category: String,
    suggestion_type: String,
    suggestion_text: String,
    system_settings_path: Option<String>,
    command_text: Option<String>,
}
```
<!-- ? id:19;status:open;date:2026-06-09T14:30 get_adjustment_suggestions依赖baseline_manager.get_current_snapshot()和is_suboptimal()判断，需确认：(1)Wizard Step 3到达时snapshot是否已采集完成 (2)is_suboptimal判断标准是否已在F001中实现 (3)generate_suggestion()函数是否需要新增实现 -->

**Wizard 步骤顺序修正**（G110-21 同步处理）：

```
Step 1: 欢迎 + 安装后网络评估
Step 2: 展示当前网络状态（分类展示）
Step 3: 手工调整引导 ← 本设计核心新增
Step 4: 确认 baseline（二次确认）
Step 5: 选择部署模式 ← 从 Step 2 移回此处
Step 6: 选择目标站点
Step 7: 完成
```

## 3. Batch B — P1 修改

### 3.1 D-110-4: 组件独立化（G110-4）

#### 3.1.1 拆分方案

| 当前页面文件 | 拆出组件 | 目标路径 |
|-------------|---------|----------|
| `SitesPage.tsx` 内嵌站点卡片 | `SiteCard` | `components/sites/SiteCard.tsx` |
| `RulesPage.tsx` 内嵌规则列表 | `RuleTable` | `components/rules/RuleTable.tsx` |
| `DiagnosticsPage.tsx` 内嵌诊断面板 | `DiagPanel` | `components/diagnostics/DiagPanel.tsx` |
| `DiagnosticsPage.tsx` 内嵌节点池 | `NodePoolTable` | `components/diagnostics/NodePoolTable.tsx` |
| `DiagnosticsPage.tsx` 内嵌审计日志 | `AuditLogTable` | `components/diagnostics/AuditLogTable.tsx` |
| `WizardPage.tsx` 内嵌向导 | `Wizard` + `WizardStep` | `components/wizard/Wizard.tsx` + `WizardStep.tsx` |

拆分原则：
1. 页面文件仅做 `<ComponentA /> <ComponentB />` 组合 + 数据绑定（Zustand store hooks）
2. 渲染逻辑全部移入独立组件
3. Props 类型从页面导出移入组件文件
4. 事件回调通过 store actions 传递，不通过 props drilling
<!-- ? id:18;status:open;date:2026-06-09T14:30 拆分原则中"不通过props drilling"与"页面仅做组件组合"可能矛盾——拆分后子组件如何获取数据？若全部通过store hook直连，页面层级的数据筛选/转换逻辑放在哪里？建议明确：纯展示组件用props，有状态交互的组件用store直连 -->

### 3.2 D-110-5: 五要素诊断完整性（G110-5）

#### 3.2.1 修改方案

后端已有 `FiveElementPrompt` 结构（`engines/site_rule_engine.rs`），前端 `DiagnosticsPage` 仅展示了 2/5 要素。

修改 `DiagPanel` 组件（拆分后）：

```tsx
// components/diagnostics/DiagPanel.tsx
interface FiveElementDiagProps {
  reason: string;
  attemptedActions: string[];
  attemptCount: { current: number; max: number };
  suggestedAction: string;
  needsManualIntervention: boolean;
}

function DiagPanel({ reason, attemptedActions, attemptCount, suggestedAction, needsManualIntervention }: FiveElementDiagProps) {
  return (
    <div className="diag-panel">
      <div className="diag-section">
        <h4>原因</h4>
        <p>{reason}</p>
      </div>
      <div className="diag-section">
        <h4>已尝试动作</h4>
        <ul>{attemptedActions.map(a => <li>{a}</li>)}</ul>
      </div>
      <div className="diag-section">
        <h4>尝试次数</h4>
        <p>{attemptCount.current}/{attemptCount.max}</p>
      </div>
      <div className="diag-section">
        <h4>建议动作</h4>
        <CodeBlock code={suggestedAction} />
      </div>
      <div className="diag-section">
        <h4>是否需要手动处理</h4>
        <StatusBadge type={needsManualIntervention ? 'warning' : 'success'}>
          {needsManualIntervention ? '需要手动处理' : '已自动恢复'}
        </StatusBadge>
      </div>
    </div>
  );
}
```

**IPC 数据映射**：`get_diagnosis` command 已返回 `FiveElementPrompt` 结构，前端 `types.ts` 已有对应类型定义，仅需在组件中完整使用。

### 3.3 D-110-6: 规则预览增强（G110-6）

#### 3.3.1 修改方案

`RuleTable` 组件（拆分后）需展示：
1. 用户覆盖标记—— `preview_rules` 返回的规则列表中 `source` 字段区分 `auto` / `user_override`
2. MATCH,DIRECT 兜底—— 固定展示在规则列表末尾
3. 待应用变更—— 比较当前生效规则 vs 预览规则，差异项高亮展示

```tsx
// components/rules/RuleTable.tsx
function RuleTable({ rules, pendingChanges }) {
  return (
    <>
      {rules.map(rule => (
        <RuleRow
          key={rule.domain}
          domain={rule.domain}
          strategy={rule.strategy}
          source={rule.source}  // 'auto' | 'user_override'
          isPending={pendingChanges?.includes(rule.domain)}
        />
      ))}
      <RuleRow domain="*" strategy="DIRECT" source="system" isSystemRule />
    </>
  );
}
```

### 3.4 D-110-7: 通知语义化（G110-7）

#### 3.4.1 修改方案

**NotifStore** 修改：

```ts
// stores/notif-store.ts — 类型修改
interface AppNotification {
  id: string;
  type: 'rule-rollback' | 'recovery' | 'audit-change' | 'node-pool';
  title: string;
  message: string;
  timestamp: Date;
  actions?: { label: string; command: string }[];
}
```

**Tauri 通知**（替代 Web Notification API）：

```ts
// lib/tauri-ipc.ts — 新增
export async function sendSystemNotification(title: string, body: string): Promise<void> {
  // 使用 Tauri notification plugin
  // import { sendNotification } from '@tauri-apps/plugin-notification';
  // 降级：Web Notification API fallback
  try {
    const { sendNotification } = await import('@tauri-apps/plugin-notification');
    sendNotification({ title, body });
  } catch {
    // Web Notification API fallback
    new Notification(title, { body });
  }
<!-- TODO id:14;status:open;date:2026-06-09T14:30 dynamic import + fallback模式下，sendNotification可能因权限被拒抛出异常（Linux/WSL无桌面通知服务），catch应区分"插件不可用"和"权限被拒"，后者需向用户展示权限引导提示而非静默降级 -->
}
```

需要在 `Cargo.toml` 添加 `tauri-plugin-notification` 依赖。

### 3.5 D-110-8: StatusBar 动态绑定（G110-8）

#### 3.5.1 修改方案

```tsx
// components/layout/StatusBar.tsx
function StatusBar() {
  const { mihomoRunning } = useServiceStore();
  const { hasBaseline, deviatedItems } = useBaselineStore();
  const { deploymentMode } = useSettingsStore(); // 或新增 DeploymentStore

  const serviceStatus = mihomoRunning ? '运行中' : '已停止';
  const baselineStatus = !hasBaseline ? '未确认' :
    deviatedItems.length > 0 ? '已偏离' : '已确认';

  return (
    <footer className="status-bar">
      <span>服务: {serviceStatus}</span>
      <span>Baseline: {baselineStatus}</span>
      <span>部署: {deploymentMode}</span>
    </footer>
  );
}
```

数据来源于 Tauri Event 推送（已在 F106 补齐），Store 中的状态已实时更新。

<!-- ? id:17;status:open;date:2026-06-09T14:30 StatusBar依赖useSettingsStore获取deploymentMode，但当前settingsStore是否已包含deploymentMode字段？若未包含需同步扩展Store，此处design未提及Store变更 -->

### 3.6 D-110-9: Header 状态指示灯+通知铃铛（G110-9）

#### 3.6.1 修改方案

```tsx
// components/layout/Header.tsx
function Header() {
  const { mihomoRunning } = useServiceStore();
  const { unreadCount } = useNotifStore();

  return (
    <header className="app-header">
      <h1>GoGuo</h1>
      <StatusBadge type={mihomoRunning ? 'success' : 'stopped'} />
      <button className="notification-bell" onClick={toggleNotifBar}>
        🔔 {unreadCount > 0 ? unreadCount : ''}
      </button>
      <button onClick={handleRestore}>立即恢复</button>
    </header>
  );
}
```

### 3.7 D-110-10: 仪表盘部署模式卡片（G110-10）

```tsx
// pages/DashboardPage.tsx — 新增卡片
<DeploymentModeCard
  mode={deploymentMode}
  windowsStatus={windowsConfigStatus}
  wslStatus={wslConfigStatus}
  hasDifference={hasCrossPlatformDifference}
/>
```

数据来源：`get_deployment_mode` + `get_state_summary` 返回的两侧配置状态。

### 3.8 D-110-11: 不可达站点内嵌诊断（G110-11）

```tsx
// components/sites/SiteCard.tsx
function SiteCard({ site, reachability }) {
  const [showDiag, setShowDiag] = useState(false);

  return (
    <div className="site-card">
      <SiteName>{site.name}</SiteName>
      <StatusBadge type={reachability.reachable ? 'success' : 'warning'} />
      {!reachability.reachable && (
        <button onClick={() => setShowDiag(true)}>查看诊断</button>
      )}
      {showDiag && <DiagPanel {...reachability.fiveElementPrompt} />}
    </div>
  );
}
```

### 3.9 D-110-12: NotifBar "查看全部"入口（G110-12）

```tsx
// components/shared/NotifBar.tsx — 新增
{unreadCount > MAX_VISIBLE && (
  <button onClick={navigateToFullNotifications}>
    查看全部通知 ({unreadCount} 条)
  </button>
)}
```

新增通知历史页面（或展开模式），支持按类型和时间筛选。

## 4. Batch C — P2 修改

### 4.1 D-110-13: IpScanner 候选 IP 扩展（G110-13）

`IpScanner::candidate_ips()` 当前仅返回 GitHub 候选 IP。修改为从 SiteDefinition 的 `IpDirect` 标记站点读取候选 IP：

```rust
fn candidate_ips(&self, site_id: &str) -> Vec<IpCandidate> {
    match site_id {
        "github" => GITHUB_CANDIDATES.clone(),
        other => {
            // 从 SiteDefinition 中读取 ip_direct_config.candidates
            // 若无配置 → 空列表 → 降级为 PROXY 策略
            self.site_store.get_ip_candidates(other)
        }
    }
}
```

SiteDefinition 结构扩展（可选字段）：

```rust
struct IpDirectConfig {
    candidates: Vec<IpCandidate>,
    verification_method: VerificationMethod,
}
```

### 4.2 D-110-14: Mihomo reload 响应解析（G110-14）

`MihomoManager::reload_config()` 当前仅检查响应状态码 200/204。修改为解析响应体：

```rust
fn reload_config(&mut self, yaml_path: &Path) -> Result<(), MihomoError> {
    // ... existing TCP request logic ...
    let response = read_http_response(tcp_stream);

    if response.status_code != 200 && response.status_code != 204 {
        // NEW: Extract error message from response body
        let error_msg = extract_error_from_body(&response.body);
        return Err(MihomoError::ReloadFailed(error_msg));
    }
    Ok(())
}

fn extract_error_from_body(body: &str) -> String {
    // mihomo API returns JSON: { "message": "error detail" }
    serde_json::from_str::<serde_json::Value>(body)
        .and_then(|v| v.get("message").and_then(|m| m.as_str()).ok_or(...))
        .unwrap_or(body)
}
```

### 4.3 D-110-15: 审计日志过滤 UI（G110-15）

`AuditLogTable` 组件（拆分后）新增筛选栏：

```tsx
// components/diagnostics/AuditLogTable.tsx
function AuditLogTable() {
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [actionType, setActionType] = useState('');

  const { loadMore, logs } = useDiagStore();

  const filteredLogs = applyFilters(logs, { dateFrom, dateTo, actionType });

  return (
    <>
      <div className="audit-filters">
        <DateRangePicker from={dateFrom} to={dateTo} />
        <ActionTypeSelect value={actionType} />
      </div>
      <table>{filteredLogs.map(log => <AuditRow {...log} />)}</table>
      <button onClick={loadMore}>加载更多</button>
    </>
  );
}
```

后端 `get_audit_log` 已支持 `action_type/from/to` 过滤参数（F109 补齐）。

### 4.4 D-110-16: NodePoolTable 元数据补齐（G110-16）

后端 `get_node_pool_status` 返回的 `NodeInfo` 已含元数据字段（joined_at、last_check、last_check_method），前端仅需在组件中展示：

```tsx
// components/diagnostics/NodePoolTable.tsx
function NodePoolTable({ nodes }) {
  return (
    <table>
      <thead>
        <tr>
          <th>节点名称</th>
          <th>协议</th>
          <th>状态</th>
          <th>入池时间</th>
          <th>最近检测</th>
          <th>检测方式</th>
          <th>延迟</th>
        </tr>
      </thead>
      <tbody>
        {nodes.map(node => (
          <tr key={node.name}>
            <td>{node.name}</td>
            <td>{node.protocol}</td>
            <td><StatusBadge type={node.status} /></td>
            <td>{formatDate(node.joinedAt)}</td>
            <td>{formatDate(node.lastCheck)}</td>
            <td>{node.lastCheckMethod}</td>
            <td>{node.latencyMs}ms</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
```

### 4.5 D-110-17: Settings 探测+通知配置（G110-17）

```tsx
// pages/SettingsPage.tsx — 新增配置项
<div className="settings-section">
  <h3>探测配置</h3>
  <label>探测间隔（秒）</label>
  <input type="number" value={probeInterval} onChange={updateProbeInterval} />
</div>

<div className="settings-section">
  <h3>通知偏好</h3>
  <checkbox checked={enableInAppNotif}>应用内通知</checkbox>
  <checkbox checked={enableSystemNotif}>系统通知</checkbox>
</div>
```

后端需新增 Tauri command `update_app_config`，持久化修改到 `AppConfig`。

### 4.6 D-110-18: CodeBlock 语法高亮（G110-18）

引入轻量语法高亮库（如 `highlight.js` 或 `prism-js`，需检查与 shadcn/ui 兼容性）：

```tsx
// components/shared/CodeBlock.tsx
import Highlight from 'highlight.js';

function CodeBlock({ code, language = 'shell' }) {
  const highlighted = Highlight.highlight(code, { language }).value;

  return (
    <div className="code-block">
      <pre><code dangerouslySetInnerHTML={{ __html: highlighted }} /></pre>
      <button onClick={() => navigator.clipboard.writeText(code)}>复制</button>
    </div>
  );
}
```
<!-- TODO id:15;status:open;date:2026-06-09T14:30 dangerouslySetInnerHTML直接渲染highlight.js输出存在XSS风险，虽然highlight.js默认转义HTML实体但仍建议增加sanitize步骤，或改用React组件式高亮方案（如react-syntax-highlighter）避免innerHTML -->

### 4.7 D-110-19: UI 术语对齐（G110-19）

| 当前 | 修改后 | 文件 |
|------|--------|------|
| "站点管理" | "需要访问的网站" | SitesPage、Sidebar |
| "规则预览" | "代理规则" | RulesPage、Sidebar |
| "诊断" | "网站状态" | DiagnosticsPage、Sidebar |
| "代理节点池" / "节点" | "访问通道" / "通道" | NodePoolTable、DiagnosticsPage |

修改仅涉及文案常量，不涉及逻辑变更。

### 4.8 D-110-20: RecoveryAckDialog 文案对齐（G110-20）

```tsx
// components/shared/RecoveryAckDialog.tsx — 按钮文案修改
<button onClick={handleRetry}>重新恢复</button>      // 原 "重试"
<button onClick={handleAcknowledge}>确认已修复</button>  // 原 "确认"
```

### 4.9 D-110-22: apply_rules 独立命令（G110-22）

新增 Tauri command `apply_rules`：

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn apply_rules(
    state: State<'_, SiteRulesState>,
    app: AppHandle,
) -> Result<ApplyRulesResponse, String> {
    let engine = state.site_rule_engine.lock().expect("lock");

    // 1. Generate rules
    let rules = engine.generate_rules()?;

    // 2. Static validation (MATCH,DIRECT check)
    engine.validate_match_direct(&rules)?;

    // 3. Pre-apply probe (non-target sites, DIRECT)
    let pre_results = engine.probe_non_target_sites_direct();

    // 4. Write config + reload mihomo
    let mihomo_mgr = state.mihomo_manager.lock().expect("lock");
    mihomo_mgr.write_config_file("config.yaml", &rules.to_yaml())?;
    mihomo_mgr.reload_config(&config_path)?;

    // 5. Post-apply probe
    let post_results = engine.probe_non_target_sites_via_proxy();

    // 6. Compare + rollback if needed
    if any_regression(&pre_results, &post_results) {
        mihomo_mgr.rollback_config()?;
        return Ok(ApplyRulesResponse { success: false, rollback: true, ... });
    }

    Ok(ApplyRulesResponse { success: true, rollback: false, ... })
}
```

<!-- TODO id:20;status:open;date:2026-06-09T14:30 apply_rules涉及多把锁（site_rule_engine + mihomo_manager），且与import_subscription可能并发调用（用户导入订阅后立即手动应用规则），需确认锁获取顺序一致以避免死锁。建议在design中明确全局锁顺序约定 -->

前端新增 IPC 函数和 UI 入口（规则预览页"应用规则"按钮）。

### 4.10 D-110-23: shadcn/ui 引入（G110-23）

#### 4.10.1 渐进迁移策略

分两阶段：
1. **Phase 1（本 feature）**: 安装 shadcn/ui + 替换核心交互组件（ConfirmDialog、StatusBadge、NotifBar、CodeBlock、RecoveryOverlay）+ 设置 CSS 变量体系
2. **Phase 2（后续）**: 页面级组件逐步迁移

**Phase 1 执行步骤**：

```
1. pnpm dlx shadcn@latest init
   - 样式: New York
   - 基色: Slate (暗色主题对齐)
   - CSS 变量: yes
   
2. pnpm dlx shadcn@latest add dialog badge alert code-block overlay
   - 替换现有 ConfirmDialog → shadcn Dialog
   - 替换 StatusBadge → shadcn Badge (扩展语义化 type)
   - 替换 NotifBar → shadcn Alert + 自定义组合
   - 替换 CodeBlock → shadcn CodeBlock 或自定义 (语法高亮)
   - 替换 RecoveryOverlay → shadcn Overlay (自定义蒙层)

3. 全局样式迁移
   - globals.css 中的手写 CSS → shadcn/ui CSS 变量体系
   - 暗色主题通过 shadcn/ui dark mode 变量管理
```
<!-- ? id:16;status:open;date:2026-06-09T14:30 Phase 1替换5个核心组件+全局CSS变量体系迁移，变更面仍然很大。结合shadcn/ui初始化会修改globals.css和tailwind配置，建议Phase 0仅安装shadcn/ui+设置CSS变量映射层，组件替换放在后续迭代逐个进行 -->

**与现有暗色主题对齐**：

当前 `globals.css` 定义了暗色主题颜色。迁移策略：
- 保留暗色主题语义
- 将硬编码颜色值映射到 shadcn/ui CSS 变量
- 例如: `--goguo-bg: #0a0a0f` → `--background: 222 84% 4.9%`（HSL 格式）

### 3.10 D-110-24: IpDirect 运行时健康维护（G110-24）

#### 3.10.1 当前问题

IpDirect 策略的核心链路：`IpScanner` 扫描域名→ `IpCache` 持久化 → `MihomoConfigManager` 生成 `hosts:` 映射 + `DOMAIN-SUFFIX,*,DIRECT` 规则 → mihomo 热重载。但存在 4 个健康维护 gap：

1. **`refresh_ip_cache()` 未集成到任何定时任务**：方法存在（`site_rule_engine.rs:596`）但无调用方
2. **探测失败无 IP 重扫钩子**：`ProbeService` 失败后无回调机制，IpDirect 站点不可达时 IP 缓存不会更新
3. **无 fallback 路径**：IP 全不可达时仍强制 `DIRECT`，`gh auth login` 等工具报 `unexpected EOF`
4. **休眠唤醒后 IP 可能过期**：`proxy_guard_loop` 已有 wake 检测（`detect_wake_from_sleep`），但未触发 IP 缓存刷新

#### 3.10.2 修改方案

##### A. 扩展 `proxy_guard_loop` 集成 IP 健康维护

在 `proxy_guard_loop`（`commands/baseline.rs:1089`）的 `Healthy` 分支中添加 IP 缓存定期刷新。利用已有的 3s 循环间隔 + wall-clock elapsed 时间：

```rust
// proxy_guard_loop — Healthy 分支扩展
// 在 "GuardAction::Healthy => {}" 内添加：

// G110-24: 定期 IP 缓存刷新（每 IP_REFRESH_INTERVAL_SECS）
let now = std::time::SystemTime::now();
if let Some(last) = state.last_ip_refresh.lock().expect("lock").as_ref() {
    let since = now.duration_since(*last).unwrap_or_default();
    if since >= std::time::Duration::from_secs(IP_REFRESH_INTERVAL_SECS) {
        let mut engine = state.site_rules_state.lock().expect("lock");
        engine.refresh_ip_cache_and_apply();
        *state.last_ip_refresh.lock().expect("lock") = Some(now);
    }
} else {
    // 首次初始化
    *state.last_ip_refresh.lock().expect("lock") = Some(now);
}
```

**定时策略**：
- `IP_REFRESH_INTERVAL_SECS = 4 * 3600`（4 小时，= 24h TTL / 6，确保 TTL 过半前刷新）
- 基于 wall-clock 而非 loop 计数，休眠期间不会误触发

##### B. 休眠唤醒时强制 IP 刷新

在 `proxy_guard_loop` 已有的 `woke_from_sleep` 分支（`baseline.rs:1120`）中，紧跟 `flush_urltest_groups` 之后添加 IP 刷新：

```rust
// 已有代码: flush_urltest_groups(...)

// G110-24: 唤醒后强制刷新 IP 缓存 + 重载配置
{
    let mut engine = state.site_rules_state.lock().expect("lock");
    engine.refresh_ip_cache_and_apply();
    eprintln!("[GoGuo] Post-wake IP cache refreshed and config reloaded");
}
```

##### C. 探测失败触发 IP 重扫 + fallback

扩展 `SiteRuleEngine` 的 `get_reachability()` 方法（被前端定时轮询调用）。在探测结果中发现 IpDirect 站点不可达时：

```rust
// SiteRuleEngine 新增方法
pub fn get_reachability(&mut self) -> Vec<SiteReachability> {
    let results = self.probe_service.probe_all();

    // G110-24: IpDirect 站点不可达时触发 IP 重扫
    let unreachable_ip_direct: Vec<&str> = results.iter()
        .filter(|r| !r.reachable)
        .filter(|r| self.is_ip_direct_site(&r.site_id))
        .map(|r| r.site_id.as_str())
        .collect();

    if !unreachable_ip_direct.is_empty() {
        self.handle_ip_direct_failure(&unreachable_ip_direct);
    }

    results.into_iter().map(|r| SiteReachability { ... }).collect()
}

fn is_ip_direct_site(&self, site_id: &str) -> bool {
    self.site_store.get(site_id)
        .is_some_and(|s| s.access_strategy == AccessStrategy::IpDirect)
}

fn handle_ip_direct_failure(&mut self, site_ids: &[&str]) {
    // 1. 强制重扫（忽略缓存 TTL）
    let domains: Vec<String> = site_ids.iter()
        .filter_map(|id| self.site_store.get(id))
        .filter(|s| s.access_strategy == AccessStrategy::IpDirect)
        .flat_map(|s| s.all_domains())
        .collect();

    let fresh = self.ip_scanner.scan_domains(&domains);

    // 2. 判断是否所有 IP 均不可用
    let any_valid = !fresh.is_empty();
    if any_valid {
        // 有新 IP → 更新缓存 + 热重载
        for (domain, ip) in &fresh {
            self.ip_cache.update(domain.clone(), ip.clone());
        }
        let _ = self.ip_cache.save(&self.ip_cache_file);
        self.reapply_ip_direct_config();
    } else {
        // 全不可达 → fallback: 切换到 PROXY
        self.fallback_to_proxy(site_ids);
    }
}
```

##### D. DIRECT→PROXY fallback 实现

当 `IpScanner` 无法为 IpDirect 站点找到任何可用 IP 时，将该站点的 mihomo 规则临时从 `DIRECT` 切换为 `PROXY`（使用站点的 proxy group）：

```rust
// SiteRuleEngine 新增
fn fallback_to_proxy(&mut self, site_ids: &[&str]) {
    // 标记站点为 fallback 模式（内存状态，不持久化到 SiteDefinition）
    for site_id in site_ids {
        self.ip_direct_fallback.insert(site_id.to_string(), true);
    }

    // 审计日志
    if let Some(ref logger) = self.audit_logger {
        let _ = logger.log_success(
            AuditAction::IpDirectFallback,
            &site_ids.join(","),
            serde_json::json!({ "reason": "all IPs unreachable" }),
        );
    }

    // 重新生成配置（fallback 模式下跳过 hosts + 内联 DIRECT 规则）
    self.reapply_ip_direct_config();
}
```

**`apply_rules_to_mihomo` 修改**：扫描阶段检查 fallback 标记：

```rust
// scan_ip_direct_sites 扩展
fn scan_ip_direct_sites(&mut self) -> (HashMap<String, String>, Vec<String>) {
    // ... 现有逻辑 ...

    // G110-24: fallback 站点不生成 hosts/DIRECT 规则，走 proxy group
    let cached = self.ip_cache.get_all_valid();
    for domain in &ip_direct_domains {
        // 检查该域名所属站点是否在 fallback 模式
        let in_fallback = self.is_domain_in_fallback(domain);
        if let Some(ip) = cached.get(domain) {
            if !in_fallback {
                direct_domains.push(domain.clone());
                // ... hosts_key 逻辑不变 ...
            }
        }
    }

    (ip_hosts, direct_domains)
}
```

**fallback 恢复**：下次 `refresh_ip_cache_and_apply()` 发现有效 IP 时自动恢复：

```rust
pub fn refresh_ip_cache_and_apply(&mut self) {
    let mut ip_direct_domains: Vec<String> = Vec::new();
    for site_id in &self.active_sites {
        if let Some(site) = self.site_store.get(site_id) {
            if site.access_strategy == AccessStrategy::IpDirect {
                ip_direct_domains.extend(site.all_domains());
            }
        }
    }

    if ip_direct_domains.is_empty() {
        return;
    }

    // 强制重扫（忽略缓存）
    let fresh = self.ip_scanner.scan_domains(&ip_direct_domains);
    let has_valid = !fresh.is_empty();

    for (domain, ip) in &fresh {
        self.ip_cache.update(domain.clone(), ip.clone());
    }
    let _ = self.ip_cache.save(&self.ip_cache_file);

    if has_valid {
        // 有效 IP → 清除 fallback 标记
        self.ip_direct_fallback.clear();
    }

    // 重新应用配置
    self.reapply_ip_direct_config();
}
```

##### E. 新增数据结构

```rust
// site_rule_engine.rs — SiteRuleEngine 新增字段
pub struct SiteRuleEngine {
    // ... 现有字段 ...
    /// G110-24: IpDirect fallback 状态（site_id → true 表示正在使用 PROXY 替代）
    ip_direct_fallback: HashMap<String, bool>,
}
```

```rust
// AppState 新增（commands/baseline.rs）
pub struct AppState {
    // ... 现有字段 ...
    /// G110-24: 上次 IP 缓存刷新时间
    pub last_ip_refresh: Mutex<Option<std::time::SystemTime>>,
}
```

```rust
// AuditAction 新增（models/audit.rs）
pub enum AuditAction {
    // ... 现有变体 ...
    /// G110-24: IpDirect 站点因 IP 不可用而 fallback 到 PROXY
    IpDirectFallback,
    /// G110-24: IpDirect IP 缓存刷新
    IpDirectCacheRefresh,
}
```

#### 3.10.3 不修改的内容

- `IpScanner` 内部逻辑不变（扫描+验证机制已成熟）
- `IpCache` 结构不变（仅 `DEFAULT_TTL_SECS` 不变，刷新策略由外部调度）
- `MihomoConfigManager` 不变（仅接收不同的 `ip_direct_hosts` / `direct_domains` 输入）
- `SleepWakeService` 不变（已通过 `proxy_guard_loop` 的 wake 检测间接使用）
- `ProbeService` 不变（不添加回调钩子，改为在调用方 `get_reachability()` 中检测失败后触发）

#### 3.10.4 关键流程

```
proxy_guard_loop (每 3s)
  ├─ wake_from_sleep?
  │   └─ YES → refresh_ip_cache_and_apply() ← 强制刷新
  ├─ last_ip_refresh >= 4h?
  │   └─ YES → refresh_ip_cache_and_apply() ← 定期刷新
  └─ Healthy → continue

前端轮询 get_reachability()
  ├─ probe_all() → 结果
  ├─ 发现 IpDirect 站点不可达
  │   ├─ 强制重扫 IP
  │   ├─ 有新 IP → 更新缓存 + 热重载
  │   └─ 无可用 IP → fallback 到 PROXY + 审计日志
  └─ 返回可达性结果

refresh_ip_cache_and_apply()
  ├─ scan_domains() → 获取新 IP
  ├─ 有有效 IP → 清除 fallback 标记
  ├─ 更新缓存 + 持久化
  └─ reapply_ip_direct_config() → 重生成 mihomo 配置 + 热重载
```

#### 3.10.5 配置常量

| 常量 | 值 | 说明 |
|------|----|------|
| `IP_REFRESH_INTERVAL_SECS` | 14400 (4h) | IP 缓存定期刷新间隔 = 24h TTL / 6 |
| `WAKE_IP_REFRESH_DEBOUNCE_SECS` | 30 | 唤醒后防抖（避免与定期刷新重复） |
| `PROBE_FAILURE_TRIGGER_THRESHOLD` | 1 | 连续探测失败几次后触发 IP 重扫 |

#### 3.10.6 审计日志

| 事件 | AuditAction | 详情 |
|------|------------|------|
| IP 缓存定期刷新 | `IpDirectCacheRefresh` | `{ "trigger": "periodic", "domains_refreshed": N }` |
| 唤醒后 IP 刷新 | `IpDirectCacheRefresh` | `{ "trigger": "post-wake", "domains_refreshed": N }` |
| 探测失败触发 IP 重扫 | `IpDirectCacheRefresh` | `{ "trigger": "probe-failure", "site_id": "..." }` |
| fallback 到 PROXY | `IpDirectFallback` | `{ "site_ids": [...], "reason": "all IPs unreachable" }` |

## 5. 数据结构变更

| 变更 | 类型 | 说明 |
|------|------|------|
| `RealProbeClient` | 新增 struct | 替代 MockProbeClient |
| `AdjustmentSuggestion` | 新增 struct | Wizard 手工调整引导数据 |
| `NodeHealthCheckerImpl` | 新增 struct | 替代 MockNodeHealthChecker |
| `SubscriptionResponse` | 扩展 | 新增 imported_count/filtered_count |
| `ApplyRulesResponse` | 新增 struct | apply_rules 命令返回 |
| `IpDirectConfig` | 新增 struct | SiteDefinition 候选 IP 配置 |
| `AppNotification.type` | 类型变更 | 从 info/success/warning/error → rule-rollback/recovery/audit-change/node-pool |
| `AppNotification.actions` | 新增字段 | 可操作通知入口 |
| `SiteRuleEngine.ip_direct_fallback` | 新增字段 | IpDirect fallback 状态（site_id → bool） |
| `AppState.last_ip_refresh` | 新增字段 | 上次 IP 缓存刷新时间（wall-clock） |
| `AuditAction::IpDirectFallback` | 新增变体 | IpDirect fallback 到 PROXY 审计 |
| `AuditAction::IpDirectCacheRefresh` | 新增变体 | IP 缓存刷新审计 |
<!-- ? id:21;status:open;date:2026-06-09T14:30 AppNotification.type从info/success/warning/error改为语义类型属于breaking change，design未提及前端Store中已有通知数据的迁移/兼容策略。建议：前端增加类型兼容层或清空历史通知 -->

## 6. 新增 Tauri Commands

| Command | 说明 | 对应需求 |
|---------|------|----------|
| `get_adjustment_suggestions` | 获取不理想状态项的调整建议 | FR-3.3.1 |
| `apply_rules` | 独立规则应用命令 | FR-3.22.1 |
| `update_app_config` | 更新用户配置（探测间隔、通知偏好） | FR-3.17.1 |

## 7. 关键流程

### 7.1 订阅导入 → 节点池 → 规则生效

```
用户输入订阅 URL
  → SubscriptionParser.import_from_url()
  → ParseResult { supported_nodes, unsupported_nodes }
  → 逐个 NodePool::add_node(supported)
  → MihomoConfigManager::regenerate(node_pool)
  → MihomoManager::write_config_file() + reload_config()
  → NodeHealthCheckerImpl::check_all()
  → ProbeService::probe_all() (验证可达性)
  → 审计日志记录
  → UI 更新节点池状态 + 可达性
```

### 7.2 Wizard 手工调整引导

```
用户进入 Step 3
  → get_adjustment_suggestions()
  → 展示 AdjustmentItem 列表（逐项）
  → 用户选择:
    a) 一键复制命令 → CodeBlock copy
    b) 系统设置指引 → 文字引导
    c) 一键自动调整 → triggerReadjustment (仅 Restorable)
    d) 手工调整后重新采集 → start_initial_assessment
  → 进度条更新
  → 完成 → 进入 Step 4 (确认 baseline)
```

## 8. 依赖变更

| 依赖 | 变更 | 说明 |
|------|------|------|
| `tauri-plugin-notification` | 新增 (Cargo.toml) | 系统通知推送 |
| `@tauri-apps/plugin-notification` | 新增 (package.json) | 前端通知 API |
| `shadcn/ui` | 新增 (package.json) | 组件库 |
| `highlight.js` 或 `prism-js` | 新增 (package.json) | 语法高亮 |

## 9. 风险雷达

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| RealProbeClient 代理探测实现复杂度（HTTP CONNECT + 响应解析） | 高 | 中 | 复用 MihomoManager 已有的原始 TCP HTTP 请求模式；若 3s 超时无法稳定实现，降级为 DNS+HEAD 直连（不走代理） |
| shadcn/ui 与现有暗色主题冲突 | 中 | 中 | Phase 1 仅替换核心交互组件，全局样式渐进迁移；保留 CSS 变量映射层 |
| 组件拆分引入渲染行为差异 | 中 | 低 | 拆分前后快照对比测试（vitest + jest-dom） |
| 节点导入后 mihomo 配置重载失败 | 中 | 中 | 重载失败时回退到 previous config + 通知用户 |
| tauri-plugin-notification 在 WSL/Linux 不可用 | 低 | 中 | Web Notification API 降级（与 F004 OP-3 对齐） |
| IpDirect fallback 频繁切换（IP 不稳定） | 中 | 低 | 添加 cooldown 机制：fallback 后 15min 内不尝试恢复 DIRECT，避免配置反复重载 |
| `get_reachability()` 中 IP 重扫阻塞前端响应 | 中 | 中 | `scan_domains` 已并行化且 2s 超时；若仍影响 UI，改为后台线程异步执行 + 状态标记 |
<!-- TODO id:22;status:open;date:2026-06-09T14:30 风险雷达缺少一项关键风险：23项gap在单一feature中实施，批次间存在依赖（如G110-4组件拆分是G110-5/6/8/9/11/15/16的前置条件），任一P0延期会级联阻塞P1/P2。建议补充"批次依赖图"明确关键路径 -->

## 10. 测试策略

| 层级 | 覆盖范围 | 方式 |
|------|----------|------|
| Rust 单元测试 | RealProbeClient 各级探测、NodeHealthCheckerImpl、SubscriptionParser→NodePool 管道、AdjustmentSuggestion 生成 | Mock TCP + 真实 localhost probe |
| Rust 集成测试 | import_subscription 闭环（解析→导入→重载→健康检查）、apply_rules 全流程 | 真实 mihomo 进程 |
| 前端单元测试 | 组件拆分行为等价、五要素渲染完整性、通知类型语义化、StatusBar 数据绑定 | vitest + jest-dom |
| 前端集成测试 | Wizard Step 3 手工调整引导交互、规则预览增强 | Playwright 或手动走查 |
| E2E 验证 | 订阅导入→站点可达→规则生效全链路、首次引导完整流程 | 手动验证 |

## 11. 不修改的内容

- F001~F004 设计规格本身（仅实现补齐）
- baseline 数据格式
- F101~F109 已覆盖的所有功能
- MockProbeClient 保留（用于开发/测试环境，生产用 RealProbeClient）
- `triggerReadjustment` 函数保留（Wizard 中仍需使用）
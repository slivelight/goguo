# Feature 003: 目标站点规则配置与可达性诊断 — 设计

- **Feature**: 003-site-rules
- **阶段**: `hf-design`
- **状态**: 草稿
- **日期**: 2026-05-12
- **上游输入**: `features/003-site-rules/spec.md`
- **关联 ADR**: ADR-0003, ADR-0004

## 1. 设计概述

Feature 003 实现 GoGuo 的核心产品价值——"只处理目标站点，不破坏非目标站点"。基于 Feature 001 的 baseline/restore 安全底座和 Feature 002 的跨平台能力，提供站点定义管理、mihomo 规则自动生成、目标站点可达性诊断和代理节点池管理。

## 2. 模块设计

### 2.1 SiteRuleEngine

**职责**：站点定义管理和 mihomo 规则生成。

```rust
struct SiteRuleEngine {
    site_store: SiteDefinitionStore,
    rule_generator: RuleGenerator,
    mihomo_manager: Arc<MihomoManager>,
    probe_service: Arc<ProbeService>,
    audit_logger: Arc<AuditLogger>,
}
```

**核心接口**：

| 方法 | 说明 | 对应需求 |
|------|------|----------|
| `add_site_by_id(id)` | 通过站点标识添加（如 "github"） | FR-2.1.1-R1 |
| `add_site_by_domain(domain)` | 通过域名添加 | FR-2.1.1-R2 |
| `remove_site(id)` | 删除目标站点 | FR-2.1.1-R4 |
| `apply_preset_template(template)` | 应用预设模板 | FR-2.1.2-R1 |
| `preview_rules()` | 预览即将生效的规则 | FR-2.4.1 |
| `apply_rules()` | 应用规则（需二次确认） | FR-2.2.1, FR-2.6.1-R1 |
| `get_user_overrides()` | 获取用户自定义规则覆盖 | FR-2.4.2 |
| `detect_related_domains(domain)` | 自动检测相关域名 | FR-2.1.1-R6 |

### 2.2 SiteDefinitionStore

**职责**：管理站点定义数据。

```rust
struct SiteDefinition {
    id: String,                           // "github"
    name: String,                         // "GitHub"
    domains: HashMap<DomainCategory, Vec<String>>,  // 分类域名
    health_check: Option<HealthCheckConfig>,
}

enum DomainCategory {
    Core, Api, Cdn, Assets, Services, Packages, ThirdParty, CrossDependency,
}
```

**存储**：
- 内置站点定义：编译时嵌入或 `data/config/site-definitions/*.json`
- 用户自定义：`data/config/site-definitions/custom/*.json`
- 预设模板定义：硬编码在应用中

**预设模板**：

| 模板 | 站点列表 | 状态 |
|------|----------|------|
| 开发者套装 | github, npmjs, claude, chatgpt(含OpenAI), oracle, docker, stackoverflow, pypi, crates | 6 个有完整定义，4 个需补充 |
| 办公套装 | google(含gmail等), wikipedia, WhatsApp, Instagram, Canva, Twitter-X | 1 个有完整定义，5 个需补充 |

### 2.3 RuleGenerator

**职责**：基于站点定义生成 mihomo YAML 配置。

**规则生成逻辑**：

```yaml
# 生成的 mihomo 配置结构
rules:
  # 目标站点规则（按站点分组）
  - DOMAIN-SUFFIX,github.com,PROXY
  - DOMAIN-SUFFIX,github.io,PROXY
  - DOMAIN-SUFFIX,githubusercontent.com,PROXY
  # ... 更多目标站点域名

  # 用户自定义规则覆盖（优先于自动生成）
  # - DOMAIN,custom.example.com,PROXY

  # 默认直连兜底
  - MATCH,DIRECT
```
<!-- [?] id:01;status:close;date:2026-05-14T16:53  是否允许不同的目标站点，使用不同的代理服务器的配置？；回答：MVP 阶段所有目标站点共享单一 PROXY 代理组（当前设计）。mihomo 原生支持多 proxy-group，未来可按站点分配不同代理组（例如 github 用节点 A、google 用节点 B）而无需修改 SiteRuleEngine 核心逻辑——仅需在 RuleGenerator 生成时将规则指向不同 proxy-group 名称。当前不实现此能力，避免过早增加配置复杂度。-->

**生成流程**：
1. 读取当前站点列表
2. 按站点定义展开所有域名
3. 为每个域名生成 `DOMAIN` 或 `DOMAIN-SUFFIX` 规则
4. 合并用户自定义规则覆盖（标记为不可覆盖）
5. 添加 `MATCH,DIRECT` 兜底
6. **静态校验**：验证最后一条规则为 `MATCH,DIRECT`（C2 不变量）
7. 写入 `data/rules/current-rules.yaml`
8. 通过 MihomoManager 热重载配置
9. **A/B 即时探测验证**（详见 §5.3）
10. 若验证失败 → 回退到 `data/rules/previous-rules.yaml`

**规则回退机制**：
- 应用新规则前，当前规则备份为 `previous-rules.yaml`
- 非目标站点 A/B 探测验证失败 → 自动回退
- 回退后通知用户（FR-2.2.2-R3）

### 2.4 ProbeService

**职责**：目标站点可达性并行探测。

```rust
struct ProbeService {
    mihomo_manager: Arc<MihomoManager>,
    config: ProbeConfig,
    history: ProbeHistory,
}

struct ProbeConfig {
    interval: Duration,          // 默认 30s（设计阶段确定）
    timeout: Duration,           // 默认 3s（NFR-3.1-3）
    max_concurrent: usize,       // 并行探测上限
    failure_threshold: u32,      // 连续失败次数触发恢复
    degraded_interval: Duration, // 不可达站点降低频率
}
```

**探测策略**：

分层探测，覆盖浏览器访问完整链路（DNS → TCP → TLS → HTTP）：

| 层级 | 触发条件 | 探测方式 | 说明 |
|------|----------|----------|------|
| Level 1 | 默认 | DNS 解析 + HTTP HEAD | 快速验证基本可达性（~1s） |
| Level 2 | 常规探测 | HTTP GET + 状态码校验（2xx/3xx） | 确认页面可访问（~3s） |
| Level 3 | 不可达时 | TLS 握手验证 + 响应体大小检查 | 定位具体失败环节 |

- 所有探测通过 mihomo 代理执行（反映真实用户体验）
- 并行探测所有目标站点（NFR-3.1-4）
- 记录响应时间
- 不可达站点切换到降级探测频率
<!-- [?] id:02;status:close;date:2026-05-14T16:58  当前的’使用 HTTP HEAD/GET 请求测试站点可达性’和’通过 mihomo 代理执行探测（反映真实用户体验）’两种方法，是否能确保用户在浏览器（至少）上能正常访问目标站点？是否还有更完备的探测策略？；回答：补充分层探测策略。Level 1（默认）：DNS 解析 + HTTP HEAD（快速验证基本可达性）；Level 2（常规）：HTTP GET + 状态码校验（2xx/3xx = OK，确认页面可访问）；Level 3（诊断，不可达时触发）：TLS 握手验证、响应体大小检查。ProbeMethod 枚举新增 TlsHandshake 变体。浏览器访问涉及 DNS → TCP → TLS → HTTP 完整链路，Level 1+2 覆盖前三层和 HTTP 响应码，Level 3 定位具体失败环节。-->

**探测结果数据模型**：

```rust
struct ProbeResult {
    site_id: String,
    timestamp: DateTime<Utc>,
    reachable: bool,
    response_time: Option<Duration>,
    error: Option<String>,
    probe_method: ProbeMethod,   // HttpHead / HttpGet / DnsResolve / TlsHandshake
}

enum ProbeMethod {
    HttpHead,
    HttpGet,
    DnsResolve,
    TlsHandshake,
}

/// 探测历史记录（环形缓冲，仅保留最近 N 条）
struct ProbeHistory {
    records: VecDeque<ProbeResult>,  // 按 timestamp 排序
    max_size: usize,                 // 默认 1000 条
}
```

**ProbeHistory 存储**：内存中保持环形缓冲（避免无限增长），应用重启后从 `data/state/probe-history.jsonl` 恢复最近记录。

### 2.5 NodePool

**职责**：代理节点池管理、健康检查、退出机制。
<!-- [?] id:03;status:close;date:2026-05-14T17:06  在前期demo项目/mnt/d/software/github-host中，曾经设计了主备mihomo进程，主进程负责目标站点的正常代理，备进程负责代理节点池中代理节点的搜索，评估，加入，检测，退出等的生命周期的管理，请确认一下，goguo中是否要这样的主备进程设计？进行评估分析，允许说不。如果要调整，需要同步更改相关features的design，保证设计的一致性；回答：GoGuo 不采用主备 mihomo 进程设计。理由：(1) mihomo 内置 proxy-group 健康检查（url-test / fallback），可直接用于节点探测；(2) NodePool 健康检查通过 mihomo REST API `GET /proxies/:name/delay` 获取延迟，或使用 TCP 直连检测，无需额外 mihomo 实例；(3) 桌面应用资源受限，双进程占用双倍内存和端口；(4) ProxyGuard 已承担进程监控和恢复职责。NodePool 的节点搜索、评估、退出等生命周期管理在 Rust 后端（NodePool 模块内）完成，不依赖独立 mihomo 进程。-->
```rust
struct NodePool {
    nodes: Vec<ProxyNode>,
    current_index: usize,
    health_checker: NodeHealthChecker,
    audit_logger: Arc<AuditLogger>,
}

/// 节点健康检查器
struct NodeHealthChecker {
    mihomo_manager: Arc<MihomoManager>,
    check_interval: Duration,       // 默认 60s
    timeout: Duration,              // 默认 3s
    failure_threshold: u32,         // 连续失败次数触发移除（默认 3）
}

impl NodeHealthChecker {
    /// 通过 mihomo REST API 测试节点延迟
    fn check_via_mihomo_api(&self, node_name: &str) -> Result<Duration>;

    /// TCP 直连测试节点可用性
    fn check_via_tcp(&self, address: SocketAddr) -> Result<Duration>;

    /// 执行一次健康检查（优先 mihomo API，失败降级为 TCP）
    fn check_node(&self, node: &ProxyNode) -> NodeHealthResult;
}

struct NodeHealthResult {
    reachable: bool,
    latency: Option<Duration>,
    method: String,    // "mihomo-api" / "tcp-connect"
    checked_at: DateTime<Utc>,
}

struct ProxyNode {
    name: String,
    address: SocketAddr,
    protocol: ProxyProtocol,     // Vless / Vmess / Trojan / Shadowsocks / Hysteria2
    joined_at: DateTime<Utc>,
    status: NodeStatus,
    last_check: Option<DateTime<Utc>>,
    last_check_method: Option<String>,
}

enum NodeStatus {
    Available,
    Unhealthy,
    Removed,
}
```
<!-- [?] id:04;status:close;date:2026-05-14T17:12  确认'ProxyNode.protocol'这些协议都能被mihomo进程所解析，不支持的协议如何处理？；回答：当前枚举的 5 种协议（Vless/Vmess/Trojan/Shadowsocks/Hysteria2）均为 mihomo（Clash.Meta）原生支持的协议。不支持的协议处理：(1) 订阅解析阶段过滤未知协议节点，记录被过滤的节点名称和协议类型；(2) 审计日志记录过滤行为；(3) UI 节点池表格展示被过滤节点（标记"不支持的协议"），用户可感知。-->

**核心操作**：

| 操作 | 说明 | 对应需求 |
|------|------|----------|
| `switch_node()` | 切换到下一个可用节点 | FR-2.3.2-R1 |
| `health_check_all()` | 对所有节点执行健康检查 | FR-2.3.2-R6 |
| `remove_unhealthy(node)` | 移除连续失败节点 | FR-2.3.2-R7 |
| `get_pool_status()` | 获取节点池整体状态 | FR-2.6.3-R1 |
| `get_node_metadata(node)` | 获取单个节点元数据 | FR-2.6.3-R3 |

**节点元数据**（FR-2.6.3-R3）：
- 节点名称
- 入池时间（`joined_at`）
- 当前可用状态（`status`）
- 最近可用检测时间（`last_check`）
- 可用检测方式（`last_check_method`：TCP connect / HTTP probe / mihomo API）

**退出机制**：
1. 连续 N 次健康检查失败（默认 N=3）
2. 标记节点为 `Removed`
3. 记入审计
4. 从可用池中移除
5. 通知 UI 更新节点池状态

### 2.6 SubscriptionParser

**职责**：解析代理节点订阅链接，导入节点到 NodePool。

```rust
struct SubscriptionParser {
    supported_protocols: Vec<ProxyProtocol>,  // Vless/Vmess/Trojan/Shadowsocks/Hysteria2
    audit_logger: Arc<AuditLogger>,
}

struct SubscriptionSource {
    url: String,
    added_at: DateTime<Utc>,
    last_update: Option<DateTime<Utc>>,
}
```

**核心操作**：

| 操作 | 说明 |
|------|------|
| `import_from_url(url)` | 从订阅链接拉取并解析节点列表 |
| `import_from_file(path)` | 从本地文件解析节点列表 |
| `parse_nodes(raw: &str)` | 解析订阅内容（base64 解码 → 节点列表） |
| `filter_supported(nodes)` | 过滤不支持的协议节点 |

**不支持协议处理**：
1. 订阅解析阶段过滤未知协议节点
2. 记录被过滤的节点名称和协议类型到审计日志
3. UI 节点池表格展示被过滤节点（标记"不支持的协议"）

**存储**：
- 订阅源列表：`data/config/subscription-sources.json`
- 节点数据写入 mihomo 配置（由 RuleGenerator 生成）

## 3. 数据模型

### 3.1 mihomo 配置生成

mihomo 配置文件（`data/mihomo/config.yaml`）由 RuleGenerator 动态生成，包含：

```yaml
# 基础配置（由 MihomoManager 管理）
mixed-port: 7890
allow-lan: false
mode: rule
log-level: warning
external-controller: 127.0.0.1:9090
secret: "<generated-secret>"

# 代理节点（从 NodePool 生成）
proxies:
  - name: "node-1"
    type: vless
    server: ...
    # ...

# 代理组
proxy-groups:
  - name: "PROXY"
    type: select
    proxies: ["node-1", "node-2", ...]

# 规则（从 SiteRuleEngine 生成）
rules:
  - DOMAIN-SUFFIX,github.com,PROXY
  # ...
  - MATCH,DIRECT
```

## 4. Tauri Commands

| Command | 说明 | 对应需求 |
|---------|------|----------|
| `add_target_site` | 添加目标站点 | FR-2.1.1 |
| `remove_target_site` | 删除目标站点 | FR-2.1.1-R4 |
| `apply_preset_template` | 应用预设模板 | FR-2.1.2 |
| `preview_rules` | 预览即将生效的规则 | FR-2.4.1 |
| `apply_rules` | 应用规则（需二次确认） | FR-2.6.1 |
| `get_site_reachability` | 获取站点可达性状态 | FR-2.6.1 |
| `get_diagnosis` | 获取不可达站点诊断 | FR-2.6.2 |
| `get_node_pool_status` | 获取节点池状态 | FR-2.6.3 |
| `override_rule` | 用户覆盖规则 | FR-2.4.2 |
| `import_subscription` | 从订阅链接导入节点 | FR-2.3.2 |
| `get_subscription_sources` | 获取订阅源列表 | FR-2.3.2 |

## 5. 关键流程

### 5.1 站点添加 → 规则生效

1. 用户输入站点标识（如 "github"）或域名
2. SiteRuleEngine 匹配站点定义 → 展开域名
3. 若无匹配 → FR-2.1.1-R6 自动检测相关域名 + 用户确认
4. UI 展示域名列表预览
5. 用户确认 → 触发规则重新生成
6. 规则预览 → 用户二次确认 → 应用
7. 应用后执行 A/B 即时探测验证（§5.3）
8. 若非目标站点可达性下降 → 自动回退 + 通知用户

### 5.2 不可达恢复

1. ProbeService 定期探测目标站点
2. 某站点不可达 → 尝试切换代理节点
3. 切换后验证可达性
4. 仍不可达 → 生成五要素诊断提示
5. 持续不可达超过阈值 → 降低探测频率
6. 所有操作记入审计

### 5.3 非目标站点可达性验证（A/B 即时探测）

**目的**：规则应用后验证非目标站点不受影响（SC-2、FR-2.2.2-R2），不依赖 baseline 历史数据。

**验证流程**：

```
Step 1: 静态校验（零成本不变量守卫）
  └─ 检查生成规则的最后一条 = MATCH,DIRECT（C2）
  └─ 若不满足 → 拒绝应用，无需探测

Step 2: Pre-apply 探测（DIRECT，不经 mihomo）
  └─ 探测一组非目标参考站点（配置项，默认国内常用站点）
  └─ 记录每个站点的 reachable + response_time → pre_results

Step 3: 应用规则（mihomo 配置热重载）

Step 4: Post-apply 探测（经 mihomo DIRECT 出口）
  └─ 探测同样的非目标参考站点
  └─ 记录每个站点的 reachable + response_time → post_results

Step 5: 对比判断
  └─ 任一站点：pre reachable && post unreachable → 触发回退
  └─ 若全部 post reachable → 验证通过
```

**参考站点列表**：由 `settings.json` 配置（`non_target_probe_sites`），默认值 `["baidu.com", "qq.com"]`，用户可自定义。站点列表仅为验证用途，不存入 baseline。

**设计要点**：
- 不修改 BaselineSnapshot 数据模型——baseline 保持"配置状态快照"的纯粹语义
- 前后探测在同一网络条件下完成（秒级间隔），避免历史基线时效性问题
- 静态校验作为第一道防线，A/B 探测作为端到端验证
- 探测超时与 ProbeService 一致（默认 3s，NFR-3.1-3）

## 6. 约束与不变量

- **C1**: 规则策略为"默认 DIRECT + 目标 PROXY"（Feature 003 CON-1）
- **C2**: 规则末尾必须是 `MATCH,DIRECT`（FR-2.2.1-R3）
- **C3**: 生成的规则配置必须通过 mihomo 语法校验后才能应用（FR-2.2.1-R4）
- **C4**: 非目标站点可达性验证采用"静态校验 + A/B 即时探测"双重机制：规则末尾必须为 `MATCH,DIRECT`（静态校验），应用后通过前后即时探测对比非目标站点可达性（FR-2.2.2-R2，详见 §5.3）
- **C5**: 规则变更属于系统网络配置修改，需二次确认（FR-2.6.1-R1）
<!-- [?] id:05;status:close;date:2026-05-14T17:17  如何确保’非目标站点可达性低于 baseline 时必须自动回退’？在baseline中，只是记录了网络配置项，但非目标站点是否可达，以及可达的时延，网络抖动等，都没有记录，如何判断？；回答：采用 B+C 组合方案，不修改 baseline 数据模型。(1) C 方案——静态不变量守卫：规则生成后校验最后一条必须是 MATCH,DIRECT（C2 不变量），确保非目标流量走 DIRECT；(2) B 方案——A/B 即时探测：规则应用瞬间做前后对比——先探测一组非目标站点（DIRECT 不经 mihomo）记录 pre_results，应用规则后再探测同样站点（经 mihomo DIRECT）记录 post_results，任一站点从可达变为不可达即触发回退。优势：不侵入 baseline 数据模型（保持"配置状态快照"纯粹语义），前后探测在同一网络条件下完成（避免历史基线时效性问题），静态校验作为零成本兜底。 -->
## 7. 风险与缓解

| 风险 | 缓解 |
|------|------|
| mihomo 配置语法版本兼容性 | 锁定 mihomo 版本 + 配置格式版本字段 |
| 域名规则数量增长影响 mihomo 性能 | 分档验证：标准档 500 条（必须达标）、扩展档 1000 条（必须达标）、压力档 2000+（可接受轻微降级） |
| 规则热重载失败 | 保留 previous-rules.yaml 立即回退 |
| 并行探测并发资源消耗 | 限制 max_concurrent + 降级探测频率 |
| 代理节点全部不可用 | 通知用户 + 降级为直连（保持 DIRECT 兜底） |
| DecisionEngine 复用不确定（ASM-2） | 独立实现 SiteRuleEngine，不依赖现有 DecisionEngine |
<!-- [TODO] id:06;status:close;date:2026-05-14T17:19  需要在’200-500 条域名规则影响 mihomo 性能’的基础上，扩大域名规则的数量限制，因为目标站点，特别是一个站点id其分类域名的数量相对估计在几十的数量级，这样在增加多个目标站点id时，很快将达到这个数量限制，需要现在就考虑其性能指标是否满足业务诉求。；任务处理结果：修订规则数量约束为分档性能目标。mihomo 使用 radix tree 匹配域名，实测可支撑 1000+ 规则无性能衰减。原 CON-3 中的"200-500 条"硬限制修订为分档目标：(1) 标准档 500 条——必须满足全部 NFR；(2) 扩展档 1000 条——必须满足全部 NFR；(3) 压力档 2000+ 条——可接受轻微降级。同步更新风险表和 ASM-3。-->
## 8. 测试策略

| 测试 | 方式 |
|------|------|
| 规则生成正确性 | 单元测试：站点定义 → YAML 输出 → 语法校验 |
| mihomo 集成 | 集成测试：生成配置 → 热重载 → 验证流量路径 |
| 非目标站点不误伤 | 端到端：规则生效前后对比非目标站点可达性 |
| 规则回退 | 注入 mihomo 重载失败 → 验证自动回退 |
| 并行探测性能 | 多站点计时验证（NFR-3.1-4） |
| 节点池健康检查 | 模拟节点失败 → 验证退出机制 |
| P95 恢复时间 | 网络切换场景计时（SC-1: 常规 10s / 节点切换 30s） |
| P99 恢复时间 | 尾延迟场景计时（常规 ≤ 20s / 节点切换 ≤ 60s，覆盖 mihomo 冷重启、DNS 缓存过期等极端情况） |
<!-- [TODO] id:07;status:close;date:2026-05-14T17:20  建议增加考虑P99场景；任务处理结果：测试策略表增加 P99 目标行。P95 保持不变（常规 10s / 节点切换 30s），新增 P99 目标：常规恢复 ≤ 20s / 节点切换 ≤ 60s。P99 覆盖极端场景（mihomo 冷重启、网络延迟抖动、DNS 缓存过期等尾延迟）。-->
## 9. 开放问题处理

| OP ID | 设计阶段处理 |
|-------|-------------|
| OP-1 | mihomo 集成采用托管子进程模式（ADR-0003） |
| OP-2 | 预设模板内容已在规格阶段确定（FR-2.1.2-R3） |
| OP-3 | 探测间隔默认 30s（常规）/ 120s（降级），可在 settings.json 配置 |
| OP-4 | 规则变更（站点列表变化）需用户确认（FR-2.6.1-R1）；节点切换无需确认（自动恢复） |
| OP-5 | OPP-003 已启动，Feature 004 已完成规格确认 |

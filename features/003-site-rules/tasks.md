# Feature 003: 目标站点规则配置与可达性诊断 — 任务计划

- 状态: 草稿
- 主题: site-rules
- 阶段: `hf-tasks`
- 上游输入: `features/003-site-rules/spec.md`、`features/003-site-rules/design.md`
- **前置依赖**: Feature 001 M1~M6 完成（BaselineManager + MihomoManager + AuditLogger + ConfigManager）

## 1. 概述

本任务计划将 Feature 003 设计转化为可执行任务。Feature 003 实现 GoGuo 的核心产品价值——站点定义管理、mihomo 规则生成、可达性探测和代理节点池管理。

**实现范围**：SiteRuleEngine、SiteDefinitionStore、RuleGenerator、ProbeService、NodePool、NodeHealthChecker、SubscriptionParser、11 个 Tauri Commands、B+C 验证流程。

## 2. 里程碑

| 里程碑 | 目标 | 退出标准 | 对应设计章节 |
|--------|------|----------|-------------|
| **M1: 站点定义与存储** | SiteDefinition 数据模型 + Store + 预设模板 | 单元测试通过：模板加载、域名展开 | design §2.2 |
| **M2: 规则生成** | RuleGenerator：站点→mihomo YAML | 单元测试通过：规则正确性+静态校验 | design §2.3 |
| **M3: 探测服务** | ProbeService 分层探测 + ProbeHistory | 单元测试通过：探测执行+历史记录 | design §2.4 |
| **M4: 节点池** | NodePool + NodeHealthChecker | 单元测试通过：节点管理+健康检查 | design §2.5 |
| **M5: 订阅解析** | SubscriptionParser | 单元测试通过：协议解析+过滤 | design §2.6 |
| **M6: B+C 验证** | 静态校验 + A/B 即时探测 | 单元测试通过：验证+回退 | design §5.3 |
| **M7: SiteRuleEngine** | 统一编排层 | 单元测试通过：完整添加→规则→验证 | design §2.1 |
| **M8: Tauri Commands** | 11 个 IPC 命令 | 命令可调用 | design §4 |
| **M9: 集成验证** | 端到端：站点管理→规则→探测→回退 | P95 ≤ 10s / P99 ≤ 20s | design §8 |

## 3. 文件 / 工件影响图

```
src-tauri/src/
  models/
    site.rs              # SiteDefinition, DomainCategory, HealthCheckConfig
    probe.rs             # ProbeResult, ProbeMethod, ProbeConfig, ProbeHistory
    node.rs              # ProxyNode, ProxyProtocol, NodeStatus, NodeHealthResult
    subscription.rs      # SubscriptionSource
  services/
    site_definition_store.rs  # SiteDefinitionStore
    rule_generator.rs        # RuleGenerator
    probe_service.rs         # ProbeService
    node_pool.rs             # NodePool
    node_health_checker.rs   # NodeHealthChecker
    subscription_parser.rs   # SubscriptionParser
  engines/
    site_rule_engine.rs      # SiteRuleEngine（编排层）
  commands/
    site_rules.rs            # F003 Tauri Commands（11 个）
data/
  config/
    site-definitions/        # 内置 + 自定义站点定义 JSON
    subscription-sources.json
  rules/
    current-rules.yaml       # 当前生效规则
    previous-rules.yaml      # 回退备份
  state/
    probe-history.jsonl       # 探测历史
```

## 4. 需求与设计追溯

| 需求 ID | 设计章节 | 任务覆盖 |
|---------|---------|---------|
| FR-2.1.1-R1/R2/R4/R6 (站点管理) | §2.1, §2.2 | T1.1, T7.1 |
| FR-2.1.2-R1 (预设模板) | §2.2 预设模板 | T1.2 |
| FR-2.2.1 (规则生成) | §2.3 RuleGenerator | T2.1 |
| FR-2.2.2-R2 (非目标站点不误伤) | §5.3 B+C 验证 | T6.1 |
| FR-2.3.2-R1 (节点切换) | §2.5 NodePool | T4.1 |
| FR-2.3.2-R3~R7 (节点池管理) | §2.5 | T4.1 |
| FR-2.4.1 (规则预览) | §2.3 preview_rules | T2.2 |
| FR-2.6.1 (可达性) | §2.4 ProbeService | T3.1 |
| FR-2.6.2 (诊断) | §2.4 | T3.1 |
| FR-2.3.2 (订阅导入) | §2.6 SubscriptionParser | T5.1 |
| SC-1 (P95 恢复时间) | §8 | T9.1 |
| CON-3 (域名规模 500/1000/2000+) | §7 | T9.1 |

## 5. 任务拆解

### T1.1: SiteDefinition 数据模型 + Store

- **目标**: 定义 SiteDefinition、DomainCategory 枚举、HealthCheckConfig；实现 SiteDefinitionStore（内置+自定义站点定义 JSON 读写）
- **Acceptance**: 站点定义可序列化/反序列化；Store 可加载内置定义和自定义定义
- **依赖**: Feature 001 T2.1（数据模型基础设施）
- **Ready When**: F001 数据模型完成
- **初始队列状态**: pending
- **Selection Priority**: 1
- **Files / 触碰工件**: `src-tauri/src/models/site.rs`, `src-tauri/src/services/site_definition_store.rs`
- **测试设计种子**: 站点定义 JSON 往返测试；Store 加载内置 github 定义→验证域名列表；自定义定义保存/读取测试
- **Verify**: `cargo test --lib models::site && cargo test --lib services::site_definition_store`
- **预期证据**: 数据模型+Store 测试通过
- **完成条件**: SiteDefinition 模型和 Store 实现完整

### T1.2: 预设模板与域名展开

- **目标**: 实现开发者套装和办公套装预设模板；实现域名分类展开逻辑
- **Acceptance**: 模板加载返回完整站点列表；域名按 DomainCategory 展开；chatgpt 站点定义覆盖 openai.com
- **依赖**: T1.1
- **Ready When**: SiteDefinition Store 完成
- **初始队列状态**: pending
- **Selection Priority**: 1
- **Files / 触碰工件**: `src-tauri/src/services/site_definition_store.rs`
- **测试设计种子**: 开发者套装→验证 10 个站点；办公套装→验证 6 个站点；chatgpt 定义→验证 openai.com 包含；域名总数统计
- **Verify**: `cargo test --lib services::site_definition_store -- template`
- **预期证据**: 模板加载测试通过
- **完成条件**: 预设模板正确，域名展开逻辑正确

### T2.1: RuleGenerator 规则生成

- **目标**: 实现站点列表→mihomo YAML 规则生成，含 MATCH,DIRECT 兜底和静态校验
- **Acceptance**: 生成的 YAML 通过 mihomo 语法校验；最后一条规则始终为 MATCH,DIRECT；用户自定义规则优先
- **依赖**: T1.1, Feature 001 T5.2（MihomoManager reload_config）
- **Ready When**: SiteDefinition + MihomoManager 热重载完成
- **初始队列状态**: pending
- **Selection Priority**: 2
- **Files / 触碰工件**: `src-tauri/src/services/rule_generator.rs`
- **测试设计种子**: 空站点列表→仅 MATCH,DIRECT；添加 github→验证 DOMAIN-SUFFIX 规则生成；静态校验拒绝非 MATCH,DIRECT 结尾；自定义规则覆盖测试
- **Verify**: `cargo test --lib services::rule_generator`
- **预期证据**: 规则生成测试通过
- **完成条件**: 规则生成+静态校验+自定义覆盖正确

### T2.2: 规则预览与回退

- **目标**: 实现 preview_rules() 和规则回退机制（previous-rules.yaml 备份/恢复）
- **Acceptance**: 预览返回即将生效的规则列表；应用前自动备份；回退可恢复到上一版本
- **依赖**: T2.1
- **Ready When**: RuleGenerator 完成
- **初始队列状态**: pending
- **Selection Priority**: 2
- **Files / 触碰工件**: `src-tauri/src/services/rule_generator.rs`
- **测试设计种子**: 预览→验证按站点分组；备份→验证 previous-rules.yaml 生成；回退→验证恢复后配置正确
- **Verify**: `cargo test --lib services::rule_generator -- preview_rollback`
- **预期证据**: 预览+回退测试通过
- **完成条件**: 预览和回退逻辑正确

### T3.1: ProbeService 分层探测

- **目标**: 实现 ProbeService：Level 1 DNS+HEAD、Level 2 GET+状态码、Level 3 TLS 诊断；并行探测；ProbeHistory 环形缓冲
- **Acceptance**: 分层探测按触发条件正确执行；并行探测不超 max_concurrent；历史记录正确存储和恢复
- **依赖**: Feature 001 T5.1（MihomoManager）
- **Ready When**: MihomoManager 完成
- **初始队列状态**: pending
- **Selection Priority**: 2
- **Files / 触碰工件**: `src-tauri/src/services/probe_service.rs`, `src-tauri/src/models/probe.rs`
- **测试设计种子**: Mock HTTP 客户端→Level 1 返回可达/不可达；Level 2 状态码 200/403/超时；TLS 握手成功/失败；并行 5 站点探测；ProbeHistory 满时淘汰旧记录
- **Verify**: `cargo test --lib services::probe_service`
- **预期证据**: 分层探测+历史记录测试通过
- **完成条件**: 3 层探测+并行+历史记录完整实现

### T4.1: NodePool + NodeHealthChecker

- **目标**: 实现 NodePool（节点管理、切换、退出）和 NodeHealthChecker（mihomo API + TCP 直连）
- **Acceptance**: 节点切换到下一个可用节点；连续 N 次失败触发移除；健康检查结果正确
- **依赖**: Feature 001 T5.1（MihomoManager）
- **Ready When**: MihomoManager 完成
- **初始队列状态**: pending
- **Selection Priority**: 2
- **Files / 触碰工件**: `src-tauri/src/services/node_pool.rs`, `src-tauri/src/services/node_health_checker.rs`, `src-tauri/src/models/node.rs`
- **测试设计种子**: 3 节点→第 1 个不可用→切换到第 2 个；连续 3 次失败→标记 Removed；mihomo API 返回延迟→TCP 降级；节点元数据（名称、入池时间等）正确
- **Verify**: `cargo test --lib services::node_pool && cargo test --lib services::node_health_checker`
- **预期证据**: 节点池+健康检查测试通过
- **完成条件**: 节点管理+切换+退出+健康检查正确

### T5.1: SubscriptionParser 订阅解析

- **目标**: 实现订阅链接解析（base64 解码→节点列表）、5 种协议解析、不支持协议过滤+审计
- **Acceptance**: 解析返回正确的 ProxyNode 列表；不支持协议被过滤并审计；订阅源持久化
- **依赖**: T4.1, Feature 001 T6.1（AuditLogger）
- **Ready When**: NodePool + AuditLogger 完成
- **初始队列状态**: pending
- **Selection Priority**: 3
- **Files / 触碰工件**: `src-tauri/src/services/subscription_parser.rs`, `src-tauri/src/models/subscription.rs`
- **测试设计种子**: 标准订阅 base64→解析出节点；混合协议→验证过滤；空订阅→空结果；订阅源保存/读取
- **Verify**: `cargo test --lib services::subscription_parser`
- **预期证据**: 订阅解析测试通过
- **完成条件**: 解析+过滤+审计+存储正确

### T6.1: B+C 验证流程

- **目标**: 实现静态校验（MATCH,DIRECT 不变量）+ A/B 即时探测（pre/post 探测非目标参考站点）+ 自动回退
- **Acceptance**: 静态校验失败拒绝应用；A/B 探测检测到可达性下降→自动回退；全部通过→规则生效
- **依赖**: T2.1, T3.1
- **Ready When**: RuleGenerator + ProbeService 完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src-tauri/src/engines/site_rule_engine.rs`（验证流程部分）
- **测试设计种子**: 静态校验：去掉 MATCH,DIRECT→拒绝；A/B：pre 可达+post 不可达→回退；A/B：全部 post 可达→通过；参考站点 baidu.com/qq.com 默认值验证。**边界场景**：①空参考站点列表→跳过 A/B 探测（仅静态校验）；②探测全部超时→标记为不可达但不触发回退（网络问题非规则问题）；③pre 和 post 都不可达→不触发回退（非规则导致）；④参考站点部分可达部分不可达→仅关注从可达变为不可达的站点
- **Verify**: `cargo test --lib engines::site_rule_engine -- verification`
- **预期证据**: B+C 验证测试通过
- **完成条件**: 双重验证机制正确，回退逻辑正确

### T7.1: SiteRuleEngine 编排层

- **目标**: 实现 SiteRuleEngine 统一编排：add_site → 展开域名 → 生成规则 → B+C 验证 → 应用
- **Acceptance**: 完整流程端到端正确；站点删除触发规则重新生成；不可达站点触发五要素提示
- **依赖**: T1.1, T2.1, T3.1, T4.1, T6.1
- **Ready When**: 全部子模块完成
- **初始队列状态**: pending
- **Selection Priority**: 5
- **Files / 触碰工件**: `src-tauri/src/engines/site_rule_engine.rs`
- **测试设计种子**: 添加 github→展开→生成→验证→应用完整流程；删除站点→规则重新生成；不可达→五要素提示生成
- **Verify**: `cargo test --lib engines::site_rule_engine`
- **预期证据**: 编排层测试通过
- **完成条件**: 站点管理→规则生成→验证→应用完整流程正确

### T8.1: Feature 003 Tauri Commands

- **目标**: 实现 11 个 Tauri Commands：add_target_site, remove_target_site, apply_preset_template, preview_rules, apply_rules, get_site_reachability, get_diagnosis, get_node_pool_status, override_rule, import_subscription, get_subscription_sources
- **Acceptance**: 每个命令可被前端 invoke() 调用；参数和返回值类型正确
- **依赖**: T7.1
- **Ready When**: SiteRuleEngine 完成
- **初始队列状态**: pending
- **Selection Priority**: 6
- **Files / 触碰工件**: `src-tauri/src/commands/site_rules.rs`, `src-tauri/src/commands/mod.rs`
- **测试设计种子**: 每个命令参数/返回值测试；apply_rules 确认标记验证；import_subscription 订阅 URL 验证
- **Verify**: `cargo test --lib commands::site_rules`
- **预期证据**: 11 个命令测试通过
- **完成条件**: 全部 Tauri Commands 实现正确

### T9.1: 集成测试

- **目标**: 端到端验证：站点管理→规则生成→探测→B+C 验证→回退；P95 ≤ 10s / P99 ≤ 20s
- **Acceptance**: 完整流程无错误；非目标站点不误伤；节点切换可达性恢复；性能达标
- **依赖**: T8.1
- **Ready When**: Tauri Commands 完成
- **初始队列状态**: pending
- **Selection Priority**: 7
- **Files / 触碰工件**: `src-tauri/tests/integration_site_rules.rs`
- **测试设计种子**: 完整流程：添加 github→应用→验证可达→删除→验证回退；A/B 验证：注入可达性下降→自动回退；节点切换：当前节点不可用→自动切换→可达性恢复；性能计时：常规 10s / 节点切换 30s
- **Verify**: `cargo test --test integration_site_rules`
- **预期证据**: 集成测试通过，性能达标
- **完成条件**: 端到端流程正确，性能指标满足

## 6. 依赖与关键路径

```
F001 T2.1 ─→ T1.1 ─→ T1.2
F001 T5.1 ─→ T3.1 ─→ T6.1 ─→ T7.1 ─→ T8.1 ─→ T9.1
F001 T5.2 ─→ T2.1 ─→ T2.2 ─→ T6.1
F001 T5.1 ─→ T4.1 ─→ T5.1
F001 T6.1 ─→ T5.1(subscription)
T1.1 + T2.1 + T3.1 + T4.1 + T6.1 ─→ T7.1
```

**关键路径**：F001 T5.1 → T3.1 → T6.1 → T7.1 → T8.1 → T9.1

**可并行任务组**：
- T1.1 + T3.1 + T4.1（分别依赖 F001 不同任务）
- T2.1 + T5.1（分别依赖不同前置）

## 7. 完成定义与验证策略

| 里程碑 | DoD | 验证方式 |
|--------|-----|---------|
| M1 | 站点定义可管理 | `cargo test --lib services::site_definition_store` |
| M2 | 规则可生成和回退 | `cargo test --lib services::rule_generator` |
| M3 | 分层探测可工作 | `cargo test --lib services::probe_service` |
| M4 | 节点池可管理 | `cargo test --lib services::node_pool` |
| M5 | 订阅可解析 | `cargo test --lib services::subscription_parser` |
| M6 | B+C 验证有效 | `cargo test --lib engines::site_rule_engine -- verification` |
| M7 | 编排流程正确 | `cargo test --lib engines::site_rule_engine` |
| M8 | 前端可调用 | `cargo test --lib commands::site_rules` |
| M9 | 端到端达标 | `cargo test --test integration_site_rules` |

## 8. 当前活跃任务选择规则

1. Feature 003 所有任务均依赖 Feature 001 部分任务完成
2. F001 T2.1 完成后可启动 T1.1
3. F001 T5.1 完成后可启动 T3.1、T4.1
4. F001 T5.2 完成后可启动 T2.1
5. **Current Active Task**: T1.1（等待 F001 T2.1 完成后启动）

## 9. 任务队列投影视图

| 阶段 | 任务 | 状态 |
|------|------|------|
| Phase 1 | T1.1 站点模型+Store · T3.1 ProbeService · T4.1 NodePool | ⬜ pending（等待 F001） |
| Phase 2 | T1.2 预设模板 · T2.1 RuleGenerator · T5.1 SubscriptionParser | ⬜ pending |
| Phase 3 | T2.2 规则回退 | ⬜ pending |
| Phase 4 | T6.1 B+C 验证 | ⬜ pending |
| Phase 5 | T7.1 SiteRuleEngine 编排 | ⬜ pending |
| Phase 6 | T8.1 Tauri Commands | ⬜ pending |
| Phase 7 | T9.1 集成测试 | ⬜ pending |

## 10. 风险与顺序说明

| 风险 | 影响 | 缓解 |
|------|------|------|
| mihomo 配置语法版本变化 | T2.1 规则生成 | 锁定 mihomo 版本+配置格式版本字段 |
| 域名规则数 1000+ 性能 | T9.1 | 分档验证（500/1000/2000+） |
| 订阅链接格式多样化 | T5.1 | 初始支持标准 base64 格式 |
| A/B 探测网络抖动误判 | T6.1 | 探测超时 3s + 重试机制 |

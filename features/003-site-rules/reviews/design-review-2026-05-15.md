# Feature 003 设计审查记录

- 审查对象：`features/003-site-rules/design.md`
- 审查日期：2026-05-15
- 审查类型：`hf-design-review`
- 审查人：Teddy（PM/QA）
- 上游输入：`features/003-site-rules/spec.md`、7 条人工评审标注（2026-05-14）、跨文档设计评审（2026-05-15）

## 审查清单

| # | 检查维度 | 结论 | 说明 |
|---|----------|------|------|
| 1 | 上游一致性 | PASS | spec.md 全部 FR/NFR/CON/SC 在设计中均有模块和接口落地 |
| 2 | 完整性与覆盖 | PASS | 核心模块 8 个、Tauri Commands 10+、数据模型 10+、分层探测策略完整 |
| 3 | 可测试性 | PASS | P99 恢复时间目标明确（≤20s / ≤60s），分档域名性能目标可验证 |
| 4 | 内部一致性 | PASS | 跨文档评审 HIGH-2（ProbeHistory/NodeHealthChecker）、HIGH-3（SubscriptionParser）已修复 |
| 5 | 约束遵守 | PASS | CON-1~4 均在设计中体现，分档性能目标与 spec.md 对齐 |
| 6 | 数据模型 | PASS | SiteDefinition / ProxyNode / ProbeHistory / NodeHealthChecker / SubscriptionParser 完整 |
| 7 | 跨 Feature 一致性 | PASS | 与 F001 B+C 验证方案、F004 语义化展示对齐 |

## 人工评审标注处理

共处理 7 条标注（2026-05-14）：

| 标注 ID | 变更 | 状态 |
|---------|------|------|
| id:01 | MVP 阶段所有目标站点共享单一 PROXY 代理组 | 已修订 |
| id:02 | 补充分层探测策略（Level 1→2→3），新增 TlsHandshake | 已修订 |
| id:03 | 不采用主备 mihomo，NodePool 生命周期在 Rust 后端 | 已修订 |
| id:04 | 5 种代理协议 mihomo 原生支持，不支持协议在订阅解析阶段过滤 | 已修订 |
| id:05 | 非目标站点验证采用 B+C 方案（静态校验 + A/B 即时探测） | 已修订 |
| id:06 | 域名规则限制修订为分档性能目标（500/1000/2000+） | 已修订 |
| id:07 | 测试策略新增 P99 恢复时间目标 | 已修订 |

## 跨文档设计评审

| # | 问题 | 严重度 | 修订内容 | 状态 |
|---|------|--------|----------|------|
| H-2 | ProbeHistory/ProbeMethod/NodeHealthChecker 缺数据模型定义 | HIGH | 补充完整结构定义和字段说明 | 已修复 |
| H-3 | 缺少 SubscriptionParser 模块设计 | HIGH | 新增 §2.6 SubscriptionParser（订阅解析、协议过滤、审计记录） | 已修复 |

## 观察项

| # | 观察 | 建议处理时机 |
|---|------|-------------|
| O-1 | id:05 B+C 验证参考站点默认 baidu.com/qq.com，可根据用户实际访问习惯调整 | hf-tasks |

## 审查结论

**通过**。7 条人工评审标注全部修订确认，2 条跨文档 HIGH 问题已修复，1 条观察项留待 hf-tasks 处理。设计完整覆盖需求规格，数据模型和分层探测策略明确，可进入 hf-tasks。

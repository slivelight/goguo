# Feature 003 设计审批记录

- 状态：已确认
- 日期：2026-05-15
- 阶段：`hf-design`
- 审批对象：`features/003-site-rules/design.md`
- 审查记录：`features/003-site-rules/reviews/design-review-2026-05-15.md`
- 审批人：用户

## 审批结论

Feature 003 设计审查通过。

- 审查清单 7/7 PASS
- 7 条人工评审标注全部修订确认
- 2 条跨文档 HIGH 问题已修复（数据模型补全、SubscriptionParser 模块新增）
- 1 条观察项留待 hf-tasks 处理

## 标注修订摘要

本轮共处理 7 条人工评审标注 + 2 条跨文档评审问题：

| 类别 | 变更 |
|------|------|
| 代理组策略 | MVP 阶段单一 PROXY 代理组，未来可按站点分配 |
| 探测策略 | 分层探测（Level 1 DNS+HEAD → Level 2 GET+状态码 → Level 3 TLS） |
| 进程架构 | 不采用主备 mihomo，NodePool 生命周期在 Rust 后端 |
| 协议兼容 | 5 种代理协议 mihomo 原生支持，不支持协议在订阅解析阶段过滤 |
| 验证方案 | B+C 组合（静态校验 MATCH,DIRECT 不变量 + A/B 即时探测） |
| 性能目标 | 分档域名目标（500/1000/2000+）+ P99 恢复时间（≤20s/≤60s） |
| 数据模型 | 补充 ProbeHistory、ProbeMethod、NodeHealthChecker、SubscriptionParser |

## 放行范围

Feature 003 设计确认完成。可进入 `hf-tasks`。

## 约束/待办

- O-1：B+C 验证参考站点默认值可根据用户实际访问习惯调整（hf-tasks）

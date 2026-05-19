# Feature 003 任务计划审查记录

- 审查对象：`features/003-site-rules/tasks.md`
- 审查日期：2026-05-18
- 审查类型：`hf-tasks-review`
- 审查人：Teddy（PM/QA）

## 评分

| 维度 | 评分 | 说明 |
|------|------|------|
| TR1 可执行性 | 8/10 | 11 个任务粒度适当；T7.1（SiteRuleEngine 编排层）聚合 5 个子模块编排逻辑，略大但性质为集成胶水层 |
| TR2 任务合同完整性 | 9/10 | 全部 11 个任务具备 Acceptance、Files、Verify、完成条件 |
| TR3 验证与测试种子 | 8/10 | 种子具体（分层探测、A/B 验证、协议过滤）；T9.1 性能计时种子缺乏自动化方案 |
| TR4 依赖与顺序 | 9/10 | 依赖链正确：Store/RuleGenerator/ProbeService/NodePool 可并行→B+C 验证→编排层→Commands→集成 |
| TR5 追溯覆盖 | 8/10 | FR 覆盖完整；SC-1（P95 恢复时间）和 CON-3（域名分档 500/1000/2000+）在 T9.1 覆盖但测试种子偏笼统 |
| TR6 Router 重选就绪 | 8/10 | 同 F002，Current Active Task 为"无"。T1.1（blocked by F001 T2.1）应显式标注 |

**综合评分**：8.3/10

## 反模式检测

| ID | 检测结果 | 说明 |
|----|---------|------|
| TA1 大任务 | 轻微 | T7.1 编排层聚合多模块。但作为集成胶水层，拆分收益有限。保留 |
| TA2 缺 Acceptance | 无 | 全部具备 |
| TA3 缺 Files/Verify | 无 | 全部具备 |
| TA4 无 test seed | 无 | 全部具备 |
| TA5 里程碑冒充 | 无 | 分层清晰 |
| TA6 orphan task | 无 | 全部可追溯 |
| TA7 unstable active | 轻微 | 同 F002 |

## 发现项

- [minor][LLM-FIXABLE][TR5] T9.1 测试种子中"性能计时：常规 10s / 节点切换 30s"缺乏具体自动化计时方案。建议明确使用 `std::time::Instant` 或集成测试框架的计时宏
- [minor][LLM-FIXABLE][TR5] T9.1 未在测试种子中覆盖 CON-3 分档验证（标准档 500 / 扩展档 1000 / 压力档 2000+）。建议补充"生成 N 条规则→计时验证"的参数化测试
- [minor][LLM-FIXABLE][TR6] §8 Current Active Task 应改为"T1.1（blocked by F001 T2.1）"
- [important][LLM-FIXABLE][TR3] T6.1（B+C 验证）是核心安全机制，测试种子应覆盖更多边界：空参考站点列表→跳过 A/B 探测；探测全部超时→判定方式；pre 和 post 都不可达（非规则问题）→判定方式

## 缺失或薄弱项

- T6.1 B+C 验证是核心安全机制，建议其测试种子补充更多边界场景（见 finding 4）

## 结论

**通过**（附带 4 条 findings：1 important + 3 minor）

## 下一步

- `任务真人确认`：建议在确认前处理 important finding（T6.1 边界场景补充）
- 等待 F001 T2.1 + T5.1 + T5.2 完成后激活 T1.1/T3.1/T4.1

# Feature 004 任务计划审查记录

- 审查对象：`features/004-user-interaction/tasks.md`
- 审查日期：2026-05-18
- 审查类型：`hf-tasks-review`
- 审查人：Teddy（PM/QA）

## 评分

| 维度 | 评分 | 说明 |
|------|------|------|
| TR1 可执行性 | 7/10 | T2.1（7 个 Store 一次实现）和 T3.1（6 个共享组件一次实现）粒度偏大（TA1）。T9.1（Wizard 7 步）也偏大 |
| TR2 任务合同完整性 | 9/10 | 全部 13 个任务具备 Acceptance、Files、Verify、完成条件 |
| TR3 验证与测试种子 | 8/10 | 大部分种子具体；T10.3（跨平台一致性）种子仅为"截图对比"，缺乏自动化方案 |
| TR4 依赖与顺序 | 9/10 | 依赖链正确：结构→IPC→Store→组件→各页面可并行→聚合任务 |
| TR5 追溯覆盖 | 8/10 | FR/NFR 追溯完整；CON-4（主路径 ≤2 步）验证方式为"M4~M8 验证"，未显式写入单个任务的测试种子 |
| TR6 Router 重选就绪 | 8/10 | 同 F002/F003，Current Active Task 为"无" |

**综合评分**：8.2/10

## 反模式检测

| ID | 检测结果 | 说明 |
|----|---------|------|
| TA1 大任务 | 中度 | T2.1（7 Store）、T3.1（6 组件）粒度偏大。建议至少将 Store 层拆为"核心 Store（Service/Baseline/Notif）+ 扩展 Store（Site/Rule/Diag/Wizard）" |
| TA2 缺 Acceptance | 无 | 全部具备 |
| TA3 缺 Files/Verify | 无 | 全部具备 |
| TA4 无 test seed | 轻微 | T10.3 种子过于手动化 |
| TA5 里程碑冒充 | 无 | 分层清晰 |
| TA6 orphan task | 无 | 全部可追溯 |
| TA7 unstable active | 轻微 | 同 F002 |

## 发现项

- [important][LLM-FIXABLE][TR1] T2.1 一次实现 7 个 Zustand Store，范围过大。建议拆为 T2.1a（核心 3：ServiceStore + BaselineStore + NotifStore）和 T2.1b（扩展 4：SiteStore + RuleStore + DiagStore + WizardStore），前者优先，后者可与 T3.1 并行
- [minor][LLM-FIXABLE][TR1] T3.1 一次实现 6 个共享组件，范围偏大。建议至少拆出 RecoveryOverlay + RecoveryAckDialog 为独立任务（与恢复流程强相关，可推迟到 F001 T8.1 完成后）
- [minor][LLM-FIXABLE][TR5] CON-4（主路径 ≤2 步）应在 T4.1~T8.1 的测试种子中各增加一条步数验证断言
- [minor][LLM-FIXABLE][TR3] T10.3 跨平台一致性验证缺乏自动化方案。建议补充"CSS 变量回归测试"或"关键组件 snapshot 测试"作为自动化基线
- [minor][LLM-FIXABLE][TR6] §8 Current Active Task 应改为"T1.1（blocked by F001 T1.1）"

## 缺失或薄弱项

- T2.1 和 T3.1 粒度是主要薄弱项，拆分后可显著提升冷启动可执行性

## 结论

**通过**（附带 1 important + 4 minor findings）

## 下一步

- `任务真人确认`：建议在确认前处理 important finding（T2.1 拆分）
- 等待 F001 T1.1 完成后激活 T1.1（前端结构）

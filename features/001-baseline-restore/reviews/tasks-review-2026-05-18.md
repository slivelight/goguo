# Feature 001 任务计划审查记录

- 审查对象：`features/001-baseline-restore/tasks.md`
- 审查日期：2026-05-18
- 审查类型：`hf-tasks-review`
- 审查人：Teddy（PM/QA）

## 评分

| 维度 | 评分 | 说明 |
|------|------|------|
| TR1 可执行性 | 8/10 | 大部分任务粒度适中；T9.1 打包 10 个 Commands + 9 个 Events 略大（TA1 轻微） |
| TR2 任务合同完整性 | 9/10 | 全部 17 个任务均具备 Acceptance、Files、Verify、完成条件 |
| TR3 验证与测试种子 | 8/10 | 大部分种子具体可用；T1.1（构建验证）和 T9.1（命令类型测试）种子偏薄 |
| TR4 依赖与顺序 | 9/10 | 依赖链正确，关键路径合理，无循环依赖 |
| TR5 追溯覆盖 | 8/10 | FR 追溯完整；CON-5（审计不含用户访问明细）未在 T6.1 测试种子中体现 |
| TR6 Router 重选就绪 | 9/10 | 选择规则唯一（优先级→ID），queue projection 清晰，Current Active Task=T1.1 |

**综合评分**：8.5/10

## 反模式检测

| ID | 检测结果 | 说明 |
|----|---------|------|
| TA1 大任务 | 轻微 | T9.1 包含 10 个 Commands + 9 个 Events 定义。但性质为机械性集成（已有全部模块），拆分收益有限。保留，不强制修改 |
| TA2 缺 Acceptance | 无 | 全部任务具备 Acceptance |
| TA3 缺 Files/Verify | 无 | 全部任务具备 Files 和 Verify |
| TA4 无 test seed | 轻微 | T1.1 测试种子仅为"构建产物验证"，可补充"Tauri window 可创建"等更具体断言 |
| TA5 里程碑冒充 | 无 | 里程碑与任务分层清晰 |
| TA6 orphan task | 无 | 全部任务可追溯到 spec/design |
| TA7 unstable active | 无 | T1.1 唯一当前活跃任务 |

## 发现项

- [minor][LLM-FIXABLE][TR1] T9.1 范围较大（10 Commands + 9 Events）。建议将 Events 定义拆到 T9.1 之前的一个轻量任务，或在 T9.1 内明确分阶段执行顺序
- [minor][LLM-FIXABLE][TR3] T1.1 测试种子偏薄——"构建产物验证"缺乏具体断言。建议补充 `Tauri window 可创建`、`lib.rs 导出 modules` 等
- [minor][LLM-FIXABLE][TR5] T6.1（AuditLogger）测试种子未覆盖 CON-5（审计记录不含用户访问内容明细）。建议在测试种子中增加一条：写入包含 URL 的审计记录→验证序列化输出不含 URL 路径部分

## 缺失或薄弱项

- 无阻塞级缺失
- M10（集成测试）的性能计时验证方式为手动，未明确自动化计时方案

## 结论

**通过**（附带 3 条 minor findings）

## 下一步

- `任务真人确认`：3 条 minor findings 可在实现阶段顺带修复，不阻塞进入 `hf-test-driven-dev`
- 当前 Current Active Task: T1.1（项目脚手架）

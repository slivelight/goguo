# Feature 113: 三层测试重构（F001~F004 历史补齐）

- **Feature**: 113-test-restructure
- **阶段**: `hf-tasks`
- **状态**: 草稿
- **当前活跃任务**: 无（spec + design + tasks 待评审）

## 状态总览

| 阶段 | 状态 | 完成日期 |
|------|------|----------|
| hf-product-discovery | — | — |
| hf-specify | 草稿 | 2026-06-11 |
| hf-design | 草稿 | 2026-06-11 |
| hf-tasks | 草稿 | 2026-06-11 |
| hf-test-driven-dev | — | — |
| hf-finalize | — | — |

## 关键工件

| 文件 | 路径 |
|------|------|
| 需求规格 | `features/113-test-restructure/spec.md` |
| 设计文档 | `features/113-test-restructure/design.md` |
| 测试原则 | `docs/principles/testing-principles.md`（宪法层） |
| 任务拆解 | `features/113-test-restructure/tasks.md` |
| 进度 | — (待 hf-test-driven-dev) |

## 定位

**质量基础设施 feature**——为 F001~F004 的 142 条 FR 建立三层自动化验收测试体系。

- 本 feature **只写测试，不修生产代码**
- 失败测试标记 `#[ignore]`，指向修复 feature
- 发现的新问题分流到 F109/F110 或新开 F114+

## 覆盖范围

### 交付物

| 层级 | 预估数量 |
|------|---------|
| FR 验收测试（后端） | ~47 个 |
| FR 验收测试（前端） | ~15 个 |
| 契约测试 | ~20 个 |
| 管道集成测试 | ~7 个 |
| 测试基建（setup helper） | 4 个 helper |
| FR 追溯矩阵 | 142 FR × 测试函数 × 状态 |
| **合计** | **~88 个新测试** |

### 批次计划

| 批次 | 内容 | 预估工作量 |
|------|------|-----------|
| Batch 1 | 测试基建 + F001 验收测试 | 2~3 天 |
| Batch 2 | F002 + F003 验收测试 | 2 天 |
| Batch 3 | F004 验收测试（前端 + 后端） | 1.5 天 |
| Batch 4 | 契约测试 + 管道集成测试 | 2 天 |
| Batch 5 | 追溯矩阵 + CI 集成 | 0.5 天 |

## 相关文档

- 三层测试方法论（宪法层）：`docs/principles/testing-principles.md`
- F109 spec + design（已知 gap 来源）
- F110 spec + design（已知 gap 来源）
- v0.1.0 审计报告：`docs/insights/2026-05-21-v010-spec-design-impl-drift-audit.md`

## 下游依赖

- F109/F110 的修复需移除本 feature 创建的 `#[ignore]` 标记
- 未来 feature 的测试遵循 `testing-principles.md`，不依赖本 feature 完成

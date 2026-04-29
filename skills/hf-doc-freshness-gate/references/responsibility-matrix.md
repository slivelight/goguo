# Responsibility Matrix（spec §6.2 权威 cold-link）

## Purpose

本文件是 `features/001-hf-doc-freshness-gate/spec.md` §6.2 责任矩阵的**权威 cold-link**：reviewer subagent 在判定每个对外可见文档维度归属时，**只读这一份**，避免 SKILL.md 复述导致的双 source-of-truth 漂移。

## Cold-Link

权威定义在：`features/001-hf-doc-freshness-gate/spec.md` §6.2 (HYP-002 (U2) Blocking 关闭依据)

**reviewer 必须以 spec §6.2 为唯一权威**。本文件仅提供等价摘要表，便于 reviewer 在 dispatch 期内快速对照。

## 等价摘要表

下表与 spec §6.2 等价；如冲突，以 spec §6.2 为准。

| 文档维度 | hf-doc-freshness-gate | hf-finalize | hf-increment | hf-code-review | hf-traceability-review |
|---|:-:|:-:|:-:|:-:|:-:|
| 仓库根 `README.md` 产品介绍段 / Quick Start / Usage / 能力清单 | ✅ verdict + evidence | ❌ | ❌ | ❌ | ❌ |
| 仓库根 `README.md` 中 *active feature / 最近 closeout / ADR 索引行*（指针式导航） | ❌ | ✅ 同步（既有合同） | ❌ | ❌ | ❌ |
| 模块层 / 子包 README | ✅ verdict + evidence | ❌ | ❌ | ❌ | ❌ |
| 公共 API docstring / OpenAPI description / 自动文档站 | ✅ verdict + evidence | ❌ | ❌ | ⚠ 实现层正确性 review | ❌ |
| i18n 副本同步 | ✅ verdict + evidence（仅判定是否同步，不判定翻译质量） | ❌ | ❌ | ❌ | ❌ |
| `CONTRIBUTING.md` / onboarding doc | ✅ verdict + evidence | ❌ | ❌ | ❌ | ❌ |
| `docs/adr/NNNN-...md` 状态翻转 | ❌ | ✅ 同步 | ❌ | ❌ | ❌ |
| `CHANGELOG.md` 写入 vX.Y.Z 入口 | ❌ | ✅ 同步 | ❌ | ❌ | ❌ |
| `docs/architecture.md` 或 `docs/arc42/` 架构概述 | ❌ | ✅ 同步 | ❌ | ❌ | ❌ |
| `docs/runbooks/` / `docs/slo/` / `docs/diagrams/` / `docs/release-notes/` | ❌ | ✅ 同步（按存在） | ❌ | ❌ | ❌ |
| `docs/insights/` / `docs/experiments/` / discovery / spec / design / tasks / progress / closeout pack | ❌ | ❌ | ⚠ 范围变更时同步 | ❌ | ✅ 反查追溯 |
| spec / design / tasks 内单条需求的 traceability 链是否完整 | ❌ | ❌ | ❌ | ❌ | ✅（既有合同） |
| 单条需求的功能正确性、测试覆盖、设计 conformance | ❌ | ❌ | ❌ | ✅（既有合同） | ❌ |
| "代码已实现新行为，但文档仍是旧结论" 心态 Red Flag | ❌（本 gate 是 verdict + fresh evidence，不只是心态） | ❌ | ✅（既有 prose Red Flag，作为本 gate 的诊断辅助） | ❌ | ❌ |
| 范围变更（需求 / 验收 / 约束变化）触发的工件失效判断 | ❌ | ❌ | ✅（既有合同） | ❌ | ❌ |

## 使用说明

reviewer 在判定每个文档维度归属时按下表流程处理：

1. 文档维度 = 本 gate 唯一 ✅ 行 → 由本 gate 判定 verdict
2. 文档维度 = 本 gate ❌ + 其他 skill ✅ → 不在本 gate 范围；evidence 文件标 "out of scope (delegated to <skill>)"
3. 文档维度 = ⚠ 共存（与 hf-code-review 或 hf-increment）→ 按行内说明判定（本 gate 仅承担明确 ✅ 部分）
4. 文档维度 = 未在表中 → reviewer 应升级为 USER-INPUT finding，不擅自归类

## Red Flags

- reviewer 跨行归类（把"模块层 README"判到 hf-finalize；或把"CHANGELOG"判到本 gate）→ 违反 spec §6.2，整体 verdict = blocked(workflow)
- reviewer 复述本表内容到 evidence 文件而非引用 cold-link → 制造双 source-of-truth 漂移；evidence 应使用 "see spec §6.2 row N" 引用形式

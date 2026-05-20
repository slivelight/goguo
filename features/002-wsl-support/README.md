# Feature 002: PC 端 Linux/WSL 支持

## 状态

- **Feature ID**: 002
- **状态**: closed（2026-05-20 workflow closeout）
- **Closeout Type**: workflow-closeout
- **关联 OPP**: OPP-004
- **创建日期**: 2026-05-11
- **关闭日期**: 2026-05-20

## 关键日期

| 阶段 | 日期 | 产出 |
|------|------|------|
| hf-product-discovery | 2026-05-11 | `docs/insights/2026-05-11-goguo-opp-004-wsl-support-discovery.md` |
| 人工评审标注 | 2026-05-11 | 1 条标注 close |
| hf-discovery-review | 2026-05-11 | `docs/insights/2026-05-11-goguo-opp-004-discovery-review.md` |
| hf-discovery-approval | 2026-05-11 | `docs/insights/2026-05-11-goguo-opp-004-discovery-approval.md` |
| hf-specify | 2026-05-11 | `spec.md` |
| hf-design | 2026-05-14~15 | `design.md`（1 条标注修订 + 跨文档评审） |
| design-review | 2026-05-15 | `reviews/design-review-2026-05-15.md` |
| design-approval | 2026-05-15 | `approvals/design-approval-2026-05-15.md` |
| hf-tasks | 2026-05-18 | `tasks.md` |
| hf-test-driven-dev | 2026-05-20 | 8/8 任务完成 |
| hf-finalize | 2026-05-20 | `closeout.md` |

## 相关 ADR

- ADR-0004: 数据存储策略 — 安装根目录下文件式 JSON
- ADR-0005: 跨平台策略 — Platform Adapter 模式

## 工件清单

| 文件 | 状态 |
|------|------|
| `spec.md` | 已确认 |
| `design.md` | 已确认 |
| `tasks.md` | 已确认 |
| `progress.md` | 已关闭 |
| `closeout.md` | 已确认 |

## 当前 active task

- 无（workflow 已关闭）

## 上游输入

- `docs/insights/2026-04-29-goguo-strategy-discovery.md`
- `docs/insights/2026-05-11-goguo-opp-004-wsl-support-discovery.md`
- `docs/insights/2026-05-11-goguo-opp-004-discovery-approval.md`
- `features/001-baseline-restore/spec.md`（FR-2.9 交接边界）

## 下游依赖

- OPP-001（目标站点规则配置）依赖本 feature 的 WSL/Linux 侧自动配置能力

# Feature 003: 目标站点规则配置与可达性诊断

## 状态

- **Feature ID**: 003
- **状态**: 任务计划中（`hf-tasks`）
- **关联 OPP**: OPP-001
- **创建日期**: 2026-05-11

## 关键日期

| 阶段 | 日期 | 产出 |
|------|------|------|
| hf-product-discovery | 2026-05-11 | `docs/insights/2026-05-11-goguo-opp-001-site-rules-discovery.md` |
| hf-discovery-review | 2026-05-11 | `docs/insights/2026-05-11-goguo-opp-001-discovery-review.md` |
| hf-discovery-approval | 2026-05-11 | `docs/insights/2026-05-11-goguo-opp-001-discovery-approval.md` |
| hf-specify | 2026-05-11 | `spec.md` |
| hf-design | 2026-05-14~15 | `design.md`（7 条标注修订 + 跨文档评审） |
| design-review | 2026-05-15 | `reviews/design-review-2026-05-15.md` |
| design-approval | 2026-05-15 | `approvals/design-approval-2026-05-15.md` |
| hf-tasks | 进行中 | `tasks.md` |

## 相关 ADR

- ADR-0003: mihomo 集成架构 — 托管子进程
- ADR-0004: 数据存储策略 — 安装根目录下文件式 JSON

## 工件清单

| 文件 | 状态 |
|------|------|
| `spec.md` | 已确认 |
| `design.md` | 已确认 |
| `tasks.md` | 草稿 |
| `progress.md` | 活跃 |

## 当前 active task

- 执行 `tasks.md` 任务计划（等待 F001 M1~M6 完成）

## 上游输入

- `docs/insights/2026-04-29-goguo-strategy-discovery.md`
- `docs/insights/2026-05-11-goguo-opp-001-site-rules-discovery.md`
- `features/001-baseline-restore/spec.md`
- `features/002-wsl-support/spec.md`

## 上游依赖

- Feature 001（baseline/restore 安全底座）
- Feature 002（WSL/Linux 侧配置能力）

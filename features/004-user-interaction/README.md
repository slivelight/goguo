# Feature 004: 用户交互界面

## 状态

- **Feature ID**: 004
- **状态**: 任务计划中（`hf-tasks`）
- **关联 OPP**: OPP-003
- **创建日期**: 2026-05-11

## 关键日期

| 阶段 | 日期 | 产出 |
|------|------|------|
| hf-product-discovery | 2026-05-11 | `docs/insights/2026-05-11-goguo-opp-003-user-interaction-discovery.md` |
| hf-discovery-review | 2026-05-11 | `docs/insights/2026-05-11-goguo-opp-003-discovery-review.md` |
| hf-discovery-approval | 2026-05-11 | `docs/insights/2026-05-11-goguo-opp-003-discovery-approval.md` |
| hf-specify | 2026-05-11 | `spec.md` |
| hf-design | 2026-05-14~15 | `design.md` + `ui-design.md`（30 条 UI 标注修订 + 跨文档评审） |
| design-review | 2026-05-15 | `reviews/design-review-2026-05-15.md` |
| design-approval | 2026-05-15 | `approvals/design-approval-2026-05-15.md` |
| hf-tasks | 进行中 | `tasks.md` |

## 相关 ADR

- ADR-0002: Desktop App Framework — Tauri
- ADR-0004: 数据存储策略 — 安装根目录下文件式 JSON
- ADR-0006: 前端框架选型 — React + TypeScript

## 工件清单

| 文件 | 状态 |
|------|------|
| `spec.md` | 已确认 |
| `design.md` | 已确认 |
| `ui-design.md` | 已确认 |
| `tasks.md` | 草稿 |
| `progress.md` | 活跃 |

## 当前 active task

- 执行 `tasks.md` 任务计划（等待 F001~003 Commands 完成）

## 上游输入

- `docs/insights/2026-04-29-goguo-strategy-discovery.md`
- `docs/insights/2026-05-11-goguo-opp-003-user-interaction-discovery.md`
- `features/001-baseline-restore/spec.md`
- `features/002-wsl-support/spec.md`
- `features/003-site-rules/spec.md`

## 上游依赖

- Feature 001（baseline/restore 安全底座 + 交互语义）
- Feature 002（WSL/Linux 配置能力 + 部署模式选择）
- Feature 003（站点管理 + 规则配置 + 可达性诊断）

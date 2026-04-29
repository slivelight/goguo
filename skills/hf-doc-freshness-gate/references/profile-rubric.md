# Profile Rubric（lightweight / standard / full 强制维度判定细则）

## Purpose

本文件定义 reviewer 在不同 `Workflow Profile` 下激活的强制维度集合，以及维度判定的优先级。spec FR-004 + design §10.3 + design §16 NFR-002 是上游依据。

## 强制维度激活表

| Profile | 强制激活维度 | 可选维度 |
|---|---|---|
| `lightweight` | (1) 仓库根 README 产品介绍段 / Quick Start / Usage / 能力清单<br>(2) Conventional Commits `docs:` 标记自检 | 模块层 README / 公共 API doc / i18n / CONTRIBUTING / 用户文档站 |
| `standard` | (1) lightweight 全部 + (2) 公共 API docstring / OpenAPI description / 自动文档站 + (3) 已存在的 i18n 副本 + (4) CONTRIBUTING.md / onboarding doc | 模块层 README / 用户文档站 |
| `full` | spec §6.2 责任矩阵中本 gate 全部 ✅ 行 | — |

## 判定优先级

reviewer 在形成 user-visible behavior change list 时按下列可信度优先级：

1. **spec FR / NFR 关联** — 最权威（已经过 spec-review + 真人确认）
2. **tasks Acceptance** — 次权威（经过 tasks-review + 真人确认）
3. **Conventional Commits** — 辅助来源（`feat:` / `fix:` / `BREAKING CHANGE:` / `docs:`）

来源全缺 → verdict = `blocked`，next = `hf-traceability-review`（FR-001 负路径）。

## 维度判定流程

每个强制维度按以下流程：

```
1. 文件系统检测：项目是否启用此载体？
   - 不存在 → verdict = N/A，evidence 标 "项目当前未启用此资产"，跳到下一维度
   - 存在 → 进入步骤 2

2. user-visible change list 触发性检测：本 task / feature 的 change list 是否触发该载体的同步需求？
   - 否 → verdict = N/A，evidence 标 "本 task / feature 未触发该资产变化"，跳到下一维度
   - 是 → 进入步骤 3

3. 同步状态判定（按可信度优先级）：
   - 已同步（commits / file diff 显示对应文档已更新）→ verdict = pass
   - 部分同步（部分维度未同步且不阻塞 closeout）→ verdict = partial
   - 关键维度漂移（仓库根 README 产品介绍段与本次行为相关部分明显过期）→ verdict = blocked

4. spec ↔ commits 一致性检测：
   - spec 与 commits 实质不一致（commits 引入新行为但 spec 未更新）→ verdict = blocked，next = hf-increment
```

## 整体 verdict 聚合规则

按 SKILL.md §3 末尾：

- 任一维度 = `blocked` → 整体 verdict = `blocked`
- 否则任一维度 = `partial` → 整体 verdict = `partial`
- 否则全部 ∈ `{pass, N/A}`：
  - 至少一个 `pass` → 整体 verdict = `pass`
  - 全部 `N/A` → 整体 verdict = `N/A`

## Lightweight 性能预算（NFR-002）

`lightweight` profile 下：

- 整轮 reviewer 派发 + 判定 + verdict 落盘 ≤ 5 分钟人工耗时
- verdict 文件 ≤ 30 行（含 metadata header + judgments + reviewer-return JSON）
- 使用 `templates/lightweight-checklist-template.md` 作为模板

## Red Flags

- reviewer 在 lightweight profile 下激活了 standard / full 强制维度（违反 NFR-002 性能预算）
- reviewer 在 standard / full 下漏掉 lightweight 强制维度（违反单调激活原则）
- 判定优先级颠倒（凭 commits 判定而忽略 spec）
- 跳过文件系统检测就直接判 blocked（违反 sync-on-presence + NFR-004）

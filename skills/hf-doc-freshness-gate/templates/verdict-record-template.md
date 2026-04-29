# Doc Freshness Verdict Record Template

> 用于 `features/<active>/verification/doc-freshness-YYYY-MM-DD.md`。
> standard / full profile 必读；lightweight 用 `lightweight-checklist-template.md`。

```markdown
# Doc Freshness Verdict — <feature-or-task-id>

## Metadata

- Reviewer Subagent ID:
- Workflow Profile: standard | full
- Execution Mode: interactive | auto
- Commit Hash:
- Date: YYYY-MM-DD
- Tested Object: features/<active>/ 或 task <T-N>
- Cold-Link to Authority: features/<active>/spec.md §6.2 + skills/hf-doc-freshness-gate/references/responsibility-matrix.md

## User-Visible Behavior Change List (FR-001)

来源（按可信度优先级）：

1. spec FR/NFR 关联：
   - FR-XXX: <statement> (file:spec.md:<line>)
   - NFR-XXX: <statement> (file:spec.md:<line>)
2. tasks Acceptance：
   - T-N: <acceptance> (file:tasks.md:<line>)
3. Conventional Commits：
   - feat: <commit subject> (commit:<hash>)
   - docs: <commit subject> (commit:<hash>)

整理后的 user-visible behavior change list：

- 变化 1: <一句话>
- 变化 2: <一句话>
- ...

## Profile-Activated Mandatory Dimensions

按 `references/profile-rubric.md` <profile> 强制激活：

- 维度 A: <name>
- 维度 B: <name>
- ...

## 维度判定明细

| 维度 | verdict | 理由 / evidence 引用 |
|---|---|---|
| 仓库根 README 产品介绍段 | pass / partial / N/A / blocked | 例：commits abc1234 已更新 §Quick Start；或 "项目当前未启用此资产" / "本 task / feature 未触发" |
| Conventional Commits docs 标记自检（lightweight 强制） | pass / partial / N/A | git log --grep '^docs:' 的最新条目摘要 |
| 公共 API docstring / OpenAPI（standard+ 强制） | pass / partial / N/A / blocked | 例：openapi.yaml 中 GET /foo description 已更新；或 N/A "项目无 OpenAPI" |
| 已存在的 i18n 副本（standard+ 强制） | pass / partial / N/A | 例：README.zh-CN.md 已与 README.md 同步；或 N/A "项目无 i18n 副本" |
| CONTRIBUTING.md / onboarding doc（standard+ 强制） | pass / partial / N/A | |
| 模块层 / 子包 README（full 强制） | pass / partial / N/A | |
| 用户文档站 source（full 强制） | pass / partial / N/A | |

## 整体 Verdict 聚合

按 SKILL §3 末尾聚合规则：

- 任一 blocked → blocked
- 否则任一 partial → partial
- 否则全部 ∈ {pass, N/A}：至少一个 pass → pass；全部 N/A → N/A

**整体 verdict**: pass | partial | N/A | blocked
**聚合理由**: <一句话说明哪些维度触发了上述结论>

## Next Action

- pass / partial / N/A → next = `hf-completion-gate`（verdict 路径作为 evidence bundle 一项）
- blocked (内容) → next = `hf-test-driven-dev`（补文档变更；spec FR-005 第三条 acceptance）
- blocked (workflow, route/stage/profile/证据冲突) → next = `hf-workflow-router` + `reroute_via_router=true`
- spec ↔ commits 实质不一致 → next = `hf-increment`（FR-007）
- user-visible change list 三类来源全缺 → next = `hf-traceability-review`（FR-001 负路径）

**实际选定 next**: <one of the above>

## Reviewer-Return JSON

```json
{
  "conclusion": "pass" | "partial" | "N/A" | "blocked",
  "next_action_or_recommended_skill": "...",
  "record_path": "features/<active>/verification/doc-freshness-YYYY-MM-DD.md",
  "key_findings": ["[severity][classification][rule_id] description"],
  "needs_human_confirmation": false,
  "reroute_via_router": false,
  "dimension_breakdown": [
    {"dimension": "...", "verdict": "...", "reason": "..."}
  ]
}
```

## Optional Evidence References

- diff log: `features/<active>/evidence/doc-freshness-diff-<topic>.log`（如 reviewer 在判定时引用了具体 file diff）
```

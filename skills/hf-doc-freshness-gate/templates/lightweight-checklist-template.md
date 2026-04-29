# Lightweight Doc Freshness Checklist Template

> 用于 lightweight profile，verdict 文件 ≤ 30 行（NFR-002 上限）

```markdown
# Doc Freshness (lightweight) — <feature-or-task-id>

- Reviewer: <subagent-id>; Profile: lightweight; Date: YYYY-MM-DD
- 被测对象: features/<active>/

## 1. user-visible behavior change（≤ 1 句）

<例：本 task 引入新 skill `hf-doc-freshness-gate`，影响 README 能力清单>

## 2. 仓库根 README 产品介绍段

- 是否需要更新? yes / no / N/A
- verdict: pass | partial | N/A | blocked
- 理由: <≤ 1 句>

## 3. Conventional Commits docs 标记自检

- 最新相关 commit: <hash> "<subject>"
- 含 `docs:` 前缀? yes / no / N/A
- verdict: pass | partial | N/A

## 4. 整体 verdict

<one of: pass | partial | N/A | blocked>

## 5. Next Action

<one of: hf-completion-gate | hf-test-driven-dev | hf-increment | hf-traceability-review | hf-workflow-router>
```

> **NFR-002 兜底**：本模板填完后 wc -l 应 ≤ 30 行（含 metadata header）；超出说明该 case 应升级到 standard profile。

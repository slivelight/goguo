# HYP-001 (A1) Probe Result — User preference: new gate vs extend `hf-finalize`

- 状态: 已结束
- 主题: HF 用户在"task / 增量级文档刷新合同应该长在哪里"上的偏好（A1 新 gate vs A2 扩 finalize vs A3 嵌 review）
- 对应 plan: `./probe-plan.md`
- 结论: **Pass**

## 1. 结果摘要

- 一句话结论: **5 / 5 desk-research 维度全部命中"A1 新独立 gate 在 HF 现行治理框架下严格优于 A2 扩 finalize 与 A3 嵌入 review"**，HYP-001 通过 ≥ 3 / 5 门槛，且未发现任何反向硬证据；HYP-001 (A1) Confidence 由 medium 升至 high，Blocking 由"是"翻为"否"，可推进 `hf-specify`。

## 2. 与事先 Success / Failure Threshold 的对照

| 维度 | 事先阈值 | 实际结果 | 判定 |
|---|---|---|---|
| E1 角色分离纪律 | A1 严格遵守 ↔ A2 / A3 至少一处违反；命中即 +1 | A2 让 `hf-finalize`（closeout 角色）同时承担 gate 角色；A3 让 reviewer 节点同时承担 gate 职责；A1 严格四角色分离 | **A1 命中** |
| E2 既有"不允许替代"规则模式 | A2 / A3 命中既有"扩展现有节点取代独立节点"被显式禁止的同构模式 | `methodology-coherence.md` §二 line 137-138 已显式禁止 `hf-bug-patterns` 升 mandatory gate / Fagan 全流程取代独立 review；A2 / A3 命中同构禁止模式 | **A1 命中** |
| E3 Gate 节点形态可比性 | A1 可直接套既有 gate 三段合同；A2 / A3 不能 | A1 可直接复用 Hard Gates + Verification + fresh evidence 模板；A2 是 Closeout/PMBOK 形态；A3 是 review 形态 | **A1 命中** |
| E4 Profile 分级机制可继承（lightweight 不退化） | A1 可继承 sync-on-presence + profile；A2 把强制同步加到 `task closeout` 会破坏 lightweight | A2 直接破坏 `lightweight` 的"task closeout 走最小路径"承诺；A1 完全可继承 | **A1 命中** |
| E5 Task closeout 既有合同不被破坏 | A1 不动 `hf-finalize`；A2 直接改合同形状 | A2 必须修改 `hf-finalize` §3A + §4；A1 完全不动 | **A1 命中** |
| **合计** | **≥ 3 / 5 → Pass** | **5 / 5** | **Pass** |

详见 `artifacts/desk-research-evidence.md`，五维度每条带 file:line 引用 + 关键句节选。

## 3. 关键证据

证据归档：`docs/experiments/2026-04-23-hf-doc-freshness-gate-hyp-001/artifacts/desk-research-evidence.md`

核心引用源：

- `docs/principles/hf-sdd-tdd-skill-design.md` §"不做'一个总 skill'，而做 family"（line 109-111）—— 四层职责分离原则
- `docs/principles/methodology-coherence.md` §二 "不允许替代清单"（line 122-139）—— 同构禁止模式
- `docs/principles/methodology-coherence.md` §评审层 / 验证 / 门禁层（line 86-100）—— author / reviewer / gate 三角色分工
- `skills/hf-finalize/SKILL.md` §3A（line 102）+ §4（line 112-137）+ §"和其他 Skill 的区别"（line 225）—— `hf-finalize` 既有合同形状
- `skills/docs/hf-workflow-shared-conventions.md` §长期资产同步规则（line 221）—— sync-on-presence + finalize 消费 gate 结论

最小必要证据 = 5 项，全部由权威治理文档与既有 SKILL.md 直接引用，无猜测。

## 4. 与假设 Impact If False 的对照

- **假设站住（Pass 路径）** Confidence 由 medium 升至 high；HYP-001 不再 Blocking。具体置信度提升幅度：5 / 5 维度命中（远超 3 / 5 门槛），且无反向硬证据，置信度提升合理。
- **是否仍保留为 Key Hypothesis**：不再保留为 Blocking，但建议在 spec Key Hypotheses 中以 `confidence: high, source: docs/experiments/2026-04-23-hf-doc-freshness-gate-hyp-001/probe-result.md` 形式留痕，便于 spec / design 阶段反向追溯。
- 如本结果在未来真实 HF 用户访谈中被反向证伪，应通过 `hf-increment` 或新一轮 `hf-experiment` 修订；不在本轮主链中预先承诺。

## 5. 回流动作

| 动作 | 状态 | 备注 |
|---|---|---|
| 更新 HYP-001 的 Confidence (medium → high) | **本 commit 后续任务** | 在 discovery 草稿 §6 / §11 标 "Closed by HYP-001 probe; confidence: high" |
| 更新 HYP-001 的 Blocking (是 → 否) | **本 commit 后续任务** | discovery §6 + spec §4 Key Hypotheses |
| 更新 discovery / spec 的 Next Action Or Recommended Skill | **本 commit 后续任务** | discovery §15 → `hf-specify`；新建 spec 时 `Current Stage` = `hf-specify` |
| (假设未被证伪) 修订 candidate wedges / 排除项 | N/A | 假设站住，无需修订；A2 / A3 仍保留为 spec 中 *Considered Alternative* |

## 6. 是否继续再做一次 probe？

否。结论 = **Pass**，5 / 5 命中，无反向证据，desk research 已经足以回到主链。

未来如能接入真实 HF 用户访谈通道（当前 cloud agent 环境不具备），可在 spec / design 阶段顺带追问一次以提升外部置信度（属于 nice-to-have，非阻塞）。

## 7. 学习点

- HF 现有的 `methodology-coherence.md` "不允许替代清单"是判断"应该新增独立节点 vs 扩展现有节点"决策的强工具；建议未来类似立项 probe 时优先把它作为 desk research 第一证据源。
- 推迟到 spec 阶段考虑：把"sync-on-presence + profile 分级"两条纪律作为 spec NFR 的硬门槛之一，避免新 gate 在 lightweight 下退化为跳过。

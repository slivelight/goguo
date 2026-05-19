# ADR-0001：记录架构决策

- 状态：已接受
- 日期：2026-04-29

## 背景

GoGuo 正从 `hf-strategy-discovery` 进入 HarnessFlow 产品规划阶段。项目需要一个仓库级、可长期保留的位置，用于记录能够跨越单个 feature 周期的架构决策。

项目工件布局约定将 `docs/adr/` 作为 Architecture Decision Records 的唯一权威池。ADR 编号在仓库范围内统一管理，从 `0001` 开始递增，永不复用。

## 决策

所有重要架构决策都使用 Architecture Decision Records（ADR）记录。

- ADR 文件统一放在 `docs/adr/` 下。
- ADR 文件名采用 `NNNN-<slug>.md` 格式。
- 编号从 `0001` 开始。
- 编号永不复用，即使某个决策后续被替代。
- feature 级设计文档通过 ADR ID 引用决策，例如 `ADR-0001`。

## 影响

- 后续架构决策必须写入 ADR，而不是隐藏在临时 feature 说明中。
- feature 级设计文档可以通过链接 ADR 保持简洁，不必重复承载长期决策原文。
- 评审者可以追溯重大技术决策的原因，并检查后续工作是否仍然遵守这些决策。

# 战略发现阶段评测

这个目录包含 `hf-strategy-discovery` 的评测 prompts。

## 目的

这些评测用于验证 strategy discovery 节点是否真正做到：

- 先收敛市场洞察、竞品分析和战略方向，而不是直接写功能规格
- 区分 confirmed facts、assumptions 和 strategic decisions
- 把战略洞察写成可交给用户评审的草稿
- 输出满足 `hf-product-discovery` 输入契约的 Bridge to Product Discovery
- 在项目初始化阶段正确补齐档0必需文档

## Running

每条 eval 使用 `prompt` 模拟用户请求，用 `expectations` 描述必须满足的行为 contract。

## Fixture-Backed Cases

部分评测会在 `files` 字段里附带真实战略洞察输入或评审回修工件，用来模拟：

- 从零散行业/市场信息收敛成可评审战略洞察草稿
- 在用户评审打回后，针对具体战略决策做定向回修
- 项目初始化阶段补齐档0必需文档

## 产品规模档位覆盖

评测覆盖三档产品规模（small/medium/large）和 HF profile 激活条件：

- small：MVP/单功能模块（本 skill 在 full profile 下仍激活）
- medium：功能扩展/垂直应用
- large：产品组合/企业战略

**重要**：本 skill 仅在 HF 框架 **full profile** 下激活。standard / lightweight profile 的评测用于验证 skill 正确识别激活条件并拒绝执行或跳过。
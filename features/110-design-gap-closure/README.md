# Feature 110: F001~004 设计-实现 Gap 闭环

- **Feature**: 110-design-gap-closure
- **阶段**: `hf-specify`
- **状态**: 草稿
- **当前活跃任务**: 无（spec + design 待评审）

## 状态总览

| 阶段 | 状态 | 完成日期 |
|------|------|----------|
| hf-product-discovery | — | — |
| hf-specify | 草稿 | 2026-06-09 |
| hf-design | 草稿 | 2026-06-09 |
| hf-tasks | — | — |
| hf-test-driven-dev | — | — |
| hf-finalize | — | — |

## 关键工件

| 文件 | 路径 |
|------|------|
| 需求规格 | `features/110-design-gap-closure/spec.md` |
| 设计文档 | `features/110-design-gap-closure/design.md` |
| 任务拆解 | — (待 hf-tasks) |
| 进度 | — (待 hf-test-driven-dev) |

## 覆盖范围

本 feature 对 F001~F004 设计规格与当前 codebase 之间的 20 项剩余差距做系统性闭环。已排除 F101~F109 已覆盖的项。
<!-- ? id:23;status:open;date:2026-06-09T14:30 "20项剩余差距"与实际gap数不一致，G110-1~G110-23共23项（P0:3+P1:9+P2:11=23），建议修正为23项 -->

### P0（阻塞发布）3 项

- G110-1: ProbeService 真实探测实现（替代 MockProbeClient）
- G110-2: NodePool ↔ SubscriptionParser 导入管道闭环
- G110-3: Wizard Step 3 手工调整引导实现

### P1（核心交互完整性）9 项

- G110-4~G110-12: 组件独立化、五要素诊断、规则预览、通知语义化、StatusBar/Header、仪表盘部署模式、内嵌诊断、通知入口

### P2（渐进改善）11 项

- G110-13~G110-23: IpScanner扩展、reload解析、审计过滤、节点元数据、Settings配置、CodeBlock高亮、UI术语、按钮文案、Wizard顺序、apply_rules命令、shadcn/ui引入
<!-- ? id:24;status:open;date:2026-06-09T14:30 P2含11项，其中G110-23(shadcn/ui引入)单项工作量可能超过P0三项之和（全局样式体系变更+5组件迁移+CSS变量体系），建议评估G110-23是否需拆为独立feature -->

## 相关 ADR

- ADR-0002: Tauri Desktop Framework
- ADR-0006: React + TypeScript + shadcn/ui

## 下游依赖

- 无（本 feature 是补齐型，不产生新接口供下游消费）
<!-- ? id:25;status:open;date:2026-06-09T14:30 spec §1.3显示F109状态为"进行中"，但本feature假设F109已覆盖的gap不再列入。若F109未完成，部分gap可能仍需本feature处理。建议确认F109关闭状态后再冻结本feature范围 -->
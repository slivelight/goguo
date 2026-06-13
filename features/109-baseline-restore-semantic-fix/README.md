# Feature 109: Baseline 恢复语义修复

- **Feature**: 109-baseline-restore-semantic-fix
- **状态**: `hf-specify` → `hf-design`（待评审）
- **日期**: 2026-06-08
- **上游输入**:
  - F001 `spec.md` / `design.md` — 原始需求与设计锚点
  - OPP-002 Gap 分析报告（2026-06-07 会话产出）
  - F108 commit (`dd7a014`) 引入的 `apply_proxy_env` + `activate_system_proxy` 硬编码
- **相关 ADR**: 待分配
- **关键工件**:
  - [spec.md](spec.md)
  - [design.md](design.md)
- **Active Task**: 待 tasks.md 完成后确定
- **Closeout 状态**: 未开始

## 一句话定位

修复"立即恢复"按钮的语义反转（实际重启代理而非恢复 baseline），以及 F001 设计-实现之间 2 个 P0 + 3 个 P1 + 4 个 P2 级 gap。
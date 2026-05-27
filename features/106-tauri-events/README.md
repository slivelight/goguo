# F106: Tauri Events 补齐

**类型**: 修复 Feature（引用已有 design，跳过 specify/design，保留 TDD + review + gate）

**问题**: F001 design §4 定义 6 个 Tauri 事件，v0.1.0 仅发射 3 个（`baseline:confirmed`, `baseline:deviation-detected`, `service:stopped`）。缺失：`recovery:started`, `recovery:completed`/`recovery:failed`, `service:started`, `proxy-guard:recovery-triggered`。

**Authority Sources**:
- F001 design: `features/001-baseline-restore/design.md` §4

**状态**: 实施完成，待 review + closeout

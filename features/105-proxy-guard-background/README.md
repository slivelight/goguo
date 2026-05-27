# F105: ProxyGuard 后台定时监控

**类型**: 修复 Feature（引用已有 spec/design，跳过 specify/design，保留 TDD + review + gate）

**问题**: `ProxyGuard` 有 `check_and_recover()` 方法和 `check_interval_secs: 3` 配置，但无后台定时调度线程。spec FR-2.5.2-R4 要求"检测到不可达时立即恢复到 baseline"。

**Authority Sources**:
- F001 spec: `features/001-baseline-restore/spec.md` §FR-2.5.2-R4

**状态**: 实施完成，待 review + closeout

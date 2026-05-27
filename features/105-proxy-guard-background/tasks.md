# F105 Tasks

### Phase 1: 后台守护线程
- [x] T1.1: `proxy_guard_loop()` — 周期调用 `check_and_recover()`，按 GuardAction 分发事件
- [x] T1.2: `trigger_baseline_restore()` — ProxyGuard 触发恢复时调用，CAS 防并发
- [x] T1.3: Tauri setup 中 spawn 后台线程

### Phase 2: 事件发射
- [x] T2.1: `GuardAction::Restarted` → emit `service:started` + `proxy-guard:recovery-triggered`
- [x] T2.2: `GuardAction::RecoveryTriggered` → 调用恢复 + emit `recovery:started/completed/failed`
- [x] T2.3: `GuardAction::Healthy` → 无操作

### Phase 3: 收口
- [x] T3.1: 全量回归测试 — 567 tests
- [x] T3.2: clippy 零警告

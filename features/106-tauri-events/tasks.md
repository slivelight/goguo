# F106 Tasks

### Phase 1: 事件发射补齐
- [x] T1.1: `tauri_stop_service` 中发射 `recovery:started`（恢复前）
- [x] T1.2: `tauri_stop_service` 中发射 `recovery:completed`（恢复成功后）
- [x] T1.3: `proxy_guard_loop` 中发射 `service:started`（mihomo 重启后）
- [x] T1.4: `proxy_guard_loop` 中发射 `proxy-guard:recovery-triggered`（重启尝试时）
- [x] T1.5: `trigger_baseline_restore` 中发射 `recovery:completed`/`recovery:failed`

### Phase 2: 收口
- [x] T2.1: 全量回归测试 — 567 tests
- [x] T2.2: clippy 零警告

## 未实现（future work）
- `recovery:item-completed`：当前 `restore_to_baseline()` 是同步批量执行，无逐项回调。需要重构 BaselineManager API。

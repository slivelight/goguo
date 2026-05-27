# F104 Progress

## Status: 实施完成，待 closeout

## Test Evidence
- Baseline: 560 tests (post F101)
- After F103+F104: 567 tests (+7)
- `app_state_is_restoring_default_false`: 1 test
- `is_restoring_set_and_clear`: 1 test
- `stop_service_blocked_when_restoring`: 1 test
- `stop_service_allowed_when_not_restoring`: 1 test
- Clippy: zero warnings

## Files Changed

### New Files
- `src/stores/recovery-store.ts` — Zustand store (isRestoring, progress, fetchRecoveryStatus)

### Modified Files
- `src-tauri/src/commands/baseline.rs` — AppState.is_restoring, check_not_restoring(), tauri_get_is_restoring
- `src-tauri/src/lib.rs` — 注册 tauri_get_is_restoring 命令
- `src/lib/tauri-ipc.ts` — 新增 getIsRestoring()
- `src/pages/DashboardPage.tsx` — RecoveryOverlay 集成 + polling + 按钮禁用

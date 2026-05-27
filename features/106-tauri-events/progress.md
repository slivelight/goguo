# F106 Progress

## Status: 实施完成，待 closeout

## Test Evidence
- Baseline: 567 tests
- After F106: 567 tests（事件发射为副作用，无额外单元测试——Payload 结构已有 roundtrip 测试）
- Clippy: zero warnings

## Event Coverage

| 事件 | 触发位置 | 状态 |
|------|---------|------|
| `baseline:confirmed` | `tauri_confirm_baseline` | ✅ 已有 |
| `baseline:deviation-detected` | `tauri_get_baseline_status` | ✅ 已有 |
| `service:stopped` | `tauri_stop_service` | ✅ 已有 |
| `recovery:started` | `tauri_stop_service` + `trigger_baseline_restore` | ✅ F106 新增 |
| `recovery:completed` | `tauri_stop_service` + `trigger_baseline_restore` | ✅ F106 新增 |
| `recovery:failed` | `trigger_baseline_restore` | ✅ F106 新增 |
| `service:started` | `proxy_guard_loop` | ✅ F105+F106 新增 |
| `proxy-guard:recovery-triggered` | `proxy_guard_loop` | ✅ F105+F106 新增 |
| `recovery:item-completed` | — | ❌ future work |

**覆盖率**：8/9 事件已实现（89%）

## Files Changed

### Modified Files
- `src-tauri/src/commands/baseline.rs` — `tauri_stop_service` 添加 `recovery:started`/`recovery:completed` 发射；`trigger_baseline_restore` 添加全套 recovery 事件

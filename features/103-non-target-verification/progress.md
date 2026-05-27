# F103 Progress

## Status: 实施完成，待 closeout

## Test Evidence
- Baseline: 560 tests (post F101)
- After F103+F104: 567 tests (+7)
- Clippy: zero warnings

## Files Changed

### Modified Files
- `src-tauri/src/managers/baseline_manager.rs` — 新增 `NonTargetVerification`, `SiteProbeDetail`, `verify_non_target_sites()`；`restore_to_baseline()` 调用验证
- `src-tauri/src/commands/baseline.rs` — `ServiceStoppedPayload` 扩展；`stop_service()` 透传验证结果
- `src/lib/types.ts` — 新增 `SiteProbeDetail`, `NonTargetVerification` 接口
- `src/stores/__tests__/service-store.test.ts` — 测试更新

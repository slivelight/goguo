# F105 Progress

## Status: 实施完成，待 closeout

## Test Evidence
- Baseline: 567 tests (post F103+F104)
- After F105+F106: 567 tests（纯行为新增，无额外单元测试——后台循环需集成环境验证）
- Clippy: zero warnings

## Files Changed

### Modified Files
- `src-tauri/src/commands/baseline.rs` — 新增 `proxy_guard_loop()` + `trigger_baseline_restore()`
- `src-tauri/src/lib.rs` — setup 中 spawn 后台线程

## 设计决策

- 锁获取顺序始终 `proxy_guard` → `mihomo_manager`，避免死锁
- `trigger_baseline_restore` 使用 `compare_exchange` CAS 防止并发恢复
- `check_interval` 硬编码为 3 秒（ProxyGuardConfig default），后续可从 config 暴露

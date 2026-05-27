# F105+F106 验证记录：回归测试

- 日期：2026-05-22
- 类型：`hf-regression-gate`
- 验证对象：F105 + F106 全部变更（未提交，working tree）

## 回归测试

```
$ cargo test
test result: ok. 567 passed; 0 failed; 0 ignored

$ cargo clippy --all-targets -- -D warnings
Finished dev profile — zero warnings
```

## 新增代码覆盖

| 模块 | 说明 |
|------|------|
| `proxy_guard_loop()` | 后台循环（集成级验证，单元测试依赖已有 ProxyGuard tests） |
| `trigger_baseline_restore()` | CAS 恢复触发 + 事件发射 |
| `tauri_stop_service` 事件 | `recovery:started` + `recovery:completed` |
| Payload roundtrip tests | 已有 6 个 payload 结构的 roundtrip 测试 |

## 结论

**PASS** — 567 测试全绿，clippy 零警告，无回归。

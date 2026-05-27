# F103+F104 验证记录：回归测试

- 日期：2026-05-22
- 类型：`hf-regression-gate`
- 验证对象：F103 + F104 全部变更（commit `852b32e`）

## 回归测试

```
$ cargo test
test result: ok. 567 passed; 0 failed; 0 ignored

$ cargo clippy --all-targets -- -D warnings
Finished dev profile — zero warnings
```

## 新增测试覆盖

| 模块 | 测试数 | Feature |
|------|--------|---------|
| baseline_manager::verify_non_target_sites | 2 | F103 |
| baseline.rs::NonTargetVerification payload | 1 | F103 |
| baseline.rs::is_restoring tests | 4 | F104 |
| service-store.test.ts | 1 (updated) | F103 |

## 结论

**PASS** — 567 测试全绿，clippy 零警告，无回归。

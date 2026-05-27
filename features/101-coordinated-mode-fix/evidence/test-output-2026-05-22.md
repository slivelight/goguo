# F101 Evidence：测试命令输出

- 日期：2026-05-22
- 提交范围：`4f472bf..185dcd4`（F101 三个 feat 提交）

## Fresh Evidence

```
$ cargo test
test result: ok. 560 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo clippy --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## 关键测试项

- `command_executor::tests::*` — 14 tests (mock executor, WSL bridge, PS bridge)
- `windows_base::tests::*` — 18 tests (6 parse functions)
- `wsl_remote::tests::*` — 15 tests (7 state items read/write)
- `windows_remote::tests::*` — 23 tests (9 state items read/write)
- `deployment_manager_create_adapters_per_mode` — Coordinated creates 2 adapters
- `dual_adapter_coordinated_mode` — 端到端协调模式

# F102 Tasks

### Phase 1: 基础设施
- [x] T1.1: `LinuxBaseAdapter.write_proxy_env()` — 读-改-写策略写入 `/etc/environment`
- [x] T1.2: `is_proxy_env_line()` 辅助函数 — 识别大小写变体

### Phase 2: 替换空操作
- [x] T2.1: `WslAdapter.write_state(ID_PROXY_ENV)` — 调用 `write_proxy_env`
- [x] T2.2: `LinuxAdapter.write_state(ID_PROXY_ENV)` — 调用 `write_proxy_env`

### Phase 3: 收口
- [x] T3.1: 全量回归测试 — 576 tests, zero regressions
- [x] T3.2: clippy 零警告

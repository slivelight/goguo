# F108 Closeout

## 修复内容
1. ProxyGuard 不区分"用户主动停止"和"意外崩溃"，一律自动拉起 mihomo
2. proxy-env 被当作基线 Restorable 项管理，但它是服务生命周期覆盖层，与基线模型冲突

## 行为变更
| 场景 | 旧行为 | 新行为 |
|------|--------|--------|
| 用户点击"停止服务" | mihomo kill → 3s 后 ProxyGuard 重启 | mihomo kill → ProxyGuard 跳过重启 |
| 用户点击"立即恢复" | 重新评估 | 重新评估 + 写入 proxy-env + 清除暂停标记 |
| proxy-env 与基线 | Restorable，参与基线采集/恢复/比对 | Excluded，由服务生命周期管理 |
| 停止服务后 proxy-env | 可能残留（取决于基线数据） | 强制清除 |
| 启动服务后 proxy-env | 依赖基线恢复 | 强制写入 127.0.0.1:{mixed_port} |

## 代码变更
### F108-1: service_paused
- `src-tauri/src/commands/baseline.rs`:
  - `AppState`: 新增 `service_paused: AtomicBool`
  - `tauri_stop_service`: 停止后设置 `service_paused = true`
  - `proxy_guard_loop`: 检查 `service_paused`，为 true 时跳过重启
  - `tauri_trigger_readjustment`: 清除 `service_paused = false`
  - 新增 4 个测试

### F108-2: proxy-env 服务生命周期
- `src-tauri/src/adapters/wsl.rs`: proxy-env category `Restorable` → `Excluded`
- `src-tauri/src/adapters/linux.rs`: 同上
- `src-tauri/src/managers/baseline_manager.rs`:
  - 新增 `clear_proxy_env()` — 清除全局代理配置
  - 新增 `apply_proxy_env(mixed_port)` — 写入代理配置
  - 新增 `write_proxy_env_values()` 内部辅助方法
- `src-tauri/src/commands/baseline.rs`:
  - `tauri_stop_service`: 恢复基线后调用 `clear_proxy_env()`
  - `tauri_trigger_readjustment`: 恢复时调用 `apply_proxy_env(mixed_port)`
- `src-tauri/src/managers/mihomo_manager.rs`:
  - 新增 `mixed_port()` getter
  - 修复 F107 遗留 clippy 警告
- `src-tauri/tests/integration_wsl_linux.rs`:
  - Restorable count 4 → 3，Excluded count 0 → 1
  - category 断言增加 Excluded 变体

## 测试结果
- 单元测试: 550 passed（含 4 个 F108 新增）
- 全量测试: 585 passed（550 lib + 11 + 5 + 19 integration）
- Clippy: 零警告

## 交互流程
```
停止服务:
  mihomo.stop()
  → restore_to_baseline()（不含 proxy-env）
  → clear_proxy_env()（强制清除 /etc/environment 中的代理行）
  → service_paused = true（ProxyGuard 不再重启）

恢复服务:
  service_paused = false
  → apply_proxy_env(7890)（写入代理配置到 /etc/environment）
  → start_initial_assessment()（重新评估）
```

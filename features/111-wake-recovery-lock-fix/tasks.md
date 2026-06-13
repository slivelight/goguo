# F111 Tasks: 休眠唤醒后窗口冻结 + 代理节点退化自愈

## Authority Source

- F001 spec §FR-2.5.2-R4 (ProxyGuard)
- F001 design §4 (Tauri Events, F106 待实现)
- `src-tauri/src/commands/baseline.rs:1078-1126` (proxy_guard_loop)
- `src-tauri/src/managers/mihomo_manager.rs:339-348` (check_api_health)
- `src-tauri/src/services/proxy_guard.rs` (check_and_recover)

## Task Dependency Graph

```
T1 (TCP 超时) ─┐
                ├→ T3 (锁粒度改造) → T5 (时间跳跃检测) → T6 (Tauri 事件)
T2 (check_and_recover 签名) ─┘                              │
                                                            ▼
T4 (flush_urltest_groups) ──────────────────────────────→ T5
```

---

## T1: L0-B — TCP 健康检查超时缩短

**文件**: `src-tauri/src/managers/mihomo_manager.rs:345`

**改动**:
- `check_api_health` 中 `connect_timeout` 从 `Duration::from_secs(2)` → `Duration::from_millis(500)`
- mihomo 在本机，正常响应 <10ms，500ms 充分判定异常，消除最长 2s 阻塞

**测试**: 现有 proxy_guard 测试不受影响（不涉及真实 TCP 连接）

---

## T2: L0-A — ProxyGuard.check_and_recover 签名拆分

**文件**: `src-tauri/src/services/proxy_guard.rs`

**改动**:
将 `check_and_recover(&mut self, mihomo: &mut MihomoManager)` 拆为：
```rust
pub fn check_and_recover(&mut self, is_running: bool, mihomo: &mut MihomoManager) -> GuardAction
```
- 调用方在外部执行健康检查（可能阻塞），传入布尔结果
- `is_running == true` → 重置计数器，返回 Healthy
- `is_running == false` → 走原有重启/恢复逻辑

**测试**:
- 更新 `proxy_guard.rs` 中所有 `check_and_recover` 调用：传入 `false`（模拟 mihomo 不在）
- 新增测试：传入 `true` 验证 Healthy 路径（当前无法测试，因为需要真实 mihomo）

---

## T3: L0-A — proxy_guard_loop 锁粒度改造

**文件**: `src-tauri/src/commands/baseline.rs:1096-1100`

**现状**:
```rust
let action = {
    let mut guard = state.proxy_guard.lock().unwrap();     // 锁 1
    let mut mihomo = state.mihomo_manager.lock().unwrap();  // 锁 2
    guard.check_and_recover(&mut mihomo)                    // 持双锁做 TCP 阻塞
};
```

**改为**:
```rust
// 1. 锁外执行阻塞健康检查
let api_addr = {
    let mihomo = state.mihomo_manager.lock().unwrap();
    mihomo.api_address().to_string()
};  // 锁立即释放
let is_running = TcpStream::connect_timeout(&addr, 500ms).is_ok();

// 2. 仅在需要重启时才持双锁
let action = {
    let mut guard = state.proxy_guard.lock().unwrap();
    if is_running {
        guard.reset_restart_count();
        GuardAction::Healthy
    } else {
        let mut mihomo = state.mihomo_manager.lock().unwrap();
        guard.check_and_recover(false, &mut mihomo)  // T2 的新签名
    }
};
```

**测试**: 行为不变，仅锁持有时间缩短。现有集成测试覆盖。

---

## T4: L1-B — MihomoManager 新增 flush_urltest_groups

**文件**: `src-tauri/src/managers/mihomo_manager.rs`

**新增方法**:
```rust
pub fn flush_urltest_groups(&self) -> Result<usize, MihomoError>
```
1. `GET /proxies` → 解析 JSON → 筛选所有 `"type": "URLTest"` 的组
2. 对每个组：`GET /proxies/{name}/delay?url=http://www.gstatic.com/generate_204&timeout=2000`
3. 返回刷新的组数量

**实现方式**: 复用 `reload_config` 的 raw TCP + HTTP 模式（项目无 reqwest 依赖）

**注意**: `is_running()` 为 false 时直接返回 `NotRunning` 错误

**测试**:
- Mock mihomo API 响应，验证 URLTest 组筛选逻辑
- 验证 API 不可达时优雅返回错误

---

## T5: L1-A + L1-B — 时间跳跃检测 + 唤醒恢复

**文件**: `src-tauri/src/commands/baseline.rs` (proxy_guard_loop)

**改动**:
```rust
let mut last_tick = Instant::now();
loop {
    thread::sleep(Duration::from_secs(3));
    let elapsed = last_tick.elapsed();
    last_tick = Instant::now();

    // ... is_restoring / service_paused 检查 ...

    // 唤醒检测：3s 间隔出现 >30s 的跳跃
    if elapsed > Duration::from_secs(30) {
        let _ = app.emit("proxy:recovering", serde_json::json!({
            "reason": "post-wake",
            "sleep_duration_secs": elapsed.as_secs(),
        }));
        {
            let mihomo = state.mihomo_manager.lock().unwrap();
            let _ = mihomo.flush_urltest_groups();  // T4
        }
        let _ = app.emit("proxy:recovered", serde_json::json!({
            "flushed_groups": true,
        }));
    }

    // T3: 改造后的健康检查（锁外做阻塞 IO）
    ...
}
```

**测试**:
- 直接验证逻辑：mock `elapsed` 值 > 30s 触发恢复
- 验证事件被正确 emit

---

## T6: L1-C — 前端事件监听 + 通知

**文件**:
- `src/lib/types.ts` — 新增 `ProxyRecoveringPayload` / `ProxyRecoveredPayload`
- `src/lib/events.ts` — 新增 `proxy:recovering` / `proxy:recovered` 事件类型和 subscribe 函数
- `src/stores/service-store.ts` — 新增 `isRecovering` 状态 + handlers
- `src/stores/notif-store.ts` — 新增 wake recovery 通知 handlers

**改动**:

types.ts:
```typescript
export interface ProxyRecoveringPayload {
  reason: string;
  sleep_duration_secs: number;
}
export interface ProxyRecoveredPayload {
  flushed_groups: boolean;
}
```

events.ts:
```typescript
| 'proxy:recovering'
| 'proxy:recovered'
```

notif-store.ts:
```typescript
handleProxyRecovering: (payload) => addNotification('info', '正在恢复连接', `系统唤醒后自动恢复代理...`)
handleProxyRecovered: (payload) => addNotification('success', '连接已恢复', `代理节点已刷新`)
```

**测试**: 前端测试验证事件订阅和状态更新

---

## 验收标准

- [ ] PC 休眠/唤醒后，GoGuo 窗口可正常拖动和操作（无冻结）
- [ ] 唤醒后 3~6 秒内，代理节点自动切换到存活节点
- [ ] 唤醒后前端 NotifBar 显示"正在恢复连接"→ "连接已恢复"
- [ ] `cargo clippy` 零警告
- [ ] `cargo test` 全部通过
- [ ] 不引入新的外部依赖（reqwest 等）

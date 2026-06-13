# F111: 休眠唤醒后窗口冻结 + 代理节点退化自愈

## 级别
BLOCKER

## 问题描述

PC 休眠/唤醒后，GoGuo 出现两个连锁故障：

1. **窗口冻结**（BLOCKER）：主窗口完全无响应，无法拖动、无法点击。用户无法通过 UI 进行任何操作。
2. **代理节点退化**（HIGH）：mihomo 所有代理节点 TCP 长连接因休眠断裂，url-test 组选中节点已死（alive=false），但 60s 定时器因休眠暂停未触发切换。GitHub 极慢，Oracle 返回 403。

### 因果链

```
PC 休眠
  → mihomo 代理节点 TCP 长连接全部断裂
  → url-test 定时器暂停，未触发节点切换
  → 用户访问走死节点 → 超时/极慢/403
  → 同时：mihomo 死连接积压，API 响应变慢
  → ProxyGuard 每 3s 持双 Mutex 做 TCP connect(2s) 阻塞调用
  → 前端 invoke() 等同一 Mutex → GLib 主循环饥饿 → 窗口冻结
```

## 复现步骤

1. GoGuo 运行中，mihomo 代理正常工作
2. PC 进入休眠（合盖 / 电源设置）
3. 唤醒 PC
4. 观察：(a) 窗口冻结，无法拖动 (b) 浏览器访问代理站点极慢或失败

## 修复方案

### L0: 防御性基础设施（解除窗口冻结）

**L0-A: 锁粒度改造**
- `proxy_guard_loop` 中将"持锁做阻塞 IO"拆为"快拷贝→释放锁→阻塞操作→加锁写回"
- 消除 `proxy_guard.lock()` + `mihomo_manager.lock()` 双锁同时持有的阻塞区间

**L0-B: TCP 健康检查加速**
- `check_api_health` 的 `connect_timeout` 从 2s 缩短到 500ms
- mihomo 在本机，正常响应 <10ms，500ms 足够判定异常

### L1: 唤醒感知（休眠后自动恢复）

**L1-A: 时间跳跃检测**
- `proxy_guard_loop` 中通过 `Instant::elapsed()` 检测异常间隔（>30s）
- 检测到时间跳跃视为系统休眠唤醒事件

**L1-B: 强制刷新 URLTest 组**
- 唤醒检测后，调 mihomo REST API `GET /proxies` 遍历所有 URLTest 类型组
- 对每组触发 `PUT /proxies/{name}` 重新测速，强制切换到存活节点

**L1-C: Tauri 事件通知**
- 检测到唤醒时 emit `proxy:recovering`（前端显示"正在恢复连接…"）
- 刷新完成后 emit `proxy:recovered`（前端更新状态）

## Authority Source

- ProxyGuard 设计: F001 spec §FR-2.5.2-R4, F001 design §4
- MihomoManager: `src-tauri/src/managers/mihomo_manager.rs`
- proxy_guard_loop: `src-tauri/src/commands/baseline.rs:1078-1126`
- Tauri Events: F001 design §4（F106 待实现事件）
- mihomo REST API: https://wiki.metacubex.one/api/

## 影响范围

- `src-tauri/src/commands/baseline.rs`（proxy_guard_loop 改造）
- `src-tauri/src/managers/mihomo_manager.rs`（check_api_health 超时、新增 flush_urltest_groups）
- `src-tauri/src/services/proxy_guard.rs`（check_and_recover 锁拆分）
- `src/stores/service-store.ts`（新增事件监听）
- `src/lib/events.ts`（新增事件类型）

## 不在范围内

- 自动站点探测循环（L2，F112）
- NodePool 智能自愈（L3，F112）
- 节点健康可视化（F112）
- 系统级睡眠/唤醒事件监听（Windows WM_POWERBROADCAST / Linux D-Bus，长期方案）

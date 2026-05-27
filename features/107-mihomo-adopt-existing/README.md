# F107: MihomoManager 启动时检测并纳管已有 mihomo 进程

## 级别
BUG（HIGH）

## 问题描述
每次 `pnpm tauri dev` 或 GoGuo 启动时，`MihomoManager::start()` 无条件 `spawn` 新的 mihomo 子进程，不检查端口是否已被占用。导致：
- 端口冲突错误（7890/9090 bind: address already in use）
- 多个 mihomo 僵尸进程累积
- ProxyGuard 后台线程拉起的实例与 dev 实例冲突

## 复现步骤
1. GoGuo 已运行（mihomo 占用 7890/9090）
2. 执行 `pnpm tauri dev`
3. 观察：新 mihomo 启动失败，端口冲突，僵尸进程残留

## 修复方案
`start()` 方法在 spawn 新进程前，检测 API 端口是否已有 mihomo 监听：
- 端口空闲 → 启动新进程（现有逻辑）
- 端口已有 mihomo 且配置目录匹配 → 纳管（标记为外部启动，不持有 Child handle）
- 端口已有 mihomo 但配置不匹配 → kill 后重启
- 端口被非 mihomo 占用 → 返回 PortConflict 错误

## Authority Source
- MihomoManager 设计: `src-tauri/src/managers/mihomo_manager.rs`
- ProxyGuard 交互: `src-tauri/src/commands/baseline.rs`

## 影响范围
- `MihomoManager::start()` 方法
- `MihomoManager::stop()` 方法（外部纳管的进程不应被 stop kill）
- 新增 `MihomoError::PortConflict` 变体

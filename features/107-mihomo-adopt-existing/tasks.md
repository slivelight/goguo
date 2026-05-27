# F107 Tasks

## T1: RED — 编写纳管逻辑失败测试
- [ ] 测试 `start()` 在端口已有 mihomo 且配置匹配时 → 纳管成功
- [ ] 测试 `start()` 在端口被非 mihomo 占用时 → 返回 PortConflict
- [ ] 测试纳管后 `stop()` 不 kill 外部进程
- [ ] 测试纳管后 `is_running()` 通过 API 检测返回 true

## T2: GREEN — 实现纳管逻辑
- [ ] 新增 `MihomoError::PortConflict` 错误变体
- [ ] 新增 `process_is_externally_managed` 字段
- [ ] 修改 `start()`: 先检测 API 端口，有响应则纳管
- [ ] 修改 `stop()`: 外部纳管的进程不 kill
- [ ] 修改 `is_running()`: 无 Child handle 时仍可通过 API 检测

## T3: 验证
- [ ] cargo test 全部通过
- [ ] cargo clippy 零警告
- [ ] `pnpm tauri dev` 不再产生端口冲突

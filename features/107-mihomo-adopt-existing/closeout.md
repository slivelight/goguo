# F107 Closeout

## 修复内容
MihomoManager::start() 在 spawn 新 mihomo 子进程前，先检测 API 端口是否已有响应。

## 行为变更
| 场景 | 旧行为 | 新行为 |
|------|--------|--------|
| API 端口已有 mihomo 监听 | spawn 新进程 → 端口冲突 → 僵尸进程 | 纳管（标记 externally_managed）→ Ok |
| stop() 外部纳管的进程 | N/A | 不 kill，仅清除 handle |
| Drop 外部纳管的进程 | N/A | 不 kill |
| is_running() 无 Child handle | 返回 false | 通过 API 端口检测返回 true |

## 代码变更
- `src-tauri/src/managers/mihomo_manager.rs`:
  - 新增 `MihomoError::PortConflict` 错误变体
  - 新增 `externally_managed` 字段
  - `start()`: 先检测 API 端口，有响应则纳管
  - `stop()`: 外部纳管的进程不 kill
  - `is_running()`: 无 Child handle 时通过 API 检测
  - `Drop`: 外部纳管的进程不 kill
  - 新增 5 个测试（FakeApi + OS-assigned port 避免并行冲突）

## 测试结果
- 单元测试: 17 passed (含 5 个新增)
- 全量测试: 542 passed
- Clippy: 零警告

## 遗留
- 未实现"检测非 mihomo 进程占用端口"的场景（PortConflict 错误变体已预留）
- 未实现"配置目录不匹配时 kill + 重启"的场景（当前一律纳管）

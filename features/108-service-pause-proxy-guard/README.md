# F108: Service Pause + Proxy-Env Lifecycle

## 级别
HIGH

## Authority Source
- F001 spec §FR-2.6.2-R2（恢复到基线能力）
- F001 spec §FR-2.5.2-R4（ProxyGuard 后台监控）
- F001 design §4（ProxyGuard 自动恢复）

## 问题描述

用户点击"停止服务"后：
1. mihomo 被终止，但 3 秒后 ProxyGuard 自动拉起 → 停止无效
2. 全局代理配置仍然指向 127.0.0.1:7890 → 断网
3. proxy-env 作为基线项管理 → 基线恢复可能写回代理值 → 与停止意图冲突

## 变更包

### F108-1: service_paused
| 类型 | 项 | 说明 |
|------|-----|------|
| New | `AppState.service_paused` | AtomicBool，用户主动停止时设 true |
| Modified | `tauri_stop_service` | 停止后设置 service_paused=true |
| Modified | `proxy_guard_loop` | 检查 service_paused，跳过重启 |
| Modified | `tauri_trigger_readjustment` | 恢复时清除 service_paused=false |

### F108-2: proxy-env 服务生命周期
| 类型 | 项 | 说明 |
|------|-----|------|
| Modified | wsl.rs/linux.rs proxy-env category | Restorable → Excluded |
| New | `BaselineManager::clear_proxy_env()` | 停止时清除全局代理 |
| New | `BaselineManager::apply_proxy_env(port)` | 启动时写入全局代理 |
| Modified | `tauri_stop_service` | 恢复基线后调用 clear_proxy_env |
| Modified | `tauri_trigger_readjustment` | 恢复时调用 apply_proxy_env |
| New | `MihomoManager::mixed_port()` | 暴露配置端口 |

## 不做的事
- 不修改 ProxyGuard 自身逻辑（GuardAction、check_and_recover 不变）
- 不新增前端命令（通过现有 trigger_readjustment 恢复）
- 不修改远程 adapter（wsl_remote/windows_remote）的 proxy-env（协同模式由 F101 管控）

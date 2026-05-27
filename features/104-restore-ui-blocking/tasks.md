# F104 Tasks

### Phase 1: 后端状态锁
- [x] T1.1: `AppState.is_restoring: AtomicBool` — 原子标志位
- [x] T1.2: `check_not_restoring()` — 操作前检查守卫
- [x] T1.3: `tauri_stop_service` — 设置/清除 is_restoring + 检查守卫
- [x] T1.4: `tauri_get_is_restoring` Tauri 命令

### Phase 2: 前端阻塞
- [x] T2.1: `recovery-store.ts` — Zustand store（isRestoring, progress, polling）
- [x] T2.2: `DashboardPage.tsx` — 集成 RecoveryOverlay + polling + 按钮禁用
- [x] T2.3: `getIsRestoring()` IPC 调用

### Phase 3: 收口
- [x] T3.1: 全量回归测试
- [x] T3.2: clippy 零警告

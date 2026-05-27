# F103 Tasks

### Phase 1: 后端验证
- [x] T1.1: `BaselineManager.restore_to_baseline()` — 恢复成功后调用 `verify_non_target_sites()`
- [x] T1.2: `NonTargetVerification` + `SiteProbeDetail` 结构体
- [x] T1.3: `verify_non_target_sites()` — curl 探测 + latency 测量

### Phase 2: 透传到前端
- [x] T2.1: `ServiceStoppedPayload` 扩展 `non_target_verification` 字段
- [x] T2.2: `stop_service()` 提取并透传验证结果
- [x] T2.3: 前端 `types.ts` + 测试更新

### Phase 3: 收口
- [x] T3.1: 全量回归测试
- [x] T3.2: clippy 零警告

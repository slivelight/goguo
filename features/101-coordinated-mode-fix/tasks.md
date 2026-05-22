# F101: 协同部署模式修复

## Authority Sources
- F002 spec: `features/002-wsl-support/spec.md` §FR-2.2.1-R4, SC-2
- F002 design: `features/002-wsl-support/design.md` §2.5
- ADR-0005: `docs/adr/0005-platform-adapter-pattern.md`
- ADR-0007: `docs/adr/0007-*.md`（新增）

## Tasks

### Phase 0: 基础设施
- [x] T0.1: CommandExecutor trait + MockCommandExecutor + SystemCommandExecutor (14 tests)
- [x] T0.2: WslBridgeExecutor (Windows→WSL bridge)
- [x] T0.3: PowershellBridgeExecutor (WSL→Windows bridge)

### Phase 1: WslRemoteAdapter（方向 A: Win→WSL）
- [x] T1.1~T1.3: WslRemoteAdapter with 7 state items (15 tests)

### Phase 2: WindowsRemoteAdapter（方向 B: WSL→Win）
- [x] T2.1~T2.3: windows_base.rs (18 tests) + WindowsRemoteAdapter (23 tests)

### Phase 3: 核心重构
- [x] T3.1: adapters/mod.rs — ungate linux_base, add new modules
- [x] T3.2: DeploymentManager.create_adapters() rewrite — Coordinated creates 2 adapters
- [x] T3.3: AppState::new() uses DeploymentManager for dynamic adapter selection

### Phase 4: 收口
- [x] T3.4: 全量回归测试 — 560 tests, zero regressions
- [x] T3.5: HF 工件
- [ ] T3.6: ADR-0007

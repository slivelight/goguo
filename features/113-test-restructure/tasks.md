# Feature 113: 三层测试重构 — 任务拆解

- **Feature**: 113-test-restructure
- **阶段**: `hf-tasks`
- **状态**: 草稿
- **日期**: 2026-06-11
- **上游设计**: `features/113-test-restructure/design.md`
- **执行约束**: 只写测试，不修生产代码

## 任务依赖图

```
T-01 ──┬── T-02 ──┬── T-06 ──┬── T-10
       │          │          │
       ├── T-03 ──┤          ├── T-11
       │          │          │
       ├── T-04 ──┼── T-07 ──┤
       │          │          │
       └── T-05 ──┘          └── T-12
                                        ── T-13
```

## Batch 1: 测试基建 + F001 验收测试

### T-01: 创建测试基建目录结构与模块入口

**优先级**: P0（所有后续任务的前置依赖）
**预估**: 0.5 天
**验收标准**:
- [ ] `src-tauri/tests/common/mod.rs` 存在，可被 `#[path = "../common/mod.rs"]` 引用
- [ ] `src-tauri/tests/fr_acceptance/mod.rs` 存在，声明子模块（f001, f002, f003, f004, contract, pipeline）
- [ ] `src/__tests__/fr-acceptance/` 目录存在
- [ ] `cargo test --test fr_acceptance` 可执行（0 个测试，编译通过）
- [ ] `cargo clippy --all-targets -- -D warnings` 零新增警告
**实现说明**:
- 创建 `src-tauri/tests/common/mod.rs`（初始为空，占位）
- 创建 `src-tauri/tests/fr_acceptance/` 目录及各子模块空文件
- 验证 Rust 测试发现机制能识别新测试入口
- 不修改 Cargo.toml（使用默认发现）

### T-02: 实现 setup helper 库（TestState + 4 个 helper）

**优先级**: P0（所有 FR 验收测试的前置依赖）
**依赖**: T-01
**预估**: 1 天
**验收标准**:
- [ ] `setup_baseline_confirmed()` 返回 TestState，其 baseline_mgr 内含已确认 baseline
- [ ] `setup_service_running()` 在 baseline_confirmed 基础上模拟 mihomo 运行中
- [ ] `setup_service_stopped()` 在 baseline_confirmed 基础上模拟 mihomo 已停止
- [ ] `setup_site_rules_with_nodes()` 返回 SiteRulesTestState，含节点池和站点规则
- [ ] TestState 包含 `baseline_manager`, `mihomo_manager`, `audit_logger` 字段
- [ ] 所有 helper 内部使用已有 Mock（MockCommandExecutor, MockAuditLog 等），不依赖真实系统
- [ ] helper 的 mock 行为与 spec 描述一致（write_state 记录调用、read_state 返回预期值）
- [ ] `cargo test --test fr_acceptance` 编译通过
**实现说明**:
- 复用已有 Mock 生态：MockCommandExecutor, MockAuditLog, MockProbeClient, MockMihomoReloader, MockShellExecutor
- 参考 `integration_baseline.rs` 中 `setup_env()` 模式
- TestState 使用 `Arc<Mutex<>>` 包裹，与生产代码 AppState 结构对齐
- SiteRulesTestState 额外包含 SiteRuleEngine 和 NodePool

### T-03: F001 验收测试（19 个，核心路径）

**优先级**: P0
**依赖**: T-02
**预估**: 1.5 天
**验收标准**:
- [ ] `src-tauri/tests/fr_acceptance/f001_baseline.rs` 包含 19 个 `fr_` 前缀测试函数
- [ ] 覆盖 F001 的 52 条 FR，按功能节分组（FR-2.1~FR-2.9 + SC-1）
- [ ] 每个测试断言 spec 描述的用户可观测结果（非函数返回值）
- [ ] 已知 gap 对应测试标记 `#[ignore = "F109/F110: 具体描述"]`
- [ ] 预期标记 `#[ignore]` 的测试（8 个）：
  - `fr_2_3_3_restore_only_restorable` → `#[ignore = "F109-P2-109-9"]`
  - `fr_2_4_2_restore_execution_and_audit` → `#[ignore = "F109-P1-109-3"]`
  - `fr_2_4_3_non_target_verification` → `#[ignore = "F109-P1-109-5"]`
  - `fr_2_5_1_proxy_guard_scope` → `#[ignore = "F109-P1-109-4"]`
  - `fr_2_5_2_proxy_guard_response` → `#[ignore = "F109-P1-109-4"]`
  - `fr_2_5_3_proxy_guard_strategy` → `#[ignore = "F109-P1-109-4"]`
  - `fr_2_6_1_recovery_task_persistence` → `#[ignore = "F109-续跑未实现"]`
  - `fr_2_7_1_audit_scope` → `#[ignore = "F109-P1-109-3"]`
  - `fr_2_7_2_audit_format` → `#[ignore = "F110-G110-15"]`
  - `fr_2_8_1_five_element_prompt` → `#[ignore = "F110-G110-5"]`
  - `fr_sc_1_restore_all_restorable` → `#[ignore = "F109-P0-109-1"]`
- [ ] 非忽略测试全部通过（`cargo test --test fr_acceptance f001`）
**实现说明**:
- 测试分组参见 design.md §3.2
- 断言示例：审计日志包含特定操作、baseline 版本递增、恢复操作触发 write_state 调用
- 使用 `setup_baseline_confirmed()` / `setup_service_running()` 构建测试状态

## Batch 2: F002 + F003 验收测试

### T-04: F002 验收测试（8 个，WSL 支持）

**优先级**: P1
**依赖**: T-02
**预估**: 1 天
**验收标准**:
- [ ] `src-tauri/tests/fr_acceptance/f002_wsl.rs` 包含 8 个 `fr_` 前缀测试函数
- [ ] 覆盖 F002 的 19 条 FR，按功能节分组
- [ ] WSL 特定测试在非 WSL 环境标记 `#[ignore]` 并注释原因
- [ ] 已知 gap 对应测试标记 `#[ignore]`：
  - `fr_2_2_1_coordinated_mode` → `#[ignore = "F101: 协同模式不可用"]`
  - `fr_2_5_1_wsl_failure_prompt` → `#[ignore = "F110-G110-5"]`
- [ ] 非忽略测试全部通过

### T-05: F003 验收测试（12 个，站点规则）

**优先级**: P1
**依赖**: T-02
**预估**: 1 天
**验收标准**:
- [ ] `src-tauri/tests/fr_acceptance/f003_site_rules.rs` 包含 12 个 `fr_` 前缀测试函数
- [ ] 覆盖 F003 的 28 条 FR
- [ ] 已知 gap 对应测试标记 `#[ignore]`：
  - `fr_2_3_1_continuous_probe` → `#[ignore = "F110-G110-1"]`
  - `fr_2_3_2_unreachable_recovery` → `#[ignore = "F110-G110-1"]`
  - `fr_2_4_1_rule_preview` → `#[ignore = "F110-G110-6"]`
  - `fr_2_5_1_rule_effectiveness` → `#[ignore = "F101"]`
  - `fr_sc_1_p95_recovery` → `#[ignore = "F110-G110-1"]`
  - `fr_sc_5_five_element_diag` → `#[ignore = "F110-G110-5"]`
- [ ] 非忽略测试全部通过

## Batch 3: F004 验收测试

### T-06: F004 后端验收测试（8 个）

**优先级**: P1
**依赖**: T-02
**预估**: 0.5 天
**验收标准**:
- [ ] `src-tauri/tests/fr_acceptance/f004_backend.rs` 包含 8 个 `fr_` 前缀测试函数
- [ ] 覆盖 F004 后端相关 FR（Tauri command 行为）
- [ ] 已知 gap 对应测试标记 `#[ignore]`：
  - `fr_2_2_1_wizard_baseline_flow` → `#[ignore = "F110-G110-3"]`
  - `fr_2_5_1_rule_preview` → `#[ignore = "F110-G110-6"]`
  - `fr_2_6_1_site_reachability` → `#[ignore = "F110-G110-1"]`
  - `fr_2_7_1_notification` → `#[ignore = "F110-G110-7"]`
- [ ] 非忽略测试全部通过

### T-07: F004 前端验收测试（~15 个）

**优先级**: P1
**依赖**: T-01（目录结构）
**预估**: 1 天
**验收标准**:
- [ ] `src/__tests__/fr-acceptance/f004-ui.test.tsx` 包含 ~15 个 `fr_` 前缀测试
- [ ] 覆盖 F004 前端相关 FR（UI 渲染、交互行为）
- [ ] 断言 DOM 渲染内容（文本、元素存在性），不断言 store 内部状态
- [ ] 使用 vitest + jest-dom，不引入新 npm 依赖
- [ ] 使用 `renderWithRouter` 包裹（复用已有测试模式）
- [ ] `pnpm test -- fr_` 全部通过（非忽略部分）

## Batch 4: 契约测试 + 管道集成测试

### T-08: 契约测试（~20 个）

**优先级**: P1
**依赖**: T-02
**预估**: 1.5 天
**验收标准**:
- [ ] `src-tauri/tests/fr_acceptance/contract.rs` 包含 ~20 个 `contract_` 前缀测试
- [ ] 三类契约全覆盖：
  - 适配器一致性：`contract_adapter_category_consistent` 等（~7 个）
  - DTO 往返：`contract_dto_roundtrip_*`（~8 个）
  - Trait 行为：`contract_trait_*`（~5 个）
- [ ] 适配器契约：验证 Windows/Linux/WSL/WslRemote 对同一 state_item_id 的 category 一致
- [ ] DTO 契约：验证所有 Tauri command 的 Request/Response 类型序列化往返正确
- [ ] Trait 契约：ProbeClient 和 PlatformAdapter 的所有实现通过共享行为测试套件
- [ ] `cargo test --test fr_acceptance contract_` 全部通过（无 `#[ignore]`）

### T-09: 管道集成测试（7 个）

**优先级**: P1
**依赖**: T-02
**预估**: 1 天
**验收标准**:
- [ ] `src-tauri/tests/fr_acceptance/pipeline.rs` 包含 7 个 `pipeline_` 前缀测试
- [ ] 7 条端到端链路全覆盖：
  1. `pipeline_assess_confirm_restore_audit`（F001 全链路）
  2. `pipeline_subscription_to_node_pool_to_rules`（F003 链路）
  3. `pipeline_site_add_probe_reachability`（F003+F004）
  4. `pipeline_probe_five_element_audit`（F001+F003）
  5. `pipeline_proxy_guard_restore`（F001）
  6. `pipeline_wizard_readjustment`（F004）
  7. `pipeline_rule_preview_apply_verify_rollback`（F003）
- [ ] 已知 gap 对应测试标记 `#[ignore]`：
  - `pipeline_subscription_to_node_pool_to_rules` → `#[ignore = "F110-G110-2"]`
  - `pipeline_probe_five_element_audit` → `#[ignore = "F110-G110-5"]`
  - `pipeline_proxy_guard_restore` → `#[ignore = "F109-P1-109-4"]`
  - `pipeline_wizard_readjustment` → `#[ignore = "F110-G110-3"]`
- [ ] 非忽略测试全部通过
- [ ] 每条管道断言终点状态符合 spec 描述

## Batch 5: 追溯矩阵 + 验证

### T-10: FR 追溯矩阵

**优先级**: P0
**依赖**: T-03, T-04, T-05, T-06, T-07, T-08, T-09
**预估**: 1 天
**验收标准**:
- [ ] `docs/test-trace-matrix.md` 存在，包含 142 条 FR 的完整映射
- [ ] 每条记录包含：FR ID、测试函数名、状态（通过/忽略/缺失）、修复 feature ID（如适用）
- [ ] 标注 FR 分组关系（多条 FR 共享同一验收测试时标注）
- [ ] 追溯矩阵中的测试函数名与代码中实际函数名完全一致
- [ ] 状态列与实际 `cargo test` 输出一致

### T-11: 全量验证 + CI 集成

**优先级**: P0
**依赖**: T-10
**预估**: 0.5 天
**验收标准**:
- [ ] `cargo test --test fr_acceptance` 全量执行，非忽略测试全部通过
- [ ] `pnpm test -- fr_` 全量执行，非忽略测试全部通过
- [ ] `cargo clippy --all-targets -- -D warnings` 零新增警告
- [ ] 现有测试不受影响：`cargo test` 全量通过（非 F113 测试不变）
- [ ] `#[ignore]` 测试数量与追溯矩阵记录一致
- [ ] 验收测试套件全量执行 ≤ 60s

### T-12: 发现分流记录

**优先级**: P1
**依赖**: T-10
**预估**: 0.5 天
**验收标准**:
- [ ] 编写验收测试过程中发现的所有新问题记录到 `features/113-test-restructure/progress.md`
- [ ] 每项新问题按分流规则归入：F109 / F110 / 新开 F114+
- [ ] 分流结果在追溯矩阵中标注
- [ ] 非 F109/F110 的 P0/P1 问题已开新 feature 跟踪

## 汇总

| 任务 | 批次 | 优先级 | 预估 | 依赖 | 测试数 |
|------|------|--------|------|------|--------|
| T-01 目录结构 | B1 | P0 | 0.5d | — | 0 |
| T-02 Setup Helper | B1 | P0 | 1d | T-01 | 0 |
| T-03 F001 验收 | B1 | P0 | 1.5d | T-02 | ~19 |
| T-04 F002 验收 | B2 | P1 | 1d | T-02 | ~8 |
| T-05 F003 验收 | B2 | P1 | 1d | T-02 | ~8 |
| T-06 F004 后端 | B3 | P1 | 0.5d | T-02 | ~8 |
| T-07 F004 前端 | B3 | P1 | 1d | T-01 | ~15 |
| T-08 契约测试 | B4 | P1 | 1.5d | T-02 | ~20 |
| T-09 管道测试 | B4 | P1 | 1d | T-02 | ~7 |
| T-10 追溯矩阵 | B5 | P0 | 1d | T-03~09 | 0 |
| T-11 全量验证 | B5 | P0 | 0.5d | T-10 | 0 |
| T-12 发现分流 | B5 | P1 | 0.5d | T-10 | 0 |
| **合计** | | | **10天** | | **~85** |

## 风险提示

- [ ] T-02 的 TestState 设计需仔细对齐生产代码 AppState 结构，否则后续测试无法编译
- [ ] T-03 是最大验收测试组（19 个），编写过程可能发现 F109/F110 之外的新 gap
- [ ] T-08 契约测试需验证 4 种适配器，当前 WSL/WslRemote 适配器可能需要额外的 mock 构建
- [ ] T-09 管道测试中订阅导入管道（#2）当前后端实现可能不完整，需标记 ignore 并记录

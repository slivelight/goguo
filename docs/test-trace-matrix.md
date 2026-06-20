# FR 追溯矩阵

> **配套文档**：能力的 L1~L5 分工见 [test-level-matrix.md](./test-level-matrix.md)（本文档追溯"FR → 测试函数"1:1 关系，level-matrix 追溯"能力 → 测试等级"多对多分工）。

- **Feature**: F113 三层测试重构
- **日期**: 2026-06-12
- **测试套件**: `cargo test --test fr_acceptance` + `pnpm test -- fr_`
- **总测试数**: 88（后端 74 + 前端 14）
- **通过**: 60 | **忽略**: 28 | **失败**: 0

## 统计摘要

| Feature | 测试数 | 通过 | 忽略 | 文件 |
|---------|--------|------|------|------|
| F001 基线恢复 | 19 | 8 | 11 | `f001_baseline.rs` |
| F002 WSL 支持 | 8 | 6 | 2 | `f002_wsl.rs` |
| F003 站点规则 | 12 | 5 | 7 | `f003_site_rules.rs` |
| F004 后端 | 8 | 4 | 4 | `f004_backend.rs` |
| F004 前端 | 14 | 14 | 0 | `f004-ui.test.tsx` |
| 契约测试 | 20 | 20 | 0 | `contract.rs` |
| 管道集成 | 7 | 3 | 4 | `pipeline.rs` |
| **合计** | **88** | **60** | **28** | |

## 忽略测试分流统计

| 修复 Feature | 数量 | 占比 |
|-------------|------|------|
| F109（基线恢复语义修复） | 9 | 32% |
| F110（设计 gap 闭环） | 13 | 46% |
| F101（协同模式） | 2 | 7% |
| F113-discovery（新发现） | 1 | 4% |
| 合并覆盖（pipeline 与 FR 测试指向同一 gap） | 3 | 11% |

---

## F001: 基线恢复（19 个测试）

| FR ID | 测试函数 | 状态 | 修复 Feature |
|-------|---------|------|-------------|
| FR-2.1.1-R1 | `fr_2_1_1_snapshot_before_modification` | ✅ 通过 | — |
| FR-2.1.2-R1 | `fr_2_1_2_assessment_readonly` | ✅ 通过 | — |
| FR-2.2.1-R1 | `fr_2_2_1_baseline_formation` | ✅ 通过 | — |
| FR-2.2.2-R1 | `fr_2_2_2_confirmation_interaction` | ✅ 通过 | — |
| FR-2.3.1-R1 | `fr_2_3_1_state_item_classification` | ✅ 通过 | — |
| FR-2.3.2-R1 | `fr_2_3_2_compare_with_baseline` | ✅ 通过 | — |
| FR-2.3.3-R1 | `fr_2_3_3_restore_only_restorable` | ⏭ 忽略 | F109-P2-109-9 |
| FR-2.4.1-R1 | `fr_2_4_1_stop_triggers_restore` | ✅ 通过 | — |
| FR-2.4.2-R1~R3 | `fr_2_4_2_restore_execution_and_audit` | ⏭ 忽略 | F109-P1-109-3 |
| FR-2.4.3-R1 | `fr_2_4_3_non_target_verification` | ⏭ 忽略 | F109-P1-109-5 |
| FR-2.5.1-R1~R4 | `fr_2_5_1_proxy_guard_scope` | ⏭ 忽略 | F109-P1-109-4 |
| FR-2.5.2-R1~R4 | `fr_2_5_2_proxy_guard_response` | ⏭ 忽略 | F109-P1-109-4 |
| FR-2.5.3-R1 | `fr_2_5_3_proxy_guard_strategy` | ⏭ 忽略 | F109-P1-109-4 |
| FR-2.6.1-R1~R2 | `fr_2_6_1_recovery_task_persistence` | ⏭ 忽略 | F109（续跑未实现） |
| FR-2.7.1-R1~R3 | `fr_2_7_1_audit_scope` | ⏭ 忽略 | F109-P1-109-3 |
| FR-2.7.2-R1~R3 | `fr_2_7_2_audit_format` | ⏭ 忽略 | F110-G110-15 |
| FR-2.8.1-R1 | `fr_2_8_1_five_element_prompt` | ⏭ 忽略 | F110-G110-5 |
| FR-2.9.1-R1 | `fr_2_9_1_deployment_identification` | ✅ 通过 | — |
| SC-1 | `fr_sc_1_restore_all_restorable` | ⏭ 忽略 | F109-P0-109-1 |

## F002: WSL 支持（8 个测试）

| FR ID | 测试函数 | 状态 | 修复 Feature |
|-------|---------|------|-------------|
| FR-2.1.1-R1~R3 | `fr_2_1_1_config_within_baseline_scope` | ✅ 通过 | — |
| FR-2.1.2-R1~R2 | `fr_2_1_2_config_execution` | ✅ 通过 | — |
| FR-2.1.3-R1 | `fr_2_1_3_config_restore` | ✅ 通过 | — |
| FR-2.2.1-R1~R4 | `fr_2_2_1_coordinated_mode` | ⏭ 忽略 | F101 |
| FR-2.2.2-R1 | `fr_2_2_2_state_item_reuse` | ✅ 通过 | — |
| FR-2.3.1-R1 | `fr_2_3_1_network_mode_detection` | ✅ 通过 | — |
| FR-2.3.2-R1 | `fr_2_3_2_strategy_selection` | ✅ 通过 | — |
| FR-2.5.1-R1 | `fr_2_5_1_wsl_failure_prompt` | ⏭ 忽略 | F110-G110-5 |

## F003: 站点规则（12 个测试）

| FR ID | 测试函数 | 状态 | 修复 Feature |
|-------|---------|------|-------------|
| FR-2.1.1-R1~R3 | `fr_2_1_1_add_remove_site` | ✅ 通过 | — |
| FR-2.1.2-R1 | `fr_2_1_2_template_selection` | ✅ 通过 | — |
| FR-2.2.1-R1~R2 | `fr_2_2_1_rule_generation` | ✅ 通过 | — |
| FR-2.2.2-R1 | `fr_2_2_2_default_direct_strategy` | ✅ 通过 | — |
| FR-2.3.1-R1~R2 | `fr_2_3_1_continuous_probe` | ⏭ 忽略 | F110-G110-1 |
| FR-2.3.2-R1~R2 | `fr_2_3_2_unreachable_recovery` | ⏭ 忽略 | F110-G110-1 |
| FR-2.4.1-R1~R2 | `fr_2_4_1_rule_preview` | ⏭ 忽略 | F110-G110-6 |
| FR-2.4.2-R1~R3 | `fr_2_4_2_rule_override` | ⏭ 忽略 | F113-discovery |
| FR-2.5.1-R1~R3 | `fr_2_5_1_rule_effectiveness` | ⏭ 忽略 | F101 |
| FR-2.6.1-R1 | `fr_2_6_1_rule_change_safety` | ✅ 通过 | — |
| SC-1 | `fr_sc_1_p95_recovery` | ⏭ 忽略 | F110-G110-1 |
| SC-5 | `fr_sc_5_five_element_diag` | ⏭ 忽略 | F110-G110-5 |

## F004: 后端（8 个测试）

| FR ID | 测试函数 | 状态 | 修复 Feature |
|-------|---------|------|-------------|
| FR-2.1.1-R1 | `fr_2_1_1_cross_platform_app` | ✅ 通过 | — |
| FR-2.2.1-R1~R3 | `fr_2_2_1_wizard_baseline_flow` | ⏭ 忽略 | F110-G110-3 |
| FR-2.3.1-R1~R2 | `fr_2_3_1_service_control` | ✅ 通过 | — |
| FR-2.4.1-R1~R2 | `fr_2_4_1_site_add_remove` | ✅ 通过 | — |
| FR-2.5.1-R1 | `fr_2_5_1_rule_preview` | ⏭ 忽略 | F110-G110-6 |
| FR-2.6.1-R1 | `fr_2_6_1_site_reachability` | ⏭ 忽略 | F110-G110-1 |
| FR-2.7.1-R1 | `fr_2_7_1_notification` | ⏭ 忽略 | F110-G110-7 |
| SC-2 | `fr_sc_2_status_matches_backend` | ✅ 通过 | — |

## F004: 前端（14 个测试）

| FR ID | 测试函数 | 状态 | 修复 Feature |
|-------|---------|------|-------------|
| FR-2.1.1 | `fr_2_1_1_displays_dashboard_with_local_data` | ✅ 通过 | — |
| FR-2.3.1 | `fr_2_3_1_service_controls_exist` | ✅ 通过 | — |
| FR-2.3.2 | `fr_2_3_2_displays_service_status_running` | ✅ 通过 | — |
| FR-2.3.2 | `fr_2_3_2_displays_baseline_status` | ✅ 通过 | — |
| FR-2.4.1 | `fr_2_4_1_site_list_shows_empty_state` | ✅ 通过 | — |
| FR-2.4.1 | `fr_2_4_1_site_list_shows_sites_with_domains` | ✅ 通过 | — |
| FR-2.4.2 | `fr_2_4_2_template_buttons_exist` | ✅ 通过 | — |
| FR-2.5.1 | `fr_2_5_1_rule_preview_shows_rules` | ✅ 通过 | — |
| FR-2.5.1 | `fr_2_5_1_rule_preview_shows_strategy_badges` | ✅ 通过 | — |
| FR-2.6.1 | `fr_2_6_1_diagnostics_shows_reachability_section` | ✅ 通过 | — |
| FR-2.6.1 | `fr_2_6_1_diagnostics_shows_node_pool` | ✅ 通过 | — |
| FR-2.7.1 | `fr_2_7_1_notification_area_exists` | ✅ 通过 | — |
| SC-2 | `fr_sc_2_service_status_displays_correctly` | ✅ 通过 | — |
| SC-2 | `fr_sc_2_baseline_status_displays_correctly` | ✅ 通过 | — |

## 契约测试（20 个）

| 类别 | 测试函数 | 状态 |
|------|---------|------|
| DTO 往返 | `contract_state_item_roundtrip` | ✅ 通过 |
| DTO 往返 | `contract_state_item_platform_variants` | ✅ 通过 |
| DTO 往返 | `contract_state_item_category_roundtrip` | ✅ 通过 |
| DTO 往返 | `contract_probe_result_roundtrip` | ✅ 通过 |
| DTO 往返 | `contract_probe_result_unreachable_roundtrip` | ✅ 通过 |
| DTO 往返 | `contract_probe_method_snake_case` | ✅ 通过 |
| DTO 往返 | `contract_site_reachability_roundtrip` | ✅ 通过 |
| DTO 往返 | `contract_non_target_verification_roundtrip` | ✅ 通过 |
| Adapter 契约 | `contract_adapter_definitions_match_read_items` | ✅ 通过 |
| Adapter 契约 | `contract_adapter_write_restorable_succeeds` | ✅ 通过 |
| Adapter 契约 | `contract_adapter_write_non_restorable_rejected` | ✅ 通过 |
| Adapter 契约 | `contract_adapter_trait_object_dispatch` | ✅ 通过 |
| ProbeClient 契约 | `contract_probe_client_default_reachable` | ✅ 通过 |
| ProbeClient 契约 | `contract_probe_client_override_works` | ✅ 通过 |
| ProbeClient 契约 | `contract_probe_client_trait_object_dispatch` | ✅ 通过 |
| ProbeClient 契约 | `contract_probe_client_method_match` | ✅ 通过 |
| AuditAction 一致性 | `contract_audit_action_serialization` | ✅ 通过 |
| AuditAction 一致性 | `contract_audit_action_roundtrip` | ✅ 通过 |
| Write 日志 | `contract_adapter_write_logging` | ✅ 通过 |
| Optional 字段 | `contract_site_reachability_optional_probe` | ✅ 通过 |

## 管道集成测试（7 个）

| 管道 | 测试函数 | 状态 | 修复 Feature |
|------|---------|------|-------------|
| P1 | `pipeline_assess_confirm_restore_audit` | ✅ 通过 | — |
| P2 | `pipeline_subscription_to_node_pool_to_rules` | ⏭ 忽略 | F110-G110-2 |
| P3 | `pipeline_site_add_probe_reachability` | ✅ 通过 | — |
| P4 | `pipeline_probe_five_element_audit` | ⏭ 忽略 | F110-G110-5 |
| P5 | `pipeline_proxy_guard_restore` | ⏭ 忽略 | F109-P1-109-4 |
| P6 | `pipeline_wizard_readjustment` | ⏭ 忽略 | F110-G110-3 |
| P7 | `pipeline_rule_preview_apply_verify_rollback` | ✅ 通过 | — |

---

## F113 新发现（T-12 待分流）

| 测试函数 | 发现 | 建议 |
|---------|------|------|
| `fr_2_4_2_rule_override` | `add_user_override()` 不记审计日志 | F114+（审计覆盖 gap） |

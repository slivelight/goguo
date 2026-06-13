//! F001 baseline-restore FR 验收测试（19 个）
//!
//! 覆盖 F001 的 52 条 FR，按功能节分组。
//! 断言 spec 描述的用户可观测结果（适配器写入、审计日志、快照持久化）。

#[path = "../common/mod.rs"]
mod common;

use std::collections::HashMap;

use common::*;
use goguo_lib::commands::baseline as cmds;
use goguo_lib::managers::baseline_manager::BaselineManager;
use goguo_lib::managers::mihomo_manager::MihomoManager;
use goguo_lib::models::audit::AuditAction;
use goguo_lib::models::baseline::StateItemCategory;
use goguo_lib::storage::baseline_storage::BaselineStorage;

// ── §2.1 安装后网络评估与初始状态快照 ─────────────────────────────────────

/// FR-2.1.1-R1~R4: 快照在 GoGuo 修改任何网络配置之前采集完成。
/// 可观测结果：快照时间戳存在；含 OS 信息；adapter 无 write_state 调用。
#[test]
fn fr_2_1_1_snapshot_before_modification() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let items = standard_baseline_items();
    let adapter = std::sync::Arc::new(TestAdapter::new(items));
    let storage = BaselineStorage::new(dir.path().join("baseline"));
    let mgr = BaselineManager::new(
        vec![Box::new((*adapter).clone())],
        storage,
        dir.path().to_path_buf(),
    );

    // R1: snapshot collected before any modification
    let snapshot = mgr.collect_initial_snapshot().expect("snapshot");
    // Adapter should NOT have been written to
    assert!(adapter.written_values().is_empty(),
        "FR-2.1.1-R1: snapshot must be collected before any write_state");

    // R3: timestamp exists and environment info present
    assert!(!snapshot.timestamp.is_empty(),
        "FR-2.1.1-R3: snapshot must contain timestamp");
    assert!(!snapshot.environment.os_name.is_empty(),
        "FR-2.1.1-R3: snapshot must contain OS info");
    assert!(!snapshot.environment.hostname.is_empty(),
        "FR-2.1.1-R3: snapshot must contain hostname");

    // R2: items persisted
    assert!(!snapshot.items.is_empty(),
        "FR-2.1.1-R2: snapshot must contain state items");

    // R4: each item has classification_reason (failure reason if failed)
    for item in &snapshot.items {
        assert!(!item.classification_reason.is_empty(),
            "FR-2.1.1-R4: item '{}' must have classification reason", item.id);
    }
}

/// FR-2.1.2-R1~R4: 评估过程不修改系统网络配置。
/// 可观测结果：评估后 adapter 无 write_state 调用；评估结果可检索。
#[test]
fn fr_2_1_2_assessment_readonly() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let items = standard_baseline_items();
    let adapter = std::sync::Arc::new(TestAdapter::new(items));
    let storage = BaselineStorage::new(dir.path().join("baseline"));
    let mgr = BaselineManager::new(
        vec![Box::new((*adapter).clone())],
        storage,
        dir.path().to_path_buf(),
    );

    // Assessment via command layer
    let response = cmds::start_initial_assessment(&mgr).expect("assess");

    // R4: assessment did NOT modify network config
    assert!(adapter.written_values().is_empty(),
        "FR-2.1.2-R4: assessment must not modify network config");

    // R3: results saved and retrievable
    let summary = cmds::get_state_summary(&mgr).expect("summary");
    assert!(summary.total > 0,
        "FR-2.1.2-R3: assessment results must be saved");

    // Verify assessment response
    assert_eq!(response.item_count, summary.total,
        "assessment response item_count must match summary");
}

// ── §2.2 用户确认基线 ────────────────────────────────────────────────────

/// FR-2.2.1-R1~R4: baseline 确认后只读、版本递增。
/// 可观测结果：confirm 产生 version > 0；再次确认产生更高版本。
#[test]
fn fr_2_2_1_baseline_formation() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let items = standard_baseline_items();
    let adapter = std::sync::Arc::new(TestAdapter::new(items));
    let storage = BaselineStorage::new(dir.path().join("baseline"));
    let mgr = BaselineManager::new(
        vec![Box::new((*adapter).clone())],
        storage,
        dir.path().to_path_buf(),
    );

    // R1: baseline defaults to initial snapshot values
    let initial = mgr.collect_initial_snapshot().expect("initial");
    assert_eq!(initial.version, 0, "initial snapshot is version 0");

    let confirmed = mgr.confirm_baseline().expect("confirm");

    // R4: version incremented after confirm
    assert!(confirmed.version > 0,
        "FR-2.2.1-R4: confirmed baseline must have version > 0");

    // R3: confirmed baseline contains all items (restorable + detectable)
    assert_eq!(confirmed.items.len(), initial.items.len(),
        "FR-2.2.1-R3: baseline must contain all collected items");

    // Persisted and retrievable
    let retrieved = mgr.get_confirmed_baseline().expect("retrieve").expect("exists");
    assert_eq!(retrieved.version, confirmed.version,
        "FR-2.2.1-R4: confirmed baseline must be persistently readable");
}

/// FR-2.2.2-R1~R4: 确认前展示摘要；确认记入审计。
/// 可观测结果：get_state_summary 提供分组摘要；confirm 产生审计记录。
#[test]
fn fr_2_2_2_confirmation_interaction() {
    let state = setup_baseline_confirmed();

    // R1: summary distinguishes restorable vs detectable
    let summary = cmds::get_state_summary(&state.baseline_manager).expect("summary");
    assert!(summary.restorable_count > 0,
        "FR-2.2.2-R1: summary must show restorable count");
    assert!(summary.detectable_count > 0,
        "FR-2.2.2-R1: summary must show detectable count");
    assert_eq!(summary.total, summary.restorable_count + summary.detectable_count + summary.excluded_count,
        "FR-2.2.2-R1: total must equal sum of categories");

    // R4: confirm_baseline returns version (user confirmation result)
    // Already confirmed in setup; verify snapshot exists
    let snap = state.baseline_manager.get_confirmed_baseline()
        .expect("get").expect("baseline exists");
    assert!(snap.version > 0, "FR-2.2.2-R4: confirmation produces versioned baseline");
}

// ── §2.3 基线状态项管理 ───────────────────────────────────────────────────

/// FR-2.3.1-R1~R3: 状态项分类含理由，持久化可查。
/// 可观测结果：每个 item 有 category 和 classification_reason；快照持久化后可检索。
#[test]
fn fr_2_3_1_state_item_classification() {
    let state = setup_baseline_confirmed();
    let snap = state.baseline_manager.get_confirmed_baseline()
        .expect("get").expect("baseline");

    // R1: each item has a category
    let categories: Vec<StateItemCategory> = snap.items.iter().map(|i| i.category.clone()).collect();
    assert!(categories.contains(&StateItemCategory::Restorable),
        "FR-2.3.1-R1: at least one Restorable item");
    assert!(categories.contains(&StateItemCategory::Detectable),
        "FR-2.3.1-R1: at least one Detectable item");

    // R2: each item has classification_reason
    for item in &snap.items {
        assert!(!item.classification_reason.is_empty(),
            "FR-2.3.1-R2: item '{}' must have classification reason", item.id);
    }

    // R3: classification persisted (snapshot is stored and retrievable)
    let snap2 = state.baseline_manager.get_confirmed_baseline()
        .expect("get").expect("baseline");
    assert_eq!(snap.items.len(), snap2.items.len(),
        "FR-2.3.1-R3: classification must be persisted");
}

/// FR-2.3.2-R1~R3: 对比输出差异清单，区分偏离/一致。
/// 可观测结果：compare_with_baseline 返回差异清单，正确标记 match/deviated。
#[test]
fn fr_2_3_2_compare_with_baseline() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let items = standard_baseline_items();
    let adapter = std::sync::Arc::new(TestAdapter::new(items));
    let storage = BaselineStorage::new(dir.path().join("baseline"));
    let mgr = BaselineManager::new(
        vec![Box::new((*adapter).clone())],
        storage,
        dir.path().to_path_buf(),
    );

    // Assess and confirm
    let _ = mgr.collect_initial_snapshot().expect("assess");
    let _ = mgr.confirm_baseline().expect("confirm");

    // R2 & R3: compare when no deviation → all match
    let comparison = mgr.compare_with_baseline().expect("compare");
    assert!(!comparison.is_empty(),
        "FR-2.3.2-R2: comparison must output item list");
    assert!(comparison.iter().all(|c| matches!(c.result,
        goguo_lib::managers::baseline_manager::ComparisonResult::Match)),
        "FR-2.3.2-R3: no deviation → all items should be Match");
}

/// FR-2.3.3-R1~R4: 仅 Restorable 项被恢复；不可恢复项仅提示。
/// 可观测结果：restore_to_baseline 仅对 Restorable 项调用 write_state。
#[test]
#[ignore = "F109-P2-109-9: restore 后需立即验证当前值与 baseline 一致"]
fn fr_2_3_3_restore_only_restorable() {
    let state = setup_baseline_confirmed();

    let result = state.baseline_manager.restore_to_baseline().expect("restore");

    // R1: only Restorable items restored
    let written = state.adapter.written_values();
    for (id, _) in &written {
        let snap = state.baseline_manager.get_confirmed_baseline()
            .expect("get").expect("baseline");
        let item = snap.items.iter().find(|i| &i.id == id).expect("item");
        assert_eq!(item.category, StateItemCategory::Restorable,
            "FR-2.3.3-R1: only Restorable items should be restored, got '{}'", id);
    }

    // R2: only deviated items restored (no deviation in fresh baseline → all match)
    // After fresh confirm, all values match baseline, so restore should succeed

    // R3: verify after restore — check adapter received correct baseline values
    assert!(result.succeeded > 0, "FR-2.3.3-R3: some items should succeed");
    assert_eq!(result.failed, 0, "no failures expected");
}

// ── §2.4 停止服务恢复流程 ────────────────────────────────────────────────

/// FR-2.4.1-R1~R3: 停止操作触发恢复检查。
/// 可观测结果：stop_service 返回 recovery_triggered=true；adapter 收到 write 调用。
#[test]
fn fr_2_4_1_stop_triggers_restore() {
    let state = setup_baseline_confirmed();

    // R1 & R2: stop triggers baseline restore check
    let result = cmds::stop_service(
        &mut state.mihomo_manager.lock().expect("lock"),
        &state.baseline_manager,
    ).expect("stop");

    // R1: restore was triggered
    assert!(result.recovery_triggered,
        "FR-2.4.1-R1: stop must trigger restore check");

    // Adapter should have received writes (restore executed)
    let written = state.adapter.written_values();
    assert!(!written.is_empty(),
        "FR-2.4.1-R2: stop must execute restore, not just terminate process");
}

/// FR-2.4.2-R1~R5: 恢复结果记入审计；结果展示区分已恢复/需手动。
#[test]
#[ignore = "F109-P1-109-3: 恢复结果需完整记入审计日志（当前 AuditLogger 未接入 BaselineManager）"]
fn fr_2_4_2_restore_execution_and_audit() {
    let state = setup_baseline_confirmed();

    let _result = state.baseline_manager.restore_to_baseline().expect("restore");

    // R3: restore results recorded in audit
    let records = state.audit_logger.records();
    let restore_records: Vec<_> = records.iter()
        .filter(|r| r.action == AuditAction::StateRestore)
        .collect();
    assert!(!restore_records.is_empty(),
        "FR-2.4.2-R3: restore must produce audit records");

    // R1 & R5: results distinguish succeeded/failed
    // R4: verification after restore
}

/// FR-2.4.3-R1~R2: 恢复后非目标站点可达性不降。
#[test]
#[ignore = "F109-P1-109-5: 非目标站点可达性验证未实现（F103 SC-5）"]
fn fr_2_4_3_non_target_verification() {
    let state = setup_baseline_confirmed();

    let result = state.baseline_manager.restore_to_baseline().expect("restore");

    // R1: non-target verification present
    assert!(result.non_target_verification.is_some(),
        "FR-2.4.3-R1: restore must verify non-target site reachability");

    let verification = result.non_target_verification.expect("verification");
    assert!(verification.sites_reachable > 0,
        "FR-2.4.3-R1: at least some sites should be reachable");
}

// ── §2.5 Proxy Guard ─────────────────────────────────────────────────────

/// FR-2.5.1-R1: ProxyGuard 监控对象列表完整。
#[test]
#[ignore = "F109-P1-109-4: ProxyGuard 后台任务未实现（F105）"]
fn fr_2_5_1_proxy_guard_scope() {
    // ProxyGuard should monitor: mihomo process, port reachability, API health, system proxy consistency
    // This requires a running monitoring loop which is not yet implemented.
}

/// FR-2.5.2-R1~R5: 异常检测→系统代理清除→审计记录。
#[test]
#[ignore = "F109-P1-109-4: ProxyGuard 后台任务未实现（F105）"]
fn fr_2_5_2_proxy_guard_response() {
    use goguo_lib::models::config::ProxyGuardConfig;
    use goguo_lib::services::proxy_guard::ProxyGuard;

    let config = ProxyGuardConfig {
        check_interval_secs: 3,
        max_restart_attempts: 3,
        restart_cooldown_secs: 10,
    };
    let mut guard = ProxyGuard::new(config);

    // When mihomo is not running, guard should detect anomaly
    let dir = tempfile::TempDir::new().expect("temp dir");
    let mihomo_config = test_mihomo_config(dir.path());
    let mut mihomo = MihomoManager::new(mihomo_config);

    let action = guard.check_and_recover(false, &mut mihomo);
    // R1: anomaly detected and action taken
    assert!(!matches!(action, goguo_lib::services::proxy_guard::GuardAction::Healthy),
        "FR-2.5.2-R1: must detect anomaly when mihomo not running");
}

/// FR-2.5.3-R1~R4: 重启策略、阈值可配置。
#[test]
#[ignore = "F109-P1-109-4: ProxyGuard 后台任务未实现（F105）"]
fn fr_2_5_3_proxy_guard_strategy() {
    use goguo_lib::models::config::ProxyGuardConfig;
    use goguo_lib::services::proxy_guard::ProxyGuard;

    // R4: threshold configurable, default ≤ 3
    let config = ProxyGuardConfig {
        check_interval_secs: 3,
        max_restart_attempts: 3,
        restart_cooldown_secs: 10,
    };
    let guard = ProxyGuard::new(config);
    assert!(guard.max_restart_attempts() <= 3,
        "FR-2.5.3-R4: default threshold must be ≤ 3");

    // R1-R3: restart strategy tested via check_and_recover
    // (requires mihomo mock which is not available in this context)
}

// ── §2.6 工具重启后续跑 ──────────────────────────────────────────────────

/// FR-2.6.1-R1~R3: 恢复任务持久化可读。
#[test]
#[ignore = "F109: 续跑功能未完整实现（RecoveryManager 未集成到 BaselineManager.restore_to_baseline）"]
fn fr_2_6_1_recovery_task_persistence() {
    use goguo_lib::models::recovery::{ItemResult, RecoveryItem};
    use goguo_lib::services::recovery::RecoveryManager;

    let dir = tempfile::TempDir::new().expect("temp dir");
    let rm = RecoveryManager::new(dir.path().to_path_buf()).expect("recovery mgr");

    // R2: task persisted to disk
    let pending = vec![RecoveryItem {
        state_item_id: "system-proxy".to_string(),
        target_value: serde_json::json!("off"),
        result: None,
        failure_reason: None,
    }];
    let task = rm.create_task(pending).expect("create");
    assert!(!task.id.is_empty(), "FR-2.6.1-R2: task must have ID");

    // R1: task loadable on restart
    let loaded = rm.load_task().expect("load");
    assert!(loaded.is_some(), "FR-2.6.1-R1: task must survive restart");
    let loaded_task = loaded.expect("task");
    assert_eq!(loaded_task.pending_items.len(), 1, "FR-2.6.1-R3: pending items preserved");

    // R3: task contains state items, attempted actions, failure reasons
    assert_eq!(loaded_task.pending_items[0].state_item_id, "system-proxy");
    assert_eq!(loaded_task.pending_items[0].target_value, serde_json::json!("off"));
}

// ── §2.7 本地审计 ────────────────────────────────────────────────────────

/// FR-2.7.1-R1~R8: 审计包含所有操作类型，不含隐私数据。
#[test]
#[ignore = "F109-P1-109-3: BaselineManager 操作未接入审计日志（AuditLogger 未传入）"]
fn fr_2_7_1_audit_scope() {
    let state = setup_baseline_confirmed();

    let _ = state.baseline_manager.restore_to_baseline().expect("restore");
    let records = state.audit_logger.records();

    // R1: baseline collect recorded
    assert!(records.iter().any(|r| r.action == AuditAction::BaselineCollect),
        "FR-2.7.1-R1: audit must contain BaselineCollect");

    // R5: user confirm recorded
    assert!(records.iter().any(|r| r.action == AuditAction::BaselineConfirm),
        "FR-2.7.1-R5: audit must contain BaselineConfirm");

    // R3: restore actions recorded
    assert!(records.iter().any(|r| r.action == AuditAction::StateRestore),
        "FR-2.7.1-R3: audit must contain StateRestore");

    // R6-R8: no privacy data in audit details
    for record in &records {
        let details_str = serde_json::to_string(&record.details).expect("serialize");
        assert!(!details_str.contains("password"), "FR-2.7.1-R6: no credentials in audit");
        assert!(!details_str.contains("token"), "FR-2.7.1-R7: no tokens in audit");
    }
}

/// FR-2.7.2-R1~R3: 审计含 5 要素，支持筛选。
#[test]
#[ignore = "F110-G110-15: 审计查询接口缺少结构化筛选支持"]
fn fr_2_7_2_audit_format() {
    let state = setup_baseline_confirmed();

    let _ = state.baseline_manager.restore_to_baseline().expect("restore");
    let records = state.audit_logger.records();

    // R1: each record has 5 elements: timestamp, action, target, result, reason/details
    for record in &records {
        assert!(!record.timestamp.is_empty(), "FR-2.7.2-R1: timestamp required");
        // action is enum, always present
        assert!(!record.target.is_empty(), "FR-2.7.2-R1: target required");
        // result is enum, always present
        // details is serde_json::Value, always present
    }
}

// ── §2.8 失败解释 ────────────────────────────────────────────────────────

/// FR-2.8.1-R1~R3: 五要素完整，建议可执行。
#[test]
#[ignore = "F110-G110-5: 五要素诊断提示未实现"]
fn fr_2_8_1_five_element_prompt() {
    // This test verifies that failure explanations contain all 5 elements:
    // 原因 (cause), 已尝试动作 (attempted), 尝试次数 (attempt count),
    // 建议动作 (suggested action), 是否需要手动处理 (needs manual intervention)
    //
    // Requires a failure scenario that produces a FiveElementPrompt.
    // Not yet implemented in production code.
}

// ── §2.9 部署模式识别 ─────────────────────────────────────────────────────

/// FR-2.9.1~2.9.2: 三种部署模式识别，双侧采集。
/// 可观测结果：DeploymentMode 包含 4 种变体；ConfigManager 可识别当前模式。
#[test]
fn fr_2_9_1_deployment_identification() {
    use goguo_lib::models::config::DeploymentMode;

    // R1: all deployment modes are defined
    let modes = [
        DeploymentMode::WindowsOnly,
        DeploymentMode::WslOnly,
        DeploymentMode::LinuxOnly,
        DeploymentMode::Coordinated,
    ];
    assert_eq!(modes.len(), 4, "FR-2.9.1-R1: must support 4 deployment modes");

    // Verify serde roundtrip (R2: data format defined for OPP-004 consumption)
    for mode in &modes {
        let json = serde_json::to_string(mode).expect("serialize");
        let deserialized: DeploymentMode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(format!("{:?}", mode), format!("{:?}", deserialized),
            "FR-2.9.2-R2: DeploymentMode must roundtrip through serde");
    }
}

// ── SC-1 成功标准 ─────────────────────────────────────────────────────────

/// SC-1: 停止后 100% Restorable 项回 baseline。
#[test]
#[ignore = "F109-P0-109-1: 立即恢复按钮调用 triggerReadjustment 而非 restore_to_baseline"]
fn fr_sc_1_restore_all_restorable() {
    let state = setup_baseline_confirmed();

    // Get baseline to know expected values
    let baseline = state.baseline_manager.get_confirmed_baseline()
        .expect("get").expect("baseline");
    let restorable_count = baseline.items.iter()
        .filter(|i| i.category == StateItemCategory::Restorable)
        .count();

    let result = state.baseline_manager.restore_to_baseline().expect("restore");

    // SC-1: 100% of restorable items restored
    assert_eq!(result.succeeded, restorable_count,
        "SC-1: all {} restorable items must succeed, got {}", restorable_count, result.succeeded);
    assert_eq!(result.failed, 0,
        "SC-1: no restorable items should fail");
}

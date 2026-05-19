//! End-to-end integration tests for Feature 001: Baseline Restore lifecycle.
//!
//! Scenarios covered:
//! 1. Full lifecycle: assess → confirm → deviate → restore → verify
//! 2. Audit trail completeness
//! 3. `ProxyGuard` auto-recovery (simulated mihomo crash)
//! 4. Resume recovery after interruption

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use goguo_lib::adapters::{PlatformAdapter, StateItemDefinition};
use goguo_lib::commands::baseline::{
    self as cmds, AuditLogParams, ComparisonResultDto,
};
use goguo_lib::managers::baseline_manager::BaselineManager;
use goguo_lib::managers::mihomo_manager::MihomoManager;
use goguo_lib::models::audit::AuditAction;
use goguo_lib::models::baseline::{Platform, StateItem, StateItemCategory};
use goguo_lib::models::config::{MihomoConfig, ProxyGuardConfig};
use goguo_lib::services::audit_logger::AuditLogger;
use goguo_lib::services::proxy_guard::{GuardAction, ProxyGuard};
use goguo_lib::services::recovery::RecoveryManager;
use goguo_lib::storage::baseline_storage::BaselineStorage;

// ── Test Helpers ───────────────────────────────────────────────────────────

/// Shared write tracker used by all clones of `IntegrationAdapter`.
type WriteLog = Arc<Mutex<Vec<(String, serde_json::Value)>>>;

/// Integration mock adapter that tracks write calls via shared state.
struct IntegrationAdapter {
    items: Vec<StateItem>,
    written: WriteLog,
}

impl IntegrationAdapter {
    fn new(items: Vec<StateItem>) -> Self {
        Self {
            items,
            written: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn written_values(&self) -> Vec<(String, serde_json::Value)> {
        self.written.lock().expect("lock").clone()
    }
}

impl Clone for IntegrationAdapter {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            written: Arc::clone(&self.written),
        }
    }
}

impl PlatformAdapter for IntegrationAdapter {
    fn platform(&self) -> Platform {
        Platform::Windows
    }

    fn state_item_definitions(&self) -> Vec<StateItemDefinition> {
        self.items
            .iter()
            .map(|i| StateItemDefinition {
                id: i.id.clone(),
                category: i.category.clone(),
                description: String::new(),
            })
            .collect()
    }

    fn read_state_items(&self) -> Vec<StateItem> {
        self.items.clone()
    }

    fn write_state(&self, item: &StateItem) -> Result<(), String> {
        self.written
            .lock()
            .expect("lock")
            .push((item.id.clone(), item.value.clone()));
        Ok(())
    }
}

fn make_item(id: &str, category: StateItemCategory, value: &str) -> StateItem {
    StateItem {
        id: id.to_string(),
        platform: Platform::Windows,
        category,
        value: serde_json::json!(value),
        collected_at: "2026-05-19T12:00:00Z".to_string(),
        classification_reason: "integration test".to_string(),
    }
}

fn make_restorable(id: &str, value: &str) -> StateItem {
    make_item(id, StateItemCategory::Restorable, value)
}

fn make_detectable(id: &str, value: &str) -> StateItem {
    make_item(id, StateItemCategory::Detectable, value)
}

fn test_mihomo_config(dir: &std::path::Path) -> MihomoConfig {
    MihomoConfig {
        binary_path: dir.join("fake-mihomo"),
        config_dir: dir.join("mihomo"),
        api_address: "127.0.0.1:19999".to_string(),
        api_secret: "test".to_string(),
        mixed_port: 19999,
        log_level: "warning".to_string(),
    }
}

fn setup_env(
    dir: &std::path::Path,
    items: Vec<StateItem>,
) -> (BaselineManager, std::sync::Arc<IntegrationAdapter>) {
    let adapter = std::sync::Arc::new(IntegrationAdapter::new(items));
    let storage = BaselineStorage::new(dir.join("baseline"));
    let mgr = BaselineManager::new(
        vec![Box::new((*adapter).clone())],
        storage,
        dir.to_path_buf(),
    );
    (mgr, adapter)
}

// ── Scenario 1: Full Lifecycle ─────────────────────────────────────────────

#[test]
fn full_lifecycle_assess_confirm_restore() {
    let dir = tempfile::TempDir::new().expect("temp dir");

    // Initial state: 2 restorable + 1 detectable.
    let items = vec![
        make_restorable("win-system-proxy", "proxy-off"),
        make_restorable("win-hosts", "127.0.0.1 localhost"),
        make_detectable("win-dns-cache", "cached"),
    ];
    let (mgr, adapter) = setup_env(dir.path(), items);

    // Step 1: Initial assessment.
    let assess = cmds::start_initial_assessment(&mgr).expect("assess");
    assert_eq!(assess.version, 0);
    assert_eq!(assess.item_count, 3);

    // Step 2: Confirm baseline.
    let confirmed = cmds::confirm_baseline(&mgr).expect("confirm");
    assert_eq!(confirmed.version, 1);
    assert_eq!(confirmed.item_count, 3);

    // Step 3: Check baseline status (should be all match).
    let status = cmds::get_baseline_status(&mgr).expect("status");
    assert!(status.has_baseline);
    assert_eq!(status.items.len(), 3);
    assert!(status.items.iter().all(|i| i.result == ComparisonResultDto::Match));

    // Step 4: Restore to baseline (no changes, but verify restore succeeds).
    let result = mgr.restore_to_baseline().expect("restore");
    assert_eq!(result.succeeded, 2); // 2 restorable items
    assert_eq!(result.failed, 0);

    // Step 5: Verify adapter received write calls for restorable items.
    let written = adapter.written_values();
    assert_eq!(written.len(), 2);
    let written_ids: Vec<&str> = written.iter().map(|(id, _)| id.as_str()).collect();
    assert!(written_ids.contains(&"win-system-proxy"));
    assert!(written_ids.contains(&"win-hosts"));
}

#[test]
fn full_lifecycle_with_deviation_detection() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let baseline_dir = dir.path().join("baseline");

    // Step 1: Collect + confirm with baseline values.
    let baseline_items = vec![
        make_restorable("a", "original-a"),
        make_restorable("b", "original-b"),
    ];
    let adapter = IntegrationAdapter::new(baseline_items);
    let storage = BaselineStorage::new(baseline_dir.clone());
    let mgr = BaselineManager::new(
        vec![Box::new(adapter)],
        storage,
        dir.path().to_path_buf(),
    );

    cmds::start_initial_assessment(&mgr).expect("assess");
    cmds::confirm_baseline(&mgr).expect("confirm");

    // Step 2: Create new manager with modified values (item "a" changed).
    let modified_items = vec![
        make_restorable("a", "modified-a"),
        make_restorable("b", "original-b"),
    ];
    let modified_adapter = IntegrationAdapter::new(modified_items);
    let storage2 = BaselineStorage::new(baseline_dir);
    let mgr2 = BaselineManager::new(
        vec![Box::new(modified_adapter)],
        storage2,
        dir.path().to_path_buf(),
    );

    // Step 3: Check baseline status — should detect deviation on "a".
    let status = cmds::get_baseline_status(&mgr2).expect("status");
    assert!(status.has_baseline);
    let deviated: Vec<_> = status
        .items
        .iter()
        .filter(|i| i.result == ComparisonResultDto::Deviated)
        .collect();
    assert_eq!(deviated.len(), 1);
    assert_eq!(deviated[0].state_item_id, "a");
    assert_eq!(deviated[0].baseline_value, Some(serde_json::json!("original-a")));
    assert_eq!(deviated[0].current_value, Some(serde_json::json!("modified-a")));

    // Step 4: Restore — should write baseline values back.
    let result = mgr2.restore_to_baseline().expect("restore");
    assert_eq!(result.succeeded, 2);
    assert_eq!(result.failed, 0);
}

// ── Scenario 2: Audit Trail Completeness ───────────────────────────────────

#[test]
fn audit_trail_captures_full_lifecycle() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let logger = AuditLogger::new(dir.path().join("audit")).expect("logger");

    // Simulate a full lifecycle with audit logging.
    logger
        .log_success(
            AuditAction::BaselineCollect,
            "all",
            serde_json::json!({"item_count": 3}),
        )
        .expect("log collect");

    logger
        .log_success(
            AuditAction::BaselineConfirm,
            "baseline-v1",
            serde_json::json!({"version": 1}),
        )
        .expect("log confirm");

    logger
        .log_success(
            AuditAction::StateRestore,
            "win-system-proxy",
            serde_json::json!({"restored_to": "proxy-off"}),
        )
        .expect("log restore-1");

    logger
        .log_success(
            AuditAction::StateRestore,
            "win-hosts",
            serde_json::json!({"restored_to": "127.0.0.1 localhost"}),
        )
        .expect("log restore-2");

    // Query via command function.
    let resp = cmds::get_audit_log(&logger, &AuditLogParams::default()).expect("audit");
    assert_eq!(resp.total_count, 4);
    assert_eq!(resp.records[0].action, "baseline_collect");
    assert_eq!(resp.records[1].action, "baseline_confirm");
    assert_eq!(resp.records[2].action, "state_restore");
    assert_eq!(resp.records[3].action, "state_restore");
    // All should be success.
    assert!(resp.records.iter().all(|r| r.result == "success"));
}

#[test]
fn audit_trail_with_failure_recorded() {
    let dir = tempfile::TempDir::new().expect("dir");
    let logger = AuditLogger::new(dir.path().join("audit")).expect("logger");

    logger
        .log_success(
            AuditAction::StateRestore,
            "win-system-proxy",
            serde_json::json!({}),
        )
        .expect("log ok");

    logger
        .log_failure(
            AuditAction::StateRestore,
            "win-hosts",
            "Permission denied",
            serde_json::json!({"path": "/etc/hosts"}),
        )
        .expect("log fail");

    let params = AuditLogParams {
        action_type: Some("state_restore".to_string()),
        ..Default::default()
    };
    let resp = cmds::get_audit_log(&logger, &params).expect("audit");
    assert_eq!(resp.total_count, 2);

    let failures: Vec<_> = resp
        .records
        .iter()
        .filter(|r| r.result == "failure")
        .collect();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].target, "win-hosts");
    assert_eq!(failures[0].reason, Some("Permission denied".to_string()));
}

// ── Scenario 3: ProxyGuard Auto-Recovery ───────────────────────────────────

#[test]
fn proxy_guard_auto_recovery_flow() {
    let dir = tempfile::TempDir::new().expect("dir");
    let config = test_mihomo_config(dir.path());
    let mut mihomo = MihomoManager::new(config);
    let mut guard = ProxyGuard::new(ProxyGuardConfig {
        check_interval_secs: 1,
        max_restart_attempts: 3,
        restart_cooldown_secs: 1,
    });

    // Mihomo never started — each check should attempt restart.
    // Attempts 1 and 2 succeed in incrementing count but start fails → Restarted.
    // Attempt 3: count=3, start fails, count >= max → RecoveryTriggered.
    for i in 1..=2 {
        let action = guard.check_and_recover(&mut mihomo);
        assert!(
            matches!(action, GuardAction::Restarted { attempt } if attempt == i),
            "Expected Restarted with attempt {i}, got {action:?}"
        );
    }

    // 3rd attempt: start fails, restart_count == max → RecoveryTriggered.
    let action = guard.check_and_recover(&mut mihomo);
    assert_eq!(action, GuardAction::RecoveryTriggered);

    // After max attempts, next check still triggers recovery.
    let action = guard.check_and_recover(&mut mihomo);
    assert_eq!(action, GuardAction::RecoveryTriggered);

    // Verify command-layer service status.
    let status = cmds::get_service_status(&mut mihomo, &guard);
    assert!(!status.mihomo_running);
    assert_eq!(status.proxy_guard_restart_count, 3);
}

#[test]
fn proxy_guard_resets_on_healthy() {
    let dir = tempfile::TempDir::new().expect("dir");
    let config = test_mihomo_config(dir.path());
    let mut mihomo = MihomoManager::new(config);
    let mut guard = ProxyGuard::new(ProxyGuardConfig {
        check_interval_secs: 1,
        max_restart_attempts: 3,
        restart_cooldown_secs: 1,
    });

    // Simulate some restart attempts.
    let _ = guard.check_and_recover(&mut mihomo);
    let _ = guard.check_and_recover(&mut mihomo);
    assert_eq!(guard.restart_count(), 2);

    // Reset (simulates healthy period).
    guard.reset_restart_count();
    assert_eq!(guard.restart_count(), 0);

    // Can restart again.
    let action = guard.check_and_recover(&mut mihomo);
    assert!(matches!(action, GuardAction::Restarted { attempt: 1 }));
}

// ── Scenario 4: Resume Recovery After Interruption ─────────────────────────

#[test]
fn resume_recovery_after_interruption() {
    let dir = tempfile::TempDir::new().expect("dir");

    // Step 1: Setup baseline with restorable items.
    let items = vec![
        make_restorable("a", "value-a"),
        make_restorable("b", "value-b"),
        make_restorable("c", "value-c"),
    ];
    let (mgr, _) = setup_env(dir.path(), items);

    cmds::start_initial_assessment(&mgr).expect("assess");
    cmds::confirm_baseline(&mgr).expect("confirm");

    // Step 2: Create a recovery task manually (simulating interruption mid-restore).
    let recovery_mgr =
        RecoveryManager::new(dir.path().join("state")).expect("recovery mgr");

    let baseline_values: HashMap<String, serde_json::Value> = {
        let snapshot = mgr
            .get_confirmed_baseline()
            .expect("load")
            .expect("exists");
        snapshot
            .items
            .iter()
            .map(|i| (i.id.clone(), i.value.clone()))
            .collect()
    };

    // Create a pending task (as if restore was interrupted before starting).
    recovery_mgr
        .create_task(vec![
            goguo_lib::models::recovery::RecoveryItem {
                state_item_id: "a".to_string(),
                target_value: serde_json::json!("value-a"),
                result: None,
                failure_reason: None,
            },
            goguo_lib::models::recovery::RecoveryItem {
                state_item_id: "b".to_string(),
                target_value: serde_json::json!("value-b"),
                result: None,
                failure_reason: None,
            },
            goguo_lib::models::recovery::RecoveryItem {
                state_item_id: "c".to_string(),
                target_value: serde_json::json!("value-c"),
                result: None,
                failure_reason: None,
            },
        ])
        .expect("create task");

    // Step 3: Resume with fresh adapters.
    let fresh_adapter = IntegrationAdapter::new(vec![
        make_restorable("a", "modified-a"),
        make_restorable("b", "modified-b"),
        make_restorable("c", "modified-c"),
    ]);
    let adapters: Vec<Box<dyn PlatformAdapter>> = vec![Box::new(fresh_adapter)];

    let result = recovery_mgr
        .resume_recovery(&adapters, &baseline_values)
        .expect("resume");
    let task = result.expect("should have task");
    assert_eq!(task.status, goguo_lib::models::recovery::RecoveryStatus::Completed);
    assert_eq!(task.completed_items.len(), 3);

    // Step 4: Verify no pending task remains.
    let progress = cmds::get_recovery_progress(&mgr).expect("progress");
    assert!(!progress.has_task);
}

#[test]
fn resume_recovery_partial_failure_then_acknowledge() {
    struct FailOnYAdapter;
    impl PlatformAdapter for FailOnYAdapter {
        fn platform(&self) -> Platform {
            Platform::Windows
        }
        fn state_item_definitions(&self) -> Vec<StateItemDefinition> {
            vec![
                StateItemDefinition {
                    id: "x".to_string(),
                    category: StateItemCategory::Restorable,
                    description: String::new(),
                },
                StateItemDefinition {
                    id: "y".to_string(),
                    category: StateItemCategory::Restorable,
                    description: String::new(),
                },
            ]
        }
        fn read_state_items(&self) -> Vec<StateItem> {
            vec![]
        }
        fn write_state(&self, item: &StateItem) -> Result<(), String> {
            if item.id == "y" {
                Err("Simulated write failure for y".to_string())
            } else {
                Ok(())
            }
        }
    }

    let dir = tempfile::TempDir::new().expect("dir");

    let items = vec![
        make_restorable("x", "val-x"),
        make_restorable("y", "val-y"),
    ];
    let (mgr, _) = setup_env(dir.path(), items);

    cmds::start_initial_assessment(&mgr).expect("assess");
    cmds::confirm_baseline(&mgr).expect("confirm");

    let recovery_mgr =
        RecoveryManager::new(dir.path().join("state")).expect("recovery mgr");

    let baseline_values: HashMap<String, serde_json::Value> = {
        let snapshot = mgr
            .get_confirmed_baseline()
            .expect("load")
            .expect("exists");
        snapshot
            .items
            .iter()
            .map(|i| (i.id.clone(), i.value.clone()))
            .collect()
    };

    // Create pending task.
    recovery_mgr
        .create_task(vec![
            goguo_lib::models::recovery::RecoveryItem {
                state_item_id: "x".to_string(),
                target_value: serde_json::json!("val-x"),
                result: None,
                failure_reason: None,
            },
            goguo_lib::models::recovery::RecoveryItem {
                state_item_id: "y".to_string(),
                target_value: serde_json::json!("val-y"),
                result: None,
                failure_reason: None,
            },
        ])
        .expect("create");

    let adapters: Vec<Box<dyn PlatformAdapter>> = vec![Box::new(FailOnYAdapter)];

    let result = recovery_mgr
        .resume_recovery(&adapters, &baseline_values)
        .expect("resume");
    let task = result.expect("task");
    assert_eq!(task.status, goguo_lib::models::recovery::RecoveryStatus::Failed);

    // Acknowledge the failure.
    let ack = recovery_mgr.acknowledge_task().expect("ack");
    assert_eq!(
        ack.status,
        goguo_lib::models::recovery::RecoveryStatus::UserAcknowledged
    );

    // Task is now terminal and cleaned up.
    assert!(recovery_mgr.load_task().expect("load").is_none());
}

// ── Scenario 5: State Summary and Baseline Status Commands ─────────────────

#[test]
fn state_summary_reflects_category_counts() {
    let dir = tempfile::TempDir::new().expect("dir");
    let items = vec![
        make_restorable("a", "v1"),
        make_restorable("b", "v2"),
        make_detectable("c", "v3"),
        make_detectable("d", "v4"),
        make_detectable("e", "v5"),
    ];
    let (mgr, _) = setup_env(dir.path(), items);

    let summary = cmds::get_state_summary(&mgr).expect("summary");
    assert_eq!(summary.total, 5);
    assert_eq!(summary.restorable_count, 2);
    assert_eq!(summary.detectable_count, 3);
    assert_eq!(summary.excluded_count, 0);
}

#[test]
fn baseline_status_empty_when_no_snapshot() {
    let dir = tempfile::TempDir::new().expect("dir");
    let items = vec![make_restorable("a", "v1")];
    let (mgr, _) = setup_env(dir.path(), items);

    let status = cmds::get_baseline_status(&mgr).expect("status");
    assert!(!status.has_baseline);
    assert!(status.items.is_empty());
}

#[test]
fn trigger_readjustment_overwrites_initial_snapshot() {
    let dir = tempfile::TempDir::new().expect("dir");
    let items = vec![make_restorable("a", "first")];
    let (mgr, _) = setup_env(dir.path(), items);

    let first = cmds::start_initial_assessment(&mgr).expect("first");
    assert_eq!(first.item_count, 1);

    // Trigger readjustment (same values, should overwrite).
    let second = cmds::trigger_readjustment(&mgr).expect("second");
    assert_eq!(second.item_count, 1);

    // Confirm both produced version 0.
    assert_eq!(first.version, 0);
    assert_eq!(second.version, 0);
}

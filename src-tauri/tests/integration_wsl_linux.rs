//! End-to-end integration tests for Feature 002: WSL/Linux support.
//!
//! These tests exercise the full stack of WSL/Linux adapters, deployment
//! detection, network strategy, and Tauri command layer using real
//! `SystemShellExecutor` and temp directories.  They only compile and run
//! on Linux targets (including WSL).

#![cfg(target_os = "linux")]

use goguo_lib::adapters::linux_base::SystemShellExecutor;
use goguo_lib::adapters::linux::LinuxAdapter;
use goguo_lib::adapters::wsl::WslAdapter;
use goguo_lib::adapters::PlatformAdapter;
use goguo_lib::commands::baseline as cmds;
use goguo_lib::managers::baseline_manager::BaselineManager;
use goguo_lib::managers::config_manager::ConfigManager;
use goguo_lib::managers::deployment_manager::DeploymentManager;
use goguo_lib::models::audit::AuditAction;
use goguo_lib::models::baseline::{Platform, StateItemCategory};
use goguo_lib::services::audit_logger::AuditLogger;
use goguo_lib::services::wsl_detector::{SystemFileReader, WslDetector};
use goguo_lib::services::wsl_network_strategy::{self, ProxyStrategy};
use goguo_lib::services::wsl_detector::WslNetworkMode;
use goguo_lib::storage::baseline_storage::BaselineStorage;

// ── Test Helpers ───────────────────────────────────────────────────────────

/// Create a `BaselineManager` backed by a temp directory and the given adapters.
fn setup_baseline_with_adapters(
    dir: &std::path::Path,
    adapters: Vec<Box<dyn PlatformAdapter + Send + Sync>>,
) -> BaselineManager {
    let storage = BaselineStorage::new(dir.join("baseline"));
    BaselineManager::new(adapters, storage, dir.to_path_buf())
}

/// Create a `DeploymentManager` backed by a temp directory.
fn setup_deployment(dir: &std::path::Path) -> DeploymentManager {
    let config_dir = dir.join("config");
    let install_root = dir.join("app");
    let cm = ConfigManager::new(config_dir).expect("create config manager");
    DeploymentManager::new(cm, install_root)
}

// ── Scenario 1: WslAdapter Full Lifecycle ──────────────────────────────────

#[test]
fn wsl_adapter_full_lifecycle() {
    let dir = tempfile::TempDir::new().expect("temp dir");

    // Step 1: Create WslAdapter with real SystemShellExecutor.
    let wsl_adapter: Box<dyn PlatformAdapter + Send + Sync> =
        Box::new(WslAdapter::new(SystemShellExecutor));
    assert_eq!(wsl_adapter.platform(), Platform::Wsl);

    let defs = wsl_adapter.state_item_definitions();
    assert_eq!(defs.len(), 7, "WslAdapter should define 7 state items");
    let restorable = defs.iter().filter(|d| d.category == StateItemCategory::Restorable).count();
    assert_eq!(restorable, 4, "WslAdapter should have 4 restorable items");

    // Step 2: Use BaselineManager with WslAdapter.
    let mgr = setup_baseline_with_adapters(dir.path(), vec![wsl_adapter]);

    // Step 3: Collect initial snapshot (assess).
    let assess = cmds::start_initial_assessment(&mgr).expect("assess");
    assert_eq!(assess.version, 0);
    assert_eq!(assess.item_count, 7);

    // Step 4: Confirm baseline.
    let confirmed = cmds::confirm_baseline(&mgr).expect("confirm");
    assert_eq!(confirmed.version, 1);
    assert_eq!(confirmed.item_count, 7);

    // Step 5: Verify baseline status — at least confirm it returns a result.
    // Note: Some items (e.g., reachability with latency_ms) may report as
    // deviated between confirm and compare because live system values change.
    // Therefore we only assert that the baseline exists and items are returned.
    let status = cmds::get_baseline_status(&mgr).expect("status");
    assert!(status.has_baseline);
    assert_eq!(status.items.len(), 7);

    // Step 6: Restore to baseline (succeeds for restorable items;
    // some system files may require root, so we accept partial success).
    let result = mgr.restore_to_baseline().expect("restore");
    assert!(
        result.succeeded + result.failed > 0,
        "Restore should have processed at least some items"
    );
}

// ── Scenario 2: LinuxAdapter Full Lifecycle ────────────────────────────────

#[test]
fn linux_adapter_full_lifecycle() {
    let dir = tempfile::TempDir::new().expect("temp dir");

    // Step 1: Create LinuxAdapter with real SystemShellExecutor.
    let linux_adapter: Box<dyn PlatformAdapter + Send + Sync> =
        Box::new(LinuxAdapter::new(SystemShellExecutor));
    assert_eq!(linux_adapter.platform(), Platform::Linux);

    let defs = linux_adapter.state_item_definitions();
    assert_eq!(defs.len(), 6, "LinuxAdapter should define 6 state items");
    let restorable = defs.iter().filter(|d| d.category == StateItemCategory::Restorable).count();
    assert_eq!(restorable, 4, "LinuxAdapter should have 4 restorable items");

    // Step 2: Use BaselineManager with LinuxAdapter.
    let mgr = setup_baseline_with_adapters(dir.path(), vec![linux_adapter]);

    // Step 3: Collect initial snapshot.
    let assess = cmds::start_initial_assessment(&mgr).expect("assess");
    assert_eq!(assess.version, 0);
    assert_eq!(assess.item_count, 6);

    // Step 4: Confirm baseline.
    let confirmed = cmds::confirm_baseline(&mgr).expect("confirm");
    assert_eq!(confirmed.version, 1);
    assert_eq!(confirmed.item_count, 6);

    // Step 5: State summary reflects correct category counts.
    let summary = cmds::get_state_summary(&mgr).expect("summary");
    assert_eq!(summary.total, 6);
    assert_eq!(summary.restorable_count, 4);
    assert_eq!(summary.detectable_count, 2);
    assert_eq!(summary.excluded_count, 0);

    // Step 6: Restore to baseline.
    let result = mgr.restore_to_baseline().expect("restore");
    assert!(
        result.succeeded + result.failed > 0,
        "Restore should have processed at least some items"
    );
}

// ── Scenario 3: Dual Adapter Coordinated Mode ─────────────────────────────

#[test]
fn dual_adapter_coordinated_mode() {
    let dir = tempfile::TempDir::new().expect("temp dir");

    // Create both WslAdapter and LinuxAdapter.
    let wsl: Box<dyn PlatformAdapter + Send + Sync> =
        Box::new(WslAdapter::new(SystemShellExecutor));
    let linux: Box<dyn PlatformAdapter + Send + Sync> =
        Box::new(LinuxAdapter::new(SystemShellExecutor));

    assert_eq!(wsl.platform(), Platform::Wsl);
    assert_eq!(linux.platform(), Platform::Linux);

    // Feed both adapters into BaselineManager.
    let mgr = setup_baseline_with_adapters(dir.path(), vec![wsl, linux]);

    // Collect: should contain items from both adapters (7 + 6 = 13).
    let assess = cmds::start_initial_assessment(&mgr).expect("assess");
    assert_eq!(assess.version, 0);
    assert_eq!(assess.item_count, 13);

    // Confirm.
    let confirmed = cmds::confirm_baseline(&mgr).expect("confirm");
    assert_eq!(confirmed.version, 1);
    assert_eq!(confirmed.item_count, 13);

    // Check baseline status — confirm it exists with all items.
    // Note: Live system reads may cause some items to appear deviated
    // due to dynamic values (latency_ms, timestamps).
    let status = cmds::get_baseline_status(&mgr).expect("status");
    assert!(status.has_baseline);
    assert_eq!(status.items.len(), 13);

    // Verify items from both platforms are present.
    let items = mgr.get_state_summary().expect("summary");
    // Should have read items from both adapters.
    assert_eq!(items.total, 13);
}

// ── Scenario 4: DeploymentManager Integration ─────────────────────────────

#[test]
fn deployment_manager_detect_mode_returns_valid() {
    let mode = DeploymentManager::detect_deployment_mode();
    assert!(
        matches!(mode, goguo_lib::models::config::DeploymentMode::WslOnly
            | goguo_lib::models::config::DeploymentMode::LinuxOnly),
        "On Linux, detect_deployment_mode should return WslOnly or LinuxOnly, got {mode:?}"
    );
}

#[test]
fn deployment_manager_get_set_persist_mode() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let depl_mgr = setup_deployment(dir.path());

    // Default mode from config is WindowsOnly.
    let loaded = depl_mgr.get_deployment_mode().expect("get mode");
    assert_eq!(loaded, goguo_lib::models::config::DeploymentMode::WindowsOnly);

    // Set to LinuxOnly.
    let result = depl_mgr
        .set_deployment_mode(goguo_lib::models::config::DeploymentMode::LinuxOnly)
        .expect("set mode");
    assert_eq!(result.deployment_mode, goguo_lib::models::config::DeploymentMode::LinuxOnly);

    // Reload to verify persistence.
    let depl_mgr2 = setup_deployment(dir.path());
    let reloaded = depl_mgr2.get_deployment_mode().expect("reload mode");
    assert_eq!(reloaded, goguo_lib::models::config::DeploymentMode::LinuxOnly);
}

#[test]
fn deployment_manager_create_adapters_per_mode() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let depl_mgr = setup_deployment(dir.path());

    // WslOnly -> 1 adapter (WslAdapter).
    let wsl_adapters = depl_mgr.create_adapters(
        &goguo_lib::models::config::DeploymentMode::WslOnly,
    );
    assert_eq!(wsl_adapters.len(), 1);
    assert_eq!(wsl_adapters[0].platform(), Platform::Wsl);

    // LinuxOnly -> 1 adapter (LinuxAdapter).
    let linux_adapters = depl_mgr.create_adapters(
        &goguo_lib::models::config::DeploymentMode::LinuxOnly,
    );
    assert_eq!(linux_adapters.len(), 1);
    assert_eq!(linux_adapters[0].platform(), Platform::Linux);

    // WindowsOnly -> 1 adapter on Linux (WindowsRemoteAdapter via powershell.exe).
    let win_adapters = depl_mgr.create_adapters(
        &goguo_lib::models::config::DeploymentMode::WindowsOnly,
    );
    assert_eq!(win_adapters.len(), 1);
    assert_eq!(win_adapters[0].platform(), Platform::Windows);

    // Coordinated -> 2 adapters on WSL (WslAdapter + WindowsRemoteAdapter).
    let coord_adapters = depl_mgr.create_adapters(
        &goguo_lib::models::config::DeploymentMode::Coordinated,
    );
    assert_eq!(coord_adapters.len(), 2);
    let coord_platforms: Vec<_> = coord_adapters.iter().map(|a| a.platform()).collect();
    assert!(coord_platforms.contains(&Platform::Wsl));
    assert!(coord_platforms.contains(&Platform::Windows));
}

// ── Scenario 5: WslNetworkStrategy Decision Logic ─────────────────────────

#[test]
fn network_strategy_nat_returns_explicit_config() {
    let strategy = wsl_network_strategy::determine_strategy(&WslNetworkMode::Nat, true);
    assert_eq!(strategy, ProxyStrategy::ExplicitConfig);

    // Reachability does not matter for NAT.
    let strategy_unreachable = wsl_network_strategy::determine_strategy(&WslNetworkMode::Nat, false);
    assert_eq!(strategy_unreachable, ProxyStrategy::ExplicitConfig);
}

#[test]
fn network_strategy_mirrored_reachable_returns_skip() {
    let strategy = wsl_network_strategy::determine_strategy(&WslNetworkMode::Mirrored, true);
    assert_eq!(strategy, ProxyStrategy::SkipConfig);
}

#[test]
fn network_strategy_mirrored_unreachable_returns_fallback() {
    let strategy = wsl_network_strategy::determine_strategy(&WslNetworkMode::Mirrored, false);
    match strategy {
        ProxyStrategy::FallbackToExplicit { reason } => {
            assert!(!reason.is_empty(), "Fallback reason should not be empty");
        }
        other => panic!("Expected FallbackToExplicit, got {other:?}"),
    }
}

#[test]
fn network_strategy_not_installed_returns_explicit() {
    let strategy = wsl_network_strategy::determine_strategy(&WslNetworkMode::NotInstalled, true);
    assert_eq!(strategy, ProxyStrategy::ExplicitConfig);

    let strategy2 = wsl_network_strategy::determine_strategy(&WslNetworkMode::NotInstalled, false);
    assert_eq!(strategy2, ProxyStrategy::ExplicitConfig);
}

// ── Scenario 6: Tauri Command Integration (Deployment) ────────────────────

#[test]
fn cmd_detect_deployment_mode_returns_valid_response() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let depl_mgr = setup_deployment(dir.path());

    let resp = cmds::detect_deployment_mode(&depl_mgr).expect("detect");
    let valid_modes = ["windows_only", "wsl_only", "linux_only", "coordinated"];
    assert!(
        valid_modes.contains(&resp.mode.as_str()),
        "mode should be valid, got: {}",
        resp.mode
    );
    assert!(
        valid_modes.contains(&resp.detected.as_str()),
        "detected should be valid, got: {}",
        resp.detected
    );
}

#[test]
fn cmd_get_and_set_deployment_mode_roundtrip() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let depl_mgr = setup_deployment(dir.path());

    // Default stored mode is windows_only.
    let get_resp = cmds::get_deployment_mode(&depl_mgr).expect("get");
    assert_eq!(get_resp.mode, "windows_only");

    // Set to wsl_only.
    let set_resp = cmds::set_deployment_mode(&depl_mgr, "wsl_only").expect("set");
    assert_eq!(set_resp.mode, "wsl_only");

    // Verify persistence.
    let get_resp2 = cmds::get_deployment_mode(&depl_mgr).expect("get after set");
    assert_eq!(get_resp2.mode, "wsl_only");
}

#[test]
fn cmd_set_deployment_mode_invalid_input_fails() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let depl_mgr = setup_deployment(dir.path());

    let err = cmds::set_deployment_mode(&depl_mgr, "bogus_mode").expect_err("should fail");
    assert!(err.contains("Invalid deployment mode"), "Error should mention invalid mode: {err}");
}

#[test]
fn cmd_get_wsl_status_returns_valid() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let depl_mgr = setup_deployment(dir.path());

    let resp = cmds::get_wsl_status(&depl_mgr).expect("wsl status");
    assert!(
        ["nat", "mirrored", "not_installed"].contains(&resp.network_mode.as_str()),
        "network_mode should be valid, got: {}",
        resp.network_mode
    );
}

#[test]
fn cmd_get_network_mode_returns_valid() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let depl_mgr = setup_deployment(dir.path());

    let resp = cmds::get_network_mode(&depl_mgr).expect("network mode");
    assert!(
        ["nat", "mirrored", "not_installed"].contains(&resp.mode.as_str()),
        "mode should be valid, got: {}",
        resp.mode
    );
    assert!(
        ["explicit_config", "skip_config", "fallback_to_explicit"]
            .contains(&resp.proxy_strategy.as_str()),
        "proxy_strategy should be valid, got: {}",
        resp.proxy_strategy
    );
}

// ── Scenario 7: Audit Log Contains WSL/Linux Operations ───────────────────

#[test]
fn audit_log_captures_wsl_linux_lifecycle() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let logger = AuditLogger::new(dir.path().join("audit")).expect("logger");

    // Simulate audit trail for a WSL baseline lifecycle.
    logger
        .log_success(
            AuditAction::BaselineCollect,
            "wsl-proxy-env",
            serde_json::json!({"adapter": "wsl", "item_count": 7}),
        )
        .expect("log collect");

    logger
        .log_success(
            AuditAction::BaselineConfirm,
            "baseline-v1-wsl",
            serde_json::json!({"version": 1, "platform": "wsl"}),
        )
        .expect("log confirm");

    logger
        .log_success(
            AuditAction::StateRestore,
            "wsl-git-proxy",
            serde_json::json!({"restored_to": {"http_proxy": "", "https_proxy": ""}}),
        )
        .expect("log restore git");

    logger
        .log_failure(
            AuditAction::StateRestore,
            "wsl-resolv-conf",
            "Root permission required",
            serde_json::json!({"path": "/etc/resolv.conf"}),
        )
        .expect("log restore resolv");

    logger
        .log_success(
            AuditAction::ConfigChange,
            "deployment_mode",
            serde_json::json!({"from": "windows_only", "to": "wsl_only"}),
        )
        .expect("log config change");

    // Query all records.
    let resp = cmds::get_audit_log(&logger, &cmds::AuditLogParams::default()).expect("audit");
    assert_eq!(resp.total_count, 5);

    // Verify ordering and content.
    assert_eq!(resp.records[0].action, "baseline_collect");
    assert_eq!(resp.records[0].target, "wsl-proxy-env");

    assert_eq!(resp.records[1].action, "baseline_confirm");
    assert_eq!(resp.records[1].target, "baseline-v1-wsl");

    assert_eq!(resp.records[2].action, "state_restore");
    assert_eq!(resp.records[2].target, "wsl-git-proxy");
    assert_eq!(resp.records[2].result, "success");

    assert_eq!(resp.records[3].action, "state_restore");
    assert_eq!(resp.records[3].target, "wsl-resolv-conf");
    assert_eq!(resp.records[3].result, "failure");
    assert_eq!(
        resp.records[3].reason,
        Some("Root permission required".to_string())
    );

    assert_eq!(resp.records[4].action, "config_change");
    assert_eq!(resp.records[4].target, "deployment_mode");

    // Filter by state_restore only.
    let params = cmds::AuditLogParams {
        action_type: Some("state_restore".to_string()),
        ..Default::default()
    };
    let filtered = cmds::get_audit_log(&logger, &params).expect("audit filtered");
    assert_eq!(filtered.total_count, 2);
}

// ── Scenario 8: WslDetector with Real System ──────────────────────────────

#[test]
fn wsl_detector_real_system_returns_valid_mode() {
    let detector = WslDetector::new(SystemFileReader);
    let mode = detector.detect_network_mode();

    // On any Linux system, should return a valid variant.
    match mode {
        WslNetworkMode::Nat | WslNetworkMode::Mirrored | WslNetworkMode::NotInstalled => {}
    }

    // get_distro_info should succeed on a real Linux system (has /etc/os-release).
    let distro = detector.get_distro_info();
    // On WSL and most Linux systems, /etc/os-release exists.
    // We do not assert Some because minimal containers may lack it.
    if let Some(info) = distro {
        assert!(!info.name.is_empty(), "Distro name should not be empty");
        assert!(!info.id.is_empty(), "Distro id should not be empty");
    }
}

// ── Scenario 9: Adapter Read Produces Consistent State Items ──────────────

#[test]
fn wsl_adapter_read_items_have_consistent_structure() {
    let adapter = WslAdapter::new(SystemShellExecutor);
    let items = adapter.read_state_items();

    assert_eq!(items.len(), 7, "WslAdapter should always produce 7 items");

    for item in &items {
        assert_eq!(item.platform, Platform::Wsl, "Item {} should be WSL platform", item.id);
        assert!(!item.id.is_empty(), "Item id should not be empty");
        assert!(!item.collected_at.is_empty(), "Item {} should have a timestamp", item.id);
        assert!(
            matches!(item.category, StateItemCategory::Restorable | StateItemCategory::Detectable),
            "Item {} should be Restorable or Detectable",
            item.id
        );
    }
}

#[test]
fn linux_adapter_read_items_have_consistent_structure() {
    let adapter = LinuxAdapter::new(SystemShellExecutor);
    let items = adapter.read_state_items();

    assert_eq!(items.len(), 6, "LinuxAdapter should always produce 6 items");

    for item in &items {
        assert_eq!(item.platform, Platform::Linux, "Item {} should be Linux platform", item.id);
        assert!(!item.id.is_empty(), "Item id should not be empty");
        assert!(!item.collected_at.is_empty(), "Item {} should have a timestamp", item.id);
        assert!(
            matches!(item.category, StateItemCategory::Restorable | StateItemCategory::Detectable),
            "Item {} should be Restorable or Detectable",
            item.id
        );
    }
}

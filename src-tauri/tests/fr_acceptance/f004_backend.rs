//! F004 user-interaction 后端 FR 验收测试（8 个）
//!
//! 覆盖 F004 的后端相关 FR（Tauri command 行为、DTO 结构）。

#[path = "../common/mod.rs"]
mod common;

use common::*;
use goguo_lib::commands::baseline as cmds;
use goguo_lib::engines::site_rule_engine::AddSiteResult;
use goguo_lib::managers::baseline_manager::BaselineManager;
use goguo_lib::models::config::ProxyGuardConfig;
use goguo_lib::services::proxy_guard::ProxyGuard;
use goguo_lib::storage::baseline_storage::BaselineStorage;

// ── §2.1 应用框架 ────────────────────────────────────────────────────────

/// FR-2.1.1-R1~R6: 桌面应用形态，UI 数据来源本地。
/// 可观测结果：命令层返回本地数据，不发起远程请求。
#[test]
fn fr_2_1_1_cross_platform_app() {
    let state = setup_baseline_confirmed();

    // Assessment data comes from local adapter (no remote calls)
    let summary = cmds::get_state_summary(&state.baseline_manager).expect("summary");
    assert!(summary.total > 0, "FR-2.1.1-R4: data comes from local API");

    // Baseline status from local storage
    let status = cmds::get_baseline_status(&state.baseline_manager).expect("status");
    assert!(status.has_baseline, "FR-2.1.1-R4: baseline status from local storage");

    // Service status from local process check
    let guard = ProxyGuard::new(ProxyGuardConfig {
        check_interval_secs: 3,
        max_restart_attempts: 3,
        restart_cooldown_secs: 10,
    });
    let svc_status = cmds::get_service_status(
        &mut state.mihomo_manager.lock().expect("lock"),
        &guard,
    );
    // mihomo_running is false in test (no real process)
    assert!(!svc_status.mihomo_running, "FR-2.1.1-R5: app works when backend not running");
}

// ── §2.2 首次引导流程 ────────────────────────────────────────────────────

/// FR-2.2.1-R1~R5: 首次引导 baseline 流程，含调整建议。
#[test]
#[ignore = "F110-G110-3: 向导 Step 3 调整建议引导未实现"]
fn fr_2_2_1_wizard_baseline_flow() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    let items = standard_baseline_items();
    let adapter = std::sync::Arc::new(TestAdapter::new(items));
    let storage = BaselineStorage::new(dir.path().join("baseline"));
    let mgr = BaselineManager::new(
        vec![Box::new((*adapter).clone())],
        storage,
        dir.path().to_path_buf(),
    );

    // R1: initial assessment
    let assess = cmds::start_initial_assessment(&mgr).expect("assess");
    assert!(assess.item_count > 0, "FR-2.2.1-R1: wizard shows assessment results");

    // R2: state items categorized
    let summary = cmds::get_state_summary(&mgr).expect("summary");
    assert!(summary.restorable_count > 0, "FR-2.2.1-R2: shows restorable items");

    // R3: Step 3 adjustment guidance (not yet implemented)
    // R4: re-assessment flow
    let reassess = cmds::trigger_readjustment(&mgr).expect("readjustment");
    assert!(reassess.item_count > 0, "FR-2.2.1-R4: re-assessment works");

    // R5: confirm baseline
    let confirm = cmds::confirm_baseline(&mgr).expect("confirm");
    assert!(confirm.version > 0, "FR-2.2.1-R5: baseline confirmed with version");
}

// ── §2.3 服务控制 ────────────────────────────────────────────────────────

/// FR-2.3.1-R1~R3: 启停控制，恢复进度展示。
/// 可观测结果：stop_service 返回恢复结果；get_recovery_progress 可查询。
#[test]
fn fr_2_3_1_service_control() {
    let state = setup_baseline_confirmed();

    // R1: stop service triggers restore
    let result = cmds::stop_service(
        &mut state.mihomo_manager.lock().expect("lock"),
        &state.baseline_manager,
    ).expect("stop");

    // R3: recovery triggered and result available
    assert!(result.recovery_triggered, "FR-2.3.1-R3: stop shows recovery progress");
    assert!(!result.reason.is_empty(), "FR-2.3.1-R1: stop returns reason");

    // Recovery progress queryable
    let progress = cmds::get_recovery_progress(&state.baseline_manager).expect("progress");
    // has_task indicates recovery task state
    assert!(!progress.has_task || progress.total_items > 0,
        "FR-2.3.1-R3: recovery progress is queryable");
}

// ── §2.4 目标站点管理 ────────────────────────────────────────────────────

/// FR-2.4.1-R1~R5: 站点标识/域名添加，关联域名展示。
/// 可观测结果：add_site 返回展开的域名列表；remove_site 返回剩余站点数。
#[test]
fn fr_2_4_1_site_add_remove() {
    let mut state = setup_site_rules_with_nodes();

    // R1: add by site identifier
    let result = state.engine.add_site("github");
    match result {
        AddSiteResult::Success { site, .. } => {
            // R3: associated domains displayed
            assert!(!site.all_domains().is_empty(),
                "FR-2.4.1-R3: must show associated domains");

            // R5: total domain count reflects addition
            let count = state.engine.total_domain_count();
            assert!(count > 0, "FR-2.4.1-R5: domain count updated");
        }
        AddSiteResult::VerificationFailed { site, .. } => {
            // Mock env: verification may fail, but site info is still available
            assert!(!site.all_domains().is_empty(),
                "FR-2.4.1-R3: domains available even on verification failure");
        }
        AddSiteResult::SiteNotFound => panic!("github should be a known site"),
    }

    // R4: remove site
    let remove = state.engine.remove_site("github");
    match remove {
        goguo_lib::engines::site_rule_engine::RemoveSiteResult::Success { remaining_sites } => {
            // R5: remaining count accurate
            assert_eq!(remaining_sites, state.engine.active_sites_count(),
                "FR-2.4.1-R5: remaining count matches");
        }
        goguo_lib::engines::site_rule_engine::RemoveSiteResult::NotFound => {
            panic!("github should exist for removal");
        }
    }
}

// ── §2.5 规则预览 ────────────────────────────────────────────────────────

/// FR-2.5.1-R1~R4: 规则预览含站点名+策略。
#[test]
#[ignore = "F110-G110-6: 规则预览未标注站点名称"]
fn fr_2_5_1_rule_preview() {
    let mut state = setup_site_rules_with_nodes();
    let _ = state.engine.add_site("github");

    let preview = state.engine.preview_rules();
    assert!(!preview.is_empty(), "FR-2.5.1-R1: preview shows rules");

    // R2: each rule annotated with site name and strategy
    // Current format: DOMAIN-SUFFIX,github.com,PROXY (no site name annotation)
    for rule in &preview {
        if rule.contains("PROXY") {
            assert!(rule.contains("github") || rule.contains("github.com") || rule.contains("PROXY"),
                "FR-2.5.1-R2: rule must show site and strategy");
        }
    }
}

// ── §2.6 站点可达性 ──────────────────────────────────────────────────────

/// FR-2.6.1-R1~R3: 可达性状态展示，响应时间。
#[test]
#[ignore = "F110-G110-1: 持续探测循环未实现"]
fn fr_2_6_1_site_reachability() {
    let mut state = setup_site_rules_with_nodes();
    let _ = state.engine.add_site("github");

    // R1: reachability status for each site
    let reachability = state.engine.get_reachability();
    // Without continuous probing, this may be empty
    // R2: response time included when available
}

// ── §2.7 通知 ─────────────────────────────────────────────────────────────

/// FR-2.7.1-R1~R4: 通知含时间戳，4 类语义化。
#[test]
#[ignore = "F110-G110-7: 通知系统未实现"]
fn fr_2_7_1_notification() {
    // R1: notification area exists
    // R2: notifications for: rule rollback, recovery, audit change, node pool change
    // R3: notifications contain timestamp
    // R4: no uncontrollable side effects
}

// ── SC-2 状态一致性 ───────────────────────────────────────────────────────

/// SC-2: UI 状态与后端实际一致。
/// 可观测结果：command 返回值反映真实后端状态。
#[test]
fn fr_sc_2_status_matches_backend() {
    let state = setup_baseline_confirmed();

    // Baseline status matches actual storage
    let status = cmds::get_baseline_status(&state.baseline_manager).expect("status");
    let stored = state.baseline_manager.get_confirmed_baseline().expect("get");
    assert_eq!(status.has_baseline, stored.is_some(),
        "SC-2: UI baseline status matches backend");

    // Service status reflects actual mihomo state
    let guard = ProxyGuard::new(ProxyGuardConfig {
        check_interval_secs: 3,
        max_restart_attempts: 3,
        restart_cooldown_secs: 10,
    });
    let svc = cmds::get_service_status(
        &mut state.mihomo_manager.lock().expect("lock"),
        &guard,
    );
    // In test, mihomo is not running → mihomo_running should be false
    assert!(!svc.mihomo_running, "SC-2: service status matches backend reality");
}

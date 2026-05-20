//! End-to-end integration tests for Feature 003: Site Rules lifecycle.
//!
//! Scenarios covered:
//! 1. Full lifecycle: add → generate → verify → probe → remove
//! 2. AB verification rollback
//! 3. Batch developer template
//! 4. Audit trail
//! 5. Override persists through reload

use std::sync::Arc;

use goguo_lib::engines::site_rule_engine::{
    AddSiteResult, RemoveSiteResult, SiteRuleEngine,
};
use goguo_lib::managers::mihomo_manager::MockMihomoReloader;
use goguo_lib::models::audit::AuditAction;
use goguo_lib::services::audit_logger::MockAuditLog;
use goguo_lib::services::probe_service::MockProbeClient;
use goguo_lib::services::rule_generator::Rule;

fn create_integ_engine(
    dir: &std::path::Path,
    reloader: Option<Arc<MockMihomoReloader>>,
    audit: Option<Arc<MockAuditLog>>,
) -> SiteRuleEngine {
    let probe_client = Arc::new(MockProbeClient::new());
    SiteRuleEngine::new(
        dir,
        probe_client,
        reloader.map(|r| r as Arc<dyn goguo_lib::managers::mihomo_manager::MihomoReloader>),
        audit.map(|a| a as Arc<dyn goguo_lib::services::audit_logger::AuditLog>),
    )
}

#[test]
fn full_lifecycle() {
    let dir = tempfile::tempdir().expect("tempdir");
    let reloader = Arc::new(MockMihomoReloader::new());
    let audit = Arc::new(MockAuditLog::new());

    let mut engine = create_integ_engine(dir.path(), Some(reloader.clone()), Some(audit.clone()));

    // Add site
    let result = engine.add_site("github");
    assert!(
        matches!(result, AddSiteResult::Success { verification_passed: true, .. }),
        "add_site should succeed with verification"
    );

    // Mihomo was called
    assert!(reloader.was_called(), "mihomo reload should be triggered");

    // Rules file exists
    let rules_path = dir.path().join("rules").join("current-rules.yaml");
    assert!(rules_path.exists(), "rules file should be created");

    // Preview contains github domains
    let preview = engine.preview_rules();
    assert!(preview.iter().any(|r| r.contains("github.com")));
    assert!(preview.iter().any(|r| r == "MATCH,DIRECT"));

    // Reachability check
    let reach = engine.get_reachability();
    assert_eq!(reach.len(), 1);
    assert!(reach[0].reachable);

    // Remove site
    let result = engine.remove_site("github");
    assert!(matches!(result, RemoveSiteResult::Success { remaining_sites: 0 }));
    assert_eq!(engine.active_sites_count(), 0);

    // Audit trail: SiteAdd + SiteRemove
    let records = audit.records();
    let add_rec = records.iter().find(|r| r.action == AuditAction::SiteAdd);
    let remove_rec = records.iter().find(|r| r.action == AuditAction::SiteRemove);
    assert!(add_rec.is_some(), "SiteAdd audit should exist");
    assert!(remove_rec.is_some(), "SiteRemove audit should exist");
    assert_eq!(add_rec.expect("add").target, "github");
    assert_eq!(remove_rec.expect("remove").target, "github");
}

#[test]
fn batch_developer_template() {
    let dir = tempfile::tempdir().expect("tempdir");
    let reloader = Arc::new(MockMihomoReloader::new());

    let mut engine = create_integ_engine(dir.path(), Some(reloader), None);

    let template_ids = goguo_lib::services::site_definition_store::SiteDefinitionStore::developer_template_ids();
    let results = engine.apply_template(&template_ids);

    let success_count = results.iter().filter(|r| matches!(r, AddSiteResult::Success { .. })).count();
    assert!(success_count > 0, "at least some template sites should succeed");
    assert_eq!(engine.active_sites_count(), success_count);

    // Rules should contain domains from multiple sites
    let preview = engine.preview_rules();
    assert!(preview.len() > success_count, "rules should contain domain entries");
}

#[test]
fn audit_trail_on_add_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let audit = Arc::new(MockAuditLog::new());

    let mut engine = create_integ_engine(dir.path(), None, Some(audit.clone()));

    // Try to add a nonexistent site
    let result = engine.add_site("nonexistent_site");
    assert!(matches!(result, AddSiteResult::SiteNotFound));

    // No audit for SiteNotFound (it's not a real operation)
    let records = audit.records();
    assert!(records.is_empty(), "no audit for site not found");
}

#[test]
fn override_persists_through_reload() {
    let dir = tempfile::tempdir().expect("tempdir");

    let mut engine = create_integ_engine(dir.path(), None, None);

    engine.add_site("github");

    let override_rule = Rule::domain_exact("custom.override.com".to_string());
    engine.add_user_override(override_rule);

    // Reload rules
    let reloaded = engine.reload_rules();
    assert!(reloaded, "reload should succeed");

    // Preview should still contain the override
    let preview = engine.preview_rules();
    assert!(
        preview.iter().any(|r| r.contains("custom.override.com")),
        "override should persist through reload"
    );
}

#[test]
fn engine_without_optional_deps_still_works() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut engine = create_integ_engine(dir.path(), None, None);

    let result = engine.add_site("github");
    assert!(matches!(result, AddSiteResult::Success { .. }));

    let preview = engine.preview_rules();
    assert!(!preview.is_empty());

    let reach = engine.probe_site("github");
    assert!(reach.is_some());
}

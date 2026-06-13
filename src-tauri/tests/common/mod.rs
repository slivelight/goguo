//! F113 测试基建 — Setup Helper 库
//!
//! 为 FR 验收测试提供统一的测试状态构建函数。
//! 所有 helper 内部使用 mock 适配器，不依赖真实系统状态。

use std::sync::{Arc, Mutex};

use goguo_lib::adapters::{PlatformAdapter, StateItemDefinition};
use goguo_lib::engines::site_rule_engine::SiteRuleEngine;
use goguo_lib::managers::baseline_manager::BaselineManager;
use goguo_lib::managers::mihomo_manager::{MihomoManager, MockMihomoReloader};
use goguo_lib::models::baseline::{Platform, StateItem, StateItemCategory};
use goguo_lib::models::config::MihomoConfig;
use goguo_lib::services::audit_logger::MockAuditLog;
use goguo_lib::services::probe_service::MockProbeClient;
use goguo_lib::storage::baseline_storage::BaselineStorage;

// ── Test Adapter ──────────────────────────────────────────────────────────

/// Shared write tracker: records all `write_state` calls across adapter clones.
pub type WriteLog = Arc<Mutex<Vec<(String, serde_json::Value)>>>;

/// Configurable mock adapter that tracks write calls via shared state.
pub struct TestAdapter {
    items: Vec<StateItem>,
    written: WriteLog,
    platform: Platform,
}

impl TestAdapter {
    pub fn new(items: Vec<StateItem>) -> Self {
        Self {
            items,
            written: Arc::new(Mutex::new(Vec::new())),
            platform: Platform::Linux,
        }
    }

    pub fn with_platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    pub fn written_values(&self) -> Vec<(String, serde_json::Value)> {
        self.written.lock().expect("lock").clone()
    }

    pub fn write_log(&self) -> &WriteLog {
        &self.written
    }

    /// Update the items the adapter returns from `read_state_items`.
    /// Useful for simulating state deviation after baseline confirmation.
    pub fn set_items(&mut self, items: Vec<StateItem>) {
        self.items = items;
    }
}

impl Clone for TestAdapter {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            written: Arc::clone(&self.written),
            platform: self.platform.clone(),
        }
    }
}

impl PlatformAdapter for TestAdapter {
    fn platform(&self) -> Platform {
        self.platform.clone()
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

// ── Item Constructors ─────────────────────────────────────────────────────

pub fn make_item(id: &str, category: StateItemCategory, value: &str) -> StateItem {
    StateItem {
        id: id.to_string(),
        platform: Platform::Linux,
        category,
        value: serde_json::json!(value),
        collected_at: "2026-06-11T12:00:00Z".to_string(),
        classification_reason: "fr-acceptance test".to_string(),
    }
}

pub fn make_restorable(id: &str, value: &str) -> StateItem {
    make_item(id, StateItemCategory::Restorable, value)
}

pub fn make_detectable(id: &str, value: &str) -> StateItem {
    make_item(id, StateItemCategory::Detectable, value)
}

pub fn make_excluded(id: &str, value: &str) -> StateItem {
    make_item(id, StateItemCategory::Excluded, value)
}

/// Standard set of state items representing a typical baseline.
/// 3 restorable + 2 detectable + 1 excluded = 6 items.
pub fn standard_baseline_items() -> Vec<StateItem> {
    vec![
        make_restorable("system-proxy", "off"),
        make_restorable("proxy-env", ""),
        make_restorable("hosts-file", "127.0.0.1 localhost"),
        make_detectable("dns-cache", "cached"),
        make_detectable("network-interfaces", "eth0"),
        make_excluded("os-version", "Ubuntu 22.04"),
    ]
}

// ── Config Helpers ────────────────────────────────────────────────────────

pub fn test_mihomo_config(dir: &std::path::Path) -> MihomoConfig {
    MihomoConfig {
        binary_path: dir.join("fake-mihomo"),
        config_dir: dir.join("mihomo"),
        api_address: "127.0.0.1:19999".to_string(),
        api_secret: "test".to_string(),
        mixed_port: 19999,
        log_level: "warning".to_string(),
    }
}

// ── TestState ─────────────────────────────────────────────────────────────

/// Core test state for baseline/service FR acceptance tests.
pub struct TestState {
    pub baseline_manager: BaselineManager,
    pub mihomo_manager: Arc<Mutex<MihomoManager>>,
    pub audit_logger: Arc<MockAuditLog>,
    pub adapter: Arc<TestAdapter>,
    /// Kept alive to prevent temp directory cleanup during test.
    pub temp_dir: tempfile::TempDir,
}

impl TestState {
    /// Path to the test's temp data directory.
    pub fn data_dir(&self) -> std::path::PathBuf {
        self.temp_dir.path().to_path_buf()
    }
}

// ── SiteRulesTestState ────────────────────────────────────────────────────

/// Test state for site-rules FR acceptance tests.
pub struct SiteRulesTestState {
    pub engine: SiteRuleEngine,
    pub probe_client: Arc<MockProbeClient>,
    pub reloader: Arc<MockMihomoReloader>,
    pub audit_logger: Arc<MockAuditLog>,
    /// Kept alive to prevent temp directory cleanup during test.
    pub temp_dir: tempfile::TempDir,
}

impl SiteRulesTestState {
    pub fn data_dir(&self) -> std::path::PathBuf {
        self.temp_dir.path().to_path_buf()
    }
}

// ── Setup Helpers ─────────────────────────────────────────────────────────

/// Build a baseline-confirmed test state.
///
/// Steps: create temp dir → create adapter with standard items →
///        build BaselineManager → collect snapshot → confirm baseline.
pub fn setup_baseline_confirmed() -> TestState {
    let dir = tempfile::TempDir::new().expect("temp dir");

    let items = standard_baseline_items();
    let adapter = Arc::new(TestAdapter::new(items));
    let audit_logger = Arc::new(MockAuditLog::new());

    let storage = BaselineStorage::new(dir.path().join("baseline"));
    let baseline_manager = BaselineManager::new(
        vec![Box::new((*adapter).clone())],
        storage,
        dir.path().to_path_buf(),
    );

    // Assess + confirm to establish a baseline
    let _snapshot = baseline_manager.collect_initial_snapshot().expect("assess");
    let _confirmed = baseline_manager.confirm_baseline().expect("confirm");

    let mihomo_config = test_mihomo_config(dir.path());
    let mihomo_manager = Arc::new(Mutex::new(MihomoManager::new(mihomo_config)));

    TestState {
        baseline_manager,
        mihomo_manager,
        audit_logger,
        adapter,
        temp_dir: dir,
    }
}

/// Build a test state where mihomo is conceptually running.
///
/// The baseline is already confirmed. MihomoManager exists but
/// doesn't spawn a real process in tests.
pub fn setup_service_running() -> TestState {
    setup_baseline_confirmed()
}

/// Build a test state where mihomo is stopped.
///
/// Equivalent to `setup_baseline_confirmed` — the manager exists
/// but no process has been started.
pub fn setup_service_stopped() -> TestState {
    setup_baseline_confirmed()
}

/// Build a site-rules test state with engine, probe client, and reloader.
///
/// The engine is initialized with mock dependencies.
/// No sites are pre-loaded — call `engine.add_target_site()` in tests.
pub fn setup_site_rules_with_nodes() -> SiteRulesTestState {
    let dir = tempfile::TempDir::new().expect("temp dir");

    let probe_client = Arc::new(MockProbeClient::new());
    let reloader = Arc::new(MockMihomoReloader::new());
    let audit_logger = Arc::new(MockAuditLog::new());

    let engine = SiteRuleEngine::new(
        dir.path(),
        probe_client.clone(),
        Some(reloader.clone()),
        Some(audit_logger.clone()),
    );

    SiteRulesTestState {
        engine,
        probe_client,
        reloader,
        audit_logger,
        temp_dir: dir,
    }
}

use crate::engines::site_rule_engine::{
    AddSiteResult, FiveElementPrompt, RemoveSiteResult, SiteReachability, SiteRuleEngine,
};
use crate::models::subscription::SubscriptionSource;
use crate::services::node_pool::{NodePool, NodePoolConfig};
use crate::services::probe_service::MockProbeClient;
use crate::services::subscription_parser::SubscriptionParser;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SiteInfo {
    pub id: String,
    pub name: String,
    pub domain_count: usize,
    pub domains: std::collections::HashMap<String, Vec<String>>,
}

impl From<crate::models::site::SiteDefinition> for SiteInfo {
    fn from(site: crate::models::site::SiteDefinition) -> Self {
        let domain_count = site.all_domains().len();
        let domains = site
            .domains
            .into_iter()
            .map(|(cat, list)| {
                let key = serde_json::to_string(&cat)
                    .expect("serialize category")
                    .trim_matches('"')
                    .to_string();
                (key, list)
            })
            .collect();
        Self {
            id: site.id,
            name: site.name,
            domain_count,
            domains,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AddSiteResponse {
    pub success: bool,
    pub site: Option<SiteInfo>,
    pub rules_generated: usize,
    pub verification_passed: bool,
    pub error: Option<String>,
    pub five_element_prompt: Option<FiveElementPrompt>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RemoveSiteResponse {
    pub success: bool,
    pub remaining_sites: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateResponse {
    pub added_count: usize,
    pub failed_count: usize,
    pub sites: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReachabilityResponse {
    pub sites: Vec<SiteReachability>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeInfo {
    pub name: String,
    pub protocol: String,
    pub status: String,
    pub latency_ms: Option<u64>,
    pub address: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NodePoolStatus {
    pub total_nodes: usize,
    pub available_nodes: usize,
    pub current_node: Option<String>,
    pub nodes: Vec<NodeInfo>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SubscriptionResponse {
    pub imported: usize,
    pub unsupported: usize,
    pub source_url: String,
}

pub struct SiteRulesState {
    pub engine: Mutex<SiteRuleEngine>,
    pub node_pool: Mutex<NodePool>,
    pub subscription_parser: Mutex<SubscriptionParser>,
    pub site_definition_store: crate::services::site_definition_store::SiteDefinitionStore,
}

impl SiteRulesState {
    #[must_use]
    pub fn new(
        install_root: &Path,
        mihomo_manager: Arc<Mutex<crate::managers::mihomo_manager::MihomoManager>>,
    ) -> Self {
        let data_dir = install_root.join("data");
        let probe_client = Arc::new(MockProbeClient::new());

        // Create RulesetWriter pointing to mihomo config dir
        let config_dir = data_dir.join("mihomo");
        let ruleset_writer =
            Some(crate::services::ruleset_writer::RulesetWriter::new(&config_dir));

        // Create MihomoManagerReloader wrapper
        let mihomo_reloader: Option<Arc<dyn crate::managers::mihomo_manager::MihomoReloader>> =
            Some(Arc::new(MihomoManagerReloader {
                manager: mihomo_manager,
                config_path: config_dir.join("config.yaml"),
            }));

        // Create MihomoConfigManager for per-site proxy groups
        let config_manager =
            crate::services::mihomo_config_manager::MihomoConfigManager::open(
                config_dir.join("config.yaml"),
            )
            .map_err(|e| {
                eprintln!("Warning: MihomoConfigManager open failed: {e}");
                e
            })
            .ok();

        let engine = SiteRuleEngine::new_with_config_manager(
            &data_dir,
            probe_client,
            mihomo_reloader,
            None,
            ruleset_writer,
            config_manager,
        );
        let node_pool = NodePool::new(NodePoolConfig::default());
        let subscription_parser = SubscriptionParser::new(
            data_dir.join("config").join("subscription-sources.json"),
        );
        let site_definition_store =
            crate::services::site_definition_store::SiteDefinitionStore::new(
                data_dir.join("config").join("site-definitions"),
            );
        Self {
            engine: Mutex::new(engine),
            node_pool: Mutex::new(node_pool),
            subscription_parser: Mutex::new(subscription_parser),
            site_definition_store,
        }
    }
}

/// Wrapper that adapts `Arc<Mutex<MihomoManager>>` to the `MihomoReloader` trait.
struct MihomoManagerReloader {
    manager: Arc<Mutex<crate::managers::mihomo_manager::MihomoManager>>,
    config_path: std::path::PathBuf,
}

impl crate::managers::mihomo_manager::MihomoReloader for MihomoManagerReloader {
    fn reload_config(&self, _config_path: &str) -> Result<(), crate::managers::mihomo_manager::MihomoError> {
        let mgr = self.manager.lock().expect("lock");
        mgr.reload_config(&self.config_path.to_string_lossy())
    }
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn add_target_site(site_id: String, state: tauri::State<'_, SiteRulesState>) -> AddSiteResponse {
    let mut engine = state.engine.lock().expect("lock");
    let result = engine.add_site(&site_id);
    
    match result {
        AddSiteResult::Success {
            site,
            rules_generated,
            verification_passed,
        } => AddSiteResponse {
            success: true,
            site: Some(SiteInfo::from(site)),
            rules_generated,
            verification_passed,
            error: None,
            five_element_prompt: None,
        },
        AddSiteResult::VerificationFailed { site, prompt } => AddSiteResponse {
            success: false,
            site: Some(SiteInfo::from(site)),
            rules_generated: 0,
            verification_passed: false,
            error: Some("Verification failed".to_string()),
            five_element_prompt: Some(prompt),
        },
        AddSiteResult::SiteNotFound => AddSiteResponse {
            success: false,
            site: None,
            rules_generated: 0,
            verification_passed: false,
            error: Some("Site not found".to_string()),
            five_element_prompt: None,
        },
    }
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn remove_target_site(
    site_id: String,
    state: tauri::State<'_, SiteRulesState>,
) -> RemoveSiteResponse {
    let mut engine = state.engine.lock().expect("lock");
    let result = engine.remove_site(&site_id);

    match result {
        RemoveSiteResult::Success { remaining_sites } => RemoveSiteResponse {
            success: true,
            remaining_sites,
            error: None,
        },
        RemoveSiteResult::NotFound => RemoveSiteResponse {
            success: false,
            remaining_sites: 0,
            error: Some("Site not found".to_string()),
        },
    }
}

/// Returns the list of currently active site IDs (F115 FR-2.2.5).
///
/// Read-only command backing the e2e `resetGoGuoState()` helper
/// (`e2e/helpers/state.ts`). Reuses the existing
/// `SiteRuleEngine::active_sites()` accessor (returns `&Vec<String>`);
/// holds the engine lock only for the duration of the clone — no mihomo
/// reload, no audit-log write, safe for `beforeEach` invocation.
#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn list_target_sites(state: tauri::State<'_, SiteRulesState>) -> Vec<String> {
    let engine = state.engine.lock().expect("lock");
    engine.active_sites().clone()
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn apply_preset_template(
    template: String,
    state: tauri::State<'_, SiteRulesState>,
) -> TemplateResponse {
    let mut engine = state.engine.lock().expect("lock");

    let template_ids = if template == "developer" {
        crate::services::site_definition_store::SiteDefinitionStore::developer_template_ids()
    } else if template == "office" {
        crate::services::site_definition_store::SiteDefinitionStore::office_template_ids()
    } else {
        return TemplateResponse {
            added_count: 0,
            failed_count: 0,
            sites: vec![],
        };
    };
    
    let results = engine.apply_template(&template_ids);
    
    let added: Vec<String> = results
        .iter()
        .filter_map(|r| match r {
            AddSiteResult::Success { site, .. } => Some(site.id.clone()),
            _ => None,
        })
        .collect();
    
    TemplateResponse {
        added_count: added.len(),
        failed_count: results.len() - added.len(),
        sites: added,
    }
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn preview_rules(state: tauri::State<'_, SiteRulesState>) -> Vec<String> {
    let engine = state.engine.lock().expect("lock");
    engine.preview_rules()
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn refresh_ip_cache(state: tauri::State<'_, SiteRulesState>) {
    let mut engine = state.engine.lock().expect("lock");
    engine.refresh_ip_cache();
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn get_site_reachability(
    state: tauri::State<'_, SiteRulesState>,
) -> ReachabilityResponse {
    let mut engine = state.engine.lock().expect("lock");
    ReachabilityResponse {
        sites: engine.get_reachability(),
    }
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn get_diagnosis(
    site_id: String,
    state: tauri::State<'_, SiteRulesState>,
) -> Option<SiteReachability> {
    let mut engine = state.engine.lock().expect("lock");
    engine.probe_site(&site_id)
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn get_node_pool_status(state: tauri::State<'_, SiteRulesState>) -> NodePoolStatus {
    let pool = state.node_pool.lock().expect("lock");
    let nodes: Vec<NodeInfo> = pool
        .nodes()
        .iter()
        .map(|n| NodeInfo {
            name: n.name.clone(),
            protocol: format!("{:?}", n.protocol).to_lowercase(),
            status: format!("{:?}", n.status).to_lowercase(),
            latency_ms: n.last_latency_ms,
            address: n.address.clone(),
        })
        .collect();
    NodePoolStatus {
        total_nodes: pool.node_count(),
        available_nodes: pool.available_count(),
        current_node: pool.current_node().map(|n| n.name.clone()),
        nodes,
    }
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn override_rule(
    rule_type: String,
    domain: String,
    state: tauri::State<'_, SiteRulesState>,
) -> bool {
    let mut engine = state.engine.lock().expect("lock");

    let rule = if rule_type == "DOMAIN-SUFFIX" {
        crate::services::rule_generator::Rule::domain_suffix(domain)
    } else {
        crate::services::rule_generator::Rule::domain_exact(domain)
    };

    engine.add_user_override(rule);
    true
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn import_subscription(
    url: String,
    _state: tauri::State<'_, SiteRulesState>,
) -> SubscriptionResponse {
    let result = crate::services::subscription_parser::SubscriptionParser::parse_raw_content(&url);
    
    SubscriptionResponse {
        imported: result.supported_count,
        unsupported: result.unsupported_count,
        source_url: url,
    }
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn get_subscription_sources(
    state: tauri::State<'_, SiteRulesState>,
) -> Vec<SubscriptionSource> {
    let parser = state.subscription_parser.lock().expect("lock");
    parser.load_sources().expect("load sources")
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SiteDefinitionInfo {
    pub id: String,
    pub name: String,
    pub domain_count: usize,
    pub domains: std::collections::HashMap<String, Vec<String>>,
}

impl From<crate::models::site::SiteDefinition> for SiteDefinitionInfo {
    fn from(site: crate::models::site::SiteDefinition) -> Self {
        let domain_count = site.domain_count();
        let domains = site
            .domains
            .into_iter()
            .map(|(cat, list)| {
                let key = serde_json::to_string(&cat)
                    .expect("serialize category")
                    .trim_matches('"')
                    .to_string();
                (key, list)
            })
            .collect();
        Self {
            id: site.id,
            name: site.name,
            domain_count,
            domains,
        }
    }
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn list_site_definitions(state: tauri::State<'_, SiteRulesState>) -> Vec<SiteDefinitionInfo> {
    state
        .site_definition_store
        .list_all()
        .into_iter()
        .map(SiteDefinitionInfo::from)
        .collect()
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn lookup_site(
    input: String,
    state: tauri::State<'_, SiteRulesState>,
) -> Option<SiteDefinitionInfo> {
    let domain = crate::services::url_parser::extract_domain(&input)?;
    let site = state.site_definition_store.lookup_by_domain(&domain)?;
    Some(SiteDefinitionInfo::from(site))
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CreateSiteResponse {
    pub success: bool,
    pub site: Option<SiteDefinitionInfo>,
    pub rules_generated: usize,
    pub error: Option<String>,
}

/// Creates a custom site and adds it to the active rule engine.
/// Used internally by tests and by the Tauri command.
fn create_custom_site(
    name: &str,
    display_name: &str,
    domains: Vec<String>,
    state: &SiteRulesState,
) -> CreateSiteResponse {
    use crate::models::site::{AccessStrategy, DomainCategory, HealthCheckConfig, SiteDefinition};

    // Generate id from name: lowercase, replace spaces with hyphens
    let site_id = name.to_lowercase().replace(' ', "-");

    // Reject if site_id collides with built-in
    if state.site_definition_store.get(&site_id).is_some() {
        return CreateSiteResponse {
            success: false,
            site: None,
            rules_generated: 0,
            error: Some(format!("Site ID '{site_id}' already exists")),
        };
    }

    let mut domain_map = std::collections::HashMap::new();
    domain_map.insert(DomainCategory::Core, domains);

    let health_check_url = domain_map
        .values()
        .flatten()
        .next()
        .map_or_else(|| format!("https://{site_id}"), |d| format!("https://{d}"));

    let site = SiteDefinition {
        id: site_id.clone(),
        name: display_name.to_string(),
        domains: domain_map,
        health_check: Some(HealthCheckConfig {
            url: health_check_url,
            timeout_secs: 5,
            failure_threshold: 3,
        }),
        access_strategy: AccessStrategy::default(),
    };

    // Save custom definition to disk
    if let Err(e) = state.site_definition_store.save_custom(&site) {
        return CreateSiteResponse {
            success: false,
            site: None,
            rules_generated: 0,
            error: Some(format!("Failed to save: {e}")),
        };
    }

    // Add to engine (generates rules, probes, etc.)
    let mut engine = state.engine.lock().expect("lock");
    let result = engine.add_site(&site_id);

    match result {
        crate::engines::site_rule_engine::AddSiteResult::Success {
            rules_generated,
            verification_passed: _,
            ..
        } => CreateSiteResponse {
            success: true,
            site: Some(SiteDefinitionInfo::from(site)),
            rules_generated,
            error: None,
        },
        crate::engines::site_rule_engine::AddSiteResult::VerificationFailed { .. } => {
            CreateSiteResponse {
                success: false,
                site: Some(SiteDefinitionInfo::from(site)),
                rules_generated: 0,
                error: Some("Verification failed".to_string()),
            }
        }
        crate::engines::site_rule_engine::AddSiteResult::SiteNotFound => {
            CreateSiteResponse {
                success: false,
                site: None,
                rules_generated: 0,
                error: Some("Site not found after save".to_string()),
            }
        }
    }
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn tauri_create_site(
    name: String,
    display_name: String,
    domains: Vec<String>,
    state: tauri::State<'_, SiteRulesState>,
) -> CreateSiteResponse {
    create_custom_site(&name, &display_name, domains, &state)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateSiteDomainsResponse {
    pub success: bool,
    pub site: Option<SiteDefinitionInfo>,
    pub rules_generated: usize,
    pub error: Option<String>,
}

/// Updates a custom site's domains (add/remove) and regenerates rules.
fn update_custom_site_domains(
    site_id: &str,
    add_domains: &[String],
    remove_domains: &[String],
    state: &SiteRulesState,
) -> UpdateSiteDomainsResponse {
    use crate::models::site::DomainCategory;

    // Load existing definition (builtin or custom override).
    // After template application, sites are independent instances;
    // editing a builtin site creates a custom override (lazy copy-on-write).
    let Some(mut site) = state.site_definition_store.get(site_id) else {
        return UpdateSiteDomainsResponse {
            success: false,
            site: None,
            rules_generated: 0,
            error: Some(format!("Site '{site_id}' not found")),
        };
    };

    // Add new domains to Core category
    let core = site.domains.entry(DomainCategory::Core).or_default();
    for domain in add_domains {
        if !core.contains(domain) {
            core.push(domain.clone());
        }
    }

    // Remove domains from any category
    for domain in remove_domains {
        for list in site.domains.values_mut() {
            list.retain(|d| d != domain);
        }
    }
    // Clean up empty categories
    site.domains.retain(|_, list| !list.is_empty());

    // Remove from engine first (to clear old rules)
    {
        let mut engine = state.engine.lock().expect("lock");
        engine.remove_site(site_id);
    }

    // Save updated definition
    if let Err(e) = state.site_definition_store.save_custom(&site) {
        return UpdateSiteDomainsResponse {
            success: false,
            site: None,
            rules_generated: 0,
            error: Some(format!("Failed to save: {e}")),
        };
    }

    // Re-add to engine (regenerates rules)
    let mut engine = state.engine.lock().expect("lock");
    let result = engine.add_site(site_id);

    match result {
        crate::engines::site_rule_engine::AddSiteResult::Success {
            rules_generated, ..
        } => UpdateSiteDomainsResponse {
            success: true,
            site: Some(SiteDefinitionInfo::from(site)),
            rules_generated,
            error: None,
        },
        _ => UpdateSiteDomainsResponse {
            success: false,
            site: Some(SiteDefinitionInfo::from(site)),
            rules_generated: 0,
            error: Some("Verification failed after update".to_string()),
        },
    }
}

#[tauri::command]
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn tauri_update_site_domains(
    site_id: String,
    add_domains: Vec<String>,
    remove_domains: Vec<String>,
    state: tauri::State<'_, SiteRulesState>,
) -> UpdateSiteDomainsResponse {
    update_custom_site_domains(&site_id, &add_domains, &remove_domains, &state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Helper to create a shared MihomoManager for tests.
    fn test_mihomo_manager(dir: &std::path::Path) -> Arc<Mutex<crate::managers::mihomo_manager::MihomoManager>> {
        let config = crate::models::config::AppConfig::default_for(dir.to_path_buf());
        Arc::new(Mutex::new(crate::managers::mihomo_manager::MihomoManager::new(config.mihomo)))
    }

    #[test]
    fn site_info_from_definition() {
        let site = crate::models::site::SiteDefinition::github_default();
        let info = SiteInfo::from(site);
        assert_eq!(info.id, "github");
        assert_eq!(info.name, "GitHub");
        assert!(info.domain_count >= 47);
        assert!(!info.domains.is_empty());
        assert!(info.domains.contains_key("core"));
        let core = info.domains.get("core").expect("core domains");
        assert!(core.contains(&"github.com".to_string()));
    }

    #[test]
    fn add_site_response_success() {
        let response = AddSiteResponse {
            success: true,
            site: Some(SiteInfo {
                id: "test".to_string(),
                name: "Test".to_string(),
                domain_count: 5,
                domains: std::collections::HashMap::new(),
            }),
            rules_generated: 10,
            verification_passed: true,
            error: None,
            five_element_prompt: None,
        };
        assert!(response.success);
        assert!(response.site.is_some());
    }

    #[test]
    fn remove_site_response_success() {
        let response = RemoveSiteResponse {
            success: true,
            remaining_sites: 2,
            error: None,
        };
        assert!(response.success);
        assert_eq!(response.remaining_sites, 2);
    }

    #[test]
    fn template_response_counts() {
        let response = TemplateResponse {
            added_count: 5,
            failed_count: 2,
            sites: vec!["a".to_string(), "b".to_string()],
        };
        assert_eq!(response.added_count, 5);
        assert_eq!(response.failed_count, 2);
    }

    #[test]
    fn reachability_response_empty() {
        let response = ReachabilityResponse { sites: vec![] };
        assert!(response.sites.is_empty());
    }

    #[test]
    fn node_pool_status_empty() {
        let status = NodePoolStatus {
            total_nodes: 0,
            available_nodes: 0,
            current_node: None,
            nodes: vec![],
        };
        assert_eq!(status.total_nodes, 0);
        assert!(status.current_node.is_none());
        assert!(status.nodes.is_empty());
    }

    #[test]
    fn node_pool_status_includes_node_details() {
        let status = NodePoolStatus {
            total_nodes: 2,
            available_nodes: 1,
            current_node: Some("node-1".to_string()),
            nodes: vec![
                NodeInfo {
                    name: "node-1".to_string(),
                    protocol: "vless".to_string(),
                    status: "available".to_string(),
                    latency_ms: Some(120),
                    address: "1.2.3.4:443".to_string(),
                },
                NodeInfo {
                    name: "node-2".to_string(),
                    protocol: "vmess".to_string(),
                    status: "unhealthy".to_string(),
                    latency_ms: None,
                    address: "5.6.7.8:443".to_string(),
                },
            ],
        };
        assert_eq!(status.nodes.len(), 2);
        assert_eq!(status.nodes[0].protocol, "vless");
        assert_eq!(status.nodes[1].status, "unhealthy");
        assert_eq!(status.current_node.as_deref(), Some("node-1"));
    }

    #[test]
    fn subscription_response_counts() {
        let response = SubscriptionResponse {
            imported: 10,
            unsupported: 2,
            source_url: "https://example.com".to_string(),
        };
        assert_eq!(response.imported, 10);
        assert_eq!(response.unsupported, 2);
    }

    #[test]
    fn site_rules_state_new() {
        let dir = tempdir().expect("tempdir");
        let state = SiteRulesState::new(dir.path(), test_mihomo_manager(dir.path()));
        assert_eq!(state.engine.lock().unwrap().active_sites_count(), 0);
        assert_eq!(state.node_pool.lock().unwrap().node_count(), 0);
    }

    #[test]
    fn list_site_definitions_returns_all_built_in() {
        let dir = tempdir().expect("tempdir");
        let state = SiteRulesState::new(dir.path(), test_mihomo_manager(dir.path()));
        let definitions: Vec<SiteDefinitionInfo> = state
            .site_definition_store
            .list_all()
            .into_iter()
            .map(SiteDefinitionInfo::from)
            .collect();
        assert_eq!(definitions.len(), 15);
        let github = definitions.iter().find(|d| d.id == "github").expect("github");
        assert_eq!(github.name, "GitHub");
        assert!(github.domain_count >= 47);
        assert!(github.domains.contains_key("core"));
    }

    #[test]
    fn site_definition_info_has_domains() {
        let site = crate::models::site::SiteDefinition::chatgpt_default();
        let info = SiteDefinitionInfo::from(site);
        assert_eq!(info.id, "chatgpt");
        assert!(info.domains.contains_key("core"));
        assert!(info.domains.contains_key("third_party"));
        let core = info.domains.get("core").expect("core");
        assert!(core.contains(&"chatgpt.com".to_string()));
    }

    #[test]
    fn lookup_site_by_exact_domain() {
        let dir = tempdir().expect("tempdir");
        let state = SiteRulesState::new(dir.path(), test_mihomo_manager(dir.path()));
        let domain = crate::services::url_parser::extract_domain("github.com").expect("domain");
        let site = state.site_definition_store.lookup_by_domain(&domain).expect("site");
        let info = SiteDefinitionInfo::from(site);
        assert_eq!(info.id, "github");
        assert!(info.domain_count >= 47);
    }

    #[test]
    fn lookup_site_by_url() {
        let dir = tempdir().expect("tempdir");
        let state = SiteRulesState::new(dir.path(), test_mihomo_manager(dir.path()));
        let domain = crate::services::url_parser::extract_domain("https://www.google.com/search?q=test").expect("domain");
        let site = state.site_definition_store.lookup_by_domain(&domain).expect("site");
        assert_eq!(site.id, "google");
    }

    #[test]
    fn lookup_site_returns_none_for_unknown() {
        let dir = tempdir().expect("tempdir");
        let state = SiteRulesState::new(dir.path(), test_mihomo_manager(dir.path()));
        let domain = crate::services::url_parser::extract_domain("https://unknown-site-xyz.com/page").expect("domain");
        let result = state.site_definition_store.lookup_by_domain(&domain);
        assert!(result.is_none());
    }

    #[test]
    fn create_custom_site_saves_and_adds() {
        let dir = tempdir().expect("tempdir");
        let state = SiteRulesState::new(dir.path(), test_mihomo_manager(dir.path()));

        let domains = vec![
            "custom.example.com".to_string(),
            "api.custom.example.com".to_string(),
        ];
        let result = create_custom_site("my-site", "My Site", domains, &state);
        assert!(result.success);
        let site = result.site.expect("site");
        assert_eq!(site.id, "my-site");
        assert_eq!(site.name, "My Site");
        assert_eq!(site.domain_count, 2);

        // Verify persisted
        let loaded = state.site_definition_store.get("my-site").expect("loaded");
        assert_eq!(loaded.name, "My Site");

        // Verify added to engine
        let engine = state.engine.lock().unwrap();
        assert!(engine.active_sites().contains(&"my-site".to_string()));
    }

    #[test]
    fn create_custom_site_rejects_builtin_id() {
        let dir = tempdir().expect("tempdir");
        let state = SiteRulesState::new(dir.path(), test_mihomo_manager(dir.path()));

        let result = create_custom_site("github", "Fake", vec!["evil.com".to_string()], &state);
        assert!(!result.success);
    }

    #[test]
    fn update_custom_site_domains_adds_and_removes() {
        let dir = tempdir().expect("tempdir");
        let state = SiteRulesState::new(dir.path(), test_mihomo_manager(dir.path()));

        // Create a custom site first
        let created = create_custom_site(
            "my-site", "My Site",
            vec!["a.example.com".to_string(), "b.example.com".to_string()],
            &state,
        );
        assert!(created.success);

        // Update: remove b, add c
        let result = update_custom_site_domains(
            "my-site",
            &vec!["c.example.com".to_string()],
            &vec!["b.example.com".to_string()],
            &state,
        );
        assert!(result.success);
        let site = result.site.expect("site");
        let all = site.domains.values().flatten().cloned().collect::<Vec<_>>();
        assert!(all.contains(&"a.example.com".to_string()));
        assert!(all.contains(&"c.example.com".to_string()));
        assert!(!all.contains(&"b.example.com".to_string()));
    }

    #[test]
    fn update_builtin_site_creates_custom_override() {
        let dir = tempdir().expect("tempdir");
        let state = SiteRulesState::new(dir.path(), test_mihomo_manager(dir.path()));

        // Editing a builtin site should succeed by creating a custom override
        let result = update_custom_site_domains(
            "github",
            &vec!["evil.com".to_string()],
            &vec![],
            &state,
        );
        assert!(result.success);
        let site = result.site.expect("site");
        let all: Vec<String> = site.domains.values().flatten().cloned().collect();
        assert!(all.contains(&"evil.com".to_string()));
        // Original builtin domains should also be present
        assert!(all.contains(&"github.com".to_string()));
    }

    /// Minimal valid mihomo config.yaml for integration tests.
    fn minimal_mihomo_config() -> &'static str {
        r#"tcp-concurrent: true
mixed-port: 7890
mode: rule
dns:
  enable: true
  nameserver:
    - 223.5.5.5
rule-providers:
  custom-direct:
    type: file
    behavior: classical
    path: ./ruleset/custom-direct.yaml
proxies:
  - name: SS-Node-A
    type: ss
    server: 1.2.3.4
    port: 443
  - name: "VMESS-Node-B"
    type: vmess
    server: 5.6.7.8
    port: 8080
proxy-groups:
  - name: PROXY
    type: select
    proxies:
      - DIRECT
rules:
  - MATCH,DIRECT
"#
    }

    /// Integration test: verify the full pipeline produces correct per-site config.
    ///
    /// This test catches the class of bug where `MihomoConfigManager::open()` is
    /// never called (or the old `new()` is used without `parse()`), because it
    /// asserts the actual config.yaml content after template application.
    #[test]
    fn apply_template_produces_per_site_proxy_groups() {
        let dir = tempdir().expect("tempdir");
        let config_dir = dir.path().join("data").join("mihomo");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.yaml"), minimal_mihomo_config()).unwrap();

        let state = SiteRulesState::new(dir.path(), test_mihomo_manager(dir.path()));

        // Apply developer template (includes github)
        let template_ids =
            crate::services::site_definition_store::SiteDefinitionStore::developer_template_ids();
        let results = {
            let mut engine = state.engine.lock().expect("lock");
            engine.apply_template(&template_ids)
        };

        // At least one site should succeed
        assert!(
            results.iter().any(|r| matches!(r, crate::engines::site_rule_engine::AddSiteResult::Success { .. })),
            "at least one site should be added successfully"
        );

        // Verify config.yaml was regenerated with per-site proxy groups
        let config_content = std::fs::read_to_string(config_dir.join("config.yaml"))
            .expect("config.yaml should exist after template application");

        // Static sections preserved
        assert!(config_content.contains("tcp-concurrent: true"), "general settings should be preserved");
        assert!(config_content.contains("mixed-port: 7890"), "mixed-port should be preserved");
        assert!(config_content.contains("dns:"), "DNS section should be preserved");

        // Proxy nodes preserved verbatim
        assert!(config_content.contains("server: 1.2.3.4"), "proxy node SS-Node-A should be preserved");
        assert!(config_content.contains("server: 5.6.7.8"), "proxy node VMESS-Node-B should be preserved");

        // Per-site proxy groups generated
        assert!(config_content.contains("name: site-github"), "site-github proxy group should exist");
        assert!(config_content.contains("type: url-test"), "per-site group should be url-test");
        assert!(config_content.contains("url: https://github.com"), "health check URL should be site-specific");

        // Per-site rules generated
        assert!(config_content.contains("RULE-SET,site-github,site-github"), "site-github rule should route to site-github group");

        // Proxy nodes listed in site groups
        assert!(config_content.contains("\"SS-Node-A\""), "proxy nodes should be in site groups");
        assert!(config_content.contains("\"VMESS-Node-B\""), "proxy nodes should be in site groups");

        // Per-site ruleset file created
        assert!(
            config_dir.join("ruleset").join("site-github.yaml").exists(),
            "site-github.yaml ruleset file should be created"
        );
    }
}
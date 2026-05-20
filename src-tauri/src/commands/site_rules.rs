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
}

impl From<crate::models::site::SiteDefinition> for SiteInfo {
    fn from(site: crate::models::site::SiteDefinition) -> Self {
        let domain_count = site.all_domains().len();
        Self {
            id: site.id,
            name: site.name,
            domain_count,
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
pub struct NodePoolStatus {
    pub total_nodes: usize,
    pub available_nodes: usize,
    pub current_node: Option<String>,
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
}

impl SiteRulesState {
    #[must_use]
    pub fn new(data_dir: &Path) -> Self {
        let probe_client = Arc::new(MockProbeClient::new());
        let engine = SiteRuleEngine::new(data_dir, probe_client, None, None);
        let node_pool = NodePool::new(NodePoolConfig::default());
        let subscription_parser = SubscriptionParser::new(
            data_dir.join("config").join("subscription-sources.json"),
        );
        Self {
            engine: Mutex::new(engine),
            node_pool: Mutex::new(node_pool),
            subscription_parser: Mutex::new(subscription_parser),
        }
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
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn apply_rules(
    confirm: bool,
    state: tauri::State<'_, SiteRulesState>,
) -> AddSiteResponse {
    if !confirm {
        return AddSiteResponse {
            success: false,
            site: None,
            rules_generated: 0,
            verification_passed: false,
            error: Some("Requires confirmation".to_string()),
            five_element_prompt: None,
        };
    }
    
    let mut engine = state.engine.lock().expect("lock");
    let reloaded = engine.reload_rules();
    
    AddSiteResponse {
        success: reloaded,
        site: None,
        rules_generated: engine.total_domain_count(),
        verification_passed: reloaded,
        error: if reloaded { None } else { Some("Reload failed".to_string()) },
        five_element_prompt: None,
    }
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
    NodePoolStatus {
        total_nodes: pool.node_count(),
        available_nodes: pool.available_count(),
        current_node: pool.current_node().map(|n| n.name.clone()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn site_info_from_definition() {
        let site = crate::models::site::SiteDefinition::github_default();
        let info = SiteInfo::from(site);
        assert_eq!(info.id, "github");
        assert_eq!(info.name, "GitHub");
        assert!(info.domain_count >= 5);
    }

    #[test]
    fn add_site_response_success() {
        let response = AddSiteResponse {
            success: true,
            site: Some(SiteInfo {
                id: "test".to_string(),
                name: "Test".to_string(),
                domain_count: 5,
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
        };
        assert_eq!(status.total_nodes, 0);
        assert!(status.current_node.is_none());
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
        let state = SiteRulesState::new(dir.path());
        assert_eq!(state.engine.lock().unwrap().active_sites_count(), 0);
        assert_eq!(state.node_pool.lock().unwrap().node_count(), 0);
    }
}
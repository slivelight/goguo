use crate::models::probe::ProbeResult;
use crate::models::site::SiteDefinition;
use crate::services::probe_service::{ProbeClient, ProbeService};
use crate::services::rule_generator::{GeneratedRules, RuleGenerator, RuleStorage};
use crate::services::rule_verifier::{ProbeFailure, RuleVerifier, VerificationConfig, VerificationResult};
use crate::services::site_definition_store::SiteDefinitionStore;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FiveElementPrompt {
    pub reason: String,
    pub attempted_actions: Vec<String>,
    pub attempt_count: u32,
    pub suggested_action: String,
    pub needs_manual_handling: bool,
}

impl FiveElementPrompt {
    #[must_use]
    pub fn new(
        reason: String,
        attempted_actions: Vec<String>,
        attempt_count: u32,
        suggested_action: String,
        needs_manual_handling: bool,
    ) -> Self {
        Self {
            reason,
            attempted_actions,
            attempt_count,
            suggested_action,
            needs_manual_handling,
        }
    }

    #[must_use]
    pub fn unreachable_prompt(site_id: &str, error: &str) -> Self {
        Self::new(
            format!("Site {site_id} is unreachable: {error}"),
            vec!["HTTP HEAD probe".to_string(), "HTTP GET probe".to_string()],
            2,
            "Check network connectivity or try a different proxy node".to_string(),
            false,
        )
    }

    #[must_use]
    pub fn verification_failed_prompt(failures: &[ProbeFailure]) -> Self {
        let failed_sites: Vec<&str> = failures.iter().map(|f| f.site.as_str()).collect();
        Self::new(
            format!("Rule verification failed for: {}", failed_sites.join(", ")),
            vec!["Pre-probe reference sites".to_string(), "Post-probe reference sites".to_string()],
            2,
            "Rules rolled back automatically. Check if target sites affect reference sites".to_string(),
            true,
        )
    }
}

#[derive(Debug, Clone)]
pub enum AddSiteResult {
    Success {
        site: SiteDefinition,
        rules_generated: usize,
        verification_passed: bool,
    },
    VerificationFailed {
        site: SiteDefinition,
        prompt: FiveElementPrompt,
    },
    SiteNotFound,
}

#[derive(Debug, Clone)]
pub enum RemoveSiteResult {
    Success { remaining_sites: usize },
    NotFound,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SiteReachability {
    pub site_id: String,
    pub reachable: bool,
    pub response_time_ms: Option<u64>,
    pub last_probe: Option<ProbeResult>,
}

#[derive(Clone)]
pub struct SiteRuleEngine {
    site_store: SiteDefinitionStore,
    rule_generator: RuleGenerator,
    rule_storage: RuleStorage,
    probe_service: ProbeService,
    verifier: RuleVerifier,
    active_sites: Vec<String>,
}

impl SiteRuleEngine {
    #[must_use]
    pub fn new(
        data_dir: &std::path::Path,
        probe_client: Arc<dyn ProbeClient>,
    ) -> Self {
        let site_store = SiteDefinitionStore::new(data_dir.join("config").join("site-definitions"));
        let rule_generator = RuleGenerator::new();
        let rule_storage = RuleStorage::new(data_dir.join("rules"));
        let probe_service = ProbeService::new(
            crate::models::probe::ProbeConfig::default(),
            probe_client.clone(),
        );
        let rule_storage_for_verifier = RuleStorage::new(data_dir.join("rules"));
        let verifier = RuleVerifier::new(
            probe_client,
            rule_storage_for_verifier,
            VerificationConfig::default(),
        );
        
        Self {
            site_store,
            rule_generator,
            rule_storage,
            probe_service,
            verifier,
            active_sites: vec![],
        }
    }

    pub fn add_site(&mut self, site_id: &str) -> AddSiteResult {
        let site = self.site_store.get(site_id);
        if site.is_none() {
            return AddSiteResult::SiteNotFound;
        }
        
        let site = site.expect("checked");
        
        if !self.active_sites.contains(&site.id) {
            self.active_sites.push(site.id.clone());
        }
        
        let generated = self.generate_rules();
        let rules = generated.rules;
        
        self.probe_service.register_site(site_id, &format!("https://{site_id}.com"));
        
        let verification = self.verifier.verify(&rules);
        
        match verification {
            VerificationResult::Passed => AddSiteResult::Success {
                site,
                rules_generated: rules.len(),
                verification_passed: true,
            },
            VerificationResult::RolledBack(failures) | VerificationResult::ProbeFailed(failures) => {
                AddSiteResult::VerificationFailed {
                    site,
                    prompt: FiveElementPrompt::verification_failed_prompt(&failures),
                }
            }
            VerificationResult::StaticCheckFailed(reason) => AddSiteResult::VerificationFailed {
                site,
                prompt: FiveElementPrompt::new(
                    reason,
                    vec!["Static rule validation".to_string()],
                    1,
                    "Check rule generator output".to_string(),
                    true,
                ),
            },
        }
    }

    pub fn remove_site(&mut self, site_id: &str) -> RemoveSiteResult {
        let idx = self.active_sites.iter().position(|s| s == site_id);
        if idx.is_none() {
            return RemoveSiteResult::NotFound;
        }
        
        self.active_sites.remove(idx.expect("checked"));
        self.probe_service.remove_site(site_id);
        
        let generated = self.generate_rules();
        self.rule_storage.save_current(&generated.rules).expect("save");
        
        RemoveSiteResult::Success {
            remaining_sites: self.active_sites.len(),
        }
    }

    fn generate_rules(&self) -> GeneratedRules {
        let sites: Vec<SiteDefinition> = self
            .active_sites
            .iter()
            .filter_map(|id| self.site_store.get(id))
            .collect();
        
        RuleGenerator::generate(&sites, &[])
    }

    #[must_use]
    pub fn preview_rules(&self) -> Vec<String> {
        let sites: Vec<SiteDefinition> = self
            .active_sites
            .iter()
            .filter_map(|id| self.site_store.get(id))
            .collect();
        
        self.rule_generator.preview(&sites)
    }

    pub fn get_reachability(&mut self) -> Vec<SiteReachability> {
        let results = self.probe_service.probe_all();
        
        results
            .iter()
            .map(|r| SiteReachability {
                site_id: r.site_id.clone(),
                reachable: r.reachable,
                response_time_ms: r.response_time_ms,
                last_probe: Some(r.clone()),
            })
            .collect()
    }

    #[must_use]
    pub fn active_sites(&self) -> &Vec<String> {
        &self.active_sites
    }

    #[must_use]
    pub fn active_sites_count(&self) -> usize {
        self.active_sites.len()
    }

    #[must_use]
    pub fn site_store(&self) -> &SiteDefinitionStore {
        &self.site_store
    }

    pub fn apply_template(&mut self, template_ids: &[String]) -> Vec<AddSiteResult> {
        template_ids
            .iter()
            .map(|id| self.add_site(id))
            .collect()
    }

    pub fn probe_site(&mut self, site_id: &str) -> Option<SiteReachability> {
        let result = self.probe_service.probe_site(site_id)?;
        
        Some(SiteReachability {
            site_id: result.site_id.clone(),
            reachable: result.reachable,
            response_time_ms: result.response_time_ms,
            last_probe: Some(result),
        })
    }

    #[must_use]
    pub fn total_domain_count(&self) -> usize {
        let sites: Vec<SiteDefinition> = self
            .active_sites
            .iter()
            .filter_map(|id| self.site_store.get(id))
            .collect();
        
        RuleGenerator::total_domain_count(&sites)
    }

    pub fn reload_rules(&mut self) -> bool {
        let generated = self.generate_rules();
        let verification = self.verifier.verify(&generated.rules);
        
        matches!(verification, VerificationResult::Passed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::probe_service::MockProbeClient;
    use tempfile::tempdir;

    fn create_test_engine(dir: &std::path::Path) -> SiteRuleEngine {
        let probe_client = Arc::new(MockProbeClient::new());
        SiteRuleEngine::new(dir, probe_client)
    }

    #[test]
    fn five_element_prompt_unreachable() {
        let prompt = FiveElementPrompt::unreachable_prompt("github", "timeout");
        assert!(prompt.reason.contains("github"));
        assert!(!prompt.needs_manual_handling);
    }

    #[test]
    fn five_element_prompt_verification_failed() {
        let failures = vec![ProbeFailure::new("baidu".to_string(), true, false, "timeout".to_string())];
        let prompt = FiveElementPrompt::verification_failed_prompt(&failures);
        assert!(prompt.reason.contains("baidu"));
        assert!(prompt.needs_manual_handling);
    }

    #[test]
    fn site_rule_engine_new_empty() {
        let dir = tempdir().expect("tempdir");
        let engine = create_test_engine(dir.path());
        assert_eq!(engine.active_sites_count(), 0);
    }

    #[test]
    fn add_site_success_builtin() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        let result = engine.add_site("github");
        assert!(matches!(result, AddSiteResult::Success { .. }));
        assert_eq!(engine.active_sites_count(), 1);
    }

    #[test]
    fn add_site_not_found() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        let result = engine.add_site("nonexistent");
        assert!(matches!(result, AddSiteResult::SiteNotFound));
    }

    #[test]
    fn add_site_duplicate_ignored() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        engine.add_site("github");
        engine.add_site("github");
        
        assert_eq!(engine.active_sites_count(), 1);
    }

    #[test]
    fn remove_site_success() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        engine.add_site("github");
        let result = engine.remove_site("github");
        
        assert!(matches!(result, RemoveSiteResult::Success { remaining_sites: 0 }));
        assert_eq!(engine.active_sites_count(), 0);
    }

    #[test]
    fn remove_site_not_found() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        let result = engine.remove_site("nonexistent");
        assert!(matches!(result, RemoveSiteResult::NotFound));
    }

    #[test]
    fn preview_rules_empty() {
        let dir = tempdir().expect("tempdir");
        let engine = create_test_engine(dir.path());
        
        let rules = engine.preview_rules();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0], "MATCH,DIRECT");
    }

    #[test]
    fn preview_rules_with_sites() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        engine.add_site("github");
        let rules = engine.preview_rules();
        
        assert!(rules.len() > 1);
        assert!(rules.iter().any(|r| r.contains("github.com")));
        assert!(rules.iter().any(|r| r.contains("MATCH,DIRECT")));
    }

    #[test]
    fn get_reachability_empty() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        let reachability = engine.get_reachability();
        assert!(reachability.is_empty());
    }

    #[test]
    fn get_reachability_with_sites() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        engine.add_site("github");
        let reachability = engine.get_reachability();
        
        assert!(!reachability.is_empty());
    }

    #[test]
    fn apply_template_developer() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        let template_ids = SiteDefinitionStore::developer_template_ids();
        let results = engine.apply_template(&template_ids);
        
        assert!(results.iter().any(|r| matches!(r, AddSiteResult::Success { .. })));
        assert!(engine.active_sites_count() > 0);
    }

    #[test]
    fn probe_site_existing() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        engine.add_site("github");
        let result = engine.probe_site("github");
        
        assert!(result.is_some());
        let reach = result.expect("found");
        assert!(reach.reachable);
    }

    #[test]
    fn probe_site_nonexistent() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        let result = engine.probe_site("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn total_domain_count_empty() {
        let dir = tempdir().expect("tempdir");
        let engine = create_test_engine(dir.path());
        
        assert_eq!(engine.total_domain_count(), 0);
    }

    #[test]
    fn total_domain_count_with_sites() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        engine.add_site("github");
        let count = engine.total_domain_count();
        
        assert!(count >= 5);
    }

    #[test]
    fn reload_rules_empty() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        assert!(engine.reload_rules());
    }

    #[test]
    fn active_sites_returns_reference() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        
        engine.add_site("github");
        engine.add_site("npmjs");
        
        let sites = engine.active_sites();
        assert_eq!(sites.len(), 2);
        assert!(sites.contains(&"github".to_string()));
        assert!(sites.contains(&"npmjs".to_string()));
    }

    #[test]
    fn site_reachability_fields() {
        let reach = SiteReachability {
            site_id: "test".to_string(),
            reachable: true,
            response_time_ms: Some(100),
            last_probe: None,
        };
        assert_eq!(reach.site_id, "test");
        assert!(reach.reachable);
    }
}
use crate::models::probe::{ProbeMethod, ProbeResult};
use crate::services::probe_service::{ProbeClient, ProbeService};
use crate::services::rule_generator::{Rule, RuleGenerator, RuleStorage};

#[derive(Debug, Clone)]
pub enum VerificationResult {
    Passed,
    StaticCheckFailed(String),
    ProbeFailed(Vec<ProbeFailure>),
    RolledBack(Vec<ProbeFailure>),
}

#[derive(Debug, Clone)]
pub struct ProbeFailure {
    pub site: String,
    pub pre_reachable: bool,
    pub post_reachable: bool,
    pub reason: String,
}

impl ProbeFailure {
    #[must_use]
    pub fn new(site: String, pre_reachable: bool, post_reachable: bool, reason: String) -> Self {
        Self {
            site,
            pre_reachable,
            post_reachable,
            reason,
        }
    }

    #[must_use]
    pub fn is_regression(&self) -> bool {
        self.pre_reachable && !self.post_reachable
    }
}

pub struct VerificationConfig {
    pub reference_sites: Vec<String>,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            reference_sites: vec![
                "https://www.baidu.com".to_string(),
                "https://www.bing.com".to_string(),
            ],
        }
    }
}

pub struct RuleVerifier {
    probe_service: ProbeService,
    rule_storage: RuleStorage,
    config: VerificationConfig,
}

impl RuleVerifier {
    #[must_use]
    pub fn new(
        probe_client: std::sync::Arc<dyn ProbeClient>,
        rule_storage: RuleStorage,
        config: VerificationConfig,
    ) -> Self {
        Self {
            probe_service: ProbeService::new(
                crate::models::probe::ProbeConfig::default(),
                probe_client,
            ),
            rule_storage,
            config,
        }
    }

    pub fn verify(&mut self, rules: &[Rule]) -> VerificationResult {
        if !RuleGenerator::validate_match_direct(rules) {
            return VerificationResult::StaticCheckFailed(
                "Last rule must be MATCH,DIRECT".to_string(),
            );
        }

        if self.config.reference_sites.is_empty() {
            return VerificationResult::Passed;
        }

        let pre_results = self.probe_reference_sites();
        let _pre_failures = self.collect_probe_failures(&pre_results);

        self.rule_storage
            .save_current(rules)
            .expect("save current rules");

        let post_results = self.probe_reference_sites();
        let regressions = self.find_regressions(&pre_results, &post_results);

        if regressions.is_empty() {
            return VerificationResult::Passed;
        }

        let rolled_back = self
            .rule_storage
            .rollback()
            .expect("rollback should succeed");
        
        if rolled_back {
            VerificationResult::RolledBack(regressions)
        } else {
            VerificationResult::ProbeFailed(regressions)
        }
    }

    fn probe_reference_sites(&mut self) -> Vec<ProbeResult> {
        for site in &self.config.reference_sites {
            let site_id = extract_site_id(site);
            self.probe_service.register_site(&site_id, site);
        }
        
        self.probe_service.probe_all()
    }

    fn collect_probe_failures(&self, results: &[ProbeResult]) -> Vec<ProbeFailure> {
        results
            .iter()
            .filter(|r| !r.reachable)
            .map(|r| ProbeFailure::new(r.site_id.clone(), false, false, r.error.clone().unwrap_or_default()))
            .collect()
    }

    fn find_regressions(&self, pre: &[ProbeResult], post: &[ProbeResult]) -> Vec<ProbeFailure> {
        let mut failures = vec![];
        
        for pre_result in pre {
            if !pre_result.reachable {
                continue;
            }
            
            let post_result = post.iter().find(|r| r.site_id == pre_result.site_id);
            
            if let Some(post_r) = post_result {
                if !post_r.reachable {
                    failures.push(ProbeFailure::new(
                        pre_result.site_id.clone(),
                        true,
                        false,
                        post_r.error.clone().unwrap_or_default(),
                    ));
                }
            }
        }
        
        failures
    }

    #[must_use]
    pub const fn config(&self) -> &VerificationConfig {
        &self.config
    }
}

fn extract_site_id(url: &str) -> String {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .replace('/', "_")
        .replace('.', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::probe_service::MockProbeClient;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_verifier(dir: &std::path::Path) -> RuleVerifier {
        let probe_client = std::sync::Arc::new(MockProbeClient::new());
        let storage = RuleStorage::new(dir.join("rules"));
        let config = VerificationConfig::default();
        RuleVerifier::new(probe_client, storage, config)
    }

    #[test]
    fn verification_config_default_has_reference_sites() {
        let config = VerificationConfig::default();
        assert!(!config.reference_sites.is_empty());
        assert!(config.reference_sites.contains(&"https://www.baidu.com".to_string()));
    }

    #[test]
    fn probe_failure_is_regression() {
        let failure = ProbeFailure::new("test".to_string(), true, false, "timeout".to_string());
        assert!(failure.is_regression());
        
        let not_regression = ProbeFailure::new("test".to_string(), false, false, "timeout".to_string());
        assert!(!not_regression.is_regression());
        
        let still_ok = ProbeFailure::new("test".to_string(), true, true, "".to_string());
        assert!(!still_ok.is_regression());
    }

    #[test]
    fn verify_static_check_fails_without_match_direct() {
        let dir = tempdir().expect("tempdir");
        let mut verifier = create_test_verifier(dir.path());
        
        let rules = vec![
            Rule::domain_suffix("github.com".to_string()),
        ];
        
        let result = verifier.verify(&rules);
        assert!(matches!(result, VerificationResult::StaticCheckFailed(_)));
    }

    #[test]
    fn verify_passes_with_valid_rules() {
        let dir = tempdir().expect("tempdir");
        let mut verifier = create_test_verifier(dir.path());
        
        let rules = vec![
            Rule::domain_suffix("github.com".to_string()),
            Rule::match_direct(),
        ];
        
        let result = verifier.verify(&rules);
        assert!(matches!(result, VerificationResult::Passed));
    }

    #[test]
    fn verify_skips_probe_if_empty_reference_sites() {
        let dir = tempdir().expect("tempdir");
        let probe_client = std::sync::Arc::new(MockProbeClient::new());
        let storage = RuleStorage::new(dir.path().join("rules"));
        let config = VerificationConfig {
            reference_sites: vec![],
        };
        let mut verifier = RuleVerifier::new(probe_client, storage, config);
        
        let rules = vec![Rule::match_direct()];
        let result = verifier.verify(&rules);
        assert!(matches!(result, VerificationResult::Passed));
    }

    #[test]
    fn verify_rollback_on_regression() {
        let dir = tempdir().expect("tempdir");
        let mut mock = MockProbeClient::new();
        
        mock.set_result(
            "head:https://www.baidu.com",
            ProbeResult::reachable("baidu".to_string(), ProbeMethod::HttpHead, 100),
        );
        mock.set_result(
            "head:https://www.bing.com",
            ProbeResult::reachable("bing".to_string(), ProbeMethod::HttpHead, 100),
        );
        
        let probe_client = std::sync::Arc::new(mock);
        let storage = RuleStorage::new(dir.path().join("rules"));
        let config = VerificationConfig::default();
        let mut verifier = RuleVerifier::new(probe_client, storage, config);
        
        let old_rules = vec![Rule::match_direct()];
        verifier.rule_storage.save_current(&old_rules).expect("save old");
        verifier.rule_storage.backup_current().expect("backup");
        
        let rules = vec![
            Rule::domain_suffix("github.com".to_string()),
            Rule::match_direct(),
        ];
        
        let result = verifier.verify(&rules);
        assert!(matches!(result, VerificationResult::Passed));
    }

    #[test]
    fn find_regressions_empty_if_all_post_reachable() {
        let dir = tempdir().expect("tempdir");
        let verifier = create_test_verifier(dir.path());
        
        let pre = vec![
            ProbeResult::reachable("a".to_string(), ProbeMethod::HttpHead, 100),
            ProbeResult::reachable("b".to_string(), ProbeMethod::HttpHead, 100),
        ];
        let post = vec![
            ProbeResult::reachable("a".to_string(), ProbeMethod::HttpHead, 100),
            ProbeResult::reachable("b".to_string(), ProbeMethod::HttpHead, 100),
        ];
        
        let regressions = verifier.find_regressions(&pre, &post);
        assert!(regressions.is_empty());
    }

    #[test]
    fn find_regressions_detects_regression() {
        let dir = tempdir().expect("tempdir");
        let verifier = create_test_verifier(dir.path());
        
        let pre = vec![
            ProbeResult::reachable("a".to_string(), ProbeMethod::HttpHead, 100),
            ProbeResult::reachable("b".to_string(), ProbeMethod::HttpHead, 100),
        ];
        let post = vec![
            ProbeResult::reachable("a".to_string(), ProbeMethod::HttpHead, 100),
            ProbeResult::unreachable("b".to_string(), ProbeMethod::HttpGet, "timeout".to_string()),
        ];
        
        let regressions = verifier.find_regressions(&pre, &post);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].site, "b");
        assert!(regressions[0].is_regression());
    }

    #[test]
    fn find_regressions_ignores_pre_unreachable() {
        let dir = tempdir().expect("tempdir");
        let verifier = create_test_verifier(dir.path());
        
        let pre = vec![
            ProbeResult::unreachable("a".to_string(), ProbeMethod::HttpGet, "pre timeout".to_string()),
            ProbeResult::reachable("b".to_string(), ProbeMethod::HttpHead, 100),
        ];
        let post = vec![
            ProbeResult::unreachable("a".to_string(), ProbeMethod::HttpGet, "post timeout".to_string()),
            ProbeResult::reachable("b".to_string(), ProbeMethod::HttpHead, 100),
        ];
        
        let regressions = verifier.find_regressions(&pre, &post);
        assert!(regressions.is_empty());
    }

    #[test]
    fn find_regressions_ignores_both_unreachable() {
        let dir = tempdir().expect("tempdir");
        let verifier = create_test_verifier(dir.path());
        
        let pre = vec![
            ProbeResult::unreachable("a".to_string(), ProbeMethod::HttpGet, "pre".to_string()),
        ];
        let post = vec![
            ProbeResult::unreachable("a".to_string(), ProbeMethod::HttpGet, "post".to_string()),
        ];
        
        let regressions = verifier.find_regressions(&pre, &post);
        assert!(regressions.is_empty());
    }

    #[test]
    fn find_regressions_partial_regression() {
        let dir = tempdir().expect("tempdir");
        let verifier = create_test_verifier(dir.path());
        
        let pre = vec![
            ProbeResult::reachable("a".to_string(), ProbeMethod::HttpHead, 100),
            ProbeResult::reachable("b".to_string(), ProbeMethod::HttpHead, 100),
            ProbeResult::reachable("c".to_string(), ProbeMethod::HttpHead, 100),
        ];
        let post = vec![
            ProbeResult::reachable("a".to_string(), ProbeMethod::HttpHead, 100),
            ProbeResult::unreachable("b".to_string(), ProbeMethod::HttpGet, "err".to_string()),
            ProbeResult::reachable("c".to_string(), ProbeMethod::HttpHead, 100),
        ];
        
        let regressions = verifier.find_regressions(&pre, &post);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].site, "b");
    }

    #[test]
    fn extract_site_id_removes_prefix() {
        assert_eq!(extract_site_id("https://www.baidu.com"), "www_baidu_com");
        assert_eq!(extract_site_id("http://example.com"), "example_com");
        assert_eq!(extract_site_id("raw.com"), "raw_com");
    }
}
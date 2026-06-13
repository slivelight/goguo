use crate::managers::mihomo_manager::MihomoReloader;
use crate::models::audit::AuditAction;
use crate::models::probe::ProbeResult;
use crate::models::site::{AccessStrategy, SiteDefinition};
use crate::services::audit_logger::AuditLog;
use crate::services::ip_cache::IpCache;
use crate::services::ip_scanner::{IpScanner, IpScannerTrait};
use crate::services::probe_service::{ProbeClient, ProbeService};
use crate::services::rule_generator::{GeneratedRules, Rule, RuleGenerator, RuleStorage};
use crate::services::rule_verifier::{ProbeFailure, RuleVerifier, VerificationConfig, VerificationResult};
use crate::services::site_definition_store::SiteDefinitionStore;
use std::collections::HashMap;
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
    pub const fn new(
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
    rule_storage: RuleStorage,
    probe_service: ProbeService,
    verifier: RuleVerifier,
    active_sites: Vec<String>,
    user_overrides: Vec<Rule>,
    mihomo_reloader: Option<Arc<dyn MihomoReloader>>,
    audit_logger: Option<Arc<dyn AuditLog>>,
    ruleset_writer: Option<crate::services::ruleset_writer::RulesetWriter>,
    active_sites_file: std::path::PathBuf,
    config_manager: Option<Arc<std::sync::Mutex<crate::services::mihomo_config_manager::MihomoConfigManager>>>,
    ip_scanner: Arc<dyn IpScannerTrait>,
    ip_cache: IpCache,
    ip_cache_file: std::path::PathBuf,
}

impl SiteRuleEngine {
    #[must_use]
    pub fn new(
        data_dir: &std::path::Path,
        probe_client: Arc<dyn ProbeClient>,
        mihomo_reloader: Option<Arc<dyn MihomoReloader>>,
        audit_logger: Option<Arc<dyn AuditLog>>,
    ) -> Self {
        Self::new_with_ruleset_writer(data_dir, probe_client, mihomo_reloader, audit_logger, None)
    }

    #[must_use]
    pub fn new_with_ruleset_writer(
        data_dir: &std::path::Path,
        probe_client: Arc<dyn ProbeClient>,
        mihomo_reloader: Option<Arc<dyn MihomoReloader>>,
        audit_logger: Option<Arc<dyn AuditLog>>,
        ruleset_writer: Option<crate::services::ruleset_writer::RulesetWriter>,
    ) -> Self {
        Self::new_with_config_manager(
            data_dir, probe_client, mihomo_reloader, audit_logger, ruleset_writer, None,
        )
    }

    #[must_use]
    pub fn new_with_config_manager(
        data_dir: &std::path::Path,
        probe_client: Arc<dyn ProbeClient>,
        mihomo_reloader: Option<Arc<dyn MihomoReloader>>,
        audit_logger: Option<Arc<dyn AuditLog>>,
        ruleset_writer: Option<crate::services::ruleset_writer::RulesetWriter>,
        config_manager: Option<crate::services::mihomo_config_manager::MihomoConfigManager>,
    ) -> Self {
        Self::new_with_scanner(
            data_dir,
            probe_client,
            mihomo_reloader,
            audit_logger,
            ruleset_writer,
            config_manager,
            Arc::new(IpScanner::new()),
        )
    }

    #[must_use]
    pub fn new_with_scanner(
        data_dir: &std::path::Path,
        probe_client: Arc<dyn ProbeClient>,
        mihomo_reloader: Option<Arc<dyn MihomoReloader>>,
        audit_logger: Option<Arc<dyn AuditLog>>,
        ruleset_writer: Option<crate::services::ruleset_writer::RulesetWriter>,
        config_manager: Option<crate::services::mihomo_config_manager::MihomoConfigManager>,
        ip_scanner: Arc<dyn IpScannerTrait>,
    ) -> Self {
        let site_store = SiteDefinitionStore::new(data_dir.join("config").join("site-definitions"));
        let rule_storage = RuleStorage::new(data_dir.join("rules"));
        let mut probe_service = ProbeService::new(
            crate::models::probe::ProbeConfig::default(),
            probe_client.clone(),
        );
        let rule_storage_for_verifier = RuleStorage::new(data_dir.join("rules"));
        let verifier = RuleVerifier::new(
            probe_client,
            rule_storage_for_verifier,
            VerificationConfig::default(),
        );

        let active_sites_file = data_dir.join("config").join("active-sites.json");
        let active_sites = Self::load_active_sites(&active_sites_file);

        let ip_cache_file = data_dir.join("config").join("ip-cache.json");
        let ip_cache = IpCache::load(&ip_cache_file);

        // Register persisted sites with ProbeService
        for site_id in &active_sites {
            if let Some(site) = site_store.get(site_id) {
                let probe_url = site.health_check
                    .as_ref()
                    .map_or_else(
                        || site.all_domains().first().cloned().unwrap_or_default(),
                        |hc| hc.url.clone(),
                    );
                probe_service.register_site(site_id, &probe_url);
            }
        }

        let mut engine = Self {
            site_store,
            rule_storage,
            probe_service,
            verifier,
            active_sites,
            user_overrides: vec![],
            mihomo_reloader,
            audit_logger,
            ruleset_writer,
            active_sites_file,
            config_manager: config_manager.map(|cm| Arc::new(std::sync::Mutex::new(cm))),
            ip_scanner,
            ip_cache,
            ip_cache_file,
        };

        engine.restore_on_startup();
        engine
    }

    /// On startup, regenerate `config.yaml` and reload mihomo to match persisted state.
    fn restore_on_startup(&mut self) {
        if self.active_sites.is_empty() {
            return;
        }

        let site_info = self.collect_active_site_info();
        let (ip_hosts, direct_domains) = self.scan_ip_direct_sites();

        if let Some(ref cm) = self.config_manager {
            if let Ok(cm_lock) = cm.lock() {
                if let Err(e) = cm_lock.regenerate(&site_info, &ip_hosts, &direct_domains) {
                    eprintln!("Warning: startup config restore failed: {e}");
                    return;
                }
            }
        }
        if let Some(ref reloader) = self.mihomo_reloader {
            if let Err(e) = reloader.reload_config("") {
                eprintln!("Warning: startup mihomo reload failed: {e}");
            }
        }
    }

    /// Scan IPs for all `IpDirect` active sites, using cache when available.
    /// Returns (`hosts_mapping`, `direct_domain_names`).
    fn scan_ip_direct_sites(&mut self) -> (HashMap<String, String>, Vec<String>) {
        let mut ip_hosts = HashMap::new();
        let mut direct_domains = Vec::new();

        // Collect all IpDirect domains
        let mut ip_direct_domains: Vec<String> = Vec::new();
        for site_id in &self.active_sites {
            if let Some(site) = self.site_store.get(site_id) {
                if site.access_strategy == AccessStrategy::IpDirect {
                    ip_direct_domains.extend(site.all_domains());
                }
            }
        }

        if ip_direct_domains.is_empty() {
            return (ip_hosts, direct_domains);
        }

        // Trigger scan only if cache is empty or expired
        if self.ip_cache.needs_scan(&ip_direct_domains) {
            let fresh = self.ip_scanner.scan_domains(&ip_direct_domains);
            for (domain, ip) in &fresh {
                self.ip_cache.update(domain.clone(), ip.clone());
            }
            let _ = self.ip_cache.save(&self.ip_cache_file);
        }

        // Build hosts + direct_domains from cache
        let cached = self.ip_cache.get_all_valid();
        for domain in &ip_direct_domains {
            if let Some(ip) = cached.get(domain) {
                direct_domains.push(domain.clone());
                let hosts_key = if IpScanner::is_parent_match(domain) {
                    format!("+.{domain}")
                } else {
                    domain.clone()
                };
                ip_hosts.insert(hosts_key, ip.clone());
            }
        }

        (ip_hosts, direct_domains)
    }

    /// # Panics
    ///
    /// Panics if rule storage save fails after site removal.
    pub fn add_site(&mut self, site_id: &str) -> AddSiteResult {
        let site = self.site_store.get(site_id);
        if site.is_none() {
            return AddSiteResult::SiteNotFound;
        }
        
        let site = site.expect("checked");
        
        if !self.active_sites.contains(&site.id) {
            self.active_sites.push(site.id.clone());
            self.persist_active_sites();
        }
        
        let generated = self.generate_rules();
        let rules = generated.rules;
        
        let probe_url = site.health_check
            .as_ref()
            .map_or_else(
                || site.all_domains().first().cloned().unwrap_or_default(),
                |hc| hc.url.clone(),
            );
        self.probe_service.register_site(site_id, &probe_url);
        
        let verification = self.verifier.verify(&rules);
        
        match verification {
            VerificationResult::Passed => {
                self.apply_rules_to_mihomo(&rules);
                self.log_audit_success(AuditAction::SiteAdd, site_id);
                AddSiteResult::Success {
                    site,
                    rules_generated: rules.len(),
                    verification_passed: true,
                }
            }
            VerificationResult::RolledBack(failures) | VerificationResult::ProbeFailed(failures) => {
                self.log_audit_failure(AuditAction::SiteAdd, site_id, "Verification failed");
                AddSiteResult::VerificationFailed {
                    site,
                    prompt: FiveElementPrompt::verification_failed_prompt(&failures),
                }
            }
            VerificationResult::StaticCheckFailed(reason) => {
                self.log_audit_failure(AuditAction::SiteAdd, site_id, "Static check failed");
                AddSiteResult::VerificationFailed {
                    site,
                    prompt: FiveElementPrompt::new(
                        reason,
                        vec!["Static rule validation".to_string()],
                        1,
                        "Check rule generator output".to_string(),
                        true,
                    ),
                }
            }
        }
    }

    /// # Panics
    ///
    /// Panics if rule storage save fails.
    pub fn remove_site(&mut self, site_id: &str) -> RemoveSiteResult {
        let idx = self.active_sites.iter().position(|s| s == site_id);
        if idx.is_none() {
            return RemoveSiteResult::NotFound;
        }
        
        self.active_sites.remove(idx.expect("checked"));
        self.persist_active_sites();
        self.probe_service.remove_site(site_id);
        
        let generated = self.generate_rules();
        self.rule_storage.save_current(&generated.rules).expect("save");
        self.apply_rules_to_mihomo(&generated.rules);
        self.log_audit_success(AuditAction::SiteRemove, site_id);

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

        RuleGenerator::generate(&sites, &self.user_overrides)
    }

    fn apply_rules_to_mihomo(&mut self, rules: &[Rule]) {
        // Step 1: Save to internal storage (audit/rollback)
        if let Err(e) = self.rule_storage.save_current(rules) {
            eprintln!("Warning: failed to save rules: {e}");
            return;
        }

        // Step 2: Scan IPs for IpDirect sites
        let (ip_hosts, direct_domains) = self.scan_ip_direct_sites();
        let direct_domain_set: std::collections::HashSet<&String> = direct_domains.iter().collect();

        // Step 3: Write per-site ruleset files (filtered by site domains + strategy)
        if let Some(ref writer) = self.ruleset_writer {
            for site_id in &self.active_sites {
                if let Some(site) = self.site_store.get(site_id) {
                    let site_domains = site.all_domains();
                    let is_ip_direct = site.access_strategy == AccessStrategy::IpDirect;

                    let site_rules: Vec<Rule> = rules
                        .iter()
                        .filter(|r| {
                            if r.rule_type == "MATCH" {
                                return false;
                            }
                            if !site_domains.contains(&r.domain) {
                                return false;
                            }
                            // For IpDirect sites: only include domains that DON'T have
                            // verified IPs (they'll use inline DIRECT rules instead)
                            if is_ip_direct && direct_domain_set.contains(&r.domain) {
                                return false;
                            }
                            true
                        })
                        .cloned()
                        .collect();
                    if let Err(e) = writer.write_site_ruleset(site_id, &site_rules) {
                        eprintln!("Warning: failed to write ruleset for {site_id}: {e}");
                    }
                }
            }
            // Cleanup stale ruleset files
            if let Err(e) = writer.cleanup_site_rulesets(&self.active_sites) {
                eprintln!("Warning: failed to cleanup rulesets: {e}");
            }
        }

        // Step 4: Update config.yaml with per-site proxy groups + hosts + DIRECT rules
        if let Some(ref cm) = self.config_manager {
            let site_info = self.collect_active_site_info();
            if let Ok(cm_lock) = cm.lock() {
                if let Err(e) = cm_lock.regenerate(&site_info, &ip_hosts, &direct_domains) {
                    eprintln!("Warning: failed to regenerate config: {e}");
                }
            }
        }

        // Step 4: Trigger mihomo reload
        if let Some(ref reloader) = self.mihomo_reloader {
            if let Err(e) = reloader.reload_config("") {
                eprintln!("Warning: mihomo reload failed: {e}");
            }
        }
    }

    /// Collect `(site_id, health_check_url)` for all active sites.
    fn collect_active_site_info(&self) -> Vec<(String, String)> {
        self.active_sites
            .iter()
            .filter_map(|id| {
                let site = self.site_store.get(id)?;
                let url = site
                    .health_check
                    .as_ref()
                    .map_or_else(
                        || {
                            site.all_domains()
                                .first()
                                .cloned()
                                .unwrap_or_else(|| format!("https://{id}"))
                        },
                        |hc| hc.url.clone(),
                    );
                Some((site.id, url))
            })
            .collect()
    }

    fn log_audit_success(&self, action: AuditAction, target: &str) {
        if let Some(ref logger) = self.audit_logger {
            let _ = logger.log_success(action, target, serde_json::json!({}));
        }
    }

    fn log_audit_failure(&self, action: AuditAction, target: &str, reason: &str) {
        if let Some(ref logger) = self.audit_logger {
            let _ = logger.log_failure(action, target, reason, serde_json::json!({}));
        }
    }

    #[must_use]
    pub fn preview_rules(&self) -> Vec<String> {
        let mut generated = self.generate_rules();

        // Collect domains from IpDirect sites — they should display as DIRECT
        let ip_direct_domains: std::collections::HashSet<String> = self
            .active_sites
            .iter()
            .filter_map(|id| self.site_store.get(id))
            .filter(|site| site.access_strategy == AccessStrategy::IpDirect)
            .flat_map(|site| site.all_domains())
            .collect();

        for rule in &mut generated.rules {
            if ip_direct_domains.contains(&rule.domain) {
                rule.policy = "DIRECT".to_string();
            }
        }

        generated.rules.iter().map(Rule::to_mihomo_line).collect()
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
    pub const fn active_sites(&self) -> &Vec<String> {
        &self.active_sites
    }

    #[must_use]
    pub const fn active_sites_count(&self) -> usize {
        self.active_sites.len()
    }

    /// Load `active_sites` from JSON file. Returns empty vec if file missing or invalid.
    fn load_active_sites(path: &std::path::Path) -> Vec<String> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Persist current `active_sites` to disk (best-effort).
    fn persist_active_sites(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.active_sites) {
            if let Some(parent) = self.active_sites_file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&self.active_sites_file, json);
        }
    }

    #[must_use]
    pub const fn site_store(&self) -> &SiteDefinitionStore {
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

    pub fn add_user_override(&mut self, rule: Rule) {
        let domain = rule.domain.clone();
        let rule_type = rule.rule_type.clone();
        if !self.user_overrides.iter().any(|r| r.domain == domain && r.rule_type == rule_type) {
            self.user_overrides.push(rule);
        }
    }

    #[must_use]
    pub const fn user_overrides_count(&self) -> usize {
        self.user_overrides.len()
    }

    /// Background refresh: scan IPs and update cache without triggering mihomo reload.
    pub fn refresh_ip_cache(&mut self) {
        let mut ip_direct_domains: Vec<String> = Vec::new();
        for site_id in &self.active_sites {
            if let Some(site) = self.site_store.get(site_id) {
                if site.access_strategy == AccessStrategy::IpDirect {
                    ip_direct_domains.extend(site.all_domains());
                }
            }
        }

        if ip_direct_domains.is_empty() {
            return;
        }

        let fresh = self.ip_scanner.scan_domains(&ip_direct_domains);
        for (domain, ip) in &fresh {
            self.ip_cache.update(domain.clone(), ip.clone());
        }
        let _ = self.ip_cache.save(&self.ip_cache_file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ip_scanner::MockIpScanner;
    use crate::services::probe_service::MockProbeClient;
    use crate::services::rule_generator::Rule;
    use tempfile::tempdir;

    fn create_test_engine(dir: &std::path::Path) -> SiteRuleEngine {
        let probe_client = Arc::new(MockProbeClient::new());
        let mock_scanner = Arc::new(MockIpScanner::new(HashMap::new()));
        SiteRuleEngine::new_with_scanner(dir, probe_client, None, None, None, None, mock_scanner)
    }

    fn create_test_engine_with_reloader(dir: &std::path::Path, reloader: Option<Arc<dyn MihomoReloader>>) -> SiteRuleEngine {
        let probe_client = Arc::new(MockProbeClient::new());
        let mock_scanner = Arc::new(MockIpScanner::new(HashMap::new()));
        SiteRuleEngine::new_with_scanner(dir, probe_client, reloader, None, None, None, mock_scanner)
    }

    fn create_test_engine_with_audit(dir: &std::path::Path, audit: Option<Arc<dyn AuditLog>>) -> SiteRuleEngine {
        let probe_client = Arc::new(MockProbeClient::new());
        let mock_scanner = Arc::new(MockIpScanner::new(HashMap::new()));
        SiteRuleEngine::new_with_scanner(dir, probe_client, None, audit, None, None, mock_scanner)
    }

    fn create_test_engine_with_probe(dir: &std::path::Path, probe_client: Arc<dyn ProbeClient>) -> SiteRuleEngine {
        let mock_scanner = Arc::new(MockIpScanner::new(HashMap::new()));
        SiteRuleEngine::new_with_scanner(dir, probe_client, None, None, None, None, mock_scanner)
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
    fn refresh_ip_cache_empty() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());

        engine.refresh_ip_cache();
        // No panic = success for empty engine
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

    #[test]
    fn add_site_uses_health_check_url_not_hardcoded() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc as StdArc;

        #[derive(Debug)]
        struct UrlTrackingClient {
            npmjs_probed: StdArc<AtomicUsize>,
        }

        impl ProbeClient for UrlTrackingClient {
            fn probe_http_head(&self, url: &str, _timeout: std::time::Duration) -> ProbeResult {
                if url == "https://www.npmjs.com" {
                    self.npmjs_probed.fetch_add(1, Ordering::SeqCst);
                }
                ProbeResult::reachable("test".to_string(), crate::models::probe::ProbeMethod::HttpHead, 100)
            }
            fn probe_http_get(&self, url: &str, _timeout: std::time::Duration) -> ProbeResult {
                if url == "https://www.npmjs.com" {
                    self.npmjs_probed.fetch_add(1, Ordering::SeqCst);
                }
                ProbeResult::reachable("test".to_string(), crate::models::probe::ProbeMethod::HttpGet, 100)
            }
            fn probe_dns(&self, _domain: &str, _timeout: std::time::Duration) -> ProbeResult {
                ProbeResult::reachable("test".to_string(), crate::models::probe::ProbeMethod::DnsResolve, 50)
            }
            fn probe_tls(&self, _domain: &str, _timeout: std::time::Duration) -> ProbeResult {
                ProbeResult::reachable("test".to_string(), crate::models::probe::ProbeMethod::TlsHandshake, 150)
            }
        }

        let dir = tempdir().expect("tempdir");
        let npmjs_probed = StdArc::new(AtomicUsize::new(0));
        let npmjs_probed_clone = npmjs_probed.clone();

        let client = UrlTrackingClient {
            npmjs_probed: npmjs_probed_clone,
        };
        let probe_client = Arc::new(client);
        let mut engine = create_test_engine_with_probe(dir.path(), probe_client);

        let _result = engine.add_site("npmjs");
        let _reach = engine.probe_site("npmjs");

        assert!(npmjs_probed.load(Ordering::SeqCst) > 0,
            "engine should probe health_check URL (https://www.npmjs.com), not hardcoded (https://npmjs.com)");
    }

    #[test]
    fn user_overrides_appear_in_generated_rules() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());

        engine.add_site("github");

        let override_rule = Rule::domain_exact("custom.example.com".to_string());
        engine.add_user_override(override_rule);

        let rules = engine.preview_rules();
        assert!(rules.iter().any(|r| r.contains("custom.example.com")),
            "user override should appear in generated rules");
    }

    #[test]
    fn user_overrides_deduplicates() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());

        engine.add_user_override(Rule::domain_exact("dup.com".to_string()));
        engine.add_user_override(Rule::domain_exact("dup.com".to_string()));

        assert_eq!(engine.user_overrides_count(), 1, "duplicate overrides should be deduplicated");
    }

    #[test]
    fn add_site_calls_mihomo_reload_on_success() {
        let dir = tempdir().expect("tempdir");
        let reloader = Arc::new(crate::managers::mihomo_manager::MockMihomoReloader::new());
        let reloader_ref = reloader.clone();

        let mut engine = create_test_engine_with_reloader(dir.path(), Some(reloader));

        let result = engine.add_site("github");
        assert!(matches!(result, AddSiteResult::Success { .. }));
        assert!(reloader_ref.was_called(), "mihomo reload should be called after successful add");
    }

    #[test]
    fn add_site_no_reload_on_verification_failure() {
        let dir = tempdir().expect("tempdir");
        let engine = create_test_engine(dir.path());
        // Engine works fine without mihomo reloader
        assert_eq!(engine.active_sites_count(), 0);
    }

    #[test]
    fn engine_works_without_mihomo_reloader() {
        let dir = tempdir().expect("tempdir");
        let mut engine = create_test_engine(dir.path());
        let result = engine.add_site("github");
        assert!(matches!(result, AddSiteResult::Success { .. }));
    }

    #[test]
    fn add_site_logs_audit_success() {
        let dir = tempdir().expect("tempdir");
        let audit = Arc::new(crate::services::audit_logger::MockAuditLog::new());
        let audit_ref = audit.clone();

        let mut engine = create_test_engine_with_audit(dir.path(), Some(audit));

        let result = engine.add_site("github");
        assert!(matches!(result, AddSiteResult::Success { .. }));

        let records = audit_ref.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].action, crate::models::audit::AuditAction::SiteAdd);
        assert_eq!(records[0].target, "github");
    }

    #[test]
    fn remove_site_logs_audit() {
        let dir = tempdir().expect("tempdir");
        let audit = Arc::new(crate::services::audit_logger::MockAuditLog::new());
        let audit_ref = audit.clone();

        let mut engine = create_test_engine_with_audit(dir.path(), Some(audit));
        engine.add_site("github");

        let result = engine.remove_site("github");
        assert!(matches!(result, RemoveSiteResult::Success { .. }));

        let records = audit_ref.records();
        let remove_records: Vec<_> = records.iter()
            .filter(|r| r.action == crate::models::audit::AuditAction::SiteRemove)
            .collect();
        assert_eq!(remove_records.len(), 1);
        assert_eq!(remove_records[0].target, "github");
    }
}
use crate::models::probe::{ProbeConfig, ProbeHistory, ProbeMethod, ProbeResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub trait ProbeClient: Send + Sync {
    fn probe_http_head(&self, url: &str, timeout: Duration) -> ProbeResult;
    fn probe_http_get(&self, url: &str, timeout: Duration) -> ProbeResult;
    fn probe_dns(&self, domain: &str, timeout: Duration) -> ProbeResult;
    fn probe_tls(&self, domain: &str, timeout: Duration) -> ProbeResult;
}

pub struct MockProbeClient {
    results: HashMap<String, ProbeResult>,
}

impl MockProbeClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    pub fn set_result(&mut self, key: &str, result: ProbeResult) {
        self.results.insert(key.to_string(), result);
    }

    #[must_use]
    pub fn get_result(&self, key: &str) -> Option<&ProbeResult> {
        self.results.get(key)
    }
}

impl Default for MockProbeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ProbeClient for MockProbeClient {
    fn probe_http_head(&self, url: &str, _timeout: Duration) -> ProbeResult {
        self.get_result(&format!("head:{url}"))
            .cloned()
            .unwrap_or_else(|| {
                ProbeResult::reachable(
                    url.to_string(),
                    ProbeMethod::HttpHead,
                    100,
                )
            })
    }

    fn probe_http_get(&self, url: &str, _timeout: Duration) -> ProbeResult {
        self.get_result(&format!("get:{url}"))
            .cloned()
            .unwrap_or_else(|| {
                ProbeResult::reachable(
                    url.to_string(),
                    ProbeMethod::HttpGet,
                    200,
                )
            })
    }

    fn probe_dns(&self, domain: &str, _timeout: Duration) -> ProbeResult {
        self.get_result(&format!("dns:{domain}"))
            .cloned()
            .unwrap_or_else(|| {
                ProbeResult::reachable(
                    domain.to_string(),
                    ProbeMethod::DnsResolve,
                    50,
                )
            })
    }

    fn probe_tls(&self, domain: &str, _timeout: Duration) -> ProbeResult {
        self.get_result(&format!("tls:{domain}"))
            .cloned()
            .unwrap_or_else(|| {
                ProbeResult::reachable(
                    domain.to_string(),
                    ProbeMethod::TlsHandshake,
                    150,
                )
            })
    }
}

#[derive(Clone)]
pub struct ProbeService {
    config: ProbeConfig,
    history: ProbeHistory,
    client: Arc<dyn ProbeClient>,
    site_urls: HashMap<String, String>,
}

impl ProbeService {
    #[must_use]
    pub fn new(config: ProbeConfig, client: Arc<dyn ProbeClient>) -> Self {
        Self {
            config,
            history: ProbeHistory::new(config.history_max_size),
            client,
            site_urls: HashMap::new(),
        }
    }

    pub fn register_site(&mut self, site_id: &str, url: &str) {
        self.site_urls.insert(site_id.to_string(), url.to_string());
    }

    pub fn remove_site(&mut self, site_id: &str) {
        self.site_urls.remove(site_id);
    }

    #[must_use]
    pub fn registered_sites(&self) -> Vec<String> {
        self.site_urls.keys().cloned().collect()
    }

    pub fn probe_site(&mut self, site_id: &str) -> Option<ProbeResult> {
        let url = self.site_urls.get(site_id)?;
        let timeout = self.config.timeout();

        let level1 = self.client.probe_http_head(url, timeout);
        self.history.push(level1.clone());

        if level1.reachable {
            return Some(level1);
        }

        let level2 = self.client.probe_http_get(url, timeout);
        self.history.push(level2.clone());

        if level2.reachable {
            return Some(level2);
        }

        let domain = extract_domain(url);
        let level3 = self.client.probe_tls(domain, timeout);
        self.history.push(level3.clone());

        Some(level3)
    }

    pub fn probe_all(&mut self) -> Vec<ProbeResult> {
        let site_ids: Vec<String> = self.site_urls.keys().cloned().collect();
        site_ids
            .iter()
            .filter_map(|id| self.probe_site(id))
            .collect()
    }

    #[must_use]
    pub const fn history(&self) -> &ProbeHistory {
        &self.history
    }

    #[must_use]
    pub const fn config(&self) -> &ProbeConfig {
        &self.config
    }

    #[must_use]
    pub fn should_probe(&self, site_id: &str) -> bool {
        let failures = self.history.consecutive_failures(site_id);
        if failures >= self.config.failure_threshold {
            return false;
        }
        true
    }

    #[must_use]
    pub fn probe_interval_for_site(&self, site_id: &str) -> Duration {
        let failures = self.history.consecutive_failures(site_id);
        if failures >= self.config.failure_threshold {
            self.config.degraded_interval()
        } else {
            self.config.interval()
        }
    }
}

fn extract_domain(url: &str) -> &str {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    
    without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .split(':')
        .next()
        .unwrap_or(without_scheme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_probe_client_default_returns_reachable() {
        let client = MockProbeClient::new();
        let result = client.probe_http_head("https://github.com", Duration::from_secs(3));
        assert!(result.reachable);
    }

    #[test]
    fn mock_probe_client_set_result_override() {
        let mut client = MockProbeClient::new();
        client.set_result(
            "head:https://github.com",
            ProbeResult::unreachable("github".to_string(), ProbeMethod::HttpHead, "timeout".to_string()),
        );
        
        let result = client.probe_http_head("https://github.com", Duration::from_secs(3));
        assert!(!result.reachable);
        assert_eq!(result.error, Some("timeout".to_string()));
    }

    #[test]
    fn probe_service_new_empty() {
        let config = ProbeConfig::default();
        let client = Arc::new(MockProbeClient::new());
        let service = ProbeService::new(config, client);
        
        assert!(service.registered_sites().is_empty());
        assert!(service.history().is_empty());
    }

    #[test]
    fn probe_service_register_site() {
        let config = ProbeConfig::default();
        let client = Arc::new(MockProbeClient::new());
        let mut service = ProbeService::new(config, client);
        
        service.register_site("github", "https://github.com");
        
        let sites = service.registered_sites();
        assert_eq!(sites.len(), 1);
        assert!(sites.contains(&"github".to_string()));
    }

    #[test]
    fn probe_service_remove_site() {
        let config = ProbeConfig::default();
        let client = Arc::new(MockProbeClient::new());
        let mut service = ProbeService::new(config, client);
        
        service.register_site("github", "https://github.com");
        service.remove_site("github");
        
        assert!(service.registered_sites().is_empty());
    }

    #[test]
    fn probe_service_probe_site_returns_result() {
        let config = ProbeConfig::default();
        let client = Arc::new(MockProbeClient::new());
        let mut service = ProbeService::new(config, client);
        
        service.register_site("github", "https://github.com");
        let result = service.probe_site("github").expect("result");
        
        assert!(result.reachable);
        assert_eq!(service.history().len(), 1);
    }

    #[test]
    fn probe_service_probe_site_falls_back_levels() {
        let config = ProbeConfig::default();
        let mut mock = MockProbeClient::new();
        
        mock.set_result(
            "head:https://github.com",
            ProbeResult::unreachable("github".to_string(), ProbeMethod::HttpHead, "L1 failed".to_string()),
        );
        mock.set_result(
            "get:https://github.com",
            ProbeResult::reachable("github".to_string(), ProbeMethod::HttpGet, 200),
        );
        
        let client = Arc::new(mock);
        let mut service = ProbeService::new(config, client);
        
        service.register_site("github", "https://github.com");
        let result = service.probe_site("github").expect("result");
        
        assert!(result.reachable);
        assert_eq!(result.probe_method, ProbeMethod::HttpGet);
        assert_eq!(service.history().len(), 2);
    }

    #[test]
    fn probe_service_probe_site_level3_on_total_failure() {
        let config = ProbeConfig::default();
        let mut mock = MockProbeClient::new();
        
        mock.set_result(
            "head:https://github.com",
            ProbeResult::unreachable("github".to_string(), ProbeMethod::HttpHead, "L1".to_string()),
        );
        mock.set_result(
            "get:https://github.com",
            ProbeResult::unreachable("github".to_string(), ProbeMethod::HttpGet, "L2".to_string()),
        );
        mock.set_result(
            "tls:github.com",
            ProbeResult::unreachable("github".to_string(), ProbeMethod::TlsHandshake, "TLS failed".to_string()),
        );
        
        let client = Arc::new(mock);
        let mut service = ProbeService::new(config, client);
        
        service.register_site("github", "https://github.com");
        let result = service.probe_site("github").expect("result");
        
        assert!(!result.reachable);
        assert_eq!(result.probe_method, ProbeMethod::TlsHandshake);
        assert_eq!(service.history().len(), 3);
    }

    #[test]
    fn probe_service_probe_all_multiple_sites() {
        let config = ProbeConfig::default();
        let client = Arc::new(MockProbeClient::new());
        let mut service = ProbeService::new(config, client);
        
        service.register_site("github", "https://github.com");
        service.register_site("npmjs", "https://npmjs.com");
        
        let results = service.probe_all();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.reachable));
    }

    #[test]
    fn probe_service_should_probe_below_threshold() {
        let config = ProbeConfig {
            failure_threshold: 10,
            ..ProbeConfig::default()
        };
        let mut mock = MockProbeClient::new();
        mock.set_result(
            "head:https://test.com",
            ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpHead, "err".to_string()),
        );
        mock.set_result(
            "get:https://test.com",
            ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpGet, "err".to_string()),
        );
        mock.set_result(
            "tls:test.com",
            ProbeResult::unreachable("test".to_string(), ProbeMethod::TlsHandshake, "err".to_string()),
        );
        
        let client = Arc::new(mock);
        let mut service = ProbeService::new(config, client);
        
        service.register_site("test", "https://test.com");
        service.probe_site("test");
        
        assert!(service.should_probe("test"));
    }

    #[test]
    fn probe_service_should_probe_at_threshold() {
        let config = ProbeConfig {
            failure_threshold: 2,
            ..ProbeConfig::default()
        };
        let mut mock = MockProbeClient::new();
        mock.set_result(
            "head:https://test.com",
            ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpHead, "err".to_string()),
        );
        mock.set_result(
            "get:https://test.com",
            ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpGet, "err".to_string()),
        );
        mock.set_result(
            "tls:test.com",
            ProbeResult::unreachable("test".to_string(), ProbeMethod::TlsHandshake, "err".to_string()),
        );
        
        let client = Arc::new(mock);
        let mut service = ProbeService::new(config, client);
        
        service.register_site("test", "https://test.com");
        service.probe_site("test");
        
        assert!(!service.should_probe("test"));
    }

    #[test]
    fn probe_service_probe_interval_normal() {
        let config = ProbeConfig::default();
        let client = Arc::new(MockProbeClient::new());
        let mut service = ProbeService::new(config, client);
        
        service.register_site("test", "https://test.com");
        service.probe_site("test");
        
        assert_eq!(service.probe_interval_for_site("test"), Duration::from_secs(30));
    }

    #[test]
    fn probe_service_probe_interval_degraded() {
        let config = ProbeConfig {
            failure_threshold: 2,
            ..ProbeConfig::default()
        };
        let mut mock = MockProbeClient::new();
        mock.set_result(
            "head:https://test.com",
            ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpHead, "err".to_string()),
        );
        mock.set_result(
            "get:https://test.com",
            ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpGet, "err".to_string()),
        );
        mock.set_result(
            "tls:test.com",
            ProbeResult::unreachable("test".to_string(), ProbeMethod::TlsHandshake, "err".to_string()),
        );
        
        let client = Arc::new(mock);
        let mut service = ProbeService::new(config, client);
        
        service.register_site("test", "https://test.com");
        service.probe_site("test");
        
        let interval = service.probe_interval_for_site("test");
        let expected = Duration::from_secs(config.degraded_interval_secs);
        assert_eq!(interval, expected);
    }

    #[test]
    fn extract_domain_simple() {
        assert_eq!(extract_domain("https://github.com"), "github.com");
        assert_eq!(extract_domain("http://example.com/path"), "example.com");
        assert_eq!(extract_domain("https://sub.domain.com:8080"), "sub.domain.com");
    }

    #[test]
    fn probe_service_probe_missing_site_returns_none() {
        let config = ProbeConfig::default();
        let client = Arc::new(MockProbeClient::new());
        let mut service = ProbeService::new(config, client);
        
        assert!(service.probe_site("nonexistent").is_none());
    }
}
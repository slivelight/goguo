use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeMethod {
    HttpHead,
    HttpGet,
    DnsResolve,
    TlsHandshake,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub site_id: String,
    pub timestamp: DateTime<Utc>,
    pub reachable: bool,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
    pub probe_method: ProbeMethod,
}

impl ProbeResult {
    #[must_use]
    pub fn reachable(site_id: String, method: ProbeMethod, response_time_ms: u64) -> Self {
        Self {
            site_id,
            timestamp: Utc::now(),
            reachable: true,
            response_time_ms: Some(response_time_ms),
            error: None,
            probe_method: method,
        }
    }

    #[must_use]
    pub fn unreachable(site_id: String, method: ProbeMethod, error: String) -> Self {
        Self {
            site_id,
            timestamp: Utc::now(),
            reachable: false,
            response_time_ms: None,
            error: Some(error),
            probe_method: method,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct ProbeConfig {
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    #[serde(default = "default_interval_secs")]
    pub interval_secs: u64,
    #[serde(default = "default_degraded_interval_secs")]
    pub degraded_interval_secs: u64,
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    #[serde(default = "default_history_max_size")]
    pub history_max_size: usize,
}

const fn default_timeout_secs() -> u64 {
    3
}

const fn default_max_concurrent() -> usize {
    10
}

const fn default_interval_secs() -> u64 {
    30
}

const fn default_degraded_interval_secs() -> u64 {
    120
}

const fn default_failure_threshold() -> u32 {
    3
}

const fn default_history_max_size() -> usize {
    1000
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout_secs(),
            max_concurrent: default_max_concurrent(),
            interval_secs: default_interval_secs(),
            degraded_interval_secs: default_degraded_interval_secs(),
            failure_threshold: default_failure_threshold(),
            history_max_size: default_history_max_size(),
        }
    }
}

impl ProbeConfig {
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    #[must_use]
    pub const fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_secs)
    }

    #[must_use]
    pub const fn degraded_interval(&self) -> Duration {
        Duration::from_secs(self.degraded_interval_secs)
    }
}

#[derive(Debug, Clone)]
pub struct ProbeHistory {
    records: VecDeque<ProbeResult>,
    max_size: usize,
}

impl ProbeHistory {
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            records: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, result: ProbeResult) {
        if self.records.len() >= self.max_size {
            self.records.pop_front();
        }
        self.records.push_back(result);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    #[must_use]
    pub fn latest(&self) -> Option<&ProbeResult> {
        self.records.back()
    }

    #[must_use]
    pub fn latest_for_site(&self, site_id: &str) -> Option<&ProbeResult> {
        self.records.iter().rev().find(|r| r.site_id == site_id)
    }

    #[must_use]
    pub fn consecutive_failures(&self, site_id: &str) -> u32 {
        let mut count = 0u32;
        for r in self.records.iter().rev() {
            if r.site_id != site_id {
                continue;
            }
            if r.reachable {
                break;
            }
            count += 1;
        }
        count
    }

    #[must_use]
    pub fn all_for_site(&self, site_id: &str) -> Vec<&ProbeResult> {
        self.records.iter().filter(|r| r.site_id == site_id).collect()
    }

    #[must_use]
    pub const fn records(&self) -> &VecDeque<ProbeResult> {
        &self.records
    }
}

impl Default for ProbeHistory {
    fn default() -> Self {
        Self::new(default_history_max_size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_method_roundtrip() {
        for method in [
            ProbeMethod::HttpHead,
            ProbeMethod::HttpGet,
            ProbeMethod::DnsResolve,
            ProbeMethod::TlsHandshake,
        ] {
            let json = serde_json::to_string(&method).expect("serialize");
            let back: ProbeMethod = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, method);
        }
    }

    #[test]
    fn probe_method_snake_case() {
        assert_eq!(
            serde_json::to_string(&ProbeMethod::HttpHead).expect("serialize"),
            "\"http_head\""
        );
        assert_eq!(
            serde_json::to_string(&ProbeMethod::TlsHandshake).expect("serialize"),
            "\"tls_handshake\""
        );
    }

    #[test]
    fn probe_result_reachable() {
        let result = ProbeResult::reachable(
            "github".to_string(),
            ProbeMethod::HttpHead,
            150,
        );
        assert_eq!(result.site_id, "github");
        assert!(result.reachable);
        assert_eq!(result.response_time_ms, Some(150));
        assert!(result.error.is_none());
        assert_eq!(result.probe_method, ProbeMethod::HttpHead);
    }

    #[test]
    fn probe_result_unreachable() {
        let result = ProbeResult::unreachable(
            "github".to_string(),
            ProbeMethod::HttpGet,
            "timeout".to_string(),
        );
        assert_eq!(result.site_id, "github");
        assert!(!result.reachable);
        assert!(result.response_time_ms.is_none());
        assert_eq!(result.error, Some("timeout".to_string()));
    }

    #[test]
    fn probe_result_roundtrip() {
        let result = ProbeResult::reachable(
            "test".to_string(),
            ProbeMethod::HttpGet,
            200,
        );
        let json = serde_json::to_string(&result).expect("serialize");
        let back: ProbeResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.site_id, result.site_id);
        assert_eq!(back.reachable, result.reachable);
        assert_eq!(back.response_time_ms, result.response_time_ms);
    }

    #[test]
    fn probe_config_default_values() {
        let config = ProbeConfig::default();
        assert_eq!(config.timeout_secs, 3);
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.interval_secs, 30);
        assert_eq!(config.degraded_interval_secs, 120);
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.history_max_size, 1000);
    }

    #[test]
    fn probe_config_timeout_duration() {
        let config = ProbeConfig::default();
        assert_eq!(config.timeout(), Duration::from_secs(3));
    }

    #[test]
    fn probe_config_roundtrip() {
        let config = ProbeConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let back: ProbeConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.timeout_secs, config.timeout_secs);
        assert_eq!(back.max_concurrent, config.max_concurrent);
    }

    #[test]
    fn probe_history_new_empty() {
        let history = ProbeHistory::new(100);
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn probe_history_push_increments() {
        let mut history = ProbeHistory::new(100);
        let result = ProbeResult::reachable("test".to_string(), ProbeMethod::HttpHead, 100);
        history.push(result);
        assert_eq!(history.len(), 1);
        assert!(history.latest().is_some());
    }

    #[test]
    fn probe_history_evicts_old_when_full() {
        let mut history = ProbeHistory::new(3);
        
        for i in 0..5 {
            history.push(ProbeResult::reachable(
                format!("site{i}"),
                ProbeMethod::HttpHead,
                100,
            ));
        }
        
        assert_eq!(history.len(), 3);
        let records = history.records();
        assert!(records.iter().any(|r| r.site_id == "site2"));
        assert!(records.iter().any(|r| r.site_id == "site3"));
        assert!(records.iter().any(|r| r.site_id == "site4"));
        assert!(!records.iter().any(|r| r.site_id == "site0"));
        assert!(!records.iter().any(|r| r.site_id == "site1"));
    }

    #[test]
    fn probe_history_latest_for_site() {
        let mut history = ProbeHistory::new(100);
        
        history.push(ProbeResult::reachable("github".to_string(), ProbeMethod::HttpHead, 100));
        history.push(ProbeResult::reachable("npmjs".to_string(), ProbeMethod::HttpHead, 150));
        history.push(ProbeResult::unreachable("github".to_string(), ProbeMethod::HttpGet, "timeout".to_string()));
        
        let latest_github = history.latest_for_site("github").expect("found");
        assert!(!latest_github.reachable);
        assert_eq!(latest_github.error, Some("timeout".to_string()));
    }

    #[test]
    fn probe_history_consecutive_failures() {
        let mut history = ProbeHistory::new(100);
        
        history.push(ProbeResult::reachable("test".to_string(), ProbeMethod::HttpHead, 100));
        history.push(ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpGet, "error1".to_string()));
        history.push(ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpGet, "error2".to_string()));
        history.push(ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpGet, "error3".to_string()));
        history.push(ProbeResult::reachable("other".to_string(), ProbeMethod::HttpHead, 100));
        
        assert_eq!(history.consecutive_failures("test"), 3);
        assert_eq!(history.consecutive_failures("other"), 0);
    }

    #[test]
    fn probe_history_consecutive_failures_stops_at_reachable() {
        let mut history = ProbeHistory::new(100);
        
        history.push(ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpGet, "e1".to_string()));
        history.push(ProbeResult::reachable("test".to_string(), ProbeMethod::HttpHead, 100));
        history.push(ProbeResult::unreachable("test".to_string(), ProbeMethod::HttpGet, "e2".to_string()));
        
        assert_eq!(history.consecutive_failures("test"), 1);
    }

    #[test]
    fn probe_history_all_for_site() {
        let mut history = ProbeHistory::new(100);
        
        history.push(ProbeResult::reachable("github".to_string(), ProbeMethod::HttpHead, 100));
        history.push(ProbeResult::reachable("npmjs".to_string(), ProbeMethod::HttpHead, 150));
        history.push(ProbeResult::unreachable("github".to_string(), ProbeMethod::HttpGet, "err".to_string()));
        
        let github_records = history.all_for_site("github");
        assert_eq!(github_records.len(), 2);
    }

    #[test]
    fn probe_history_default_size() {
        let history = ProbeHistory::default();
        assert_eq!(history.max_size, 1000);
    }
}
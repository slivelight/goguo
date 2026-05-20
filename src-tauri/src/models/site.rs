use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainCategory {
    Core,
    Api,
    Cdn,
    Assets,
    Services,
    Packages,
    ThirdParty,
    CrossDependency,
}

impl DomainCategory {
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Core,
            Self::Api,
            Self::Cdn,
            Self::Assets,
            Self::Services,
            Self::Packages,
            Self::ThirdParty,
            Self::CrossDependency,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub url: String,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
}

const fn default_timeout_secs() -> u64 {
    5
}

const fn default_failure_threshold() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteDefinition {
    pub id: String,
    pub name: String,
    pub domains: HashMap<DomainCategory, Vec<String>>,
    #[serde(default)]
    pub health_check: Option<HealthCheckConfig>,
}

impl SiteDefinition {
    #[must_use]
    pub fn all_domains(&self) -> Vec<String> {
        self.domains
            .values()
            .flatten()
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn domain_count(&self) -> usize {
        self.domains.values().map(Vec::len).sum()
    }

    #[must_use]
    pub fn github_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "github.com".to_string(),
            "github.io".to_string(),
            "githubusercontent.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "githubassets.com".to_string(),
        ]);
        domains.insert(DomainCategory::Packages, vec![
            "ghcr.io".to_string(),
        ]);

        Self {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://github.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn npmjs_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "npmjs.com".to_string(),
            "npmjs.org".to_string(),
        ]);

        Self {
            id: "npmjs".to_string(),
            name: "npm".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://www.npmjs.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn claude_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "claude.ai".to_string(),
            "anthropic.com".to_string(),
        ]);

        Self {
            id: "claude".to_string(),
            name: "Claude".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://claude.ai".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn chatgpt_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "chatgpt.com".to_string(),
            "openai.com".to_string(),
        ]);
        domains.insert(DomainCategory::Api, vec![
            "api.openai.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "cdn.openai.com".to_string(),
            "oaistatic.com".to_string(),
            "oaiusercontent.com".to_string(),
        ]);

        Self {
            id: "chatgpt".to_string(),
            name: "ChatGPT".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://chatgpt.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn docker_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "docker.com".to_string(),
            "docker.io".to_string(),
        ]);
        domains.insert(DomainCategory::Packages, vec![
            "registry.docker.com".to_string(),
            "registry.hub.docker.com".to_string(),
        ]);

        Self {
            id: "docker".to_string(),
            name: "Docker".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://www.docker.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn google_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "google.com".to_string(),
            "googleapis.com".to_string(),
        ]);
        domains.insert(DomainCategory::Services, vec![
            "gmail.com".to_string(),
            "googlemail.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "gstatic.com".to_string(),
        ]);
        domains.insert(DomainCategory::CrossDependency, vec![
            "ytimg.com".to_string(),
            "youtube.com".to_string(),
        ]);

        Self {
            id: "google".to_string(),
            name: "Google".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://www.google.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_category_all_count() {
        let all = DomainCategory::all();
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn domain_category_roundtrip() {
        for cat in DomainCategory::all() {
            let json = serde_json::to_string(&cat).expect("serialize");
            let back: DomainCategory = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, cat);
        }
    }

    #[test]
    fn domain_category_snake_case() {
        assert_eq!(
            serde_json::to_string(&DomainCategory::Core).expect("serialize"),
            "\"core\""
        );
        assert_eq!(
            serde_json::to_string(&DomainCategory::CrossDependency).expect("serialize"),
            "\"cross_dependency\""
        );
    }

    #[test]
    fn health_check_config_default_values() {
        let config = HealthCheckConfig {
            url: "https://example.com".to_string(),
            timeout_secs: default_timeout_secs(),
            failure_threshold: default_failure_threshold(),
        };
        assert_eq!(config.timeout_secs, 5);
        assert_eq!(config.failure_threshold, 3);
    }

    #[test]
    fn health_check_config_roundtrip() {
        let config = HealthCheckConfig {
            url: "https://github.com".to_string(),
            timeout_secs: 10,
            failure_threshold: 5,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let back: HealthCheckConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.url, config.url);
        assert_eq!(back.timeout_secs, 10);
        assert_eq!(back.failure_threshold, 5);
    }

    #[test]
    fn site_definition_github_default() {
        let site = SiteDefinition::github_default();
        assert_eq!(site.id, "github");
        assert_eq!(site.name, "GitHub");
        assert!(site.health_check.is_some());
        
        let all = site.all_domains();
        assert!(all.contains(&"github.com".to_string()));
        assert!(all.contains(&"github.io".to_string()));
        assert!(all.contains(&"githubusercontent.com".to_string()));
        assert!(all.contains(&"githubassets.com".to_string()));
        assert!(all.contains(&"ghcr.io".to_string()));
        assert_eq!(site.domain_count(), 5);
    }

    #[test]
    fn site_definition_chatgpt_covers_openai() {
        let site = SiteDefinition::chatgpt_default();
        let all = site.all_domains();
        assert!(all.contains(&"chatgpt.com".to_string()));
        assert!(all.contains(&"openai.com".to_string()));
        assert!(all.contains(&"api.openai.com".to_string()));
        assert!(all.contains(&"cdn.openai.com".to_string()));
        assert!(all.contains(&"oaistatic.com".to_string()));
        assert!(all.contains(&"oaiusercontent.com".to_string()));
        assert_eq!(site.domain_count(), 6);
    }

    #[test]
    fn site_definition_roundtrip() {
        let site = SiteDefinition::github_default();
        let json = serde_json::to_string(&site).expect("serialize");
        let back: SiteDefinition = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, site.id);
        assert_eq!(back.name, site.name);
        assert_eq!(back.domain_count(), site.domain_count());
    }

    #[test]
    fn site_definition_custom_without_health_check() {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec!["custom.com".to_string()]);
        
        let site = SiteDefinition {
            id: "custom".to_string(),
            name: "Custom".to_string(),
            domains,
            health_check: None,
        };
        
        let json = serde_json::to_string(&site).expect("serialize");
        let back: SiteDefinition = serde_json::from_str(&json).expect("deserialize");
        assert!(back.health_check.is_none());
    }

    #[test]
    fn site_definition_all_domains_flattens() {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec!["a.com".to_string(), "b.com".to_string()]);
        domains.insert(DomainCategory::Api, vec!["c.com".to_string()]);
        
        let site = SiteDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            domains,
            health_check: None,
        };
        
        let all = site.all_domains();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&"a.com".to_string()));
        assert!(all.contains(&"b.com".to_string()));
        assert!(all.contains(&"c.com".to_string()));
    }
}
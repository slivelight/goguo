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
            "githubassets.com".to_string(),
            "ghcr.io".to_string(),
            "ssh.github.com".to_string(),
            "lfs.github.com".to_string(),
        ]);
        domains.insert(DomainCategory::Api, vec![
            "api.github.com".to_string(),
            "uploads.github.com".to_string(),
            "objects.githubusercontent.com".to_string(),
            "graphql.github.com".to_string(),
            "api.githubusercontent.com".to_string(),
            "actions.githubusercontent.com".to_string(),
            "pipelines.actions.githubusercontent.com".to_string(),
            "artifactsv2.githubusercontent.com".to_string(),
            "resultsv2.githubusercontent.com".to_string(),
            "github-releases.githubusercontent.com".to_string(),
        ]);
        domains.insert(DomainCategory::Assets, vec![
            "raw.githubusercontent.com".to_string(),
            "codeload.github.com".to_string(),
            "camo.githubusercontent.com".to_string(),
            "cloud.githubusercontent.com".to_string(),
            "avatars.githubusercontent.com".to_string(),
            "user-images.githubusercontent.com".to_string(),
            "media.githubusercontent.com".to_string(),
            "private-user-images.githubusercontent.com".to_string(),
            "marketplace-screenshots.githubusercontent.com".to_string(),
            "desktop.githubusercontent.com".to_string(),
        ]);
        domains.insert(DomainCategory::Services, vec![
            "gist.github.com".to_string(),
            "pages.github.com".to_string(),
            "vscode-auth.github.com".to_string(),
            "education.github.com".to_string(),
            "auth.github.com".to_string(),
            "live.github.com".to_string(),
            "packagist.githubusercontent.com".to_string(),
            "favicons.githubusercontent.com".to_string(),
            "copilot.github.com".to_string(),
            "copilot-telemetry.githubusercontent.com".to_string(),
            "copilot-proxy.githubusercontent.com".to_string(),
            "githubcopilot.com".to_string(),
            "origin-tracker.githubusercontent.com".to_string(),
            "codespaces.githubusercontent.com".to_string(),
            "githubcloudusercontent.com".to_string(),
        ]);
        domains.insert(DomainCategory::Packages, vec![
            "npm.pkg.github.com".to_string(),
            "maven.pkg.github.com".to_string(),
            "nuget.pkg.github.com".to_string(),
            "rubygems.pkg.github.com".to_string(),
            "pypi.pkg.github.com".to_string(),
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
            "registry.npmjs.org".to_string(),
            "static.npmjs.com".to_string(),
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
            "claude.com".to_string(),
            "anthropic.com".to_string(),
        ]);
        domains.insert(DomainCategory::Api, vec![
            "api.anthropic.com".to_string(),
            "statsig.anthropic.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "s-cdn.anthropic.com".to_string(),
            "a-cdn.anthropic.com".to_string(),
            "cdn.anthropic.com".to_string(),
        ]);
        domains.insert(DomainCategory::ThirdParty, vec![
            "intercom.io".to_string(),
            "intercomcdn.com".to_string(),
            "sentry.io".to_string(),
            "statsigapi.net".to_string(),
        ]);
        domains.insert(DomainCategory::CrossDependency, vec![
            "accounts.google.com".to_string(),
            "fonts.googleapis.com".to_string(),
            "fonts.gstatic.com".to_string(),
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
            "chat.openai.com".to_string(),
            "openai.com".to_string(),
            "help.openai.com".to_string(),
            "platform.openai.com".to_string(),
        ]);
        domains.insert(DomainCategory::Api, vec![
            "api.openai.com".to_string(),
            "auth0.openai.com".to_string(),
            "auth.openai.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "cdn.openai.com".to_string(),
            "oaistatic.com".to_string(),
            "oaiusercontent.com".to_string(),
            "cdn.openaimerge.com".to_string(),
        ]);
        domains.insert(DomainCategory::ThirdParty, vec![
            "cdn.workos.com".to_string(),
            "setup.workos.com".to_string(),
            "challenges.cloudflare.com".to_string(),
            "turnstile.cloudflare.com".to_string(),
            "statsigapi.net".to_string(),
            "statsig.com".to_string(),
            "intercom.io".to_string(),
            "intercomcdn.com".to_string(),
            "sentry.io".to_string(),
            "rum.browser-intake-datadoghq.com".to_string(),
            "js.stripe.com".to_string(),
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
            "google.com.hk".to_string(),
            "googleapis.com".to_string(),
        ]);
        domains.insert(DomainCategory::Api, vec![
            "accounts.google.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "gstatic.com".to_string(),
            "googleusercontent.com".to_string(),
            "fonts.googleapis.com".to_string(),
            "fonts.gstatic.com".to_string(),
            "ajax.googleapis.com".to_string(),
            "apis.google.com".to_string(),
        ]);
        domains.insert(DomainCategory::Services, vec![
            "youtube.com".to_string(),
            "ytimg.com".to_string(),
            "yt3.ggpht.com".to_string(),
            "gmail.com".to_string(),
            "drive.google.com".to_string(),
            "maps.google.com".to_string(),
            "play.google.com".to_string(),
            "scholar.google.com".to_string(),
            "google-analytics.com".to_string(),
            "googletagmanager.com".to_string(),
            "doubleclick.net".to_string(),
            "googlesyndication.com".to_string(),
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

    #[must_use]
    pub fn stackoverflow_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "stackoverflow.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "sstatic.net".to_string(),
            "cdn.sstatic.net".to_string(),
        ]);
        domains.insert(DomainCategory::Services, vec![
            "stackexchange.com".to_string(),
        ]);
        domains.insert(DomainCategory::CrossDependency, vec![
            "superuser.com".to_string(),
            "askubuntu.com".to_string(),
            "serverfault.com".to_string(),
            "mathoverflow.net".to_string(),
            "stackapps.com".to_string(),
        ]);

        Self {
            id: "stackoverflow".to_string(),
            name: "Stack Overflow".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://stackoverflow.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn pypi_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "pypi.org".to_string(),
        ]);
        domains.insert(DomainCategory::Packages, vec![
            "files.pythonhosted.org".to_string(),
        ]);

        Self {
            id: "pypi".to_string(),
            name: "PyPI".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://pypi.org".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn crates_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "crates.io".to_string(),
            "static.crates.io".to_string(),
            "index.crates.io".to_string(),
        ]);

        Self {
            id: "crates".to_string(),
            name: "Crates.io".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://crates.io".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn oracle_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "oracle.com".to_string(),
        ]);
        domains.insert(DomainCategory::Services, vec![
            "cloud.oracle.com".to_string(),
        ]);

        Self {
            id: "oracle".to_string(),
            name: "Oracle".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://www.oracle.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn wikipedia_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "wikipedia.org".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "wikimedia.org".to_string(),
        ]);

        Self {
            id: "wikipedia".to_string(),
            name: "Wikipedia".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://www.wikipedia.org".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn whatsapp_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "whatsapp.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "whatsapp.net".to_string(),
        ]);

        Self {
            id: "whatsapp".to_string(),
            name: "WhatsApp".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://www.whatsapp.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn instagram_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "instagram.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "cdninstagram.com".to_string(),
            "fbcdn.net".to_string(),
        ]);

        Self {
            id: "instagram".to_string(),
            name: "Instagram".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://www.instagram.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn canva_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "canva.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "cdn.canva.com".to_string(),
        ]);

        Self {
            id: "canva".to_string(),
            name: "Canva".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://www.canva.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        }
    }

    #[must_use]
    pub fn twitter_x_default() -> Self {
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "x.com".to_string(),
            "twitter.com".to_string(),
        ]);
        domains.insert(DomainCategory::Cdn, vec![
            "twimg.com".to_string(),
        ]);

        Self {
            id: "twitter-x".to_string(),
            name: "X (Twitter)".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://x.com".to_string(),
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
        // Core
        assert!(all.contains(&"github.com".to_string()));
        assert!(all.contains(&"github.io".to_string()));
        assert!(all.contains(&"githubusercontent.com".to_string()));
        assert!(all.contains(&"githubassets.com".to_string()));
        assert!(all.contains(&"ghcr.io".to_string()));
        assert!(all.contains(&"ssh.github.com".to_string()));
        assert!(all.contains(&"lfs.github.com".to_string()));
        // Api
        assert!(all.contains(&"api.github.com".to_string()));
        assert!(all.contains(&"graphql.github.com".to_string()));
        assert!(all.contains(&"actions.githubusercontent.com".to_string()));
        // Assets
        assert!(all.contains(&"raw.githubusercontent.com".to_string()));
        assert!(all.contains(&"codeload.github.com".to_string()));
        assert!(all.contains(&"avatars.githubusercontent.com".to_string()));
        // Services
        assert!(all.contains(&"gist.github.com".to_string()));
        assert!(all.contains(&"copilot.github.com".to_string()));
        assert!(all.contains(&"githubcopilot.com".to_string()));
        // Packages
        assert!(all.contains(&"npm.pkg.github.com".to_string()));
        // Reference: github-host project has 47 domains for GitHub
        assert_eq!(site.domain_count(), 47);
    }

    #[test]
    fn site_definition_chatgpt_covers_openai() {
        let site = SiteDefinition::chatgpt_default();
        let all = site.all_domains();
        // Core
        assert!(all.contains(&"chatgpt.com".to_string()));
        assert!(all.contains(&"openai.com".to_string()));
        assert!(all.contains(&"chat.openai.com".to_string()));
        assert!(all.contains(&"help.openai.com".to_string()));
        assert!(all.contains(&"platform.openai.com".to_string()));
        // Api
        assert!(all.contains(&"auth0.openai.com".to_string()));
        assert!(all.contains(&"auth.openai.com".to_string()));
        // Cdn
        assert!(all.contains(&"cdn.openai.com".to_string()));
        assert!(all.contains(&"oaistatic.com".to_string()));
        assert!(all.contains(&"oaiusercontent.com".to_string()));
        // ThirdParty
        assert!(all.contains(&"challenges.cloudflare.com".to_string()));
        assert!(all.contains(&"sentry.io".to_string()));
        assert!(all.contains(&"js.stripe.com".to_string()));
        // Reference: github-host project has 22 domains, GoGuo adds api.openai.com = 23
        assert_eq!(site.domain_count(), 23);
    }

    #[test]
    fn site_definition_google_domains() {
        let site = SiteDefinition::google_default();
        let all = site.all_domains();
        // Core
        assert!(all.contains(&"google.com".to_string()));
        assert!(all.contains(&"google.com.hk".to_string()));
        assert!(all.contains(&"googleapis.com".to_string()));
        // Api
        assert!(all.contains(&"accounts.google.com".to_string()));
        // Cdn
        assert!(all.contains(&"gstatic.com".to_string()));
        assert!(all.contains(&"googleusercontent.com".to_string()));
        assert!(all.contains(&"fonts.googleapis.com".to_string()));
        // Services
        assert!(all.contains(&"youtube.com".to_string()));
        assert!(all.contains(&"gmail.com".to_string()));
        assert!(all.contains(&"drive.google.com".to_string()));
        // Reference: github-host project has 22 domains for Google
        assert_eq!(site.domain_count(), 22);
    }

    #[test]
    fn site_definition_claude_domains() {
        let site = SiteDefinition::claude_default();
        let all = site.all_domains();
        // Core
        assert!(all.contains(&"claude.ai".to_string()));
        assert!(all.contains(&"claude.com".to_string()));
        assert!(all.contains(&"anthropic.com".to_string()));
        // Api
        assert!(all.contains(&"api.anthropic.com".to_string()));
        // Cdn
        assert!(all.contains(&"cdn.anthropic.com".to_string()));
        // ThirdParty
        assert!(all.contains(&"intercom.io".to_string()));
        assert!(all.contains(&"sentry.io".to_string()));
        // CrossDependency
        assert!(all.contains(&"accounts.google.com".to_string()));
        // Reference: github-host project has 15 domains for Claude
        assert_eq!(site.domain_count(), 15);
    }

    #[test]
    fn site_definition_npmjs_domains() {
        let site = SiteDefinition::npmjs_default();
        let all = site.all_domains();
        assert!(all.contains(&"npmjs.com".to_string()));
        assert!(all.contains(&"registry.npmjs.org".to_string()));
        assert!(all.contains(&"static.npmjs.com".to_string()));
        assert_eq!(site.domain_count(), 3);
    }

    #[test]
    fn site_definition_crates_domains() {
        let site = SiteDefinition::crates_default();
        let all = site.all_domains();
        assert!(all.contains(&"crates.io".to_string()));
        assert!(all.contains(&"static.crates.io".to_string()));
        assert!(all.contains(&"index.crates.io".to_string()));
        assert_eq!(site.domain_count(), 3);
    }

    #[test]
    fn site_definition_docker_domains() {
        let site = SiteDefinition::docker_default();
        let all = site.all_domains();
        assert!(all.contains(&"docker.com".to_string()));
        assert!(all.contains(&"docker.io".to_string()));
        assert!(all.contains(&"registry.docker.com".to_string()));
        assert!(all.contains(&"registry.hub.docker.com".to_string()));
        assert!(site.domain_count() >= 4);
    }

    #[test]
    fn site_definition_stackoverflow_domains() {
        let site = SiteDefinition::stackoverflow_default();
        let all = site.all_domains();
        assert!(all.contains(&"stackoverflow.com".to_string()));
        assert!(all.contains(&"sstatic.net".to_string()));
        assert!(all.contains(&"stackexchange.com".to_string()));
        assert!(all.contains(&"superuser.com".to_string()));
        assert!(site.domain_count() >= 6);
    }

    #[test]
    fn site_definition_pypi_domains() {
        let site = SiteDefinition::pypi_default();
        let all = site.all_domains();
        assert!(all.contains(&"pypi.org".to_string()));
        assert!(all.contains(&"files.pythonhosted.org".to_string()));
        assert!(site.domain_count() >= 2);
    }

    #[test]
    fn site_definition_all_sites_have_domains() {
        let sites = vec![
            SiteDefinition::github_default(),
            SiteDefinition::npmjs_default(),
            SiteDefinition::claude_default(),
            SiteDefinition::chatgpt_default(),
            SiteDefinition::docker_default(),
            SiteDefinition::google_default(),
            SiteDefinition::stackoverflow_default(),
            SiteDefinition::pypi_default(),
            SiteDefinition::crates_default(),
            SiteDefinition::oracle_default(),
            SiteDefinition::wikipedia_default(),
            SiteDefinition::whatsapp_default(),
            SiteDefinition::instagram_default(),
            SiteDefinition::canva_default(),
            SiteDefinition::twitter_x_default(),
        ];
        assert_eq!(sites.len(), 15);
        for site in &sites {
            assert!(!site.all_domains().is_empty(), "{} has no domains", site.id);
        }
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
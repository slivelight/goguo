use crate::models::site::SiteDefinition;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub rule_type: String,
    pub domain: String,
    pub policy: String,
}

impl Rule {
    #[must_use]
    pub fn domain_suffix(domain: String) -> Self {
        Self {
            rule_type: "DOMAIN-SUFFIX".to_string(),
            domain,
            policy: "PROXY".to_string(),
        }
    }

    #[must_use]
    pub fn domain_exact(domain: String) -> Self {
        Self {
            rule_type: "DOMAIN".to_string(),
            domain,
            policy: "PROXY".to_string(),
        }
    }

    #[must_use]
    pub fn match_direct() -> Self {
        Self {
            rule_type: "MATCH".to_string(),
            domain: String::new(),
            policy: "DIRECT".to_string(),
        }
    }

    #[must_use]
    pub fn to_mihomo_line(&self) -> String {
        if self.rule_type == "MATCH" {
            format!("{},{}", self.rule_type, self.policy)
        } else {
            format!("{},{},{}", self.rule_type, self.domain, self.policy)
        }
    }
}

pub struct GeneratedRules {
    pub rules: Vec<Rule>,
    pub site_counts: HashMap<String, usize>,
}

pub struct RuleGenerator {
    user_overrides: Vec<Rule>,
}

impl RuleGenerator {
    #[must_use]
    pub const fn new() -> Self {
        Self { user_overrides: vec![] }
    }

    #[must_use]
    pub fn generate(sites: &[SiteDefinition], user_overrides: &[Rule]) -> GeneratedRules {
        let mut rules: Vec<Rule> = Vec::new();
        let mut site_counts: HashMap<String, usize> = HashMap::new();

        for override_rule in user_overrides {
            rules.push(override_rule.clone());
        }

        for site in sites {
            let domains = site.all_domains();
            let count = domains.len();
            site_counts.insert(site.id.clone(), count);

            for domain in domains {
                if domain.starts_with('*') {
                    let clean = domain.trim_start_matches('*').trim_start_matches('.');
                    rules.push(Rule::domain_suffix(clean.to_string()));
                } else if domain.contains('.') && !domain.contains('*') {
                    rules.push(Rule::domain_suffix(domain.clone()));
                } else {
                    rules.push(Rule::domain_exact(domain.clone()));
                }
            }
        }

        rules.push(Rule::match_direct());

        GeneratedRules { rules, site_counts }
    }

    /// Validates that the last rule is `MATCH,DIRECT`.
    ///
    /// Returns `false` if the rules list is empty or the last rule
    /// is not `MATCH,DIRECT`.
    ///
    /// # Panics
    ///
    /// Never panics. The `expect()` call is guarded by the empty check above.
    #[must_use]
    pub fn validate_match_direct(rules: &[Rule]) -> bool {
        if rules.is_empty() {
            return false;
        }
        let last = rules.last().expect("checked non-empty");
        last.rule_type == "MATCH" && last.policy == "DIRECT"
    }

    pub fn set_user_overrides(&mut self, overrides: Vec<Rule>) {
        self.user_overrides = overrides;
    }

    #[must_use]
    pub fn preview(&self, sites: &[SiteDefinition]) -> Vec<String> {
        let generated = Self::generate(sites, &self.user_overrides);
        generated.rules.iter().map(Rule::to_mihomo_line).collect()
    }

    #[must_use]
    pub fn total_domain_count(sites: &[SiteDefinition]) -> usize {
        sites.iter().map(SiteDefinition::domain_count).sum()
    }
}

impl Default for RuleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::site::DomainCategory;
    use std::collections::HashMap as StdHashMap;

    #[test]
    fn rule_domain_suffix_format() {
        let rule = Rule::domain_suffix("github.com".to_string());
        assert_eq!(rule.rule_type, "DOMAIN-SUFFIX");
        assert_eq!(rule.domain, "github.com");
        assert_eq!(rule.policy, "PROXY");
        assert_eq!(rule.to_mihomo_line(), "DOMAIN-SUFFIX,github.com,PROXY");
    }

    #[test]
    fn rule_domain_exact_format() {
        let rule = Rule::domain_exact("exact.com".to_string());
        assert_eq!(rule.rule_type, "DOMAIN");
        assert_eq!(rule.domain, "exact.com");
        assert_eq!(rule.to_mihomo_line(), "DOMAIN,exact.com,PROXY");
    }

    #[test]
    fn rule_match_direct_format() {
        let rule = Rule::match_direct();
        assert_eq!(rule.rule_type, "MATCH");
        assert_eq!(rule.policy, "DIRECT");
        assert_eq!(rule.to_mihomo_line(), "MATCH,DIRECT");
    }

    #[test]
    fn generate_empty_sites_returns_only_match_direct() {
        let generated = RuleGenerator::generate(&[], &[]);
        assert_eq!(generated.rules.len(), 1);
        assert_eq!(generated.rules[0].rule_type, "MATCH");
        assert_eq!(generated.rules[0].policy, "DIRECT");
        assert!(generated.site_counts.is_empty());
    }

    #[test]
    fn generate_single_site_returns_domains_plus_match() {
        let mut domains = StdHashMap::new();
        domains.insert(DomainCategory::Core, vec![
            "github.com".to_string(),
            "github.io".to_string(),
        ]);

        let site = SiteDefinition {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            domains,
            health_check: None,
        };

        let generated = RuleGenerator::generate(&[site], &[]);
        assert_eq!(generated.rules.len(), 3);
        assert_eq!(generated.site_counts.get("github"), Some(&2));
        assert!(generated.rules.iter().any(|r| r.domain == "github.com"));
        assert!(generated.rules.iter().any(|r| r.domain == "github.io"));
        assert_eq!(generated.rules.last().expect("last").rule_type, "MATCH");
    }

    #[test]
    fn validate_match_direct_empty_fails() {
        assert!(!RuleGenerator::validate_match_direct(&[]));
    }

    #[test]
    fn validate_match_direct_correct_last() {
        let rules = vec![
            Rule::domain_suffix("test.com".to_string()),
            Rule::match_direct(),
        ];
        assert!(RuleGenerator::validate_match_direct(&rules));
    }

    #[test]
    fn validate_match_direct_wrong_last_fails() {
        let rules = vec![
            Rule::domain_suffix("test.com".to_string()),
            Rule::domain_suffix("another.com".to_string()),
        ];
        assert!(!RuleGenerator::validate_match_direct(&rules));
    }

    #[test]
    fn validate_match_direct_wrong_policy_fails() {
        let rule = Rule {
            rule_type: "MATCH".to_string(),
            domain: String::new(),
            policy: "PROXY".to_string(),
        };
        let rules = vec![rule];
        assert!(!RuleGenerator::validate_match_direct(&rules));
    }

    #[test]
    fn user_overrides_appear_first() {
        let mut domains = StdHashMap::new();
        domains.insert(DomainCategory::Core, vec!["github.com".to_string()]);

        let site = SiteDefinition {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            domains,
            health_check: None,
        };

        let override_rule = Rule::domain_exact("custom.override.com".to_string());
        let generated = RuleGenerator::generate(&[site], &[override_rule]);

        assert_eq!(generated.rules.first().expect("first").domain, "custom.override.com");
        assert_eq!(generated.rules.first().expect("first").rule_type, "DOMAIN");
    }

    #[test]
    fn preview_returns_mihomo_lines() {
        let mut domains = StdHashMap::new();
        domains.insert(DomainCategory::Core, vec!["test.com".to_string()]);

        let site = SiteDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            domains,
            health_check: None,
        };

        let gen = RuleGenerator::new();
        let lines = gen.preview(&[site]);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "DOMAIN-SUFFIX,test.com,PROXY");
        assert_eq!(lines[1], "MATCH,DIRECT");
    }

    #[test]
    fn total_domain_count_sum() {
        let site1 = SiteDefinition::github_default();
        let site2 = SiteDefinition::npmjs_default();
        let expected = site1.domain_count() + site2.domain_count();

        let total = RuleGenerator::total_domain_count(&[site1, site2]);
        assert_eq!(total, expected);
    }

    #[test]
    fn wildcard_domain_generates_suffix() {
        let mut domains = StdHashMap::new();
        domains.insert(DomainCategory::Core, vec!["*.example.com".to_string()]);

        let site = SiteDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            domains,
            health_check: None,
        };

        let generated = RuleGenerator::generate(&[site], &[]);
        assert_eq!(generated.rules.first().expect("first").rule_type, "DOMAIN-SUFFIX");
        assert_eq!(generated.rules.first().expect("first").domain, "example.com");
    }

    #[test]
    fn rule_generator_default() {
        let gen = RuleGenerator::default();
        assert!(gen.user_overrides.is_empty());
    }

    #[test]
    fn set_user_overrides_updates() {
        let mut gen = RuleGenerator::new();
        let override_rule = Rule::domain_exact("custom.com".to_string());
        gen.set_user_overrides(vec![override_rule]);
        assert_eq!(gen.user_overrides.len(), 1);
        assert_eq!(gen.user_overrides[0].domain, "custom.com");
    }

    #[test]
    fn generate_multiple_sites_counts_each() {
        let site1 = SiteDefinition::github_default();
        let site2 = SiteDefinition::chatgpt_default();
        let expected1 = site1.domain_count();
        let expected2 = site2.domain_count();

        let generated = RuleGenerator::generate(&[site1, site2], &[]);
        assert_eq!(generated.site_counts.get("github"), Some(&expected1));
        assert_eq!(generated.site_counts.get("chatgpt"), Some(&expected2));
    }
}
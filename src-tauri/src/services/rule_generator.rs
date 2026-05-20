use crate::models::site::SiteDefinition;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

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

#[derive(Clone)]
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

#[derive(Debug, Clone)]
pub struct RuleStorage {
    rules_dir: PathBuf,
}

impl RuleStorage {
    #[must_use]
    pub const fn new(rules_dir: PathBuf) -> Self {
        Self { rules_dir }
    }

    fn current_rules_path(&self) -> PathBuf {
        self.rules_dir.join("current-rules.yaml")
    }

    fn previous_rules_path(&self) -> PathBuf {
        self.rules_dir.join("previous-rules.yaml")
    }

    /// Saves the current rules to YAML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written.
    pub fn save_current(&self, rules: &[Rule]) -> io::Result<()> {
        fs::create_dir_all(&self.rules_dir)?;
        let content = Self::rules_to_yaml(rules);
        fs::write(self.current_rules_path(), content)?;
        Ok(())
    }

    /// Backs up current rules to previous-rules.yaml before applying new rules.
    ///
    /// # Errors
    ///
    /// Returns an error if the current rules file cannot be read or the backup cannot be written.
    pub fn backup_current(&self) -> io::Result<bool> {
        let current_path = self.current_rules_path();
        if !current_path.exists() {
            return Ok(false);
        }

        let previous_path = self.previous_rules_path();
        fs::copy(current_path, previous_path)?;
        Ok(true)
    }

    /// Rolls back to previous rules after validation failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the previous rules file cannot be read or restored.
    pub fn rollback(&self) -> io::Result<bool> {
        let previous_path = self.previous_rules_path();
        if !previous_path.exists() {
            return Ok(false);
        }

        let current_path = self.current_rules_path();
        fs::copy(previous_path, current_path)?;
        Ok(true)
    }

    /// Loads current rules from YAML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn load_current(&self) -> io::Result<Vec<Rule>> {
        let path = self.current_rules_path();
        if !path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(path)?;
        Ok(Self::yaml_to_rules(&content))
    }

    /// Loads previous rules from backup file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn load_previous(&self) -> io::Result<Vec<Rule>> {
        let path = self.previous_rules_path();
        if !path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(path)?;
        Ok(Self::yaml_to_rules(&content))
    }

    fn rules_to_yaml(rules: &[Rule]) -> String {
        let mut yaml = String::from("rules:\n");
        for rule in rules {
            yaml.push_str("  - ");
            yaml.push_str(&rule.to_mihomo_line());
            yaml.push('\n');
        }
        yaml
    }

    #[must_use]
    fn yaml_to_rules(content: &str) -> Vec<Rule> {
        let mut rules = vec![];
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") {
                let rule_line = trimmed.trim_start_matches("- ").trim();
                if let Some(rule) = Self::parse_rule_line(rule_line) {
                    rules.push(rule);
                }
            }
        }
        rules
    }

    fn parse_rule_line(line: &str) -> Option<Rule> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            return None;
        }

        let rule_type = parts[0].trim();
        if rule_type == "MATCH" {
            return Some(Rule::match_direct());
        }

        if parts.len() < 3 {
            return None;
        }

        let domain = parts[1].trim();
        let policy = parts[2].trim();

        Some(Rule {
            rule_type: rule_type.to_string(),
            domain: domain.to_string(),
            policy: policy.to_string(),
        })
    }

    #[must_use]
    pub fn has_previous_backup(&self) -> bool {
        self.previous_rules_path().exists()
    }

    #[must_use]
    pub fn current_rules_exist(&self) -> bool {
        self.current_rules_path().exists()
    }
}

impl Default for RuleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RuleStorage {
    fn default() -> Self {
        Self::new(PathBuf::from("data/rules"))
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

    mod rule_storage_tests {
        use super::*;
        use tempfile::tempdir;

        #[test]
        fn storage_new_creates_paths() {
            let dir = tempdir().expect("tempdir");
            let storage = RuleStorage::new(dir.path().join("rules"));
            assert!(!storage.current_rules_exist());
            assert!(!storage.has_previous_backup());
        }

        #[test]
        fn save_current_creates_file() {
            let dir = tempdir().expect("tempdir");
            let storage = RuleStorage::new(dir.path().join("rules"));
            
            let rules = vec![
                Rule::domain_suffix("github.com".to_string()),
                Rule::match_direct(),
            ];
            
            storage.save_current(&rules).expect("save");
            assert!(storage.current_rules_exist());
        }

        #[test]
        fn save_current_yaml_format() {
            let dir = tempdir().expect("tempdir");
            let storage = RuleStorage::new(dir.path().join("rules"));
            
            let rules = vec![
                Rule::domain_suffix("github.com".to_string()),
                Rule::match_direct(),
            ];
            
            storage.save_current(&rules).expect("save");
            let content = fs::read_to_string(storage.current_rules_path()).expect("read");
            assert!(content.contains("rules:"));
            assert!(content.contains("DOMAIN-SUFFIX,github.com,PROXY"));
            assert!(content.contains("MATCH,DIRECT"));
        }

        #[test]
        fn backup_current_creates_previous() {
            let dir = tempdir().expect("tempdir");
            let storage = RuleStorage::new(dir.path().join("rules"));
            
            let rules = vec![Rule::match_direct()];
            storage.save_current(&rules).expect("save");
            
            let backed_up = storage.backup_current().expect("backup");
            assert!(backed_up);
            assert!(storage.has_previous_backup());
        }

        #[test]
        fn backup_current_returns_false_if_no_current() {
            let dir = tempdir().expect("tempdir");
            let storage = RuleStorage::new(dir.path().join("rules"));
            
            let backed_up = storage.backup_current().expect("backup");
            assert!(!backed_up);
        }

        #[test]
        fn rollback_restores_previous() {
            let dir = tempdir().expect("tempdir");
            let storage = RuleStorage::new(dir.path().join("rules"));
            
            let rules1 = vec![Rule::domain_suffix("old.com".to_string()), Rule::match_direct()];
            storage.save_current(&rules1).expect("save");
            storage.backup_current().expect("backup");
            
            let rules2 = vec![Rule::domain_suffix("new.com".to_string()), Rule::match_direct()];
            storage.save_current(&rules2).expect("save");
            
            let rolled_back = storage.rollback().expect("rollback");
            assert!(rolled_back);
            
            let restored = storage.load_current().expect("load");
            assert!(restored.iter().any(|r| r.domain == "old.com"));
        }

        #[test]
        fn rollback_returns_false_if_no_previous() {
            let dir = tempdir().expect("tempdir");
            let storage = RuleStorage::new(dir.path().join("rules"));
            
            let rolled_back = storage.rollback().expect("rollback");
            assert!(!rolled_back);
        }

        #[test]
        fn load_current_returns_empty_if_no_file() {
            let dir = tempdir().expect("tempdir");
            let storage = RuleStorage::new(dir.path().join("rules"));
            
            let rules = storage.load_current().expect("load");
            assert!(rules.is_empty());
        }

        #[test]
        fn load_current_roundtrip() {
            let dir = tempdir().expect("tempdir");
            let storage = RuleStorage::new(dir.path().join("rules"));
            
            let original = vec![
                Rule::domain_suffix("github.com".to_string()),
                Rule::domain_exact("test.com".to_string()),
                Rule::match_direct(),
            ];
            
            storage.save_current(&original).expect("save");
            let loaded = storage.load_current().expect("load");
            
            assert_eq!(loaded.len(), 3);
            assert_eq!(loaded[0].rule_type, "DOMAIN-SUFFIX");
            assert_eq!(loaded[0].domain, "github.com");
            assert_eq!(loaded[1].rule_type, "DOMAIN");
            assert_eq!(loaded[1].domain, "test.com");
            assert_eq!(loaded[2].rule_type, "MATCH");
        }

        #[test]
        fn rules_to_yaml_format() {
            let rules = vec![
                Rule::domain_suffix("example.com".to_string()),
                Rule::match_direct(),
            ];
            
            let yaml = RuleStorage::rules_to_yaml(&rules);
            assert!(yaml.starts_with("rules:\n"));
            assert!(yaml.contains("  - DOMAIN-SUFFIX,example.com,PROXY\n"));
            assert!(yaml.contains("  - MATCH,DIRECT\n"));
        }

        #[test]
        fn yaml_to_rules_parses_correctly() {
            let yaml = "rules:\n  - DOMAIN-SUFFIX,github.com,PROXY\n  - MATCH,DIRECT\n";
            let rules = RuleStorage::yaml_to_rules(yaml);
            
            assert_eq!(rules.len(), 2);
            assert_eq!(rules[0].rule_type, "DOMAIN-SUFFIX");
            assert_eq!(rules[0].domain, "github.com");
            assert_eq!(rules[1].rule_type, "MATCH");
        }

        #[test]
        fn parse_rule_line_match() {
            let rule = RuleStorage::parse_rule_line("MATCH,DIRECT");
            assert!(rule.is_some());
            let r = rule.expect("rule");
            assert_eq!(r.rule_type, "MATCH");
            assert_eq!(r.policy, "DIRECT");
        }

        #[test]
        fn parse_rule_line_domain_suffix() {
            let rule = RuleStorage::parse_rule_line("DOMAIN-SUFFIX,github.com,PROXY");
            assert!(rule.is_some());
            let r = rule.expect("rule");
            assert_eq!(r.rule_type, "DOMAIN-SUFFIX");
            assert_eq!(r.domain, "github.com");
            assert_eq!(r.policy, "PROXY");
        }

        #[test]
        fn parse_rule_line_invalid_returns_none() {
            assert!(RuleStorage::parse_rule_line("").is_none());
            assert!(RuleStorage::parse_rule_line("INVALID").is_none());
        }

        #[test]
        fn storage_default() {
            let storage = RuleStorage::default();
            assert_eq!(storage.rules_dir, PathBuf::from("data/rules"));
        }
    }
}
use crate::services::rule_generator::Rule;
use std::fmt::Write;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Writes rules in mihomo ruleset payload format to the mihomo config directory.
///
/// Mihomo expects ruleset files (referenced by `rule-providers` in config.yaml) to use
/// the `payload:` format without policy fields:
///
/// ```yaml
/// payload:
///   - DOMAIN-SUFFIX,github.com
///   - DOMAIN-SUFFIX,api.github.com
/// ```
///
/// The policy (PROXY/DIRECT) is determined by the `rules:` section in config.yaml,
/// not by the ruleset file itself.
#[derive(Clone)]
pub struct RulesetWriter {
    ruleset_dir: PathBuf,
    config_dir: PathBuf,
}

impl RulesetWriter {
    #[must_use]
    pub fn new(config_dir: &std::path::Path) -> Self {
        Self {
            ruleset_dir: config_dir.join("ruleset"),
            config_dir: config_dir.to_path_buf(),
        }
    }

    /// Write proxy rules to `custom-proxy.yaml` using atomic write.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if directory creation, file write, or rename fails.
    pub fn write_proxy_ruleset(&self, rules: &[Rule]) -> io::Result<()> {
        let payload = Self::rules_to_payload(rules);
        self.atomic_write("custom-proxy.yaml", &payload)
    }

    /// Write per-site ruleset file: `site-{site_id}.yaml`.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if directory creation, file write, or rename fails.
    pub fn write_site_ruleset(&self, site_id: &str, rules: &[Rule]) -> io::Result<()> {
        let payload = Self::rules_to_payload(rules);
        self.atomic_write(&format!("site-{site_id}.yaml"), &payload)
    }

    /// Remove per-site ruleset files for sites no longer in the active list.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if directory listing fails.
    pub fn cleanup_site_rulesets(&self, active_site_ids: &[String]) -> io::Result<()> {
        let entries = fs::read_dir(&self.ruleset_dir)?;
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if let Some(site_id) = name_str.strip_prefix("site-").and_then(|s| s.strip_suffix(".yaml")) {
                if !active_site_ids.iter().any(|id| id == site_id) {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
        Ok(())
    }

    /// Convert rules to mihomo ruleset payload format.
    /// Strips the policy field and skips MATCH rules.
    fn rules_to_payload(rules: &[Rule]) -> String {
        let mut yaml = String::from("payload:\n");
        for rule in rules {
            // MATCH rules don't belong in rulesets
            if rule.rule_type == "MATCH" {
                continue;
            }
            // DOMAIN-SUFFIX,github.com,PROXY → DOMAIN-SUFFIX,github.com
            let _ = writeln!(yaml, "  - {},{}", rule.rule_type, rule.domain);
        }
        yaml
    }

    /// Atomic write: write to temp file, then rename.
    fn atomic_write(&self, filename: &str, content: &str) -> io::Result<()> {
        fs::create_dir_all(&self.ruleset_dir)?;
        let target = self.ruleset_dir.join(filename);
        let temp = target.with_extension("yaml.tmp");
        fs::write(&temp, content)?;
        fs::rename(&temp, &target)?;
        Ok(())
    }

    /// Rewrite the `rules:` section in mihomo's config.yaml.
    ///
    /// Removes hardcoded rule-set references (github, github-ip) and routes
    /// `custom-proxy` through the `PROXY` group. Falls back to `MATCH,DIRECT`
    /// so only GoGuo-managed sites are proxied.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if config.yaml cannot be read, parsed, or written.
    pub fn update_config_rules(&self, has_proxy_rules: bool) -> io::Result<()> {
        let config_path = self.config_dir.join("config.yaml");
        let content = fs::read_to_string(&config_path)?;

        // Find the start of the rules section — always at top level (no indent)
        let prefix = if let Some(idx) = content.find("\nrules:") {
            &content[..=idx]
        } else if content.starts_with("rules:") {
            ""
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No 'rules:' section found in config.yaml",
            ));
        };

        let mut new_content = prefix.to_string();
        new_content.push_str("rules:\n");
        new_content.push_str("  - RULE-SET,custom-direct,DIRECT\n");
        new_content.push_str("  - RULE-SET,custom-block,REJECT\n");
        if has_proxy_rules {
            new_content.push_str("  - RULE-SET,custom-proxy,PROXY\n");
        }
        new_content.push_str("  - GEOIP,CN,DIRECT\n");
        new_content.push_str("  - MATCH,DIRECT\n");

        // Atomic write config.yaml
        let tmp_path = config_path.with_extension("yaml.tmp");
        fs::write(&tmp_path, &new_content)?;
        fs::rename(&tmp_path, &config_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_to_payload_basic() {
        let rules = vec![
            Rule::domain_suffix("github.com".to_string()),
            Rule::domain_suffix("api.github.com".to_string()),
        ];
        let yaml = RulesetWriter::rules_to_payload(&rules);
        assert_eq!(yaml, "payload:\n  - DOMAIN-SUFFIX,github.com\n  - DOMAIN-SUFFIX,api.github.com\n");
    }

    #[test]
    fn rules_to_payload_skips_match() {
        let rules = vec![
            Rule::domain_suffix("github.com".to_string()),
            Rule::match_direct(),
        ];
        let yaml = RulesetWriter::rules_to_payload(&rules);
        assert_eq!(yaml, "payload:\n  - DOMAIN-SUFFIX,github.com\n");
    }

    #[test]
    fn rules_to_payload_empty() {
        let rules: Vec<Rule> = vec![];
        let yaml = RulesetWriter::rules_to_payload(&rules);
        assert_eq!(yaml, "payload:\n");
    }

    #[test]
    fn rules_to_payload_only_match() {
        let rules = vec![Rule::match_direct()];
        let yaml = RulesetWriter::rules_to_payload(&rules);
        assert_eq!(yaml, "payload:\n");
    }

    #[test]
    fn rules_to_payload_domain_exact() {
        let rules = vec![Rule::domain_exact("exact.example.com".to_string())];
        let yaml = RulesetWriter::rules_to_payload(&rules);
        assert_eq!(yaml, "payload:\n  - DOMAIN,exact.example.com\n");
    }

    #[test]
    fn write_proxy_ruleset_creates_file() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let writer = RulesetWriter::new(dir.path());

        let rules = vec![Rule::domain_suffix("github.com".to_string())];
        writer.write_proxy_ruleset(&rules).expect("write");

        let content = fs::read_to_string(dir.path().join("ruleset").join("custom-proxy.yaml"))
            .expect("read");
        assert!(content.contains("DOMAIN-SUFFIX,github.com"));
    }

    #[test]
    fn write_proxy_ruleset_atomic_no_tmp_left() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let writer = RulesetWriter::new(dir.path());

        let rules = vec![Rule::domain_suffix("test.com".to_string())];
        writer.write_proxy_ruleset(&rules).expect("write");

        // No .tmp file should remain
        assert!(!dir.path().join("ruleset/custom-proxy.yaml.tmp").exists());
        // Target file should exist
        assert!(dir.path().join("ruleset/custom-proxy.yaml").exists());
    }

    #[test]
    fn write_proxy_ruleset_overwrites() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let writer = RulesetWriter::new(dir.path());

        writer
            .write_proxy_ruleset(&[Rule::domain_suffix("first.com".to_string())])
            .expect("write 1");
        writer
            .write_proxy_ruleset(&[Rule::domain_suffix("second.com".to_string())])
            .expect("write 2");

        let content = fs::read_to_string(dir.path().join("ruleset/custom-proxy.yaml"))
            .expect("read");
        assert!(content.contains("second.com"));
        assert!(!content.contains("first.com"));
    }
}

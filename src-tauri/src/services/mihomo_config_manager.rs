use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Manages mihomo's config.yaml dynamic sections (rule-providers, proxy-groups, rules).
///
/// The static sections (general settings, DNS, proxies) are preserved verbatim.
/// Only the three dynamic sections are regenerated based on active sites.
#[derive(Clone)]
pub struct MihomoConfigManager {
    config_path: PathBuf,
    /// Cached sections from the original config.yaml
    prefix: String,            // Everything before rule-providers (general + dns)
    proxies_content: String,   // The entire proxies: section verbatim
    proxy_names: Vec<String>,  // Extracted proxy node names
}

impl MihomoConfigManager {
    /// Open and parse the given config.yaml.
    ///
    /// Returns a fully initialized manager ready for `regenerate()`.
    /// This is the only way to create a `MihomoConfigManager` — the parsed
    /// state is guaranteed to be valid.
    ///
    /// On first successful parse, saves a `.orig` backup of the original file
    /// to allow recovery if a subsequent `regenerate()` produces invalid output.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if the file cannot be read or required sections are missing.
    pub fn open(config_path: PathBuf) -> io::Result<Self> {
        let content = fs::read_to_string(&config_path)?;

        // Save .orig backup on first parse (only if backup doesn't already exist)
        let orig_path = config_path.with_extension("yaml.orig");
        if !orig_path.exists() {
            let _ = fs::copy(&config_path, &orig_path);
        }

        // Extract prefix: everything before "rule-providers:"
        let rp_start = content
            .find("\nrule-providers:")
            .or_else(|| {
                if content.starts_with("rule-providers:") {
                    Some(0)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "No 'rule-providers:' section in config.yaml")
            })?;

        let prefix = if rp_start == 0 {
            String::new()
        } else {
            let raw_prefix = content[..=rp_start].to_string();
            // Strip any existing hosts: section to avoid duplicates on regeneration
            Self::strip_hosts_section(&raw_prefix)
        };

        // Extract proxies section: from "proxies:" to "proxy-groups:"
        let proxies_start = content
            .find("\nproxies:")
            .map(|i| i + 1) // skip the leading newline
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "No 'proxies:' section in config.yaml")
            })?;

        let proxies_end = content
            .find("\nproxy-groups:")
            .or_else(|| content.find("\nrules:"))
            .unwrap_or(content.len());

        let proxies_content = content[proxies_start..proxies_end].to_string();

        // Extract proxy node names from lines matching "- name: ..."
        let proxy_names = Self::extract_proxy_names(&proxies_content);

        Ok(Self {
            config_path,
            prefix,
            proxies_content,
            proxy_names,
        })
    }

    /// Regenerate config.yaml with per-site proxy groups for the given active sites.
    ///
    /// Each `(site_id, health_check_url)` pair generates:
    /// - A `site-{id}` url-test proxy group
    /// - A `site-{id}` rule-provider referencing `./ruleset/site-{id}.yaml`
    /// - A `RULE-SET,site-{id},site-{id}` rule entry
    ///
    /// For `IpDirect` sites, additionally generates:
    /// - A `hosts:` section mapping verified domains to their IPs
    /// - Inline `DOMAIN-SUFFIX,domain,DIRECT` rules before RULE-SET entries
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if file writing fails or if critical parsed sections
    /// are empty (indicating the original config was already corrupted).
    pub fn regenerate(
        &self,
        active_sites: &[(String, String)],
        ip_direct_hosts: &HashMap<String, String>,
        direct_domains: &[String],
    ) -> io::Result<()> {
        // Guard: refuse to overwrite if we couldn't extract proxy nodes.
        // This prevents writing a broken config that loses all proxy definitions.
        if self.proxies_content.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Cannot regenerate: proxies section is empty (config may be corrupted). \
                 Restore from .orig backup or re-import subscription.",
            ));
        }
        let mut new_content = self.prefix.clone();

        // Section: hosts (only if we have IpDirect mappings)
        if !ip_direct_hosts.is_empty() {
            // Ensure prefix ends with newline before appending hosts section
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str(&Self::generate_hosts(ip_direct_hosts));
            new_content.push('\n');
        }

        // Section: rule-providers
        new_content.push_str(&Self::generate_rule_providers(active_sites));
        new_content.push('\n');

        // Section: proxies (verbatim)
        new_content.push_str(&self.proxies_content);
        new_content.push('\n');

        // Section: proxy-groups
        new_content.push_str(&Self::generate_proxy_groups(active_sites, &self.proxy_names));
        new_content.push('\n');

        // Section: rules (with inline DIRECT rules for IpDirect domains)
        new_content.push_str(&Self::generate_rules(active_sites, direct_domains));
        new_content.push('\n');

        // Atomic write
        let tmp_path = self.config_path.with_extension("yaml.tmp");
        fs::write(&tmp_path, &new_content)?;
        fs::rename(&tmp_path, &self.config_path)?;
        Ok(())
    }

    /// Extract proxy node names from the proxies section.
    /// Handles both inline (`- name: Foo`) and multi-line (`-\n    name: Foo`) YAML formats,
    /// with or without surrounding quotes.
    fn extract_proxy_names(proxies_content: &str) -> Vec<String> {
        let mut names = Vec::new();
        for line in proxies_content.lines() {
            let trimmed = line.trim();
            // Match both "- name: Foo" and "name: Foo" (multi-line list item)
            let rest = trimmed
                .strip_prefix("- name:")
                .or_else(|| trimmed.strip_prefix("name:"))
                .map(str::trim);
            if let Some(name) = rest {
                // Remove surrounding quotes if present
                let name = if (name.starts_with('"') && name.ends_with('"'))
                    || (name.starts_with('\'') && name.ends_with('\''))
                {
                    &name[1..name.len() - 1]
                } else {
                    name
                };
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
        }
        names
    }

    /// Generate the rule-providers section.
    fn generate_rule_providers(sites: &[(String, String)]) -> String {
        let mut yaml = String::from("rule-providers:\n");
        yaml.push_str("  custom-direct:\n");
        yaml.push_str("    type: file\n");
        yaml.push_str("    behavior: classical\n");
        yaml.push_str("    path: ./ruleset/custom-direct.yaml\n");
        yaml.push_str("  custom-block:\n");
        yaml.push_str("    type: file\n");
        yaml.push_str("    behavior: classical\n");
        yaml.push_str("    path: ./ruleset/custom-block.yaml\n");

        for (site_id, _) in sites {
            let _ = writeln!(yaml, "  site-{site_id}:");
            let _ = writeln!(yaml, "    type: file");
            let _ = writeln!(yaml, "    behavior: classical");
            let _ = writeln!(yaml, "    path: ./ruleset/site-{site_id}.yaml");
        }
        yaml
    }

    /// Generate the proxy-groups section with per-site url-test groups.
    fn generate_proxy_groups(sites: &[(String, String)], proxy_names: &[String]) -> String {
        let mut yaml = String::from("proxy-groups:\n");

        // Base PROXY group (select, DIRECT only as fallback)
        yaml.push_str("  - name: PROXY\n    type: select\n    proxies:\n");
        yaml.push_str("      - DIRECT\n");

        // Per-site url-test groups
        for (site_id, health_url) in sites {
            let _ = writeln!(yaml, "  - name: site-{site_id}");
            let _ = writeln!(yaml, "    type: url-test");
            let _ = writeln!(yaml, "    tolerance: 100");
            let _ = writeln!(yaml, "    interval: 60");
            let _ = writeln!(yaml, "    timeout: 2000");
            let _ = writeln!(yaml, "    lazy: false");
            let _ = writeln!(yaml, "    url: {health_url}");
            yaml.push_str("    proxies:\n");
            for name in proxy_names {
                let _ = writeln!(yaml, "      - \"{name}\"");
            }
        }
        yaml
    }

    /// Generate the rules section.
    /// Includes inline DIRECT rules for `IpDirect` domains before RULE-SET entries.
    fn generate_rules(sites: &[(String, String)], direct_domains: &[String]) -> String {
        let mut yaml = String::from("rules:\n");
        yaml.push_str("  - RULE-SET,custom-direct,DIRECT\n");
        yaml.push_str("  - RULE-SET,custom-block,REJECT\n");

        // Inline DIRECT rules for IpDirect domains (before RULE-SET for priority matching)
        for domain in direct_domains {
            let _ = writeln!(yaml, "  - DOMAIN-SUFFIX,{domain},DIRECT");
        }

        for (site_id, _) in sites {
            let _ = writeln!(yaml, "  - RULE-SET,site-{site_id},site-{site_id}");
        }

        yaml.push_str("  - GEOIP,CN,DIRECT\n");
        yaml.push_str("  - MATCH,DIRECT\n");
        yaml
    }

    /// Generate the hosts section for `IpDirect` sites.
    fn generate_hosts(ip_direct_hosts: &HashMap<String, String>) -> String {
        if ip_direct_hosts.is_empty() {
            return String::new();
        }
        let mut yaml = String::from("hosts:\n");
        // Sort for deterministic output
        let mut entries: Vec<_> = ip_direct_hosts.iter().collect();
        entries.sort_by_key(|(d, _)| *d);
        for (domain, ip) in entries {
            let _ = writeln!(yaml, "  '{domain}': \"{ip}\"");
        }
        yaml
    }

    /// Strip any existing hosts: section from the prefix to avoid duplicates.
    ///
    /// Handles both properly formatted (`\nhosts:`) and malformed (`CNhosts:` glued)
    /// cases. After stripping, the result always ends with a newline.
    #[allow(clippy::option_if_let_else)]
    fn strip_hosts_section(prefix: &str) -> String {
        // Find "hosts:" as a YAML top-level key.
        // Try proper format first (newline before hosts:), then fallback to glued format.
        let hosts_start = prefix
            .find("\nhosts:")
            .or_else(|| prefix.find("hosts:\n").filter(|&pos| pos == 0 || !prefix[..pos].ends_with('\n')));

        if let Some(pos) = hosts_start {
            let after_hosts = &prefix[pos..];
            let skip = usize::from(after_hosts.starts_with('\n'));
            let content_after_key = &after_hosts[skip..]; // skip the \n or start at "hosts:"

            let end = content_after_key
                .lines()
                .skip(1) // skip the "hosts:" line itself
                .position(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty()
                        && !trimmed.starts_with('#')
                        && !line.starts_with(' ')
                        && !line.starts_with('\t')
                        && trimmed.contains(':')
                })
                .map(|i| {
                    let mut offset = 0;
                    for (idx, line) in content_after_key.lines().enumerate() {
                        if idx == i + 1 {
                            break;
                        }
                        offset += line.len() + 1;
                    }
                    pos + skip + offset
                });

            let result = end.map_or_else(
                || prefix[..pos].to_string(),
                |end_pos| {
                    let mut r = prefix[..pos].to_string();
                    r.push_str(&prefix[end_pos..]);
                    r
                },
            );

            // Ensure result ends with newline for correct subsequent section appending
            if result.ends_with('\n') { result } else { result + "\n" }
        } else if prefix.starts_with("hosts:") {
            let end = prefix
                .lines()
                .skip(1)
                .position(|line| !line.starts_with(' ') && !line.starts_with('\t') && !line.starts_with('#') && line.contains(':'))
                .map_or(prefix.len(), |i| prefix.lines().take(i + 1).map(|l| l.len() + 1).sum::<usize>());
            prefix[end..].to_string()
        } else {
            prefix.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn minimal_config() -> &'static str {
        r#"tcp-concurrent: true
mixed-port: 7890
mode: rule
dns:
  enable: true
  nameserver:
    - 223.5.5.5
rule-providers:
  custom-direct:
    type: file
    behavior: classical
    path: ./ruleset/custom-direct.yaml
proxies:
  - name: SS-Node-A
    type: ss
    server: 1.2.3.4
    port: 443
  - name: "VMESS-Node-B"
    type: vmess
    server: 5.6.7.8
    port: 8080
  - name: "|ripaojiedian-1"
    type: trojan
    server: 9.10.11.12
    port: 443
proxy-groups:
  - name: PROXY
    type: select
    proxies:
      - DIRECT
rules:
  - MATCH,DIRECT
"#
    }

    #[test]
    fn open_extracts_proxy_names() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mgr = MihomoConfigManager::open(path).unwrap();

        assert_eq!(mgr.proxy_names, vec![
            "SS-Node-A",
            "VMESS-Node-B",
            "|ripaojiedian-1",
        ]);
    }

    #[test]
    fn open_preserves_prefix() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mgr = MihomoConfigManager::open(path).unwrap();

        assert!(mgr.prefix.contains("tcp-concurrent: true"));
        assert!(mgr.prefix.contains("mixed-port: 7890"));
        assert!(mgr.prefix.contains("dns:"));
        assert!(!mgr.prefix.contains("rule-providers:"));
    }

    #[test]
    fn open_preserves_proxies_content() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mgr = MihomoConfigManager::open(path).unwrap();

        assert!(mgr.proxies_content.starts_with("proxies:"));
        assert!(mgr.proxies_content.contains("SS-Node-A"));
        assert!(mgr.proxies_content.contains("VMESS-Node-B"));
        assert!(!mgr.proxies_content.contains("proxy-groups:"));
    }

    #[test]
    fn regenerate_no_active_sites() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mgr = MihomoConfigManager::open(path.clone()).unwrap();
        mgr.regenerate(&[], &HashMap::new(), &[]).unwrap();

        let result = fs::read_to_string(&path).unwrap();
        assert!(result.contains("rule-providers:"));
        assert!(result.contains("custom-direct:"));
        assert!(result.contains("proxies:"));
        assert!(result.contains("SS-Node-A"));
        assert!(result.contains("GEOIP,CN,DIRECT"));
        assert!(result.contains("MATCH,DIRECT"));
        assert!(!result.contains("site-"));
    }

    #[test]
    fn regenerate_single_site() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mgr = MihomoConfigManager::open(path.clone()).unwrap();

        let sites = vec![("github".to_string(), "https://github.com".to_string())];
        mgr.regenerate(&sites, &HashMap::new(), &[]).unwrap();

        let result = fs::read_to_string(&path).unwrap();
        assert!(result.contains("site-github"));
        assert!(result.contains("url: https://github.com"));
        assert!(result.contains("RULE-SET,site-github,site-github"));
        assert!(result.contains("path: ./ruleset/site-github.yaml"));
        // All nodes should be in the site-github group
        assert!(result.contains("\"SS-Node-A\""));
        assert!(result.contains("\"VMESS-Node-B\""));
    }

    #[test]
    fn regenerate_multiple_sites() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mgr = MihomoConfigManager::open(path.clone()).unwrap();

        let sites = vec![
            ("github".to_string(), "https://github.com".to_string()),
            ("google".to_string(), "https://www.google.com".to_string()),
        ];
        mgr.regenerate(&sites, &HashMap::new(), &[]).unwrap();

        let result = fs::read_to_string(&path).unwrap();
        assert!(result.contains("site-github"));
        assert!(result.contains("site-google"));
        assert!(result.contains("url: https://github.com"));
        assert!(result.contains("url: https://www.google.com"));
    }

    #[test]
    fn regenerate_preserves_dns_and_general() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mgr = MihomoConfigManager::open(path.clone()).unwrap();

        let sites = vec![("github".to_string(), "https://github.com".to_string())];
        mgr.regenerate(&sites, &HashMap::new(), &[]).unwrap();

        let result = fs::read_to_string(&path).unwrap();
        assert!(result.contains("tcp-concurrent: true"));
        assert!(result.contains("mixed-port: 7890"));
        assert!(result.contains("dns:"));
        assert!(result.contains("223.5.5.5"));
    }

    #[test]
    fn regenerate_preserves_proxies_verbatim() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        let original = minimal_config();
        fs::write(&path, original).unwrap();

        let mgr = MihomoConfigManager::open(path.clone()).unwrap();

        let sites = vec![("github".to_string(), "https://github.com".to_string())];
        mgr.regenerate(&sites, &HashMap::new(), &[]).unwrap();

        let result = fs::read_to_string(&path).unwrap();
        // Each proxy node's definition should be preserved
        assert!(result.contains("server: 1.2.3.4"));
        assert!(result.contains("server: 5.6.7.8"));
        assert!(result.contains("type: ss"));
        assert!(result.contains("type: vmess"));
        assert!(result.contains("type: trojan"));
    }

    #[test]
    fn regenerate_atomic_no_tmp_left() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mgr = MihomoConfigManager::open(path.clone()).unwrap();
        mgr.regenerate(&[], &HashMap::new(), &[]).unwrap();

        assert!(!path.with_extension("yaml.tmp").exists());
    }

    #[test]
    fn extract_proxy_names_handles_quoted() {
        let content = "proxies:\n  - name: \"My Node\"\n  - name: 'Another'\n  - name: Plain\n";
        let names = MihomoConfigManager::extract_proxy_names(content);
        assert_eq!(names, vec!["My Node", "Another", "Plain"]);
    }

    #[test]
    fn extract_proxy_names_multiline_yaml() {
        // Multi-line format: "-\n    name: Foo" (common in subscription-generated configs)
        let content = "proxies:\n  -\n    name: SS-Node-A\n    type: ss\n  -\n    name: \"VMESS-Node-B\"\n    type: vmess\n";
        let names = MihomoConfigManager::extract_proxy_names(content);
        assert_eq!(names, vec!["SS-Node-A", "VMESS-Node-B"]);
    }

    #[test]
    fn extract_proxy_names_empty() {
        let content = "proxies:\n";
        let names = MihomoConfigManager::extract_proxy_names(content);
        assert!(names.is_empty());
    }

    #[test]
    fn open_missing_rule_providers_fails() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, "mixed-port: 7890\nproxies:\n  - name: A\n    type: ss\n").unwrap();

        assert!(MihomoConfigManager::open(path).is_err());
    }

    #[test]
    fn open_missing_proxies_fails() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, "mixed-port: 7890\nrule-providers:\n  x:\n    type: file\nrules:\n  - MATCH,DIRECT\n").unwrap();

        assert!(MihomoConfigManager::open(path).is_err());
    }

    #[test]
    fn proxy_groups_all_nodes_included() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mgr = MihomoConfigManager::open(path.clone()).unwrap();

        let sites = vec![("test".to_string(), "https://example.com".to_string())];
        mgr.regenerate(&sites, &HashMap::new(), &[]).unwrap();

        let result = fs::read_to_string(&path).unwrap();
        // Each of the 3 proxy nodes should appear in the site-test group
        let site_section = result.split("name: site-test").nth(1).unwrap();
        assert!(site_section.contains("\"SS-Node-A\""));
        assert!(site_section.contains("\"VMESS-Node-B\""));
        assert!(site_section.contains("\"|ripaojiedian-1\""));
    }

    #[test]
    fn hosts_section_has_proper_newlines() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mgr = MihomoConfigManager::open(path.clone()).unwrap();

        let mut hosts = HashMap::new();
        hosts.insert("github.com".to_string(), "140.82.112.4".to_string());
        hosts.insert("api.github.com".to_string(), "140.82.114.5".to_string());

        let sites = vec![("github".to_string(), "https://github.com".to_string())];
        let direct = vec!["github.com".to_string(), "api.github.com".to_string()];
        mgr.regenerate(&sites, &hosts, &direct).unwrap();

        let result = fs::read_to_string(&path).unwrap();

        // hosts: must be on its own line (not glued to previous content)
        assert!(
            !result.contains("CNhosts:"),
            "hosts: must be separated from previous section by newline"
        );
        assert!(result.contains("\nhosts:\n"), "hosts: must start on new line");

        // Verify host entries are present
        assert!(result.contains("'api.github.com': \"140.82.114.5\""));
        assert!(result.contains("'github.com': \"140.82.112.4\""));

        // Verify inline DIRECT rules
        assert!(result.contains("DOMAIN-SUFFIX,github.com,DIRECT"));
        assert!(result.contains("DOMAIN-SUFFIX,api.github.com,DIRECT"));
    }

    #[test]
    fn regenerate_twice_no_duplicate_hosts() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, minimal_config()).unwrap();

        let mut hosts = HashMap::new();
        hosts.insert("github.com".to_string(), "140.82.112.4".to_string());
        let sites = vec![("github".to_string(), "https://github.com".to_string())];
        let direct = vec!["github.com".to_string()];

        // First regeneration
        let mgr = MihomoConfigManager::open(path.clone()).unwrap();
        mgr.regenerate(&sites, &hosts, &direct).unwrap();

        // Second regeneration (simulates restart: re-open then regenerate)
        let mgr2 = MihomoConfigManager::open(path.clone()).unwrap();
        mgr2.regenerate(&sites, &hosts, &direct).unwrap();

        let result = fs::read_to_string(&path).unwrap();

        // Count occurrences of "hosts:" — must be exactly 1
        let count = result.lines().filter(|l| *l == "hosts:").count();
        assert_eq!(count, 1, "Expected exactly 1 hosts: section, found {count}");

        // Count occurrences of each domain mapping — must be exactly 1
        let github_count = result.matches("'github.com':").count();
        assert_eq!(github_count, 1, "Expected exactly 1 github.com host entry, found {github_count}");
    }
}

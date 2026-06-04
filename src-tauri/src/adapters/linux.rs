//! Linux platform adapter for reading and writing network configuration state.
//!
//! Covers 6 Linux state items:
//! - Restorable (4): linux-proxy-env, linux-git-proxy, linux-resolv-conf,
//!   linux-etc-environment (write supported)
//! - Detectable (2): linux-shell-proxy, linux-reachability (read-only)

use std::path::Path;

use crate::adapters::linux_base::{LinuxBaseAdapter, ShellExecutor, WritePermission};
use crate::adapters::{PlatformAdapter, StateItemDefinition};
use crate::models::baseline::{Platform, StateItem, StateItemCategory};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// State item IDs
const ID_PROXY_ENV: &str = "linux-proxy-env";
const ID_GIT_PROXY: &str = "linux-git-proxy";
const ID_RESOLV_CONF: &str = "linux-resolv-conf";
const ID_ETC_ENVIRONMENT: &str = "linux-etc-environment";
const ID_SHELL_PROXY: &str = "linux-shell-proxy";
const ID_REACHABILITY: &str = "linux-reachability";

const RESOLV_CONF_PATH: &str = "/etc/resolv.conf";
const ETC_ENVIRONMENT_PATH: &str = "/etc/environment";

// ---------------------------------------------------------------------------
// Struct
// ---------------------------------------------------------------------------

/// Platform adapter for native Linux network configuration state.
///
/// Generic over `ShellExecutor` to enable dependency injection in tests.
#[allow(private_bounds)] // ShellExecutor is pub(crate) by design; adapter is re-exported publicly
pub struct LinuxAdapter<E: ShellExecutor = super::linux_base::SystemShellExecutor> {
    base: LinuxBaseAdapter<E>,
}

#[allow(clippy::similar_names)] // http_proxy/https_proxy are domain-standard names
#[allow(private_bounds)] // ShellExecutor is pub(crate) by design; adapter is re-exported publicly
impl<E: ShellExecutor> LinuxAdapter<E> {
    /// Create a new `LinuxAdapter` with the given shell executor.
    #[must_use]
    #[allow(dead_code)] // Used by downstream consumers and tests
    pub const fn new(executor: E) -> Self {
        Self {
            base: LinuxBaseAdapter::new(executor),
        }
    }

    /// Build a timestamp string for "now".
    fn now_iso() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    /// Build an error `StateItem` for a given id.
    #[allow(dead_code)] // Reserved for graceful error handling in future read operations
    fn error_item(id: &str, err: &str) -> StateItem {
        StateItem {
            id: id.to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Detectable,
            value: serde_json::json!({ "error": err }),
            collected_at: Self::now_iso(),
            classification_reason: "Collection failed".to_string(),
        }
    }

    fn push_restorable_items(&self, items: &mut Vec<StateItem>, now: &str) {
        // linux-proxy-env (Excluded from baseline — managed by service lifecycle)
        items.push(StateItem {
            id: ID_PROXY_ENV.to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Excluded,
            value: self.base.read_proxy_env_vars(),
            collected_at: now.to_string(),
            classification_reason: "Service lifecycle overlay, not baseline-managed".to_string(),
        });

        // linux-git-proxy
        items.push(StateItem {
            id: ID_GIT_PROXY.to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Restorable,
            value: self.base.read_git_proxy(),
            collected_at: now.to_string(),
            classification_reason: "Git config, writable".to_string(),
        });

        // linux-resolv-conf
        items.push(StateItem {
            id: ID_RESOLV_CONF.to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Restorable,
            value: self.base.read_resolv_conf(Path::new(RESOLV_CONF_PATH)),
            collected_at: now.to_string(),
            classification_reason: "System file, writable with root".to_string(),
        });

        // linux-etc-environment
        items.push(StateItem {
            id: ID_ETC_ENVIRONMENT.to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Restorable,
            value: self.base.read_etc_environment(Path::new(ETC_ENVIRONMENT_PATH)),
            collected_at: now.to_string(),
            classification_reason: "System file, writable with root".to_string(),
        });
    }

    fn push_detectable_items(&self, items: &mut Vec<StateItem>, now: &str) {
        // linux-shell-proxy
        items.push(StateItem {
            id: ID_SHELL_PROXY.to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Detectable,
            value: self.base.read_shell_rc_proxy(),
            collected_at: now.to_string(),
            classification_reason: "Shell RC files, not writable".to_string(),
        });

        // linux-reachability
        items.push(StateItem {
            id: ID_REACHABILITY.to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Detectable,
            value: self.base.check_reachability("https://www.google.com"),
            collected_at: now.to_string(),
            classification_reason: "Network test, not writable".to_string(),
        });
    }
}

#[allow(clippy::similar_names)] // http_proxy/https_proxy are domain-standard names
#[allow(private_bounds)] // ShellExecutor is pub(crate) by design; adapter is re-exported publicly
impl<E: ShellExecutor + Send + Sync> PlatformAdapter for LinuxAdapter<E> {
    fn platform(&self) -> Platform {
        Platform::Linux
    }

    fn state_item_definitions(&self) -> Vec<StateItemDefinition> {
        vec![
            StateItemDefinition {
                id: ID_PROXY_ENV.to_string(),
                category: StateItemCategory::Excluded,
                description: "Proxy environment variables — managed by service lifecycle, not baseline"
                    .to_string(),
            },
            StateItemDefinition {
                id: ID_GIT_PROXY.to_string(),
                category: StateItemCategory::Restorable,
                description: "Git global proxy settings".to_string(),
            },
            StateItemDefinition {
                id: ID_RESOLV_CONF.to_string(),
                category: StateItemCategory::Restorable,
                description: "DNS resolver configuration (/etc/resolv.conf)".to_string(),
            },
            StateItemDefinition {
                id: ID_ETC_ENVIRONMENT.to_string(),
                category: StateItemCategory::Restorable,
                description: "System environment variables (/etc/environment)".to_string(),
            },
            StateItemDefinition {
                id: ID_SHELL_PROXY.to_string(),
                category: StateItemCategory::Detectable,
                description: "Proxy lines in shell RC files (.bashrc, .zshrc)".to_string(),
            },
            StateItemDefinition {
                id: ID_REACHABILITY.to_string(),
                category: StateItemCategory::Detectable,
                description: "Network reachability check".to_string(),
            },
        ]
    }

    fn read_state_items(&self) -> Vec<StateItem> {
        let now = Self::now_iso();
        let mut items = Vec::with_capacity(6);

        self.push_restorable_items(&mut items, &now);
        self.push_detectable_items(&mut items, &now);

        items
    }

    fn write_state(&self, item: &StateItem) -> Result<(), String> {
        match item.id.as_str() {
            ID_PROXY_ENV => {
                let http = item
                    .value
                    .get("http_proxy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let https = item
                    .value
                    .get("https_proxy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let no_proxy = item
                    .value
                    .get("no_proxy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let path = Path::new(ETC_ENVIRONMENT_PATH);
                match self.base.write_proxy_env(path, http, https, no_proxy)? {
                    WritePermission::Granted => Ok(()),
                    WritePermission::NeedRoot { suggested_command } => Err(format!(
                        "Root required to write /etc/environment. Suggested: {suggested_command}"
                    )),
                }
            }
            ID_GIT_PROXY => {
                let http_proxy = item
                    .value
                    .get("http_proxy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let https_proxy = item
                    .value
                    .get("https_proxy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                self.base.write_git_proxy(&http_proxy, &https_proxy)
            }
            ID_RESOLV_CONF => {
                let content = item
                    .value
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'content' field in linux-resolv-conf value".to_string())?;
                let perm = self.base.write_resolv_conf(Path::new(RESOLV_CONF_PATH), content)?;
                match perm {
                    WritePermission::Granted => Ok(()),
                    WritePermission::NeedRoot { suggested_command } => {
                        Err(format!("Root required to write /etc/resolv.conf. Suggested: {suggested_command}"))
                    }
                }
            }
            ID_ETC_ENVIRONMENT => {
                let content = item
                    .value
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'content' field in linux-etc-environment value".to_string())?;
                let perm = self.base.write_etc_environment(Path::new(ETC_ENVIRONMENT_PATH), content)?;
                match perm {
                    WritePermission::Granted => Ok(()),
                    WritePermission::NeedRoot { suggested_command } => {
                        Err(format!("Root required to write /etc/environment. Suggested: {suggested_command}"))
                    }
                }
            }
            _ => Err(format!("Not restorable: {}", item.id)),
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::linux_base::MockShellExecutor;
    use crate::models::baseline::{Platform, StateItem, StateItemCategory};

    // -----------------------------------------------------------------------
    // Helper: build adapter with mock
    // -----------------------------------------------------------------------

    fn make_adapter(mock: MockShellExecutor) -> LinuxAdapter<MockShellExecutor> {
        LinuxAdapter::new(mock)
    }

    // -----------------------------------------------------------------------
    // trait_object_dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn trait_object_dispatch_returns_linux_platform() {
        let adapter: Box<dyn PlatformAdapter> = Box::new(make_adapter(MockShellExecutor::new()));
        assert_eq!(adapter.platform(), Platform::Linux);
    }

    #[test]
    fn trait_object_dispatch_definitions_not_empty() {
        let adapter: Box<dyn PlatformAdapter> = Box::new(make_adapter(MockShellExecutor::new()));
        let defs = adapter.state_item_definitions();
        assert_eq!(defs.len(), 6);
    }

    // -----------------------------------------------------------------------
    // definitions tests
    // -----------------------------------------------------------------------

    #[test]
    fn definitions_has_six_items() {
        let adapter = make_adapter(MockShellExecutor::new());
        let defs = adapter.state_item_definitions();
        assert_eq!(defs.len(), 6);

        let ids: Vec<&str> = defs.iter().map(|d| d.id.as_str()).collect();
        assert!(ids.contains(&ID_PROXY_ENV));
        assert!(ids.contains(&ID_GIT_PROXY));
        assert!(ids.contains(&ID_RESOLV_CONF));
        assert!(ids.contains(&ID_ETC_ENVIRONMENT));
        assert!(ids.contains(&ID_SHELL_PROXY));
        assert!(ids.contains(&ID_REACHABILITY));
    }

    #[test]
    fn definitions_has_three_restorable() {
        let adapter = make_adapter(MockShellExecutor::new());
        let restorable_count = adapter
            .state_item_definitions()
            .iter()
            .filter(|d| d.category == StateItemCategory::Restorable)
            .count();
        assert_eq!(restorable_count, 3);
    }

    #[test]
    fn definitions_has_two_detectable() {
        let adapter = make_adapter(MockShellExecutor::new());
        let detectable_count = adapter
            .state_item_definitions()
            .iter()
            .filter(|d| d.category == StateItemCategory::Detectable)
            .count();
        assert_eq!(detectable_count, 2);
    }

    // -----------------------------------------------------------------------
    // Mock-based read tests
    // -----------------------------------------------------------------------

    #[test]
    fn read_proxy_env_vars_via_mock() {
        let mock = MockShellExecutor::new()
            .with_env_var("http_proxy", "http://proxy:8080")
            .with_env_var("https_proxy", "http://proxy:8443")
            .with_env_var("no_proxy", "localhost,127.0.0.1");
        let adapter = make_adapter(mock);
        let items = adapter.read_state_items();
        let proxy_item = items.iter().find(|i| i.id == ID_PROXY_ENV).expect("proxy-env item");
        assert_eq!(proxy_item.value["http_proxy"], "http://proxy:8080");
        assert_eq!(proxy_item.value["https_proxy"], "http://proxy:8443");
        assert_eq!(proxy_item.value["no_proxy"], "localhost,127.0.0.1");
        assert_eq!(proxy_item.platform, Platform::Linux);
        assert_eq!(proxy_item.category, StateItemCategory::Excluded);
    }

    #[test]
    fn read_git_proxy_via_mock() {
        let mock = MockShellExecutor::new()
            .with_output(
                "git",
                &["config", "--global", "http.proxy"],
                Ok("http://proxy:8080\n".to_string()),
            )
            .with_output(
                "git",
                &["config", "--global", "https.proxy"],
                Ok("http://proxy:8443\n".to_string()),
            );
        let adapter = make_adapter(mock);
        let items = adapter.read_state_items();
        let git_item = items.iter().find(|i| i.id == ID_GIT_PROXY).expect("git-proxy item");
        assert_eq!(git_item.value["http_proxy"], "http://proxy:8080");
        assert_eq!(git_item.value["https_proxy"], "http://proxy:8443");
        assert_eq!(git_item.category, StateItemCategory::Restorable);
    }

    #[test]
    fn read_resolv_conf_from_temp_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("resolv.conf");
        std::fs::write(&path, "nameserver 8.8.8.8\nnameserver 8.8.4.4\n").expect("write");

        // We need a custom adapter that reads from the temp path instead of /etc/resolv.conf.
        // Since read_state_items uses a hardcoded path, we verify via LinuxBaseAdapter directly.
        let mock = MockShellExecutor::new();
        let base = LinuxBaseAdapter::new(mock);
        let result = base.read_resolv_conf(&path);
        let ns = result["nameservers"].as_array().unwrap();
        assert_eq!(ns.len(), 2);
        assert_eq!(ns[0], "8.8.8.8");
        assert_eq!(ns[1], "8.8.4.4");
    }

    #[test]
    fn read_shell_rc_proxy_with_no_home() {
        let mock = MockShellExecutor::new(); // no home configured
        let adapter = make_adapter(mock);
        let items = adapter.read_state_items();
        let shell_item = items.iter().find(|i| i.id == ID_SHELL_PROXY).expect("shell-proxy item");
        assert!(shell_item.value["error"].is_string());
        assert_eq!(shell_item.category, StateItemCategory::Detectable);
    }

    // -----------------------------------------------------------------------
    // write_state rejects detectable items
    // -----------------------------------------------------------------------

    #[test]
    fn write_state_rejects_detectable_item() {
        let adapter = make_adapter(MockShellExecutor::new());
        let item = StateItem {
            id: ID_SHELL_PROXY.to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Detectable,
            value: serde_json::json!({}),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        let result = adapter.write_state(&item);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("Not restorable"),
            "Expected 'Not restorable' error message"
        );
    }

    // -----------------------------------------------------------------------
    // write_state rejects unknown items
    // -----------------------------------------------------------------------

    #[test]
    fn write_state_rejects_unknown_item() {
        let adapter = make_adapter(MockShellExecutor::new());
        let item = StateItem {
            id: "linux-unknown".to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({}),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        let result = adapter.write_state(&item);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("Not restorable"),
            "Expected 'Not restorable' error for unknown item"
        );
    }

    // -----------------------------------------------------------------------
    // Root permission degradation
    // -----------------------------------------------------------------------

    #[test]
    fn write_resolv_conf_reports_root_required_for_system_path() {
        // Use LinuxBaseAdapter directly since LinuxAdapter hardcodes the path
        let mock = MockShellExecutor::new();
        let base = LinuxBaseAdapter::new(mock);
        let result = base.write_resolv_conf(Path::new("/etc/resolv.conf"), "nameserver 8.8.8.8\n");
        // The result should be Ok(WritePermission) — either Granted or NeedRoot
        assert!(result.is_ok(), "write_resolv_conf should not return Err for permission check");
        let perm = result.unwrap();
        match perm {
            WritePermission::Granted => {
                // Acceptable if test is running as root
            }
            WritePermission::NeedRoot { suggested_command } => {
                assert!(suggested_command.contains("sudo"));
            }
        }
    }

    #[test]
    fn write_etc_environment_reports_root_required_for_system_path() {
        let mock = MockShellExecutor::new();
        let base = LinuxBaseAdapter::new(mock);
        let result = base.write_etc_environment(Path::new("/etc/environment"), "http_proxy=http://proxy:8080\n");
        assert!(result.is_ok(), "write_etc_environment should not return Err for permission check");
        let perm = result.unwrap();
        match perm {
            WritePermission::Granted => {
                // Acceptable if test is running as root
            }
            WritePermission::NeedRoot { suggested_command } => {
                assert!(suggested_command.contains("sudo"));
            }
        }
    }

    // -----------------------------------------------------------------------
    // read_state_items integration
    // -----------------------------------------------------------------------

    #[test]
    fn read_state_items_returns_six_items() {
        let mock = MockShellExecutor::new()
            .with_output(
                "curl",
                &[
                    "--noproxy", "*", "-s", "-o", "/dev/null", "-w", "%{http_code}",
                    "--connect-timeout", "5", "--max-time", "10",
                    "https://www.google.com",
                ],
                Ok("200".to_string()),
            );
        let adapter = make_adapter(mock);
        let items = adapter.read_state_items();
        assert_eq!(items.len(), 6);

        let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&ID_PROXY_ENV));
        assert!(ids.contains(&ID_GIT_PROXY));
        assert!(ids.contains(&ID_RESOLV_CONF));
        assert!(ids.contains(&ID_ETC_ENVIRONMENT));
        assert!(ids.contains(&ID_SHELL_PROXY));
        assert!(ids.contains(&ID_REACHABILITY));
    }

    #[test]
    fn read_state_items_all_have_correct_platform() {
        let mock = MockShellExecutor::new()
            .with_output(
                "curl",
                &[
                    "--noproxy", "*", "-s", "-o", "/dev/null", "-w", "%{http_code}",
                    "--connect-timeout", "5", "--max-time", "10",
                    "https://www.google.com",
                ],
                Ok("200".to_string()),
            );
        let adapter = make_adapter(mock);
        let items = adapter.read_state_items();
        for item in &items {
            assert_eq!(item.platform, Platform::Linux, "Wrong platform for {}", item.id);
        }
    }

    #[test]
    fn write_git_proxy_via_mock() {
        let mock = MockShellExecutor::new()
            .with_output(
                "git",
                &["config", "--global", "http.proxy", "http://proxy:8080"],
                Ok(String::new()),
            )
            .with_output(
                "git",
                &["config", "--global", "https.proxy", "http://proxy:8443"],
                Ok(String::new()),
            );
        let adapter = make_adapter(mock);
        let item = StateItem {
            id: ID_GIT_PROXY.to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({
                "http_proxy": "http://proxy:8080",
                "https_proxy": "http://proxy:8443",
            }),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        let result = adapter.write_state(&item);
        assert!(result.is_ok(), "write_state for git proxy should succeed");
    }

    // -----------------------------------------------------------------------
    // write_state for proxy-env (F102: no longer a no-op)
    // -----------------------------------------------------------------------

    #[test]
    fn write_proxy_env_dispatches_write() {
        let adapter = make_adapter(MockShellExecutor::new());
        let item = StateItem {
            id: ID_PROXY_ENV.to_string(),
            platform: Platform::Linux,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({
                "http_proxy": "http://proxy:8080",
                "https_proxy": "http://proxy:8443",
                "no_proxy": "localhost",
            }),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        let result = adapter.write_state(&item);
        // The write targets /etc/environment — in test env may require root
        match result {
            Ok(()) => { /* writable in this environment */ }
            Err(e) => {
                assert!(
                    e.contains("Root required") || e.contains("Failed to write"),
                    "Unexpected error: {e}"
                );
            }
        }
    }
}

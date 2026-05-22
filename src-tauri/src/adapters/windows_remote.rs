//! Remote Windows platform adapter — manages Windows state from a WSL/Linux host.
//!
//! Uses `CommandExecutor` (typically `PowershellBridgeExecutor`) to bridge operations
//! from WSL to Windows via `powershell.exe`. Covers the same 9 state items as `WindowsAdapter`.

#![allow(dead_code)]

use crate::adapters::command_executor::CommandExecutor;
use crate::adapters::windows_base::{
    parse_ipconfig_displaydns, parse_netsh_dns, parse_netsh_winhttp, parse_proxy_processes,
    parse_tun_status, parse_wslconfig, ID_DNS_CACHE, ID_DNS_SERVERS, ID_HOSTS, ID_HTTP_PROXY,
    ID_PAC, ID_PROXY_PROCESSES, ID_SYSTEM_PROXY, ID_TUN_STATUS, ID_WSL2_NETWORK_MODE,
};
use crate::adapters::{PlatformAdapter, StateItemDefinition};
use crate::models::baseline::{Platform, StateItem, StateItemCategory};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const REG_INTERNET_SETTINGS: &str =
    r"HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings";

const WINDOWS_HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";

// ---------------------------------------------------------------------------
// WindowsRemoteAdapter
// ---------------------------------------------------------------------------

/// Remote adapter for managing Windows network configuration from a WSL/Linux host.
///
/// Generic over `CommandExecutor` to enable full testability via `MockCommandExecutor`.
/// In production, uses `PowershellBridgeExecutor` which delegates to `powershell.exe`.
pub(crate) struct WindowsRemoteAdapter<E: CommandExecutor> {
    executor: E,
}

impl<E: CommandExecutor> WindowsRemoteAdapter<E> {
    #[must_use]
    pub fn new(executor: E) -> Self {
        Self { executor }
    }

    fn now_iso() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    // ----- Read helpers -----

    fn read_hosts(&self) -> serde_json::Value {
        match self.executor.read_file(WINDOWS_HOSTS_PATH) {
            Ok(content) => serde_json::json!({
                "Content": content,
                "Path": WINDOWS_HOSTS_PATH,
            }),
            Err(e) => serde_json::json!({ "error": e }),
        }
    }

    fn read_system_proxy(&self) -> serde_json::Value {
        match self.executor.execute(
            "Get-ItemProperty",
            &[
                "-Path",
                REG_INTERNET_SETTINGS,
                "|",
                "Select-Object",
                "-Property",
                "ProxyEnable,ProxyServer,ProxyOverride",
                "|",
                "ConvertTo-Json",
            ],
        ) {
            Ok(output) => {
                let trimmed = output.trim();
                if trimmed.is_empty() {
                    return serde_json::json!({
                        "ProxyEnable": 0,
                        "ProxyServer": "",
                        "ProxyOverride": "",
                    });
                }
                serde_json::from_str(trimmed).unwrap_or_else(|_| {
                    serde_json::json!({ "error": "Failed to parse registry JSON" })
                })
            }
            Err(e) => serde_json::json!({ "error": e }),
        }
    }

    fn read_pac(&self) -> serde_json::Value {
        match self.executor.execute(
            "Get-ItemProperty",
            &[
                "-Path",
                REG_INTERNET_SETTINGS,
                "|",
                "Select-Object",
                "-Property",
                "AutoConfigURL",
                "|",
                "ConvertTo-Json",
            ],
        ) {
            Ok(output) => {
                let trimmed = output.trim();
                if trimmed.is_empty() {
                    return serde_json::json!({ "AutoConfigURL": "" });
                }
                serde_json::from_str(trimmed).unwrap_or_else(|_| {
                    serde_json::json!({ "error": "Failed to parse PAC registry JSON" })
                })
            }
            Err(e) => serde_json::json!({ "error": e }),
        }
    }

    fn read_wslconfig(&self) -> serde_json::Value {
        let home = match self.executor.env_var("USERPROFILE") {
            Some(h) => h,
            None => return serde_json::json!({ "error": "USERPROFILE not found" }),
        };
        let path = format!("{home}\\.wslconfig");
        match self.executor.read_file(&path) {
            Ok(content) => parse_wslconfig(&content),
            Err(e) => serde_json::json!({ "error": e }),
        }
    }

    fn read_http_proxy(&self) -> serde_json::Value {
        match self.executor.execute("netsh", &["winhttp", "show", "proxy"]) {
            Ok(output) => parse_netsh_winhttp(&output),
            Err(e) => serde_json::json!({ "error": e }),
        }
    }

    fn read_dns_cache(&self) -> serde_json::Value {
        match self.executor.execute("ipconfig", &["/displaydns"]) {
            Ok(output) => parse_ipconfig_displaydns(&output),
            Err(e) => serde_json::json!({ "error": e }),
        }
    }

    fn read_dns_servers(&self) -> serde_json::Value {
        match self.executor.execute("netsh", &["interface", "ip", "show", "dns"]) {
            Ok(output) => parse_netsh_dns(&output),
            Err(e) => serde_json::json!({ "error": e }),
        }
    }

    fn read_proxy_processes(&self) -> serde_json::Value {
        match self.executor.execute("tasklist", &["/FO", "CSV", "/NH"]) {
            Ok(output) => parse_proxy_processes(&output),
            Err(e) => serde_json::json!({ "error": e }),
        }
    }

    fn read_tun_status(&self) -> serde_json::Value {
        match self.executor.execute("netsh", &["interface", "show", "interface"]) {
            Ok(output) => parse_tun_status(&output),
            Err(e) => serde_json::json!({ "error": e }),
        }
    }

    // ----- Write helpers -----

    fn write_hosts(&self, content: &str) -> Result<(), String> {
        self.executor.write_file(WINDOWS_HOSTS_PATH, content)
    }

    fn write_system_proxy(&self, value: &serde_json::Value) -> Result<(), String> {
        let enable = value
            .get("ProxyEnable")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let server = value
            .get("ProxyServer")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let r#override = value
            .get("ProxyOverride")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        self.executor.execute(
            "Set-ItemProperty",
            &[
                "-Path",
                REG_INTERNET_SETTINGS,
                "-Name",
                "ProxyEnable",
                "-Value",
                &enable.to_string(),
            ],
        )?;
        self.executor.execute(
            "Set-ItemProperty",
            &[
                "-Path",
                REG_INTERNET_SETTINGS,
                "-Name",
                "ProxyServer",
                "-Value",
                server,
            ],
        )?;
        self.executor.execute(
            "Set-ItemProperty",
            &[
                "-Path",
                REG_INTERNET_SETTINGS,
                "-Name",
                "ProxyOverride",
                "-Value",
                r#override,
            ],
        )?;
        Ok(())
    }

    fn write_pac(&self, value: &serde_json::Value) -> Result<(), String> {
        let url = value
            .get("AutoConfigURL")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        self.executor.execute(
            "Set-ItemProperty",
            &[
                "-Path",
                REG_INTERNET_SETTINGS,
                "-Name",
                "AutoConfigURL",
                "-Value",
                url,
            ],
        )?;
        Ok(())
    }

    fn write_wslconfig(&self, content: &str) -> Result<(), String> {
        let home = self
            .executor
            .env_var("USERPROFILE")
            .ok_or_else(|| "USERPROFILE not found".to_string())?;
        let path = format!("{home}\\.wslconfig");
        self.executor.write_file(&path, content)
    }
}

// ---------------------------------------------------------------------------
// PlatformAdapter implementation
// ---------------------------------------------------------------------------

impl<E: CommandExecutor + Send + Sync> PlatformAdapter for WindowsRemoteAdapter<E> {
    fn platform(&self) -> Platform {
        Platform::Windows
    }

    fn state_item_definitions(&self) -> Vec<StateItemDefinition> {
        vec![
            StateItemDefinition {
                id: ID_HOSTS.to_string(),
                category: StateItemCategory::Restorable,
                description: "Windows hosts file content".to_string(),
            },
            StateItemDefinition {
                id: ID_SYSTEM_PROXY.to_string(),
                category: StateItemCategory::Restorable,
                description: "System proxy settings (IE/WinINET proxy)".to_string(),
            },
            StateItemDefinition {
                id: ID_PAC.to_string(),
                category: StateItemCategory::Restorable,
                description: "PAC auto-config URL setting".to_string(),
            },
            StateItemDefinition {
                id: ID_HTTP_PROXY.to_string(),
                category: StateItemCategory::Detectable,
                description: "WinHTTP proxy settings (netsh winhttp)".to_string(),
            },
            StateItemDefinition {
                id: ID_DNS_CACHE.to_string(),
                category: StateItemCategory::Detectable,
                description: "DNS resolver cache entries".to_string(),
            },
            StateItemDefinition {
                id: ID_DNS_SERVERS.to_string(),
                category: StateItemCategory::Detectable,
                description: "DNS server addresses per interface".to_string(),
            },
            StateItemDefinition {
                id: ID_PROXY_PROCESSES.to_string(),
                category: StateItemCategory::Detectable,
                description: "Detected proxy-related processes".to_string(),
            },
            StateItemDefinition {
                id: ID_TUN_STATUS.to_string(),
                category: StateItemCategory::Detectable,
                description: "TUN/TAP virtual adapter status".to_string(),
            },
            StateItemDefinition {
                id: ID_WSL2_NETWORK_MODE.to_string(),
                category: StateItemCategory::Restorable,
                description: "WSL2 networking mode from .wslconfig".to_string(),
            },
        ]
    }

    fn read_state_items(&self) -> Vec<StateItem> {
        let now = Self::now_iso();
        vec![
            StateItem {
                id: ID_HOSTS.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Restorable,
                value: self.read_hosts(),
                collected_at: now.clone(),
                classification_reason: "File, restorable via remote write".to_string(),
            },
            StateItem {
                id: ID_SYSTEM_PROXY.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Restorable,
                value: self.read_system_proxy(),
                collected_at: now.clone(),
                classification_reason: "Registry, restorable via powershell.exe".to_string(),
            },
            StateItem {
                id: ID_PAC.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Restorable,
                value: self.read_pac(),
                collected_at: now.clone(),
                classification_reason: "Registry, restorable via powershell.exe".to_string(),
            },
            StateItem {
                id: ID_HTTP_PROXY.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Detectable,
                value: self.read_http_proxy(),
                collected_at: now.clone(),
                classification_reason: "Command output, detectable only".to_string(),
            },
            StateItem {
                id: ID_DNS_CACHE.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Detectable,
                value: self.read_dns_cache(),
                collected_at: now.clone(),
                classification_reason: "Command output, detectable only".to_string(),
            },
            StateItem {
                id: ID_DNS_SERVERS.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Detectable,
                value: self.read_dns_servers(),
                collected_at: now.clone(),
                classification_reason: "Command output, detectable only".to_string(),
            },
            StateItem {
                id: ID_PROXY_PROCESSES.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Detectable,
                value: self.read_proxy_processes(),
                collected_at: now.clone(),
                classification_reason: "Process scan, detectable only".to_string(),
            },
            StateItem {
                id: ID_TUN_STATUS.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Detectable,
                value: self.read_tun_status(),
                collected_at: now.clone(),
                classification_reason: "Adapter detection, detectable only".to_string(),
            },
            StateItem {
                id: ID_WSL2_NETWORK_MODE.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Restorable,
                value: self.read_wslconfig(),
                collected_at: now,
                classification_reason: "File, restorable via remote write".to_string(),
            },
        ]
    }

    fn write_state(&self, item: &StateItem) -> Result<(), String> {
        match item.id.as_str() {
            ID_HOSTS => {
                let content = item
                    .value
                    .get("Content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'Content' field".to_string())?;
                self.write_hosts(content)
            }
            ID_SYSTEM_PROXY => self.write_system_proxy(&item.value),
            ID_PAC => self.write_pac(&item.value),
            ID_WSL2_NETWORK_MODE => {
                let content = item
                    .value
                    .get("RawContent")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'RawContent' field".to_string())?;
                self.write_wslconfig(content)
            }
            _ => Err(format!("Not restorable: {}", item.id)),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::command_executor::MockCommandExecutor;

    fn base_mock() -> MockCommandExecutor {
        MockCommandExecutor::new()
            .with_env_var("USERPROFILE", r"C:\Users\testuser")
            // Registry: system proxy (JSON output from ConvertTo-Json)
            .with_output(
                "Get-ItemProperty",
                &[
                    "-Path",
                    REG_INTERNET_SETTINGS,
                    "|",
                    "Select-Object",
                    "-Property",
                    "ProxyEnable,ProxyServer,ProxyOverride",
                    "|",
                    "ConvertTo-Json",
                ],
                Ok(
                    "{\"ProxyEnable\":1,\"ProxyServer\":\"127.0.0.1:7890\",\"ProxyOverride\":\"<local>\"}"
                        .to_string(),
                ),
            )
            // Registry: PAC
            .with_output(
                "Get-ItemProperty",
                &[
                    "-Path",
                    REG_INTERNET_SETTINGS,
                    "|",
                    "Select-Object",
                    "-Property",
                    "AutoConfigURL",
                    "|",
                    "ConvertTo-Json",
                ],
                Ok("{\"AutoConfigURL\":\"http://proxy.example.com/pac\"}".to_string()),
            )
            // netsh winhttp show proxy
            .with_output(
                "netsh",
                &["winhttp", "show", "proxy"],
                Ok("Direct access (no proxy server).".to_string()),
            )
            // ipconfig /displaydns
            .with_output(
                "ipconfig",
                &["/displaydns"],
                Ok("Windows IP Configuration\n".to_string()),
            )
            // netsh interface ip show dns
            .with_output(
                "netsh",
                &["interface", "ip", "show", "dns"],
                Ok("Configuration for interface \"Ethernet\"\n    DNS servers configured through DHCP: 8.8.8.8\n".to_string()),
            )
            // tasklist
            .with_output(
                "tasklist",
                &["/FO", "CSV", "/NH"],
                Ok("\"svchost.exe\",\"1234\",\"Services\",\"0\",\"2,456 K\"".to_string()),
            )
            // netsh interface show interface
            .with_output(
                "netsh",
                &["interface", "show", "interface"],
                Ok("Admin State    State          Type             Interface Name\n -------------------------------------------------------------------------\n Enabled        Connected      Dedicated        Wi-Fi".to_string()),
            )
    }

    fn base_mock_with_files() -> MockCommandExecutor {
        base_mock()
            .with_file(
                r"C:\Windows\System32\drivers\etc\hosts",
                "127.0.0.1 localhost\n",
            )
            .with_file(
                r"C:\Users\testuser\.wslconfig",
                "[wsl2]\nnetworkingMode=mirrored\n",
            )
    }

    // ----- Trait interface -----

    #[test]
    fn platform_returns_windows() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        assert_eq!(adapter.platform(), Platform::Windows);
    }

    #[test]
    fn definitions_count_is_nine() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        assert_eq!(adapter.state_item_definitions().len(), 9);
    }

    #[test]
    fn definitions_have_four_restorable() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let restorable = adapter
            .state_item_definitions()
            .iter()
            .filter(|d| d.category == StateItemCategory::Restorable)
            .count();
        assert_eq!(restorable, 4);
    }

    #[test]
    fn definitions_have_five_detectable() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let detectable = adapter
            .state_item_definitions()
            .iter()
            .filter(|d| d.category == StateItemCategory::Detectable)
            .count();
        assert_eq!(detectable, 5);
    }

    #[test]
    fn trait_object_dispatch_works() {
        let adapter: Box<dyn PlatformAdapter> =
            Box::new(WindowsRemoteAdapter::new(base_mock_with_files()));
        assert_eq!(adapter.platform(), Platform::Windows);
    }

    // ----- Read state items -----

    #[test]
    fn read_state_items_returns_nine() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        assert_eq!(items.len(), 9);
    }

    #[test]
    fn read_state_items_all_have_windows_platform() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        for item in &items {
            assert_eq!(item.platform, Platform::Windows, "Wrong platform for {}", item.id);
        }
    }

    #[test]
    fn read_state_items_contain_expected_ids() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&ID_HOSTS));
        assert!(ids.contains(&ID_SYSTEM_PROXY));
        assert!(ids.contains(&ID_PAC));
        assert!(ids.contains(&ID_HTTP_PROXY));
        assert!(ids.contains(&ID_DNS_CACHE));
        assert!(ids.contains(&ID_DNS_SERVERS));
        assert!(ids.contains(&ID_PROXY_PROCESSES));
        assert!(ids.contains(&ID_TUN_STATUS));
        assert!(ids.contains(&ID_WSL2_NETWORK_MODE));
    }

    #[test]
    fn read_hosts_returns_content() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        let hosts = items.iter().find(|i| i.id == ID_HOSTS).unwrap();
        assert_eq!(hosts.value["Content"], "127.0.0.1 localhost\n");
    }

    #[test]
    fn read_system_proxy_returns_registry_values() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        let proxy = items.iter().find(|i| i.id == ID_SYSTEM_PROXY).unwrap();
        assert_eq!(proxy.value["ProxyEnable"], 1);
        assert_eq!(proxy.value["ProxyServer"], "127.0.0.1:7890");
    }

    #[test]
    fn read_pac_returns_autoconfig_url() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        let pac = items.iter().find(|i| i.id == ID_PAC).unwrap();
        assert_eq!(pac.value["AutoConfigURL"], "http://proxy.example.com/pac");
    }

    #[test]
    fn read_http_proxy_uses_parse_function() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        let http = items.iter().find(|i| i.id == ID_HTTP_PROXY).unwrap();
        assert_eq!(http.value["AccessType"], "direct");
    }

    #[test]
    fn read_dns_cache_uses_parse_function() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        let dns = items.iter().find(|i| i.id == ID_DNS_CACHE).unwrap();
        assert_eq!(dns.value["EntryCount"], 0);
    }

    #[test]
    fn read_dns_servers_uses_parse_function() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        let dns = items.iter().find(|i| i.id == ID_DNS_SERVERS).unwrap();
        let interfaces = dns.value["Interfaces"].as_array().unwrap();
        assert!(!interfaces.is_empty());
    }

    #[test]
    fn read_proxy_processes_uses_parse_function() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        let procs = items.iter().find(|i| i.id == ID_PROXY_PROCESSES).unwrap();
        assert_eq!(procs.value["DetectedCount"], 0);
    }

    #[test]
    fn read_tun_status_uses_parse_function() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        let tun = items.iter().find(|i| i.id == ID_TUN_STATUS).unwrap();
        assert_eq!(tun.value["TunFound"], false);
    }

    #[test]
    fn read_wslconfig_parses_content() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let items = adapter.read_state_items();
        let wsl = items.iter().find(|i| i.id == ID_WSL2_NETWORK_MODE).unwrap();
        assert_eq!(wsl.value["NetworkingMode"], "mirrored");
    }

    // ----- Write state items -----

    #[test]
    fn write_state_rejects_detectable() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let item = StateItem {
            id: ID_HTTP_PROXY.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Detectable,
            value: serde_json::json!({}),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        assert!(adapter.write_state(&item).is_err());
    }

    #[test]
    fn write_hosts_succeeds() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let item = StateItem {
            id: ID_HOSTS.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({
                "Content": "127.0.0.1 localhost\n::1 localhost\n"
            }),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        assert!(adapter.write_state(&item).is_ok());
    }

    #[test]
    fn write_system_proxy_succeeds() {
        let mock = base_mock_with_files()
            .with_output(
                "Set-ItemProperty",
                &[
                    "-Path",
                    REG_INTERNET_SETTINGS,
                    "-Name",
                    "ProxyEnable",
                    "-Value",
                    "1",
                ],
                Ok(String::new()),
            )
            .with_output(
                "Set-ItemProperty",
                &[
                    "-Path",
                    REG_INTERNET_SETTINGS,
                    "-Name",
                    "ProxyServer",
                    "-Value",
                    "127.0.0.1:7890",
                ],
                Ok(String::new()),
            )
            .with_output(
                "Set-ItemProperty",
                &[
                    "-Path",
                    REG_INTERNET_SETTINGS,
                    "-Name",
                    "ProxyOverride",
                    "-Value",
                    "<local>",
                ],
                Ok(String::new()),
            );
        let adapter = WindowsRemoteAdapter::new(mock);
        let item = StateItem {
            id: ID_SYSTEM_PROXY.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({
                "ProxyEnable": 1,
                "ProxyServer": "127.0.0.1:7890",
                "ProxyOverride": "<local>",
            }),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        assert!(adapter.write_state(&item).is_ok());
    }

    #[test]
    fn write_pac_succeeds() {
        let mock = base_mock_with_files().with_output(
            "Set-ItemProperty",
            &[
                "-Path",
                REG_INTERNET_SETTINGS,
                "-Name",
                "AutoConfigURL",
                "-Value",
                "http://new.example.com/pac",
            ],
            Ok(String::new()),
        );
        let adapter = WindowsRemoteAdapter::new(mock);
        let item = StateItem {
            id: ID_PAC.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({
                "AutoConfigURL": "http://new.example.com/pac",
            }),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        assert!(adapter.write_state(&item).is_ok());
    }

    #[test]
    fn write_wslconfig_succeeds() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let item = StateItem {
            id: ID_WSL2_NETWORK_MODE.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({
                "RawContent": "[wsl2]\nnetworkingMode=nat\n",
            }),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        assert!(adapter.write_state(&item).is_ok());
    }

    #[test]
    fn write_hosts_missing_content_returns_error() {
        let adapter = WindowsRemoteAdapter::new(base_mock_with_files());
        let item = StateItem {
            id: ID_HOSTS.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({}),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        assert!(adapter.write_state(&item).is_err());
    }
}

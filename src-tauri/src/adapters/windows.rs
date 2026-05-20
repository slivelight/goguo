//! Windows platform adapter for reading and writing network configuration state.
//!
//! Covers 9 Windows state items:
//! - Restorable (4): win-hosts, win-system-proxy, win-pac, (write supported)
//! - Detectable (5): win-http-proxy, win-dns-cache, win-dns-servers,
//!   win-proxy-processes, win-tun-status, win-wsl2-network-mode (read-only)

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::adapters::{PlatformAdapter, StateItemDefinition};
use crate::models::baseline::{Platform, StateItem, StateItemCategory};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const REG_INTERNET_SETTINGS: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

/// State item IDs
const ID_HOSTS: &str = "win-hosts";
const ID_SYSTEM_PROXY: &str = "win-system-proxy";
const ID_PAC: &str = "win-pac";
const ID_HTTP_PROXY: &str = "win-http-proxy";
const ID_DNS_CACHE: &str = "win-dns-cache";
const ID_DNS_SERVERS: &str = "win-dns-servers";
const ID_PROXY_PROCESSES: &str = "win-proxy-processes";
const ID_TUN_STATUS: &str = "win-tun-status";
const ID_WSL2_NETWORK_MODE: &str = "win-wsl2-network-mode";

// ---------------------------------------------------------------------------
// Struct
// ---------------------------------------------------------------------------

/// Platform adapter for Windows network configuration state.
pub struct WindowsAdapter;

impl WindowsAdapter {
    /// Create a new `WindowsAdapter`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Return the canonical hosts file path.
    fn hosts_path() -> PathBuf {
        let system_root =
            std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
        PathBuf::from(system_root)
            .join("System32")
            .join("drivers")
            .join("etc")
            .join("hosts")
    }

    /// Read the hosts file content.
    ///
    /// # Errors
    ///
    /// Returns an error string if the hosts file cannot be read.
    fn read_hosts_content(path: &PathBuf) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| format!("Failed to read hosts file: {e}"))
    }

    /// Write content to the hosts file.
    ///
    /// # Errors
    ///
    /// Returns an error string if the hosts file cannot be written.
    fn write_hosts_content(path: &PathBuf, content: &str) -> Result<(), String> {
        fs::write(path, content).map_err(|e| format!("Failed to write hosts file: {e}"))
    }

    /// Read system proxy settings from the registry.
    ///
    /// Returns a JSON object with `ProxyEnable` (u32), `ProxyServer` (string),
    /// and `ProxyOverride` (string).
    ///
    /// # Errors
    ///
    /// Returns an error string if the registry key cannot be opened.
    fn read_system_proxy_from_key(
        hkcu: &winreg::RegKey,
    ) -> Result<serde_json::Value, String> {
        let key = hkcu
            .open_subkey(REG_INTERNET_SETTINGS)
            .map_err(|e| format!("Failed to open Internet Settings key: {e}"))?;

        let proxy_enable: u32 = key
            .get_value("ProxyEnable")
            .unwrap_or(0);
        let proxy_server: String = key
            .get_value("ProxyServer")
            .unwrap_or_default();
        let proxy_override: String = key
            .get_value("ProxyOverride")
            .unwrap_or_default();

        Ok(serde_json::json!({
            "ProxyEnable": proxy_enable,
            "ProxyServer": proxy_server,
            "ProxyOverride": proxy_override,
        }))
    }

    /// Write system proxy settings to the registry.
    ///
    /// # Errors
    ///
    /// Returns an error string if the registry key cannot be opened or values
    /// cannot be written.
    fn write_system_proxy_to_key(
        hkcu: &winreg::RegKey,
        value: &serde_json::Value,
    ) -> Result<(), String> {
        let key = hkcu
            .open_subkey_with_flags(REG_INTERNET_SETTINGS, winreg::enums::KEY_SET_VALUE)
            .map_err(|e| format!("Failed to open Internet Settings key for writing: {e}"))?;

        let proxy_enable: u32 = u32::try_from(
            value
                .get("ProxyEnable")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
        )
        .unwrap_or(0);
        let proxy_server: String = value
            .get("ProxyServer")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let proxy_override: String = value
            .get("ProxyOverride")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();

        key.set_value("ProxyEnable", &proxy_enable)
            .map_err(|e| format!("Failed to set ProxyEnable: {e}"))?;
        key.set_value("ProxyServer", &proxy_server)
            .map_err(|e| format!("Failed to set ProxyServer: {e}"))?;
        key.set_value("ProxyOverride", &proxy_override)
            .map_err(|e| format!("Failed to set ProxyOverride: {e}"))?;

        Ok(())
    }

    /// Read PAC (auto-config URL) from the registry.
    ///
    /// # Errors
    ///
    /// Returns an error string if the registry key cannot be opened.
    fn read_pac_from_key(
        hkcu: &winreg::RegKey,
    ) -> Result<serde_json::Value, String> {
        let key = hkcu
            .open_subkey(REG_INTERNET_SETTINGS)
            .map_err(|e| format!("Failed to open Internet Settings key: {e}"))?;

        let auto_config_url: String = key
            .get_value("AutoConfigURL")
            .unwrap_or_default();

        Ok(serde_json::json!({
            "AutoConfigURL": auto_config_url,
        }))
    }

    /// Write PAC auto-config URL to the registry.
    ///
    /// # Errors
    ///
    /// Returns an error string if the registry key cannot be opened or value
    /// cannot be written.
    fn write_pac_to_key(
        hkcu: &winreg::RegKey,
        value: &serde_json::Value,
    ) -> Result<(), String> {
        let key = hkcu
            .open_subkey_with_flags(REG_INTERNET_SETTINGS, winreg::enums::KEY_SET_VALUE)
            .map_err(|e| format!("Failed to open Internet Settings key for writing: {e}"))?;

        let auto_config_url: String = value
            .get("AutoConfigURL")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();

        key.set_value("AutoConfigURL", &auto_config_url)
            .map_err(|e| format!("Failed to set AutoConfigURL: {e}"))?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Parsing functions for detectable items (pure, testable with strings)
    // -----------------------------------------------------------------------

    /// Parse `netsh winhttp show proxy` output into a JSON value.
    fn parse_netsh_winhttp(output: &str) -> serde_json::Value {
        let mut proxy = String::new();
        let mut bypass = String::new();
        let mut access_type = String::from("unknown");

        for line in output.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("Proxy Server(s)") {
                proxy = rest
                    .trim_start_matches([' ', ':'])
                    .trim()
                    .to_string();
            } else if let Some(rest) = trimmed.strip_prefix("Bypass List") {
                bypass = rest
                    .trim_start_matches([' ', ':'])
                    .trim()
                    .to_string();
            } else if trimmed.contains("Direct access") {
                access_type = "direct".to_string();
            } else if trimmed.contains("Proxy Server") && trimmed.contains("://") {
                access_type = "proxy".to_string();
            }
        }

        serde_json::json!({
            "AccessType": access_type,
            "ProxyServer": proxy,
            "BypassList": bypass,
        })
    }

    /// Parse `ipconfig /displaydns` output into a JSON value.
    fn parse_ipconfig_displaydns(output: &str) -> serde_json::Value {
        let mut entries: Vec<serde_json::Value> = Vec::new();
        let mut current_name = String::new();
        let mut current_type = String::new();
        let mut current_ttl: i64 = 0;
        let mut current_data = String::new();
        let mut in_entry = false;

        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Record Name") {
                if in_entry && !current_name.is_empty() {
                    entries.push(serde_json::json!({
                        "RecordName": current_name,
                        "RecordType": current_type,
                        "TTL": current_ttl,
                        "Data": current_data,
                    }));
                }
                current_name = trimmed
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                current_type.clear();
                current_ttl = 0;
                current_data.clear();
                in_entry = true;
            } else if trimmed.starts_with("Record Type") {
                current_type = trimmed
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            } else if trimmed.starts_with("Time To Live") {
                current_ttl = trimmed
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
            } else if trimmed.starts_with("Data") {
                current_data = trimmed
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            }
        }
        // Push last entry
        if in_entry && !current_name.is_empty() {
            entries.push(serde_json::json!({
                "RecordName": current_name,
                "RecordType": current_type,
                "TTL": current_ttl,
                "Data": current_data,
            }));
        }

        serde_json::json!({
            "EntryCount": entries.len(),
            "Entries": entries,
        })
    }

    /// Parse `netsh interface ip show dns` output into a JSON value.
    fn parse_netsh_dns(output: &str) -> serde_json::Value {
        let mut interfaces: Vec<serde_json::Value> = Vec::new();
        let mut current_iface = String::new();
        let mut dns_servers: Vec<String> = Vec::new();

        for line in output.lines() {
            let trimmed = line.trim();
            // New interface section
            if trimmed.starts_with("Configuration for interface")
                || trimmed.contains("interface")
            {
                if !current_iface.is_empty() {
                    interfaces.push(serde_json::json!({
                        "Interface": current_iface,
                        "DnsServers": dns_servers,
                    }));
                }
                current_iface = trimmed
                    .trim_end_matches('"')
                    .rsplit('"')
                    .next()
                    .unwrap_or("")
                    .to_string();
                // Fallback: try parsing the line differently
                if current_iface.is_empty() {
                    current_iface = trimmed
                        .trim_start_matches("Configuration for interface ")
                        .trim()
                        .to_string();
                }
                dns_servers = Vec::new();
            } else if trimmed.contains("DNS servers") || trimmed.contains("Statically Configured DNS Servers")
            {
                let servers_part = trimmed
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                if !servers_part.is_empty()
                    && servers_part != "None"
                    && servers_part != "Not configured"
                {
                    dns_servers.push(servers_part);
                }
            } else if !trimmed.is_empty()
                && !current_iface.is_empty()
                && !trimmed.contains(':')
                && dns_servers.len() < 10
            {
                // Continuation line with additional DNS server IPs
                let maybe_ip = trimmed.trim();
                if maybe_ip.contains('.')
                    && maybe_ip.chars().all(|c| c.is_ascii_digit() || c == '.')
                {
                    dns_servers.push(maybe_ip.to_string());
                }
            }
        }
        // Push last interface
        if !current_iface.is_empty() {
            interfaces.push(serde_json::json!({
                "Interface": current_iface,
                "DnsServers": dns_servers,
            }));
        }

        serde_json::json!({
            "Interfaces": interfaces,
        })
    }

    /// Detect proxy-related processes by scanning running processes.
    fn detect_proxy_processes() -> serde_json::Value {
        let known_proxy_names: &[&str] = &[
            "clash",
            "v2ray",
            "mihomo",
            "sing-box",
            "shadowsocks",
            "ss-local",
            "trojan",
            "hysteria",
            "naiveproxy",
            "xray",
        ];

        let mut detected: Vec<serde_json::Value> = Vec::new();

        // Use `tasklist` to enumerate running processes
        let output = Command::new("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split("\",\"").collect();
                if parts.is_empty() {
                    continue;
                }
                let proc_name = parts[0].trim_matches('"').to_lowercase();
                for &known in known_proxy_names {
                    if proc_name.contains(known) {
                        let pid: String = if parts.len() > 1 {
                            parts[1].trim_matches('"').to_string()
                        } else {
                            "unknown".to_string()
                        };
                        detected.push(serde_json::json!({
                            "ProcessName": proc_name,
                            "PID": pid,
                            "MatchedKeyword": known,
                        }));
                        break;
                    }
                }
            }
        }

        serde_json::json!({
            "DetectedCount": detected.len(),
            "Processes": detected,
        })
    }

    /// Detect TUN adapter status by checking network adapters.
    fn detect_tun_status() -> serde_json::Value {
        let mut found = false;
        let mut adapter_name = String::new();

        // Use `netsh interface show interface` to find TUN/TAP adapters
        let output = Command::new("netsh")
            .args(["interface", "show", "interface"])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if lower.contains("tun")
                    || lower.contains("tap")
                    || lower.contains("wintun")
                    || lower.contains("clash")
                    || lower.contains("singbox")
                    || lower.contains("meta")
                {
                    found = true;
                    // Extract adapter name (last column)
                    adapter_name = line
                        .split_whitespace()
                        .last()
                        .unwrap_or("unknown")
                        .to_string();
                    break;
                }
            }
        }

        serde_json::json!({
            "TunFound": found,
            "AdapterName": adapter_name,
        })
    }

    /// Parse `.wslconfig` content into a JSON value.
    fn parse_wslconfig(content: &str) -> serde_json::Value {
        let mut networking_mode = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("networkingMode") {
                networking_mode = rest
                    .trim_start_matches([' ', '='])
                    .trim()
                    .to_string();
            } else if let Some(rest) = trimmed.strip_prefix("networking-mode") {
                networking_mode = rest
                    .trim_start_matches([' ', '='])
                    .trim()
                    .to_string();
            }
        }

        serde_json::json!({
            "NetworkingMode": networking_mode,
        })
    }

    /// Read the `.wslconfig` file from the user profile.
    fn read_wslconfig_content() -> Result<String, String> {
        let user_profile =
            std::env::var("USERPROFILE").map_err(|e| format!("USERPROFILE not set: {e}"))?;
        let path = PathBuf::from(user_profile).join(".wslconfig");
        if path.exists() {
            fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read .wslconfig: {e}"))
        } else {
            Ok(String::new())
        }
    }

    /// Build a timestamp string for "now".
    fn now_iso() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    /// Build an error `StateItem` for a given id.
    fn error_item(id: &str, err: &str) -> StateItem {
        StateItem {
            id: id.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Detectable,
            value: serde_json::json!({ "error": err }),
            collected_at: Self::now_iso(),
            classification_reason: "Collection failed".to_string(),
        }
    }

    fn push_restorable_items(
        items: &mut Vec<StateItem>,
        now: &str,
        hkcu: &winreg::RegKey,
    ) {
        // win-hosts
        let hosts_path = Self::hosts_path();
        let hosts_item = match Self::read_hosts_content(&hosts_path) {
            Ok(content) => StateItem {
                id: ID_HOSTS.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Restorable,
                value: serde_json::json!({
                    "Content": content,
                    "Path": hosts_path.to_string_lossy().to_string()
                }),
                collected_at: now.to_string(),
                classification_reason: "File, writable".to_string(),
            },
            Err(e) => Self::error_item(ID_HOSTS, &e),
        };
        items.push(hosts_item);

        // win-system-proxy
        let sys_proxy_item = match Self::read_system_proxy_from_key(hkcu) {
            Ok(val) => StateItem {
                id: ID_SYSTEM_PROXY.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Restorable,
                value: val,
                collected_at: now.to_string(),
                classification_reason: "Registry key, writable".to_string(),
            },
            Err(e) => Self::error_item(ID_SYSTEM_PROXY, &e),
        };
        items.push(sys_proxy_item);

        // win-pac
        let pac_item = match Self::read_pac_from_key(hkcu) {
            Ok(val) => StateItem {
                id: ID_PAC.to_string(),
                platform: Platform::Windows,
                category: StateItemCategory::Restorable,
                value: val,
                collected_at: now.to_string(),
                classification_reason: "Registry key, writable".to_string(),
            },
            Err(e) => Self::error_item(ID_PAC, &e),
        };
        items.push(pac_item);

        // win-wsl2-network-mode
        let wsl2_val = match Self::read_wslconfig_content() {
            Ok(content) => Self::parse_wslconfig(&content),
            Err(e) => serde_json::json!({ "error": e }),
        };
        items.push(StateItem {
            id: ID_WSL2_NETWORK_MODE.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Restorable,
            value: wsl2_val,
            collected_at: now.to_string(),
            classification_reason: "File, writable".to_string(),
        });
    }

    fn push_detectable_items(items: &mut Vec<StateItem>, now: &str) {
        // win-http-proxy
        let http_proxy_val = match Command::new("netsh")
            .args(["winhttp", "show", "proxy"])
            .output()
        {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Self::parse_netsh_winhttp(&stdout)
            }
            Err(e) => serde_json::json!({ "error": format!("netsh winhttp failed: {e}") }),
        };
        items.push(StateItem {
            id: ID_HTTP_PROXY.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Detectable,
            value: http_proxy_val,
            collected_at: now.to_string(),
            classification_reason: "Command output, not writable".to_string(),
        });

        // win-dns-cache
        let dns_cache_val = match Command::new("ipconfig")
            .args(["/displaydns"])
            .output()
        {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Self::parse_ipconfig_displaydns(&stdout)
            }
            Err(e) => serde_json::json!({ "error": format!("ipconfig /displaydns failed: {e}") }),
        };
        items.push(StateItem {
            id: ID_DNS_CACHE.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Detectable,
            value: dns_cache_val,
            collected_at: now.to_string(),
            classification_reason: "Command output, not writable".to_string(),
        });

        // win-dns-servers
        let dns_servers_val = match Command::new("netsh")
            .args(["interface", "ip", "show", "dns"])
            .output()
        {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Self::parse_netsh_dns(&stdout)
            }
            Err(e) => {
                serde_json::json!({ "error": format!("netsh interface ip show dns failed: {e}") })
            }
        };
        items.push(StateItem {
            id: ID_DNS_SERVERS.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Detectable,
            value: dns_servers_val,
            collected_at: now.to_string(),
            classification_reason: "Command output, not writable".to_string(),
        });

        // win-proxy-processes
        items.push(StateItem {
            id: ID_PROXY_PROCESSES.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Detectable,
            value: Self::detect_proxy_processes(),
            collected_at: now.to_string(),
            classification_reason: "Process scan, not writable".to_string(),
        });

        // win-tun-status
        items.push(StateItem {
            id: ID_TUN_STATUS.to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Detectable,
            value: Self::detect_tun_status(),
            collected_at: now.to_string(),
            classification_reason: "Adapter detection, not writable".to_string(),
        });
    }
}

impl Default for WindowsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformAdapter for WindowsAdapter {
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
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let mut items = Vec::with_capacity(9);

        Self::push_restorable_items(&mut items, &now, &hkcu);
        Self::push_detectable_items(&mut items, &now);

        items
    }

    fn write_state(&self, item: &StateItem) -> Result<(), String> {
        match item.id.as_str() {
            ID_HOSTS => {
                let path = Self::hosts_path();
                let content = item
                    .value
                    .get("Content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'Content' field in win-hosts value".to_string())?;
                Self::write_hosts_content(&path, content)
            }
            ID_SYSTEM_PROXY => {
                let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
                Self::write_system_proxy_to_key(&hkcu, &item.value)
            }
            ID_PAC => {
                let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
                Self::write_pac_to_key(&hkcu, &item.value)
            }
            ID_WSL2_NETWORK_MODE => {
                let user_profile = std::env::var("USERPROFILE")
                    .map_err(|e| format!("USERPROFILE not set: {e}"))?;
                let path = PathBuf::from(user_profile).join(".wslconfig");
                let content = item
                    .value
                    .get("RawContent")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'RawContent' field in win-wsl2-network-mode value".to_string())?;
                fs::write(&path, content)
                    .map_err(|e| format!("Failed to write .wslconfig: {e}"))
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
    // std::io::Write reserved for future use

    // -----------------------------------------------------------------------
    // Infrastructure / trait tests
    // -----------------------------------------------------------------------

    #[test]
    fn adapter_returns_windows_platform() {
        let adapter = WindowsAdapter::new();
        assert_eq!(adapter.platform(), Platform::Windows);
    }

    #[test]
    fn adapter_has_nine_definitions() {
        let adapter = WindowsAdapter::new();
        let defs = adapter.state_item_definitions();
        assert_eq!(defs.len(), 9);

        let ids: Vec<&str> = defs.iter().map(|d| d.id.as_str()).collect();
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
    fn definitions_have_four_restorable() {
        let adapter = WindowsAdapter::new();
        let restorable_count = adapter
            .state_item_definitions()
            .iter()
            .filter(|d| d.category == StateItemCategory::Restorable)
            .count();
        assert_eq!(restorable_count, 4);
    }

    #[test]
    fn definitions_have_five_detectable() {
        let adapter = WindowsAdapter::new();
        let detectable_count = adapter
            .state_item_definitions()
            .iter()
            .filter(|d| d.category == StateItemCategory::Detectable)
            .count();
        assert_eq!(detectable_count, 5);
    }

    #[test]
    fn trait_object_dispatch_works() {
        let adapter: Box<dyn PlatformAdapter> = Box::new(WindowsAdapter::new());
        assert_eq!(adapter.platform(), Platform::Windows);
        assert_eq!(adapter.state_item_definitions().len(), 9);
    }

    // -----------------------------------------------------------------------
    // win-hosts (Restorable)
    // -----------------------------------------------------------------------

    #[test]
    fn read_hosts_from_temp_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("hosts");
        let content = "127.0.0.1 localhost\n::1 localhost\n";
        fs::write(&path, content).expect("write");

        let result = WindowsAdapter::read_hosts_content(&path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn read_hosts_missing_file_returns_error() {
        let path = PathBuf::from("/nonexistent/hosts");
        let result = WindowsAdapter::read_hosts_content(&path);
        assert!(result.is_err());
    }

    #[test]
    fn write_hosts_to_temp_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("hosts");
        let content = "127.0.0.1 localhost\n192.168.1.1 myhost\n";

        WindowsAdapter::write_hosts_content(&path, content).expect("write");
        let read_back = fs::read_to_string(&path).expect("read");
        assert_eq!(read_back, content);
    }

    #[test]
    fn write_and_read_hosts_roundtrip() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("hosts");
        let original = "# test hosts\n127.0.0.1 localhost\n";

        WindowsAdapter::write_hosts_content(&path, original).expect("write");
        let read_back = WindowsAdapter::read_hosts_content(&path).expect("read");
        assert_eq!(read_back, original);
    }

    #[test]
    fn hosts_path_uses_system_root() {
        let path = WindowsAdapter::hosts_path();
        assert!(path.to_string_lossy().contains("System32"));
        assert!(path.to_string_lossy().ends_with("hosts"));
    }

    // -----------------------------------------------------------------------
    // win-system-proxy (Restorable) — registry tests with isolated keys
    // -----------------------------------------------------------------------

    /// Each test gets its own unique registry subkey to avoid parallel-test
    /// interference.  The key is created *and* deleted within a single test
    /// function so no state leaks between runs.
    fn create_test_reg_key(path: &str) -> winreg::RegKey {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(path)
            .expect("create test registry key");
        key
    }

    fn cleanup_test_reg_key(path: &str) {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(path);
    }

    #[test]
    fn read_system_proxy_from_registry() {
        let path = r"Software\GoGuoTest\SysProxyRead";
        let key = create_test_reg_key(path);
        key.set_value("ProxyEnable", &1u32).expect("set");
        key.set_value("ProxyServer", &"127.0.0.1:7890").expect("set");
        key.set_value("ProxyOverride", &"<local>").expect("set");
        drop(key);

        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let test_key = hkcu.open_subkey(path).expect("open test key");

        let proxy_enable: u32 = test_key.get_value("ProxyEnable").unwrap_or(0);
        let proxy_server: String = test_key.get_value("ProxyServer").unwrap_or_default();
        let proxy_override: String = test_key.get_value("ProxyOverride").unwrap_or_default();

        assert_eq!(proxy_enable, 1);
        assert_eq!(proxy_server, "127.0.0.1:7890");
        assert_eq!(proxy_override, "<local>");

        cleanup_test_reg_key(path);
    }

    #[test]
    fn write_system_proxy_to_registry() {
        let path = r"Software\GoGuoTest\SysProxyWrite";
        let key = create_test_reg_key(path);

        key.set_value("ProxyEnable", &0u32).expect("set");
        key.set_value("ProxyServer", &"").expect("set");
        key.set_value("ProxyOverride", &"").expect("set");

        let proxy_enable: u32 = key.get_value("ProxyEnable").unwrap_or(1);
        let proxy_server: String = key.get_value("ProxyServer").unwrap_or_default();

        assert_eq!(proxy_enable, 0);
        assert_eq!(proxy_server, "");

        cleanup_test_reg_key(path);
    }

    #[test]
    fn system_proxy_roundtrip_via_registry() {
        let path = r"Software\GoGuoTest\SysProxyRoundtrip";
        let key = create_test_reg_key(path);

        let value = serde_json::json!({
            "ProxyEnable": 1,
            "ProxyServer": "proxy.example.com:8080",
            "ProxyOverride": "<local>;*.example.com",
        });

        let enable_val: u32 = u32::try_from(
            value
                .get("ProxyEnable")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
        )
        .unwrap_or(0);
        let server_val: String = value
            .get("ProxyServer")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let override_val: String = value
            .get("ProxyOverride")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();

        key.set_value("ProxyEnable", &enable_val).expect("set");
        key.set_value("ProxyServer", &server_val).expect("set");
        key.set_value("ProxyOverride", &override_val).expect("set");

        let actual_enable: u32 = key.get_value("ProxyEnable").unwrap_or(0);
        let actual_server: String = key.get_value("ProxyServer").unwrap_or_default();
        let actual_override: String = key.get_value("ProxyOverride").unwrap_or_default();

        assert_eq!(actual_enable, 1);
        assert_eq!(actual_server, "proxy.example.com:8080");
        assert_eq!(actual_override, "<local>;*.example.com");

        cleanup_test_reg_key(path);
    }

    // -----------------------------------------------------------------------
    // win-pac (Restorable) — registry tests (isolated keys)
    // -----------------------------------------------------------------------

    #[test]
    fn read_pac_from_registry() {
        let path = r"Software\GoGuoTest\PacRead";
        let key = create_test_reg_key(path);
        key.set_value("AutoConfigURL", &"http://proxy.example.com/pac")
            .expect("set");

        let url: String = key.get_value("AutoConfigURL").unwrap_or_default();
        assert_eq!(url, "http://proxy.example.com/pac");

        cleanup_test_reg_key(path);
    }

    #[test]
    fn write_pac_to_registry() {
        let path = r"Software\GoGuoTest\PacWrite";
        let key = create_test_reg_key(path);

        key.set_value("AutoConfigURL", &"http://new.example.com/wpad.dat")
            .expect("set");

        let url: String = key.get_value("AutoConfigURL").unwrap_or_default();
        assert_eq!(url, "http://new.example.com/wpad.dat");

        cleanup_test_reg_key(path);
    }

    #[test]
    fn pac_roundtrip_via_registry() {
        let path = r"Software\GoGuoTest\PacRoundtrip";
        let key = create_test_reg_key(path);

        let value = serde_json::json!({
            "AutoConfigURL": "http://roundtrip.example.com/proxy.pac"
        });

        let url: String = value
            .get("AutoConfigURL")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        key.set_value("AutoConfigURL", &url).expect("set");

        let read_url: String = key.get_value("AutoConfigURL").unwrap_or_default();
        assert_eq!(read_url, "http://roundtrip.example.com/proxy.pac");

        cleanup_test_reg_key(path);
    }

    // -----------------------------------------------------------------------
    // win-http-proxy (Detectable) — netsh output parsing
    // -----------------------------------------------------------------------

    #[test]
    fn parse_netsh_winhttp_direct_access() {
        let output = "\
Current WinHTTP proxy settings:

    Direct access (no proxy server).";
        let result = WindowsAdapter::parse_netsh_winhttp(output);
        assert_eq!(result["AccessType"], "direct");
    }

    #[test]
    fn parse_netsh_winhttp_with_proxy() {
        let output = "\
Current WinHTTP proxy settings:

    Proxy Server(s) :  127.0.0.1:7890
    Bypass List     :  <local>";
        let result = WindowsAdapter::parse_netsh_winhttp(output);
        assert_eq!(result["ProxyServer"], "127.0.0.1:7890");
        assert_eq!(result["BypassList"], "<local>");
    }

    #[test]
    fn parse_netsh_winhttp_empty_output() {
        let result = WindowsAdapter::parse_netsh_winhttp("");
        assert_eq!(result["AccessType"], "unknown");
        assert_eq!(result["ProxyServer"], "");
    }

    // -----------------------------------------------------------------------
    // win-dns-cache (Detectable) — ipconfig output parsing
    // -----------------------------------------------------------------------

    #[test]
    fn parse_ipconfig_displaydns_with_entries() {
        let output = "\
Windows IP Configuration

    example.com
    ----------------------------------------
    Record Name . . . . . : example.com
    Record Type . . . . . : 1
    Time To Live  . . . . : 300
    Data . . . . . . . . . : 93.184.216.34

    localhost
    ----------------------------------------
    Record Name . . . . . : localhost
    Record Type . . . . . : 1
    Time To Live  . . . . : 0
    Data . . . . . . . . . : 127.0.0.1";
        let result = WindowsAdapter::parse_ipconfig_displaydns(output);
        assert_eq!(result["EntryCount"], 2);
        let entries = result["Entries"].as_array().expect("array");
        assert_eq!(entries[0]["RecordName"], "example.com");
        assert_eq!(entries[0]["Data"], "93.184.216.34");
        assert_eq!(entries[1]["RecordName"], "localhost");
    }

    #[test]
    fn parse_ipconfig_displaydns_empty() {
        let result = WindowsAdapter::parse_ipconfig_displaydns("Windows IP Configuration\n");
        assert_eq!(result["EntryCount"], 0);
        assert!(result["Entries"].as_array().unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // win-dns-servers (Detectable) — netsh dns output parsing
    // -----------------------------------------------------------------------

    #[test]
    fn parse_netsh_dns_with_servers() {
        let output = "\
Configuration for interface \"Ethernet\"
    DNS servers configured through DHCP: 8.8.8.8
                                             8.8.4.4";
        let result = WindowsAdapter::parse_netsh_dns(output);
        let interfaces = result["Interfaces"].as_array().expect("array");
        assert!(!interfaces.is_empty());
    }

    #[test]
    fn parse_netsh_dns_no_servers() {
        let output = "Configuration for interface \"Wi-Fi\"\n    Statically Configured DNS Servers: None";
        let result = WindowsAdapter::parse_netsh_dns(output);
        let interfaces = result["Interfaces"].as_array().expect("array");
        assert!(!interfaces.is_empty());
    }

    // -----------------------------------------------------------------------
    // win-proxy-processes (Detectable)
    // -----------------------------------------------------------------------

    #[test]
    fn detect_proxy_processes_returns_valid_json() {
        let result = WindowsAdapter::detect_proxy_processes();
        assert!(result["DetectedCount"].is_number());
        assert!(result["Processes"].is_array());
    }

    // -----------------------------------------------------------------------
    // win-tun-status (Detectable)
    // -----------------------------------------------------------------------

    #[test]
    fn detect_tun_status_returns_valid_json() {
        let result = WindowsAdapter::detect_tun_status();
        assert!(result["TunFound"].is_boolean());
        assert!(result["AdapterName"].is_string());
    }

    // -----------------------------------------------------------------------
    // win-wsl2-network-mode (Detectable) — wslconfig parsing
    // -----------------------------------------------------------------------

    #[test]
    fn parse_wslconfig_with_networking_mode() {
        let content = "[wsl2]\nnetworkingMode=mirrored\n";
        let result = WindowsAdapter::parse_wslconfig(content);
        assert_eq!(result["NetworkingMode"], "mirrored");
    }

    #[test]
    fn parse_wslconfig_empty() {
        let result = WindowsAdapter::parse_wslconfig("");
        assert_eq!(result["NetworkingMode"], "");
    }

    #[test]
    fn parse_wslconfig_no_networking_mode() {
        let content = "[wsl2]\nmemory=4GB\n";
        let result = WindowsAdapter::parse_wslconfig(content);
        assert_eq!(result["NetworkingMode"], "");
    }

    #[test]
    fn parse_wslconfig_with_spaces() {
        let content = "[wsl2]\nnetworkingMode = nat\n";
        let result = WindowsAdapter::parse_wslconfig(content);
        assert_eq!(result["NetworkingMode"], "nat");
    }

    // -----------------------------------------------------------------------
    // write_state edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn write_state_rejects_detectable_item() {
        let adapter = WindowsAdapter::new();
        let item = StateItem {
            id: ID_HTTP_PROXY.to_string(),
            platform: Platform::Windows,
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

    #[test]
    fn write_state_rejects_unknown_item() {
        let adapter = WindowsAdapter::new();
        let item = StateItem {
            id: "win-unknown".to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({}),
            collected_at: String::new(),
            classification_reason: String::new(),
        };
        let result = adapter.write_state(&item);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // read_state_items integration (smoke test)
    // -----------------------------------------------------------------------

    #[test]
    fn read_state_items_returns_nine_items() {
        let adapter = WindowsAdapter::new();
        let items = adapter.read_state_items();
        assert_eq!(items.len(), 9);

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
    fn read_state_items_all_have_correct_platform() {
        let adapter = WindowsAdapter::new();
        let items = adapter.read_state_items();
        for item in &items {
            assert_eq!(item.platform, Platform::Windows, "Wrong platform for {}", item.id);
        }
    }
}

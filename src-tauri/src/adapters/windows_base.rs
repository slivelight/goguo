//! Pure parsing functions for Windows state items.
//!
//! Extracted from `windows.rs` so they can be reused by `WindowsRemoteAdapter`
//! without depending on `winreg` or `cfg(windows)`. All functions are pure
//! string → JSON transformations.

use serde_json;

// ---------------------------------------------------------------------------
// State item IDs (shared with windows.rs and windows_remote.rs)
// ---------------------------------------------------------------------------

pub const ID_HOSTS: &str = "win-hosts";
pub const ID_SYSTEM_PROXY: &str = "win-system-proxy";
pub const ID_PAC: &str = "win-pac";
pub const ID_HTTP_PROXY: &str = "win-http-proxy";
pub const ID_DNS_CACHE: &str = "win-dns-cache";
pub const ID_DNS_SERVERS: &str = "win-dns-servers";
pub const ID_PROXY_PROCESSES: &str = "win-proxy-processes";
pub const ID_TUN_STATUS: &str = "win-tun-status";
pub const ID_WSL2_NETWORK_MODE: &str = "win-wsl2-network-mode";

/// Known proxy process names for detection.
pub const KNOWN_PROXY_NAMES: &[&str] = &[
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

// ---------------------------------------------------------------------------
// Parsing functions
// ---------------------------------------------------------------------------

/// Parse `netsh winhttp show proxy` output into a JSON value.
pub fn parse_netsh_winhttp(output: &str) -> serde_json::Value {
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
pub fn parse_ipconfig_displaydns(output: &str) -> serde_json::Value {
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
pub fn parse_netsh_dns(output: &str) -> serde_json::Value {
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

/// Parse `.wslconfig` content into a JSON value.
pub fn parse_wslconfig(content: &str) -> serde_json::Value {
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

/// Parse `tasklist /FO CSV /NH` output for proxy-related processes.
pub fn parse_proxy_processes(output: &str) -> serde_json::Value {
    let mut detected: Vec<serde_json::Value> = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split("\",\"").collect();
        if parts.is_empty() {
            continue;
        }
        let proc_name = parts[0].trim_matches('"').to_lowercase();
        for &known in KNOWN_PROXY_NAMES {
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

    serde_json::json!({
        "DetectedCount": detected.len(),
        "Processes": detected,
    })
}

/// Parse `netsh interface show interface` output for TUN/TAP adapter detection.
pub fn parse_tun_status(output: &str) -> serde_json::Value {
    let mut found = false;
    let mut adapter_name = String::new();

    for line in output.lines() {
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

    serde_json::json!({
        "TunFound": found,
        "AdapterName": adapter_name,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ----- parse_netsh_winhttp -----

    #[test]
    fn parse_netsh_winhttp_direct_access() {
        let output = "\
Current WinHTTP proxy settings:

    Direct access (no proxy server).";
        let result = parse_netsh_winhttp(output);
        assert_eq!(result["AccessType"], "direct");
    }

    #[test]
    fn parse_netsh_winhttp_with_proxy() {
        let output = "\
Current WinHTTP proxy settings:

    Proxy Server(s) :  127.0.0.1:7890
    Bypass List     :  <local>";
        let result = parse_netsh_winhttp(output);
        assert_eq!(result["ProxyServer"], "127.0.0.1:7890");
        assert_eq!(result["BypassList"], "<local>");
    }

    #[test]
    fn parse_netsh_winhttp_empty_output() {
        let result = parse_netsh_winhttp("");
        assert_eq!(result["AccessType"], "unknown");
        assert_eq!(result["ProxyServer"], "");
    }

    // ----- parse_ipconfig_displaydns -----

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
        let result = parse_ipconfig_displaydns(output);
        assert_eq!(result["EntryCount"], 2);
        let entries = result["Entries"].as_array().expect("array");
        assert_eq!(entries[0]["RecordName"], "example.com");
        assert_eq!(entries[0]["Data"], "93.184.216.34");
        assert_eq!(entries[1]["RecordName"], "localhost");
    }

    #[test]
    fn parse_ipconfig_displaydns_empty() {
        let result = parse_ipconfig_displaydns("Windows IP Configuration\n");
        assert_eq!(result["EntryCount"], 0);
        assert!(result["Entries"].as_array().unwrap().is_empty());
    }

    // ----- parse_netsh_dns -----

    #[test]
    fn parse_netsh_dns_with_servers() {
        let output = "\
Configuration for interface \"Ethernet\"
    DNS servers configured through DHCP: 8.8.8.8
                                             8.8.4.4";
        let result = parse_netsh_dns(output);
        let interfaces = result["Interfaces"].as_array().expect("array");
        assert!(!interfaces.is_empty());
    }

    #[test]
    fn parse_netsh_dns_no_servers() {
        let output = "Configuration for interface \"Wi-Fi\"\n    Statically Configured DNS Servers: None";
        let result = parse_netsh_dns(output);
        let interfaces = result["Interfaces"].as_array().expect("array");
        assert!(!interfaces.is_empty());
    }

    // ----- parse_wslconfig -----

    #[test]
    fn parse_wslconfig_with_networking_mode() {
        let content = "[wsl2]\nnetworkingMode=mirrored\n";
        let result = parse_wslconfig(content);
        assert_eq!(result["NetworkingMode"], "mirrored");
    }

    #[test]
    fn parse_wslconfig_empty() {
        let result = parse_wslconfig("");
        assert_eq!(result["NetworkingMode"], "");
    }

    #[test]
    fn parse_wslconfig_no_networking_mode() {
        let content = "[wsl2]\nmemory=4GB\n";
        let result = parse_wslconfig(content);
        assert_eq!(result["NetworkingMode"], "");
    }

    #[test]
    fn parse_wslconfig_with_spaces() {
        let content = "[wsl2]\nnetworkingMode = nat\n";
        let result = parse_wslconfig(content);
        assert_eq!(result["NetworkingMode"], "nat");
    }

    // ----- parse_proxy_processes -----

    #[test]
    fn parse_proxy_processes_detects_mihomo() {
        let output = "\"mihomo.exe\",\"1234\",\"Services\",\"0\",\"2,456 K\"\n\"svchost.exe\",\"5678\",\"Services\",\"0\",\"12,345 K\"";
        let result = parse_proxy_processes(output);
        assert_eq!(result["DetectedCount"], 1);
        let procs = result["Processes"].as_array().unwrap();
        assert_eq!(procs[0]["MatchedKeyword"], "mihomo");
        assert_eq!(procs[0]["PID"], "1234");
    }

    #[test]
    fn parse_proxy_processes_empty() {
        let result = parse_proxy_processes("");
        assert_eq!(result["DetectedCount"], 0);
        assert!(result["Processes"].as_array().unwrap().is_empty());
    }

    #[test]
    fn parse_proxy_processes_no_match() {
        let output = "\"svchost.exe\",\"1234\",\"Services\",\"0\",\"2,456 K\"";
        let result = parse_proxy_processes(output);
        assert_eq!(result["DetectedCount"], 0);
    }

    // ----- parse_tun_status -----

    #[test]
    fn parse_tun_status_found() {
        let output = "Admin State    State          Type             Interface Name\n -------------------------------------------------------------------------\n Enabled        Connected      Dedicated        Wi-Fi\n Enabled        Connected      Dedicated        Clash TUN";
        let result = parse_tun_status(output);
        assert_eq!(result["TunFound"], true);
    }

    #[test]
    fn parse_tun_status_not_found() {
        let output = "Admin State    State          Type             Interface Name\n -------------------------------------------------------------------------\n Enabled        Connected      Dedicated        Wi-Fi\n Enabled        Connected      Dedicated        Ethernet";
        let result = parse_tun_status(output);
        assert_eq!(result["TunFound"], false);
    }

    #[test]
    fn parse_tun_status_empty() {
        let result = parse_tun_status("");
        assert_eq!(result["TunFound"], false);
    }

    // ----- Constants -----

    #[test]
    fn known_proxy_names_contains_key_entries() {
        assert!(KNOWN_PROXY_NAMES.contains(&"clash"));
        assert!(KNOWN_PROXY_NAMES.contains(&"mihomo"));
        assert!(KNOWN_PROXY_NAMES.contains(&"v2ray"));
    }
}

use crate::models::node::ProxyProtocol;
use crate::models::subscription::{ParsedNode, ParseResult, SubscriptionSource};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs;
use std::io;
use std::path::PathBuf;

const SUPPORTED_PROTOCOLS: [&str; 5] = ["vless", "vmess", "trojan", "shadowsocks", "hysteria2"];

#[derive(Clone)]
pub struct SubscriptionParser {
    sources_file: PathBuf,
}

impl SubscriptionParser {
    #[must_use]
    pub fn new(sources_file: PathBuf) -> Self {
        Self { sources_file }
    }

    #[must_use] 
    pub fn parse_raw_content(content: &str) -> ParseResult {
        let decoded = Self::decode_base64(content);
        Self::parse_nodes_from_lines(&decoded)
    }

    fn decode_base64(content: &str) -> String {
        let trimmed = content.trim();
        STANDARD
            .decode(trimmed)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_else(|| trimmed.to_string())
    }

    fn parse_nodes_from_lines(content: &str) -> ParseResult {
        let mut result = ParseResult::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            let node = Self::parse_node_url(trimmed);
            if let Some(n) = node {
                if Self::is_protocol_supported(&n.protocol) {
                    result.add_supported(n);
                } else {
                    result.add_unsupported(n);
                }
            }
        }
        
        result
    }

    fn parse_node_url(url: &str) -> Option<ParsedNode> {
        let protocol = url.split("://").next()?;
        let protocol_lower = protocol.to_lowercase();
        
        let name = Self::extract_name(url, &protocol_lower)?;
        let address = Self::extract_address(url, &protocol_lower)?;
        
        Some(ParsedNode::new(
            name,
            address,
            protocol_lower,
            url.to_string(),
        ))
    }

    fn extract_name(url: &str, protocol: &str) -> Option<String> {
        let after_scheme = url.strip_prefix(&format!("{protocol}://"))?;
        
        let hash_part = after_scheme.split('#').next_back()?;
        if !hash_part.is_empty() {
            return Some(
                urlencoding_decode(hash_part)
            );
        }
        
        let name_part = after_scheme.split('@').next()?;
        Some(name_part.to_string())
    }

    fn extract_address(url: &str, protocol: &str) -> Option<String> {
        let after_scheme = url.strip_prefix(&format!("{protocol}://"))?;
        
        if protocol == "vmess" {
            return Self::extract_vmess_address(after_scheme);
        }
        
        let after_at = after_scheme.split('@').nth(1)?;
        let host_port = after_at.split('?').next()?;
        Some(host_port.to_string())
    }

    fn extract_vmess_address(encoded: &str) -> Option<String> {
        let decoded = STANDARD
            .decode(encoded)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())?;
        
        let json: serde_json::Value = serde_json::from_str(&decoded).ok()?;
        let add = json.get("add")?.as_str()?;
        let port = json.get("port")?.as_str()?;
        Some(format!("{add}:{port}"))
    }

    #[must_use]
    fn is_protocol_supported(protocol: &str) -> bool {
        SUPPORTED_PROTOCOLS.contains(&protocol)
    }

    pub fn load_sources(&self) -> io::Result<Vec<SubscriptionSource>> {
        if !self.sources_file.exists() {
            return Ok(vec![]);
        }
        
        let content = fs::read_to_string(&self.sources_file)?;
        let sources: Vec<SubscriptionSource> = serde_json::from_str(&content)?;
        Ok(sources)
    }

    pub fn save_sources(&self, sources: &[SubscriptionSource]) -> io::Result<()> {
        if let Some(parent) = self.sources_file.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(sources)?;
        fs::write(&self.sources_file, content)?;
        Ok(())
    }

    pub fn add_source(&mut self, source: SubscriptionSource) -> io::Result<bool> {
        let mut sources = self.load_sources()?;
        
        if sources.iter().any(|s| s.url == source.url) {
            return Ok(false);
        }
        
        sources.push(source);
        self.save_sources(&sources)?;
        Ok(true)
    }

    pub fn remove_source(&mut self, url: &str) -> io::Result<bool> {
        let mut sources = self.load_sources()?;
        
        let idx = sources.iter().position(|s| s.url == url);
        if let Some(i) = idx {
            sources.remove(i);
            self.save_sources(&sources)?;
            return Ok(true);
        }
        
        Ok(false)
    }

    #[must_use]
    pub fn protocol_to_proxy_protocol(protocol: &str) -> Option<ProxyProtocol> {
        match protocol {
            "vless" => Some(ProxyProtocol::Vless),
            "vmess" => Some(ProxyProtocol::Vmess),
            "trojan" => Some(ProxyProtocol::Trojan),
            "shadowsocks" => Some(ProxyProtocol::Shadowsocks),
            "hysteria2" => Some(ProxyProtocol::Hysteria2),
            _ => None,
        }
    }
}

fn urlencoding_decode(s: &str) -> String {
    let mut result = String::new();
    let chars = s.chars().collect::<Vec<_>>();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                result.push(byte as char);
                i += 3;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    
    result
}

impl Default for SubscriptionParser {
    fn default() -> Self {
        Self::new(PathBuf::from("data/config/subscription-sources.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn decode_base64_simple() {
        let encoded = STANDARD.encode("test content");
        let decoded = SubscriptionParser::decode_base64(&encoded);
        assert_eq!(decoded, "test content");
    }

    #[test]
    fn decode_base64_invalid_returns_original() {
        let content = "not base64!";
        let decoded = SubscriptionParser::decode_base64(content);
        assert_eq!(decoded, content);
    }

    #[test]
    fn parse_nodes_from_lines_empty() {
        let result = SubscriptionParser::parse_nodes_from_lines("");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_nodes_from_lines_single_vless() {
        let content = "vless://uuid@server.com:443?type=tcp#MyNode";
        let result = SubscriptionParser::parse_nodes_from_lines(content);
        
        assert_eq!(result.total_count, 1);
        assert_eq!(result.supported_count, 1);
        assert_eq!(result.supported_nodes[0].protocol, "vless");
        assert_eq!(result.supported_nodes[0].name, "MyNode");
    }

    #[test]
    fn parse_nodes_from_lines_vmess_base64() {
        let vmess_json = serde_json::json!({
            "add": "server.com",
            "port": "443",
            "ps": "VMessNode"
        });
        let encoded = STANDARD.encode(vmess_json.to_string());
        let content = format!("vmess://{encoded}");
        
        let result = SubscriptionParser::parse_nodes_from_lines(&content);
        
        assert_eq!(result.total_count, 1);
        assert_eq!(result.supported_count, 1);
        assert_eq!(result.supported_nodes[0].protocol, "vmess");
        assert!(result.supported_nodes[0].address.contains("server.com"));
    }

    #[test]
    fn parse_nodes_from_lines_trojan() {
        let content = "trojan://password@server.com:443?security=tls#TrojanNode";
        let result = SubscriptionParser::parse_nodes_from_lines(content);
        
        assert_eq!(result.supported_count, 1);
        assert_eq!(result.supported_nodes[0].protocol, "trojan");
        assert_eq!(result.supported_nodes[0].name, "TrojanNode");
    }

    #[test]
    fn parse_nodes_from_lines_shadowsocks() {
        let content = "shadowsocks://base64@server.com:8388#SSNode";
        let result = SubscriptionParser::parse_nodes_from_lines(content);
        
        assert_eq!(result.supported_count, 1);
        assert_eq!(result.supported_nodes[0].protocol, "shadowsocks");
    }

    #[test]
    fn parse_nodes_from_lines_hysteria2() {
        let content = "hysteria2://password@server.com:443#H2Node";
        let result = SubscriptionParser::parse_nodes_from_lines(content);
        
        assert_eq!(result.supported_count, 1);
        assert_eq!(result.supported_nodes[0].protocol, "hysteria2");
    }

    #[test]
    fn parse_nodes_from_lines_unsupported_protocol() {
        let content = "ssr://password@server.com:443#SSRNode\nvless://uuid@server.com:443#VlessNode";
        let result = SubscriptionParser::parse_nodes_from_lines(content);
        
        assert_eq!(result.total_count, 2);
        assert_eq!(result.supported_count, 1);
        assert_eq!(result.unsupported_count, 1);
        assert_eq!(result.unsupported_nodes[0].protocol, "ssr");
    }

    #[test]
    fn is_protocol_supported_true() {
        for p in SUPPORTED_PROTOCOLS {
            assert!(SubscriptionParser::is_protocol_supported(p));
        }
    }

    #[test]
    fn is_protocol_supported_false() {
        assert!(!SubscriptionParser::is_protocol_supported("ssr"));
        assert!(!SubscriptionParser::is_protocol_supported("http"));
    }

    #[test]
    fn protocol_to_proxy_protocol_mapping() {
        assert_eq!(
            SubscriptionParser::protocol_to_proxy_protocol("vless"),
            Some(ProxyProtocol::Vless)
        );
        assert_eq!(
            SubscriptionParser::protocol_to_proxy_protocol("vmess"),
            Some(ProxyProtocol::Vmess)
        );
        assert!(SubscriptionParser::protocol_to_proxy_protocol("unknown").is_none());
    }

    #[test]
    fn parse_raw_content_full_flow() {
        let nodes = "vless://uuid@server.com:443#Node1\nssr://pass@server.com:443#Node2\nvless://uuid2@server2.com:443#Node3";
        let encoded = STANDARD.encode(nodes);
        
        let result = SubscriptionParser::parse_raw_content(&encoded);
        
        assert_eq!(result.total_count, 3);
        assert_eq!(result.supported_count, 2);
        assert_eq!(result.unsupported_count, 1);
    }

    #[test]
    fn load_sources_empty_if_no_file() {
        let dir = tempdir().expect("tempdir");
        let parser = SubscriptionParser::new(dir.path().join("sources.json"));
        
        let sources = parser.load_sources().expect("load");
        assert!(sources.is_empty());
    }

    #[test]
    fn save_and_load_sources() {
        let dir = tempdir().expect("tempdir");
        let parser = SubscriptionParser::new(dir.path().join("sources.json"));
        
        let source = SubscriptionSource::new("https://example.com/sub".to_string());
        parser.save_sources(std::slice::from_ref(&source)).expect("save");
        
        let loaded = parser.load_sources().expect("load");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].url, source.url);
    }

    #[test]
    fn add_source_new() {
        let dir = tempdir().expect("tempdir");
        let mut parser = SubscriptionParser::new(dir.path().join("sources.json"));
        
        let source = SubscriptionSource::new("https://example.com/sub".to_string());
        let added = parser.add_source(source).expect("add");
        
        assert!(added);
        let sources = parser.load_sources().expect("load");
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn add_source_duplicate_rejected() {
        let dir = tempdir().expect("tempdir");
        let mut parser = SubscriptionParser::new(dir.path().join("sources.json"));
        
        let source = SubscriptionSource::new("https://example.com/sub".to_string());
        parser.add_source(source.clone()).expect("add");
        
        let added = parser.add_source(source).expect("add");
        assert!(!added);
        
        let sources = parser.load_sources().expect("load");
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn remove_source_existing() {
        let dir = tempdir().expect("tempdir");
        let mut parser = SubscriptionParser::new(dir.path().join("sources.json"));
        
        parser.add_source(SubscriptionSource::new("https://example.com/sub".to_string())).expect("add");
        
        let removed = parser.remove_source("https://example.com/sub").expect("remove");
        assert!(removed);
        
        let sources = parser.load_sources().expect("load");
        assert!(sources.is_empty());
    }

    #[test]
    fn remove_source_nonexistent() {
        let dir = tempdir().expect("tempdir");
        let mut parser = SubscriptionParser::new(dir.path().join("sources.json"));
        
        let removed = parser.remove_source("https://nonexistent.com").expect("remove");
        assert!(!removed);
    }

    #[test]
    fn parser_default() {
        let parser = SubscriptionParser::default();
        assert_eq!(parser.sources_file, PathBuf::from("data/config/subscription-sources.json"));
    }

    #[test]
    fn urlencoding_decode_simple() {
        let encoded = "Hello%20World";
        let decoded = urlencoding_decode(encoded);
        assert_eq!(decoded, "Hello World".to_string());
    }
}
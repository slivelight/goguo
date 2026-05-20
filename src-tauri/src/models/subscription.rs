use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionSource {
    pub url: String,
    pub name: Option<String>,
    pub added_at: DateTime<Utc>,
    pub last_update: Option<DateTime<Utc>>,
    pub node_count: Option<usize>,
}

impl SubscriptionSource {
    #[must_use]
    pub fn new(url: String) -> Self {
        Self {
            url,
            name: None,
            added_at: Utc::now(),
            last_update: None,
            node_count: None,
        }
    }

    #[must_use]
    pub fn with_name(url: String, name: String) -> Self {
        Self {
            url,
            name: Some(name),
            added_at: Utc::now(),
            last_update: None,
            node_count: None,
        }
    }

    pub fn update(&mut self, node_count: usize) {
        self.last_update = Some(Utc::now());
        self.node_count = Some(node_count);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedNode {
    pub name: String,
    pub address: String,
    pub protocol: String,
    pub raw_url: String,
}

impl ParsedNode {
    #[must_use]
    pub fn new(name: String, address: String, protocol: String, raw_url: String) -> Self {
        Self {
            name,
            address,
            protocol,
            raw_url,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub total_count: usize,
    pub supported_count: usize,
    pub unsupported_count: usize,
    pub supported_nodes: Vec<ParsedNode>,
    pub unsupported_nodes: Vec<ParsedNode>,
}

impl ParseResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_count: 0,
            supported_count: 0,
            unsupported_count: 0,
            supported_nodes: vec![],
            unsupported_nodes: vec![],
        }
    }

    pub fn add_supported(&mut self, node: ParsedNode) {
        self.total_count += 1;
        self.supported_count += 1;
        self.supported_nodes.push(node);
    }

    pub fn add_unsupported(&mut self, node: ParsedNode) {
        self.total_count += 1;
        self.unsupported_count += 1;
        self.unsupported_nodes.push(node);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_count == 0
    }
}

impl Default for ParseResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_source_new() {
        let source = SubscriptionSource::new("https://example.com/sub".to_string());
        assert_eq!(source.url, "https://example.com/sub");
        assert!(source.name.is_none());
        assert!(source.last_update.is_none());
        assert!(source.node_count.is_none());
    }

    #[test]
    fn subscription_source_with_name() {
        let source = SubscriptionSource::with_name(
            "https://example.com/sub".to_string(),
            "MySubscription".to_string(),
        );
        assert_eq!(source.name, Some("MySubscription".to_string()));
    }

    #[test]
    fn subscription_source_update() {
        let mut source = SubscriptionSource::new("https://example.com/sub".to_string());
        source.update(10);
        
        assert!(source.last_update.is_some());
        assert_eq!(source.node_count, Some(10));
    }

    #[test]
    fn subscription_source_roundtrip() {
        let source = SubscriptionSource::new("https://example.com/sub".to_string());
        let json = serde_json::to_string(&source).expect("serialize");
        let back: SubscriptionSource = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.url, source.url);
    }

    #[test]
    fn parsed_node_new() {
        let node = ParsedNode::new(
            "node1".to_string(),
            "127.0.0.1:1080".to_string(),
            "vless".to_string(),
            "vless://...".to_string(),
        );
        assert_eq!(node.name, "node1");
        assert_eq!(node.address, "127.0.0.1:1080");
        assert_eq!(node.protocol, "vless");
    }

    #[test]
    fn parsed_node_roundtrip() {
        let node = ParsedNode::new(
            "test".to_string(),
            "1.2.3.4:443".to_string(),
            "vmess".to_string(),
            "vmess://...".to_string(),
        );
        let json = serde_json::to_string(&node).expect("serialize");
        let back: ParsedNode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.name, node.name);
    }

    #[test]
    fn parse_result_new_empty() {
        let result = ParseResult::new();
        assert_eq!(result.total_count, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn parse_result_add_supported() {
        let mut result = ParseResult::new();
        result.add_supported(ParsedNode::new(
            "node1".to_string(),
            "127.0.0.1:1080".to_string(),
            "vless".to_string(),
            "url".to_string(),
        ));
        
        assert_eq!(result.total_count, 1);
        assert_eq!(result.supported_count, 1);
        assert_eq!(result.unsupported_count, 0);
        assert_eq!(result.supported_nodes.len(), 1);
    }

    #[test]
    fn parse_result_add_unsupported() {
        let mut result = ParseResult::new();
        result.add_unsupported(ParsedNode::new(
            "node2".to_string(),
            "127.0.0.1:1081".to_string(),
            "unknown".to_string(),
            "url".to_string(),
        ));
        
        assert_eq!(result.total_count, 1);
        assert_eq!(result.supported_count, 0);
        assert_eq!(result.unsupported_count, 1);
        assert_eq!(result.unsupported_nodes.len(), 1);
    }

    #[test]
    fn parse_result_mixed() {
        let mut result = ParseResult::new();
        result.add_supported(ParsedNode::new("s1".to_string(), "a1".to_string(), "vless".to_string(), "u1".to_string()));
        result.add_unsupported(ParsedNode::new("u1".to_string(), "a2".to_string(), "ssr".to_string(), "u2".to_string()));
        result.add_supported(ParsedNode::new("s2".to_string(), "a3".to_string(), "vmess".to_string(), "u3".to_string()));
        
        assert_eq!(result.total_count, 3);
        assert_eq!(result.supported_count, 2);
        assert_eq!(result.unsupported_count, 1);
    }

    #[test]
    fn parse_result_default() {
        let result = ParseResult::default();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_result_roundtrip() {
        let mut result = ParseResult::new();
        result.add_supported(ParsedNode::new("n1".to_string(), "a1".to_string(), "vless".to_string(), "u1".to_string()));
        
        let json = serde_json::to_string(&result).expect("serialize");
        let back: ParseResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.total_count, 1);
        assert_eq!(back.supported_nodes.len(), 1);
    }
}
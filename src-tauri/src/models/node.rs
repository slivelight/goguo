use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyProtocol {
    Vless,
    Vmess,
    Trojan,
    Shadowsocks,
    Hysteria2,
}

impl ProxyProtocol {
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Vless,
            Self::Vmess,
            Self::Trojan,
            Self::Shadowsocks,
            Self::Hysteria2,
        ]
    }

    #[must_use]
    pub const fn is_supported(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Available,
    Unhealthy,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyNode {
    pub name: String,
    pub address: String,
    pub protocol: ProxyProtocol,
    pub joined_at: DateTime<Utc>,
    pub status: NodeStatus,
    pub consecutive_failures: u32,
    pub last_latency_ms: Option<u64>,
}

impl ProxyNode {
    #[must_use]
    pub fn new(name: String, address: String, protocol: ProxyProtocol) -> Self {
        Self {
            name,
            address,
            protocol,
            joined_at: Utc::now(),
            status: NodeStatus::Available,
            consecutive_failures: 0,
            last_latency_ms: None,
        }
    }

    pub fn mark_healthy(&mut self, latency_ms: u64) {
        self.status = NodeStatus::Available;
        self.consecutive_failures = 0;
        self.last_latency_ms = Some(latency_ms);
    }

    pub fn mark_unhealthy(&mut self) {
        self.consecutive_failures += 1;
        self.status = NodeStatus::Unhealthy;
    }

    pub fn mark_removed(&mut self) {
        self.status = NodeStatus::Removed;
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        self.status == NodeStatus::Available
    }

    #[must_use]
    pub fn is_removed(&self) -> bool {
        self.status == NodeStatus::Removed
    }

    #[must_use]
    pub fn socket_addr(&self) -> Option<SocketAddr> {
        self.address.parse().ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealthResult {
    pub node_name: String,
    pub reachable: bool,
    pub latency_ms: Option<u64>,
    pub method: String,
    pub checked_at: DateTime<Utc>,
}

impl NodeHealthResult {
    #[must_use]
    pub fn healthy(node_name: String, latency_ms: u64, method: String) -> Self {
        Self {
            node_name,
            reachable: true,
            latency_ms: Some(latency_ms),
            method,
            checked_at: Utc::now(),
        }
    }

    #[must_use]
    pub fn unhealthy(node_name: String, method: String) -> Self {
        Self {
            node_name,
            reachable: false,
            latency_ms: None,
            method,
            checked_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_protocol_all_count() {
        assert_eq!(ProxyProtocol::all().len(), 5);
    }

    #[test]
    fn proxy_protocol_roundtrip() {
        for p in ProxyProtocol::all() {
            let json = serde_json::to_string(&p).expect("serialize");
            let back: ProxyProtocol = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, p);
        }
    }

    #[test]
    fn proxy_protocol_snake_case() {
        assert_eq!(
            serde_json::to_string(&ProxyProtocol::Vless).expect("serialize"),
            "\"vless\""
        );
        assert_eq!(
            serde_json::to_string(&ProxyProtocol::Hysteria2).expect("serialize"),
            "\"hysteria2\""
        );
    }

    #[test]
    fn proxy_protocol_is_supported() {
        for p in ProxyProtocol::all() {
            assert!(p.is_supported());
        }
    }

    #[test]
    fn node_status_roundtrip() {
        for s in [NodeStatus::Available, NodeStatus::Unhealthy, NodeStatus::Removed] {
            let json = serde_json::to_string(&s).expect("serialize");
            let back: NodeStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn proxy_node_new() {
        let node = ProxyNode::new(
            "node1".to_string(),
            "127.0.0.1:1080".to_string(),
            ProxyProtocol::Vless,
        );
        assert_eq!(node.name, "node1");
        assert_eq!(node.protocol, ProxyProtocol::Vless);
        assert_eq!(node.status, NodeStatus::Available);
        assert_eq!(node.consecutive_failures, 0);
        assert!(node.last_latency_ms.is_none());
    }

    #[test]
    fn proxy_node_mark_healthy() {
        let mut node = ProxyNode::new(
            "test".to_string(),
            "127.0.0.1:1080".to_string(),
            ProxyProtocol::Vmess,
        );
        node.consecutive_failures = 2;
        
        node.mark_healthy(100);
        
        assert_eq!(node.status, NodeStatus::Available);
        assert_eq!(node.consecutive_failures, 0);
        assert_eq!(node.last_latency_ms, Some(100));
    }

    #[test]
    fn proxy_node_mark_unhealthy() {
        let mut node = ProxyNode::new(
            "test".to_string(),
            "127.0.0.1:1080".to_string(),
            ProxyProtocol::Trojan,
        );
        
        node.mark_unhealthy();
        
        assert_eq!(node.consecutive_failures, 1);
        assert_eq!(node.status, NodeStatus::Unhealthy);
    }

    #[test]
    fn proxy_node_mark_removed() {
        let mut node = ProxyNode::new(
            "test".to_string(),
            "127.0.0.1:1080".to_string(),
            ProxyProtocol::Shadowsocks,
        );
        
        node.mark_removed();
        
        assert_eq!(node.status, NodeStatus::Removed);
    }

    #[test]
    fn proxy_node_is_available() {
        let mut node = ProxyNode::new(
            "test".to_string(),
            "127.0.0.1:1080".to_string(),
            ProxyProtocol::Hysteria2,
        );
        
        assert!(node.is_available());
        
        node.mark_unhealthy();
        assert!(!node.is_available());
        
        node.mark_removed();
        assert!(!node.is_available());
        assert!(node.is_removed());
    }

    #[test]
    fn proxy_node_socket_addr() {
        let node = ProxyNode::new(
            "test".to_string(),
            "127.0.0.1:1080".to_string(),
            ProxyProtocol::Vless,
        );
        
        let addr = node.socket_addr().expect("parse");
        assert_eq!(addr.port(), 1080);
    }

    #[test]
    fn proxy_node_socket_addr_invalid() {
        let node = ProxyNode::new(
            "test".to_string(),
            "invalid".to_string(),
            ProxyProtocol::Vless,
        );
        
        assert!(node.socket_addr().is_none());
    }

    #[test]
    fn proxy_node_roundtrip() {
        let node = ProxyNode::new(
            "node1".to_string(),
            "127.0.0.1:1080".to_string(),
            ProxyProtocol::Vless,
        );
        
        let json = serde_json::to_string(&node).expect("serialize");
        let back: ProxyNode = serde_json::from_str(&json).expect("deserialize");
        
        assert_eq!(back.name, node.name);
        assert_eq!(back.address, node.address);
        assert_eq!(back.protocol, node.protocol);
        assert_eq!(back.status, node.status);
    }

    #[test]
    fn node_health_result_healthy() {
        let result = NodeHealthResult::healthy("node1".to_string(), 150, "mihomo-api".to_string());
        assert!(result.reachable);
        assert_eq!(result.latency_ms, Some(150));
        assert_eq!(result.method, "mihomo-api");
    }

    #[test]
    fn node_health_result_unhealthy() {
        let result = NodeHealthResult::unhealthy("node1".to_string(), "tcp-connect".to_string());
        assert!(!result.reachable);
        assert!(result.latency_ms.is_none());
        assert_eq!(result.method, "tcp-connect");
    }

    #[test]
    fn node_health_result_roundtrip() {
        let result = NodeHealthResult::healthy("node1".to_string(), 100, "api".to_string());
        let json = serde_json::to_string(&result).expect("serialize");
        let back: NodeHealthResult = serde_json::from_str(&json).expect("deserialize");
        
        assert_eq!(back.node_name, result.node_name);
        assert_eq!(back.reachable, result.reachable);
        assert_eq!(back.latency_ms, result.latency_ms);
    }
}
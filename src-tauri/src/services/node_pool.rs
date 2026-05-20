use crate::models::node::{NodeHealthResult, ProxyNode};
use std::time::Duration;

pub trait NodeHealthChecker: Send + Sync {
    fn check_node(&self, node: &ProxyNode) -> NodeHealthResult;
}

pub struct MockNodeHealthChecker {
    results: std::collections::HashMap<String, NodeHealthResult>,
}

impl MockNodeHealthChecker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: std::collections::HashMap::new(),
        }
    }

    pub fn set_result(&mut self, node_name: &str, result: NodeHealthResult) {
        self.results.insert(node_name.to_string(), result);
    }
}

impl Default for MockNodeHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeHealthChecker for MockNodeHealthChecker {
    fn check_node(&self, node: &ProxyNode) -> NodeHealthResult {
        self.results
            .get(&node.name)
            .cloned()
            .unwrap_or_else(|| {
                NodeHealthResult::healthy(node.name.clone(), 100, "mock".to_string())
            })
    }
}

#[derive(Clone)]
pub struct NodePoolConfig {
    pub failure_threshold: u32,
    pub check_interval: Duration,
}

impl Default for NodePoolConfig {
    #[allow(clippy::duration_subsec)]
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            check_interval: Duration::from_mins(1),
        }
    }
}

#[derive(Clone)]
pub struct NodePool {
    nodes: Vec<ProxyNode>,
    current_index: usize,
    config: NodePoolConfig,
}

impl NodePool {
    #[must_use]
    pub fn new(config: NodePoolConfig) -> Self {
        Self {
            nodes: vec![],
            current_index: 0,
            config,
        }
    }

    pub fn add_node(&mut self, node: ProxyNode) {
        if self.nodes.iter().any(|n| n.name == node.name) {
            return;
        }
        self.nodes.push(node);
    }

    pub fn remove_node(&mut self, name: &str) -> bool {
        let idx = self.nodes.iter().position(|n| n.name == name);
        if let Some(i) = idx {
            if i < self.current_index && self.current_index > 0 {
                self.current_index -= 1;
            }
            self.nodes.remove(i);
            return true;
        }
        false
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn available_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_available()).count()
    }

    #[must_use]
    pub fn current_node(&self) -> Option<&ProxyNode> {
        self.nodes.get(self.current_index)
    }

    pub fn switch_to_next(&mut self) -> Option<&ProxyNode> {
        if self.nodes.is_empty() {
            return None;
        }

        let start = self.current_index;
        for i in 0..self.nodes.len() {
            let idx = (start + i + 1) % self.nodes.len();
            if self.nodes[idx].is_available() {
                self.current_index = idx;
                return Some(&self.nodes[idx]);
            }
        }
        None
    }

    pub fn process_health_result(&mut self, result: &NodeHealthResult) {
        let node = self.nodes.iter_mut().find(|n| n.name == result.node_name);
        if let Some(n) = node {
            if result.reachable {
                n.mark_healthy(result.latency_ms.unwrap_or(0));
            } else {
                n.mark_unhealthy();
                if n.consecutive_failures >= self.config.failure_threshold {
                    n.mark_removed();
                }
            }
        }
    }

    pub fn check_and_update(&mut self, checker: &dyn NodeHealthChecker) -> Vec<NodeHealthResult> {
        let results: Vec<NodeHealthResult> = self
            .nodes
            .iter()
            .filter(|n| !n.is_removed())
            .map(|n| checker.check_node(n))
            .collect();

        for result in &results {
            self.process_health_result(result);
        }

        results
    }

    #[must_use]
    pub fn nodes(&self) -> &Vec<ProxyNode> {
        &self.nodes
    }

    #[must_use]
    pub fn get_node(&self, name: &str) -> Option<&ProxyNode> {
        self.nodes.iter().find(|n| n.name == name)
    }

    #[must_use]
    pub const fn config(&self) -> &NodePoolConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::node::NodeStatus;
    use crate::models::node::ProxyProtocol;

    #[test]
    fn mock_health_checker_default_returns_healthy() {
        let checker = MockNodeHealthChecker::new();
        let node = ProxyNode::new("test".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless);
        let result = checker.check_node(&node);
        assert!(result.reachable);
    }

    #[test]
    fn mock_health_checker_override() {
        let mut checker = MockNodeHealthChecker::new();
        checker.set_result(
            "node1",
            NodeHealthResult::unhealthy("node1".to_string(), "tcp".to_string()),
        );
        
        let node = ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless);
        let result = checker.check_node(&node);
        assert!(!result.reachable);
    }

    #[test]
    fn node_pool_new_empty() {
        let config = NodePoolConfig::default();
        let pool = NodePool::new(config);
        assert_eq!(pool.node_count(), 0);
        assert!(pool.current_node().is_none());
    }

    #[test]
    fn node_pool_add_node() {
        let config = NodePoolConfig::default();
        let mut pool = NodePool::new(config);
        
        let node = ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless);
        pool.add_node(node);
        
        assert_eq!(pool.node_count(), 1);
        assert!(pool.current_node().is_some());
    }

    #[test]
    fn node_pool_add_duplicate_ignored() {
        let config = NodePoolConfig::default();
        let mut pool = NodePool::new(config);
        
        pool.add_node(ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless));
        pool.add_node(ProxyNode::new("node1".to_string(), "127.0.0.1:1081".to_string(), ProxyProtocol::Vmess));
        
        assert_eq!(pool.node_count(), 1);
    }

    #[test]
    fn node_pool_remove_node() {
        let config = NodePoolConfig::default();
        let mut pool = NodePool::new(config);
        
        pool.add_node(ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless));
        pool.add_node(ProxyNode::new("node2".to_string(), "127.0.0.1:1081".to_string(), ProxyProtocol::Vmess));
        
        let removed = pool.remove_node("node1");
        assert!(removed);
        assert_eq!(pool.node_count(), 1);
    }

    #[test]
    fn node_pool_remove_nonexistent() {
        let config = NodePoolConfig::default();
        let mut pool = NodePool::new(config);
        
        let removed = pool.remove_node("nonexistent");
        assert!(!removed);
    }

    #[test]
    fn node_pool_switch_to_next() {
        let config = NodePoolConfig::default();
        let mut pool = NodePool::new(config);
        
        pool.add_node(ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless));
        pool.add_node(ProxyNode::new("node2".to_string(), "127.0.0.1:1081".to_string(), ProxyProtocol::Vmess));
        pool.add_node(ProxyNode::new("node3".to_string(), "127.0.0.1:1082".to_string(), ProxyProtocol::Trojan));
        
        assert_eq!(pool.current_node().expect("current").name, "node1");
        
        let next = pool.switch_to_next();
        assert_eq!(next.expect("next").name, "node2");
        assert_eq!(pool.current_node().expect("current").name, "node2");
    }

    #[test]
    fn node_pool_switch_skips_unhealthy() {
        let config = NodePoolConfig::default();
        let mut pool = NodePool::new(config);
        
        let mut node1 = ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless);
        node1.mark_unhealthy();
        
        pool.add_node(node1);
        pool.add_node(ProxyNode::new("node2".to_string(), "127.0.0.1:1081".to_string(), ProxyProtocol::Vmess));
        
        let current = pool.current_node();
        assert_eq!(current.expect("current").name, "node1");
        
        let next = pool.switch_to_next();
        assert_eq!(next.expect("next").name, "node2");
        assert_eq!(pool.available_count(), 1);
    }

    #[test]
    fn node_pool_process_health_healthy() {
        let config = NodePoolConfig::default();
        let mut pool = NodePool::new(config);
        
        let mut node = ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless);
        node.consecutive_failures = 2;
        pool.add_node(node);
        
        let result = NodeHealthResult::healthy("node1".to_string(), 150, "api".to_string());
        pool.process_health_result(&result);
        
        let updated = pool.get_node("node1").expect("found");
        assert_eq!(updated.status, NodeStatus::Available);
        assert_eq!(updated.consecutive_failures, 0);
        assert_eq!(updated.last_latency_ms, Some(150));
    }

    #[test]
    fn node_pool_process_health_unhealthy() {
        let config = NodePoolConfig::default();
        let mut pool = NodePool::new(config);
        
        pool.add_node(ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless));
        
        let result = NodeHealthResult::unhealthy("node1".to_string(), "tcp".to_string());
        pool.process_health_result(&result);
        
        let updated = pool.get_node("node1").expect("found");
        assert_eq!(updated.status, NodeStatus::Unhealthy);
        assert_eq!(updated.consecutive_failures, 1);
    }

    #[test]
    fn node_pool_process_health_removed_after_threshold() {
        let config = NodePoolConfig {
            failure_threshold: 2,
            ..NodePoolConfig::default()
        };
        let mut pool = NodePool::new(config);
        
        let mut node = ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless);
        node.consecutive_failures = 1;
        pool.add_node(node);
        
        let result = NodeHealthResult::unhealthy("node1".to_string(), "tcp".to_string());
        pool.process_health_result(&result);
        
        let updated = pool.get_node("node1").expect("found");
        assert_eq!(updated.status, NodeStatus::Removed);
    }

    #[test]
    fn node_pool_check_and_update() {
        let config = NodePoolConfig::default();
        let mut pool = NodePool::new(config);
        
        pool.add_node(ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless));
        pool.add_node(ProxyNode::new("node2".to_string(), "127.0.0.1:1081".to_string(), ProxyProtocol::Vmess));
        
        let checker = MockNodeHealthChecker::new();
        let results = pool.check_and_update(&checker);
        
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.reachable));
        assert_eq!(pool.available_count(), 2);
    }

#[test]
    #[allow(clippy::duration_subsec)]
    fn node_pool_config_default_values() {
        let config = NodePoolConfig::default();
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.check_interval, Duration::from_mins(1));
    }

    #[test]
    fn node_pool_available_count() {
        let config = NodePoolConfig::default();
        let mut pool = NodePool::new(config);
        
        let mut node1 = ProxyNode::new("node1".to_string(), "127.0.0.1:1080".to_string(), ProxyProtocol::Vless);
        node1.mark_unhealthy();
        
        pool.add_node(node1);
        pool.add_node(ProxyNode::new("node2".to_string(), "127.0.0.1:1081".to_string(), ProxyProtocol::Vmess));
        
        assert_eq!(pool.available_count(), 1);
    }
}
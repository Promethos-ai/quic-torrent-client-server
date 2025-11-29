//! # Work Distribution System
//!
//! Implements weighted node tables for work delegation.
//! Allows nodes to hand work to each other based on weighted IP address tables.

use crate::messages::{AiRequest, AiResponse};
use crate::quic_client::QuicClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use rand::Rng;

/// Node capability type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeCapability {
    /// AI processing capability
    AiProcessing,
    /// File serving capability
    FileServing,
    /// Tracker capability
    Tracker,
    /// Custom capability
    Custom(String),
}

/// Node information for work distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub ip: String,
    pub port: u16,
    pub capabilities: Vec<NodeCapability>,
    pub weight: f64,  // Weight for load balancing (higher = more work)
    pub last_seen: SystemTime,
    pub active_requests: usize,
    pub max_concurrent: usize,
}

impl NodeInfo {
    pub fn new(ip: String, port: u16, capabilities: Vec<NodeCapability>, weight: f64) -> Self {
        Self {
            ip,
            port,
            capabilities,
            weight,
            last_seen: SystemTime::now(),
            active_requests: 0,
            max_concurrent: 100,
        }
    }

    /// Check if node can handle a capability
    pub fn can_handle(&self, capability: &NodeCapability) -> bool {
        self.capabilities.contains(capability) && self.active_requests < self.max_concurrent
    }

    /// Calculate effective weight (weight adjusted by load)
    pub fn effective_weight(&self) -> f64 {
        let load_factor = if self.max_concurrent > 0 {
            1.0 - (self.active_requests as f64 / self.max_concurrent as f64)
        } else {
            1.0
        };
        self.weight * load_factor
    }
}

/// Work distribution manager with weighted node tables
pub struct WorkDistributionManager {
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,  // node_id -> NodeInfo
    capability_tables: Arc<RwLock<HashMap<NodeCapability, Vec<String>>>>,  // capability -> node_ids
}

impl WorkDistributionManager {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            capability_tables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a node with its capabilities and weight
    pub fn register_node(&self, node_id: String, node_info: NodeInfo) {
        let mut nodes = self.nodes.write().unwrap();
        let mut tables = self.capability_tables.write().unwrap();
        
        // Remove old entries
        if let Some(old_info) = nodes.get(&node_id) {
            for cap in &old_info.capabilities {
                if let Some(node_list) = tables.get_mut(cap) {
                    node_list.retain(|id| id != &node_id);
                }
            }
        }
        
        // Add new entries
        for cap in &node_info.capabilities {
            tables.entry(cap.clone()).or_insert_with(Vec::new).push(node_id.clone());
        }
        
        nodes.insert(node_id, node_info);
        crate::log_server!("[WORK_DIST] Registered node with capabilities");
    }

    /// Select best node for a capability using weighted selection
    pub fn select_node(&self, capability: &NodeCapability) -> Option<(String, NodeInfo)> {
        let tables = self.capability_tables.read().unwrap();
        let nodes = self.nodes.read().unwrap();
        
        let node_ids = tables.get(capability)?;
        
        // Filter to available nodes and calculate weights
        let mut candidates: Vec<(String, f64)> = Vec::new();
        for node_id in node_ids {
            if let Some(node) = nodes.get(node_id) {
                if node.can_handle(capability) {
                    candidates.push((node_id.clone(), node.effective_weight()));
                }
            }
        }
        
        if candidates.is_empty() {
            return None;
        }
        
        // Weighted random selection
        let total_weight: f64 = candidates.iter().map(|(_, w)| w).sum();
        if total_weight <= 0.0 {
            return None;
        }
        
        let mut rng = rand::thread_rng();
        let mut random = rng.gen_range(0.0..total_weight);
        
        for (node_id, weight) in &candidates {
            random -= weight;
            if random <= 0.0 {
                if let Some(node) = nodes.get(node_id).cloned() {
                    return Some((node_id.clone(), node));
                }
            }
        }
        
        // Fallback to first candidate
        if let Some((node_id, _)) = candidates.first() {
            if let Some(node) = nodes.get(node_id).cloned() {
                return Some((node_id.clone(), node));
            }
        }
        
        None
    }

    /// Delegate AI work to another node
    pub async fn delegate_ai_work(
        &self,
        request: &AiRequest,
        capability: &NodeCapability,
    ) -> Result<AiResponse, String> {
        crate::log_server!("[WORK_DIST] Delegating AI work - capability: {:?}", capability);
        
        let (node_id, node_info) = match self.select_node(capability) {
            Some(n) => n,
            None => {
                return Err("No available node for AI processing".to_string());
            }
        };
        
        crate::log_server!("[WORK_DIST] Selected node: {}:{} (weight={}, load={}/{})",
            node_info.ip, node_info.port, node_info.weight,
            node_info.active_requests, node_info.max_concurrent);
        
        // Increment active requests
        {
            let mut nodes = self.nodes.write().unwrap();
            if let Some(node) = nodes.get_mut(&node_id) {
                node.active_requests += 1;
            }
        }
        
        // Send request to selected node
        crate::log_server!("[WORK_DIST] ===== WORK DELEGATION START =====");
        crate::log_server!("[WORK_DIST] Function: work_distribution::delegate_ai_work()");
        crate::log_server!("[WORK_DIST] Target node: {}:{}", node_info.ip, node_info.port);
        crate::log_server!("[WORK_DIST] Capability: {:?}", capability);
        crate::log_server!("[WORK_DIST] Request type: AiRequest");
        crate::log_server!("[WORK_DIST] Sending work request to remote node...");
        
        let client = match QuicClient::new() {
            Ok(c) => {
                crate::log_server!("[WORK_DIST] QUIC client created");
                c
            }
            Err(e) => {
                // Decrement on error
                let mut nodes = self.nodes.write().unwrap();
                if let Some(node) = nodes.get_mut(&node_id) {
                    node.active_requests = node.active_requests.saturating_sub(1);
                }
                return Err(format!("Failed to create QUIC client: {}", e));
            }
        };
        
        crate::log_server!("[WORK_DIST] Data sent to: Remote node {}:{}", node_info.ip, node_info.port);
        crate::log_server!("[WORK_DIST] Remote node will process via: quic_tracker::handle_ai_request() -> ai_processor::process_query_sync()");
        
        let result = client.send_message::<_, AiResponse>(&node_info.ip, node_info.port, request).await;
        
        // Decrement active requests
        {
            let mut nodes = self.nodes.write().unwrap();
            if let Some(node) = nodes.get_mut(&node_id) {
                node.active_requests = node.active_requests.saturating_sub(1);
                node.last_seen = SystemTime::now();
            }
        }
        
        match result {
            Ok(response) => {
                crate::log_server!("[WORK_DIST] Data received from: Remote node {}:{}", 
                    node_info.ip, node_info.port);
                crate::log_server!("[WORK_DIST] Response type: AiResponse");
                crate::log_server!("[WORK_DIST] Response size: answer_len={}", response.answer.len());
                crate::log_server!("[WORK_DIST] ===== WORK DELEGATION COMPLETE =====");
                crate::log_server!("[WORK_DIST] Work delegation successful - node: {}:{}", 
                    node_info.ip, node_info.port);
                Ok(response)
            }
            Err(e) => {
                crate::log_server!("[WORK_DIST] ===== WORK DELEGATION FAILED =====");
                crate::log_server!("[WORK_DIST] Work delegation failed - node: {}:{}, error: {}",
                    node_info.ip, node_info.port, e);
                Err(format!("{}", e))
            }
        }
    }

    /// Get all nodes for a capability
    pub fn get_nodes_for_capability(&self, capability: &NodeCapability) -> Vec<(String, NodeInfo)> {
        let tables = self.capability_tables.read().unwrap();
        let nodes = self.nodes.read().unwrap();
        
        let node_ids = match tables.get(capability) {
            Some(ids) => ids,
            None => return Vec::new(),
        };
        
        node_ids.iter()
            .filter_map(|id| {
                nodes.get(id).map(|node| (id.clone(), node.clone()))
            })
            .collect()
    }

    /// Update node status
    pub fn update_node_status(&self, node_id: &str, active_requests: usize) {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(node) = nodes.get_mut(node_id) {
            node.active_requests = active_requests;
            node.last_seen = SystemTime::now();
        }
    }
}

impl Default for WorkDistributionManager {
    fn default() -> Self {
        Self::new()
    }
}


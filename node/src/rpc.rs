//! JSON-RPC 2.0 API layer for Prova node.
//!
//! Provides a lightweight, synchronous JSON-RPC handler that routes requests
//! to the chain state. Designed to be embedded in an HTTP server or used
//! directly for testing.
//!
//! ## Supported Methods
//!
//! - `prova_getEpoch` — current epoch number
//! - `prova_getBlock` — block at given height
//! - `prova_getModel` — model registry lookup by ID hex
//! - `prova_getCommit` — inference commit by ID
//! - `prova_getStake` — stake info for an address
//! - `prova_getDisputeStatus` — dispute game status by commit ID
//! - `prova_submitCommit` — submit a new inference commit
//! - `prova_getPaymentChannel` — payment channel state
//! - `prova_nodeInfo` — node version and status

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl RpcResponse {
    pub fn ok(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn err(id: serde_json::Value, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }
}

// Standard JSON-RPC error codes
pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;

// ---------------------------------------------------------------------------
// Node state snapshot (simplified view of chain state for RPC)
// ---------------------------------------------------------------------------

/// Minimal node state exposed via RPC.
/// In production this wraps a reference to the full chain state;
/// here we use a self-contained snapshot for testability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub epoch: u64,
    pub node_version: String,
    pub models: HashMap<String, ModelInfo>,
    pub commits: HashMap<u64, CommitInfo>,
    pub stakes: HashMap<String, StakeInfo>,
    pub channels: HashMap<String, ChannelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id_hex: String,
    pub name: String,
    pub arch_group: String,
    pub layer_count: u32,
    pub registered_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub id: u64,
    pub provider: String,
    pub model_id_hex: String,
    pub activation_root_hex: String,
    pub epoch: u64,
    pub status: CommitStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommitStatus {
    Pending,
    Challenged,
    Finalized,
    Slashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub address: String,
    pub total: u128,
    pub locked: u128,
    pub available: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub id: String,
    pub payer: String,
    pub payee: String,
    pub balance: u128,
    pub rate_per_epoch: u128,
}

impl NodeState {
    /// Create a minimal default state for testing.
    pub fn new_test() -> Self {
        Self {
            epoch: 0,
            node_version: "prova-node/0.1.0".into(),
            models: HashMap::new(),
            commits: HashMap::new(),
            stakes: HashMap::new(),
            channels: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// RPC dispatcher
// ---------------------------------------------------------------------------

/// Handle a single JSON-RPC request against the given state.
pub fn handle_request(state: &NodeState, req: &RpcRequest) -> RpcResponse {
    if req.jsonrpc != "2.0" {
        return RpcResponse::err(req.id.clone(), INVALID_REQUEST, "Expected jsonrpc 2.0");
    }

    match req.method.as_str() {
        "prova_nodeInfo" => handle_node_info(state, req),
        "prova_getEpoch" => handle_get_epoch(state, req),
        "prova_getModel" => handle_get_model(state, req),
        "prova_getCommit" => handle_get_commit(state, req),
        "prova_getStake" => handle_get_stake(state, req),
        "prova_submitCommit" => handle_submit_commit(state, req),
        "prova_getPaymentChannel" => handle_get_channel(state, req),
        _ => RpcResponse::err(
            req.id.clone(),
            METHOD_NOT_FOUND,
            format!("Unknown method: {}", req.method),
        ),
    }
}

/// Parse a raw JSON string into a request, dispatch, and return JSON response.
pub fn handle_raw(state: &NodeState, raw: &str) -> String {
    let req: RpcRequest = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(e) => {
            let resp = RpcResponse::err(
                serde_json::Value::Null,
                PARSE_ERROR,
                format!("Parse error: {e}"),
            );
            return serde_json::to_string(&resp).unwrap();
        }
    };
    let resp = handle_request(state, &req);
    serde_json::to_string(&resp).unwrap()
}

// ---------------------------------------------------------------------------
// Method handlers
// ---------------------------------------------------------------------------

fn handle_node_info(state: &NodeState, req: &RpcRequest) -> RpcResponse {
    RpcResponse::ok(
        req.id.clone(),
        serde_json::json!({
            "version": state.node_version,
            "epoch": state.epoch,
            "models": state.models.len(),
            "commits": state.commits.len(),
        }),
    )
}

fn handle_get_epoch(state: &NodeState, req: &RpcRequest) -> RpcResponse {
    RpcResponse::ok(req.id.clone(), serde_json::json!(state.epoch))
}

fn handle_get_model(state: &NodeState, req: &RpcRequest) -> RpcResponse {
    let id_hex = match param_str(&req.params, 0) {
        Some(s) => s,
        None => {
            return RpcResponse::err(
                req.id.clone(),
                INVALID_PARAMS,
                "Expected model ID hex as first param",
            )
        }
    };
    match state.models.get(&id_hex) {
        Some(m) => RpcResponse::ok(req.id.clone(), serde_json::to_value(m).unwrap()),
        None => RpcResponse::err(req.id.clone(), -32000, format!("Model not found: {id_hex}")),
    }
}

fn handle_get_commit(state: &NodeState, req: &RpcRequest) -> RpcResponse {
    let id = match param_u64(&req.params, 0) {
        Some(v) => v,
        None => {
            return RpcResponse::err(
                req.id.clone(),
                INVALID_PARAMS,
                "Expected commit ID as first param",
            )
        }
    };
    match state.commits.get(&id) {
        Some(c) => RpcResponse::ok(req.id.clone(), serde_json::to_value(c).unwrap()),
        None => RpcResponse::err(req.id.clone(), -32000, format!("Commit not found: {id}")),
    }
}

fn handle_get_stake(state: &NodeState, req: &RpcRequest) -> RpcResponse {
    let addr = match param_str(&req.params, 0) {
        Some(s) => s,
        None => {
            return RpcResponse::err(
                req.id.clone(),
                INVALID_PARAMS,
                "Expected address as first param",
            )
        }
    };
    match state.stakes.get(&addr) {
        Some(s) => RpcResponse::ok(req.id.clone(), serde_json::to_value(s).unwrap()),
        None => RpcResponse::err(
            req.id.clone(),
            -32000,
            format!("No stake found for: {addr}"),
        ),
    }
}

fn handle_submit_commit(state: &NodeState, req: &RpcRequest) -> RpcResponse {
    // In a real node, this would mutate state. Here we validate params and return a mock receipt.
    let model_id = match param_str(&req.params, 0) {
        Some(s) => s,
        None => {
            return RpcResponse::err(
                req.id.clone(),
                INVALID_PARAMS,
                "Expected model_id_hex as param[0]",
            )
        }
    };
    let activation_root = match param_str(&req.params, 1) {
        Some(s) => s,
        None => {
            return RpcResponse::err(
                req.id.clone(),
                INVALID_PARAMS,
                "Expected activation_root_hex as param[1]",
            )
        }
    };

    // Validate model exists
    if !state.models.contains_key(&model_id) {
        return RpcResponse::err(
            req.id.clone(),
            -32001,
            format!("Model not registered: {model_id}"),
        );
    }

    let commit_id = state.commits.len() as u64 + 1;
    RpcResponse::ok(
        req.id.clone(),
        serde_json::json!({
            "commit_id": commit_id,
            "model_id": model_id,
            "activation_root": activation_root,
            "epoch": state.epoch,
            "status": "Pending",
        }),
    )
}

fn handle_get_channel(state: &NodeState, req: &RpcRequest) -> RpcResponse {
    let id = match param_str(&req.params, 0) {
        Some(s) => s,
        None => {
            return RpcResponse::err(
                req.id.clone(),
                INVALID_PARAMS,
                "Expected channel ID as first param",
            )
        }
    };
    match state.channels.get(&id) {
        Some(ch) => RpcResponse::ok(req.id.clone(), serde_json::to_value(ch).unwrap()),
        None => RpcResponse::err(req.id.clone(), -32000, format!("Channel not found: {id}")),
    }
}

// ---------------------------------------------------------------------------
// Param helpers
// ---------------------------------------------------------------------------

fn param_str(params: &serde_json::Value, idx: usize) -> Option<String> {
    match params {
        serde_json::Value::Array(arr) => arr.get(idx)?.as_str().map(|s| s.to_string()),
        serde_json::Value::Object(obj) if idx == 0 => {
            // Try common first-param names
            for key in &["id", "address", "model_id", "channel_id"] {
                if let Some(v) = obj.get(*key) {
                    return v.as_str().map(|s| s.to_string());
                }
            }
            None
        }
        _ => None,
    }
}

fn param_u64(params: &serde_json::Value, idx: usize) -> Option<u64> {
    match params {
        serde_json::Value::Array(arr) => arr.get(idx)?.as_u64(),
        serde_json::Value::Object(obj) if idx == 0 => {
            for key in &["id", "commit_id"] {
                if let Some(v) = obj.get(*key) {
                    return v.as_u64();
                }
            }
            None
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> NodeState {
        let mut state = NodeState::new_test();
        state.epoch = 42;

        state.models.insert(
            "abc123".into(),
            ModelInfo {
                id_hex: "abc123".into(),
                name: "llama-7b".into(),
                arch_group: "nvidia-sm89-int8".into(),
                layer_count: 32,
                registered_epoch: 10,
            },
        );

        state.commits.insert(
            1,
            CommitInfo {
                id: 1,
                provider: "0xprovider1".into(),
                model_id_hex: "abc123".into(),
                activation_root_hex: "deadbeef".into(),
                epoch: 40,
                status: CommitStatus::Pending,
            },
        );

        state.stakes.insert(
            "0xprovider1".into(),
            StakeInfo {
                address: "0xprovider1".into(),
                total: 1_000_000,
                locked: 200_000,
                available: 800_000,
            },
        );

        state.channels.insert(
            "ch-1".into(),
            ChannelInfo {
                id: "ch-1".into(),
                payer: "0xclient".into(),
                payee: "0xprovider1".into(),
                balance: 500_000,
                rate_per_epoch: 100,
            },
        );

        state
    }

    fn call(state: &NodeState, method: &str, params: serde_json::Value) -> RpcResponse {
        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params,
            id: serde_json::json!(1),
        };
        handle_request(state, &req)
    }

    #[test]
    fn test_node_info() {
        let s = test_state();
        let resp = call(&s, "prova_nodeInfo", serde_json::json!([]));
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["epoch"], 42);
        assert_eq!(r["version"], "prova-node/0.1.0");
        assert_eq!(r["models"], 1);
    }

    #[test]
    fn test_get_epoch() {
        let s = test_state();
        let resp = call(&s, "prova_getEpoch", serde_json::json!([]));
        assert_eq!(resp.result.unwrap(), 42);
    }

    #[test]
    fn test_get_model_found() {
        let s = test_state();
        let resp = call(&s, "prova_getModel", serde_json::json!(["abc123"]));
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["name"], "llama-7b");
        assert_eq!(r["layer_count"], 32);
    }

    #[test]
    fn test_get_model_not_found() {
        let s = test_state();
        let resp = call(&s, "prova_getModel", serde_json::json!(["nonexistent"]));
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32000);
    }

    #[test]
    fn test_get_commit() {
        let s = test_state();
        let resp = call(&s, "prova_getCommit", serde_json::json!([1]));
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["provider"], "0xprovider1");
        assert_eq!(r["status"], "Pending");
    }

    #[test]
    fn test_get_commit_not_found() {
        let s = test_state();
        let resp = call(&s, "prova_getCommit", serde_json::json!([999]));
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_get_stake() {
        let s = test_state();
        let resp = call(&s, "prova_getStake", serde_json::json!(["0xprovider1"]));
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["total"], 1_000_000);
        assert_eq!(r["available"], 800_000);
    }

    #[test]
    fn test_submit_commit_success() {
        let s = test_state();
        let resp = call(
            &s,
            "prova_submitCommit",
            serde_json::json!(["abc123", "cafebabe"]),
        );
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["status"], "Pending");
        assert_eq!(r["model_id"], "abc123");
    }

    #[test]
    fn test_submit_commit_unknown_model() {
        let s = test_state();
        let resp = call(
            &s,
            "prova_submitCommit",
            serde_json::json!(["unknown", "cafebabe"]),
        );
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32001);
    }

    #[test]
    fn test_get_payment_channel() {
        let s = test_state();
        let resp = call(&s, "prova_getPaymentChannel", serde_json::json!(["ch-1"]));
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["payer"], "0xclient");
        assert_eq!(r["rate_per_epoch"], 100);
    }

    #[test]
    fn test_method_not_found() {
        let s = test_state();
        let resp = call(&s, "prova_nonexistent", serde_json::json!([]));
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, METHOD_NOT_FOUND);
    }

    #[test]
    fn test_invalid_jsonrpc_version() {
        let s = test_state();
        let req = RpcRequest {
            jsonrpc: "1.0".into(),
            method: "prova_getEpoch".into(),
            params: serde_json::json!([]),
            id: serde_json::json!(1),
        };
        let resp = handle_request(&s, &req);
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, INVALID_REQUEST);
    }

    #[test]
    fn test_handle_raw_valid() {
        let s = test_state();
        let raw = r#"{"jsonrpc":"2.0","method":"prova_getEpoch","params":[],"id":1}"#;
        let out = handle_raw(&s, raw);
        let resp: RpcResponse = serde_json::from_str(&out).unwrap();
        assert_eq!(resp.result.unwrap(), 42);
    }

    #[test]
    fn test_handle_raw_parse_error() {
        let s = test_state();
        let out = handle_raw(&s, "not json");
        let resp: RpcResponse = serde_json::from_str(&out).unwrap();
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, PARSE_ERROR);
    }

    #[test]
    fn test_missing_params() {
        let s = test_state();
        let resp = call(&s, "prova_getModel", serde_json::json!([]));
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, INVALID_PARAMS);
    }

    #[test]
    fn test_named_params() {
        let s = test_state();
        let resp = call(&s, "prova_getModel", serde_json::json!({"id": "abc123"}));
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["name"], "llama-7b");
    }
}

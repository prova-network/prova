//! JSON-RPC 2.0 client for connecting to a Prova node.
//!
//! Provides `RpcClient` which wraps a node endpoint and exposes typed
//! methods for all `prova_*` RPC calls. Uses synchronous in-process
//! dispatch for testing, with a pluggable transport trait for real networking.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

// ── Transport trait ──────────────────────────────────────────

/// Transport sends a raw JSON-RPC request string and returns the raw response.
/// Implementors provide HTTP, TCP, IPC, or in-process backends.
pub trait Transport {
    fn send_raw(&self, request: &str) -> Result<String, RpcClientError>;
}

/// In-process transport that dispatches directly to a node RPC handler.
/// Used for testing without network overhead.
pub struct InProcessTransport<F: Fn(&str) -> String> {
    handler: F,
}

impl<F: Fn(&str) -> String> InProcessTransport<F> {
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F: Fn(&str) -> String> Transport for InProcessTransport<F> {
    fn send_raw(&self, request: &str) -> Result<String, RpcClientError> {
        Ok((self.handler)(request))
    }
}

/// Simulated HTTP transport (URL-based, returns canned responses for testing).
pub struct HttpTransport {
    pub url: String,
}

impl HttpTransport {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

impl Transport for HttpTransport {
    fn send_raw(&self, _request: &str) -> Result<String, RpcClientError> {
        // In production: HTTP POST to self.url with request body
        // For now: return connection error (real HTTP requires async runtime)
        Err(RpcClientError::Transport(format!(
            "HTTP transport to {} not implemented in simulation mode",
            self.url
        )))
    }
}

// ── JSON-RPC types (client-side) ─────────────────────────────

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    #[allow(dead_code)]
    id: Value,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RPC error {}: {}", self.code, self.message)
    }
}

// ── Typed response structs ───────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct NodeInfo {
    pub version: String,
    pub epoch: u64,
    pub models: u64,
    pub commits: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id_hex: String,
    pub name: String,
    pub arch_group: String,
    pub layer_count: u32,
    pub registered_epoch: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitInfo {
    pub id: u64,
    pub provider: String,
    pub model_id_hex: String,
    pub activation_root_hex: String,
    pub epoch: u64,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StakeInfo {
    pub address: String,
    pub total: u128,
    pub locked: u128,
    pub available: u128,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelInfo {
    pub id: String,
    pub payer: String,
    pub payee: String,
    pub balance: u128,
    pub rate_per_epoch: u128,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitReceipt {
    pub commit_id: u64,
    pub model_id: String,
    pub activation_root: String,
    pub epoch: u64,
    pub status: String,
}

// ── RPC Client ───────────────────────────────────────────────

/// JSON-RPC client for a Prova node.
pub struct RpcClient<T: Transport> {
    transport: T,
    next_id: AtomicU64,
}

impl<T: Transport> RpcClient<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            next_id: AtomicU64::new(1),
        }
    }

    /// Send a raw RPC call and return the parsed result.
    fn call(&self, method: &str, params: Value) -> Result<Value, RpcClientError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id,
        };
        let raw_req = serde_json::to_string(&req)
            .map_err(|e| RpcClientError::Serialization(e.to_string()))?;

        let raw_resp = self.transport.send_raw(&raw_req)?;

        let resp: JsonRpcResponse = serde_json::from_str(&raw_resp)
            .map_err(|e| RpcClientError::Serialization(format!("Bad response: {e}")))?;

        if let Some(err) = resp.error {
            return Err(RpcClientError::Rpc(err));
        }

        resp.result.ok_or(RpcClientError::Serialization(
            "Response has neither result nor error".into(),
        ))
    }

    /// Get node info (version, epoch, counts).
    pub fn node_info(&self) -> Result<NodeInfo, RpcClientError> {
        let val = self.call("prova_nodeInfo", Value::Array(vec![]))?;
        serde_json::from_value(val).map_err(|e| RpcClientError::Serialization(e.to_string()))
    }

    /// Get current epoch.
    pub fn get_epoch(&self) -> Result<u64, RpcClientError> {
        let val = self.call("prova_getEpoch", Value::Array(vec![]))?;
        val.as_u64()
            .ok_or(RpcClientError::Serialization("Expected u64 epoch".into()))
    }

    /// Look up a model by ID hex.
    pub fn get_model(&self, id_hex: &str) -> Result<ModelInfo, RpcClientError> {
        let val = self.call("prova_getModel", serde_json::json!([id_hex]))?;
        serde_json::from_value(val).map_err(|e| RpcClientError::Serialization(e.to_string()))
    }

    /// Look up a commit by ID.
    pub fn get_commit(&self, commit_id: u64) -> Result<CommitInfo, RpcClientError> {
        let val = self.call("prova_getCommit", serde_json::json!([commit_id]))?;
        serde_json::from_value(val).map_err(|e| RpcClientError::Serialization(e.to_string()))
    }

    /// Get stake info for an address.
    pub fn get_stake(&self, address: &str) -> Result<StakeInfo, RpcClientError> {
        let val = self.call("prova_getStake", serde_json::json!([address]))?;
        serde_json::from_value(val).map_err(|e| RpcClientError::Serialization(e.to_string()))
    }

    /// Submit an inference commit. Returns a receipt.
    pub fn submit_commit(
        &self,
        model_id_hex: &str,
        activation_root_hex: &str,
    ) -> Result<SubmitReceipt, RpcClientError> {
        let val = self.call(
            "prova_submitCommit",
            serde_json::json!([model_id_hex, activation_root_hex]),
        )?;
        serde_json::from_value(val).map_err(|e| RpcClientError::Serialization(e.to_string()))
    }

    /// Get payment channel info.
    pub fn get_payment_channel(&self, channel_id: &str) -> Result<ChannelInfo, RpcClientError> {
        let val = self.call("prova_getPaymentChannel", serde_json::json!([channel_id]))?;
        serde_json::from_value(val).map_err(|e| RpcClientError::Serialization(e.to_string()))
    }

    /// Poll a job until it reaches a terminal status or max attempts.
    /// Returns the final commit info, or an error if polling exhausted.
    pub fn poll_commit(
        &self,
        commit_id: u64,
        max_attempts: u32,
    ) -> Result<CommitInfo, RpcClientError> {
        for _ in 0..max_attempts {
            let info = self.get_commit(commit_id)?;
            match info.status.as_str() {
                "Finalized" | "Slashed" => return Ok(info),
                _ => continue,
            }
        }
        Err(RpcClientError::PollExhausted {
            commit_id,
            attempts: max_attempts,
        })
    }

    /// Batch call: fetch multiple commits by ID.
    pub fn get_commits(&self, ids: &[u64]) -> Vec<Result<CommitInfo, RpcClientError>> {
        ids.iter().map(|id| self.get_commit(*id)).collect()
    }
}

// ── Errors ───────────────────────────────────────────────────

#[derive(Debug)]
pub enum RpcClientError {
    Transport(String),
    Serialization(String),
    Rpc(JsonRpcError),
    PollExhausted { commit_id: u64, attempts: u32 },
}

impl std::fmt::Display for RpcClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(msg) => write!(f, "transport error: {msg}"),
            Self::Serialization(msg) => write!(f, "serialization error: {msg}"),
            Self::Rpc(e) => write!(f, "{e}"),
            Self::PollExhausted {
                commit_id,
                attempts,
            } => {
                write!(
                    f,
                    "polling commit {commit_id} exhausted after {attempts} attempts"
                )
            }
        }
    }
}

impl std::error::Error for RpcClientError {}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal mock JSON-RPC handler that simulates a Prova node.
    fn mock_handler(raw: &str) -> String {
        let req: serde_json::Value = serde_json::from_str(raw).unwrap();
        let method = req["method"].as_str().unwrap_or("");
        let params = &req["params"];
        let id = req["id"].clone();

        let (result, error) = match method {
            "prova_nodeInfo" => (
                Some(serde_json::json!({
                    "version": "prova-node/0.1.0",
                    "epoch": 100,
                    "models": 1,
                    "commits": 2,
                })),
                None,
            ),
            "prova_getEpoch" => (Some(serde_json::json!(100)), None),
            "prova_getModel" => {
                let id_hex = params[0].as_str().unwrap_or("");
                if id_hex == "model1" {
                    (
                        Some(serde_json::json!({
                            "id_hex": "model1", "name": "llama-7b",
                            "arch_group": "nvidia-sm89-int8", "layer_count": 32,
                            "registered_epoch": 10
                        })),
                        None,
                    )
                } else {
                    (
                        None,
                        Some(
                            serde_json::json!({"code": -32000, "message": format!("Model not found: {id_hex}")}),
                        ),
                    )
                }
            }
            "prova_getCommit" => {
                let cid = params[0].as_u64().unwrap_or(0);
                match cid {
                    1 => (
                        Some(serde_json::json!({
                            "id": 1, "provider": "0xprov", "model_id_hex": "model1",
                            "activation_root_hex": "aabbccdd", "epoch": 98, "status": "Pending"
                        })),
                        None,
                    ),
                    2 => (
                        Some(serde_json::json!({
                            "id": 2, "provider": "0xprov", "model_id_hex": "model1",
                            "activation_root_hex": "11223344", "epoch": 99, "status": "Finalized"
                        })),
                        None,
                    ),
                    _ => (
                        None,
                        Some(
                            serde_json::json!({"code": -32000, "message": format!("Commit not found: {cid}")}),
                        ),
                    ),
                }
            }
            "prova_getStake" => {
                let addr = params[0].as_str().unwrap_or("");
                if addr == "0xprov" {
                    (
                        Some(serde_json::json!({
                            "address": "0xprov", "total": 5_000_000u64,
                            "locked": 1_000_000u64, "available": 4_000_000u64
                        })),
                        None,
                    )
                } else {
                    (
                        None,
                        Some(
                            serde_json::json!({"code": -32000, "message": format!("No stake: {addr}")}),
                        ),
                    )
                }
            }
            "prova_submitCommit" => {
                let model_id = params[0].as_str().unwrap_or("");
                let root = params[1].as_str().unwrap_or("");
                if model_id == "model1" {
                    (
                        Some(serde_json::json!({
                            "commit_id": 3, "model_id": model_id,
                            "activation_root": root, "epoch": 100, "status": "Pending"
                        })),
                        None,
                    )
                } else {
                    (
                        None,
                        Some(
                            serde_json::json!({"code": -32001, "message": format!("Model not registered: {model_id}")}),
                        ),
                    )
                }
            }
            "prova_getPaymentChannel" => {
                let ch_id = params[0].as_str().unwrap_or("");
                if ch_id == "pay-1" {
                    (
                        Some(serde_json::json!({
                            "id": "pay-1", "payer": "0xclient", "payee": "0xprov",
                            "balance": 250_000u64, "rate_per_epoch": 50u64
                        })),
                        None,
                    )
                } else {
                    (
                        None,
                        Some(
                            serde_json::json!({"code": -32000, "message": format!("Channel not found: {ch_id}")}),
                        ),
                    )
                }
            }
            _ => (
                None,
                Some(
                    serde_json::json!({"code": -32601, "message": format!("Unknown method: {method}")}),
                ),
            ),
        };

        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "error": error,
            "id": id,
        });
        serde_json::to_string(&resp).unwrap()
    }

    fn test_client() -> RpcClient<InProcessTransport<impl Fn(&str) -> String>> {
        RpcClient::new(InProcessTransport::new(mock_handler))
    }

    #[test]
    fn client_node_info() {
        let c = test_client();
        let info = c.node_info().unwrap();
        assert_eq!(info.epoch, 100);
        assert_eq!(info.version, "prova-node/0.1.0");
        assert_eq!(info.models, 1);
    }

    #[test]
    fn client_get_epoch() {
        let c = test_client();
        assert_eq!(c.get_epoch().unwrap(), 100);
    }

    #[test]
    fn client_get_model() {
        let c = test_client();
        let m = c.get_model("model1").unwrap();
        assert_eq!(m.name, "llama-7b");
        assert_eq!(m.layer_count, 32);
    }

    #[test]
    fn client_get_model_not_found() {
        let c = test_client();
        let err = c.get_model("nonexistent").unwrap_err();
        assert!(matches!(err, RpcClientError::Rpc(_)));
    }

    #[test]
    fn client_get_commit() {
        let c = test_client();
        let ci = c.get_commit(1).unwrap();
        assert_eq!(ci.provider, "0xprov");
        assert_eq!(ci.status, "Pending");
    }

    #[test]
    fn client_get_commit_not_found() {
        let c = test_client();
        assert!(c.get_commit(999).is_err());
    }

    #[test]
    fn client_get_stake() {
        let c = test_client();
        let s = c.get_stake("0xprov").unwrap();
        assert_eq!(s.total, 5_000_000);
        assert_eq!(s.available, 4_000_000);
    }

    #[test]
    fn client_submit_commit() {
        let c = test_client();
        let receipt = c.submit_commit("model1", "deadbeef").unwrap();
        assert_eq!(receipt.status, "Pending");
        assert_eq!(receipt.model_id, "model1");
    }

    #[test]
    fn client_submit_commit_unknown_model() {
        let c = test_client();
        let err = c.submit_commit("bad_model", "deadbeef").unwrap_err();
        assert!(matches!(err, RpcClientError::Rpc(_)));
    }

    #[test]
    fn client_get_payment_channel() {
        let c = test_client();
        let ch = c.get_payment_channel("pay-1").unwrap();
        assert_eq!(ch.payer, "0xclient");
        assert_eq!(ch.rate_per_epoch, 50);
    }

    #[test]
    fn client_poll_commit_finalized() {
        let c = test_client();
        // Commit 2 is already Finalized
        let ci = c.poll_commit(2, 3).unwrap();
        assert_eq!(ci.status, "Finalized");
    }

    #[test]
    fn client_poll_commit_exhausted() {
        let c = test_client();
        // Commit 1 is Pending, will never finalize in this static state
        let err = c.poll_commit(1, 3).unwrap_err();
        assert!(matches!(
            err,
            RpcClientError::PollExhausted {
                commit_id: 1,
                attempts: 3
            }
        ));
    }

    #[test]
    fn client_batch_get_commits() {
        let c = test_client();
        let results = c.get_commits(&[1, 2, 999]);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
    }

    #[test]
    fn http_transport_errors() {
        let t = HttpTransport::new("http://localhost:9999");
        let c = RpcClient::new(t);
        let err = c.get_epoch().unwrap_err();
        assert!(matches!(err, RpcClientError::Transport(_)));
    }

    #[test]
    fn client_increments_request_ids() {
        let c = test_client();
        c.get_epoch().unwrap();
        c.get_epoch().unwrap();
        // next_id should be 3 now (started at 1, called twice)
        assert_eq!(c.next_id.load(Ordering::Relaxed), 3);
    }
}

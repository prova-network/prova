//! API Gateway client SDK — key management, inference submission, job lifecycle.
//!
//! Provides `GatewayClient` that wraps the Prova API Gateway HTTP endpoints
//! with typed methods for key management, inference submission, status polling,
//! cancellation, model listing, and webhook registration.

use sha2::{Digest, Sha256};
use std::collections::HashMap;

// ── Error types ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum GatewayError {
    Unauthorized,
    Forbidden(String),
    NotFound,
    RateLimited { retry_after_secs: u64 },
    BadRequest(String),
    ServerError(String),
    Transport(String),
    ParseError(String),
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Forbidden(m) => write!(f, "forbidden: {}", m),
            Self::NotFound => write!(f, "not found"),
            Self::RateLimited { retry_after_secs } => {
                write!(f, "rate limited, retry after {}s", retry_after_secs)
            }
            Self::BadRequest(m) => write!(f, "bad request: {}", m),
            Self::ServerError(m) => write!(f, "server error: {}", m),
            Self::Transport(m) => write!(f, "transport error: {}", m),
            Self::ParseError(m) => write!(f, "parse error: {}", m),
        }
    }
}

// ── Transport trait ──────────────────────────────────────────

/// HTTP-like transport for gateway requests.
pub trait GatewayTransport {
    fn request(
        &self,
        method: &str,
        path: &str,
        api_key: &str,
        body: Option<&str>,
    ) -> TransportResponse;
}

#[derive(Debug, Clone)]
pub struct TransportResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

/// In-process transport that dispatches to an ApiGateway directly (for testing).
pub struct InProcessGatewayTransport<F: Fn(&str, &str, &str, Option<&str>) -> TransportResponse> {
    handler: F,
}

impl<F: Fn(&str, &str, &str, Option<&str>) -> TransportResponse> InProcessGatewayTransport<F> {
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F: Fn(&str, &str, &str, Option<&str>) -> TransportResponse> GatewayTransport
    for InProcessGatewayTransport<F>
{
    fn request(
        &self,
        method: &str,
        path: &str,
        api_key: &str,
        body: Option<&str>,
    ) -> TransportResponse {
        (self.handler)(method, path, api_key, body)
    }
}

// ── Response types ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "queued" => Some(Self::Queued),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone)]
pub struct SubmitResult {
    pub job_id: String,
    pub status: JobStatus,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct JobResult {
    pub job_id: String,
    pub status: JobStatus,
    pub output: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelList {
    pub models: Vec<String>,
}

// ── Inference request builder ────────────────────────────────

#[derive(Debug, Clone)]
pub struct InferenceRequestBuilder {
    model_id: String,
    input: String,
    max_tokens: u64,
    callback_url: Option<String>,
}

impl InferenceRequestBuilder {
    pub fn new(model_id: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            input: input.into(),
            max_tokens: 256,
            callback_url: None,
        }
    }

    pub fn max_tokens(mut self, n: u64) -> Self {
        self.max_tokens = n;
        self
    }

    pub fn callback_url(mut self, url: impl Into<String>) -> Self {
        self.callback_url = Some(url.into());
        self
    }

    pub fn to_json(&self) -> String {
        let mut s = format!(
            r#"{{"model_id":"{}","input":"{}","max_tokens":"{}""#,
            self.model_id, self.input, self.max_tokens
        );
        if let Some(ref url) = self.callback_url {
            s.push_str(&format!(r#","callback_url":"{}""#, url));
        }
        s.push('}');
        s
    }
}

// ── API Key management ───────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ApiKeyConfig {
    pub key: String,
    pub label: Option<String>,
}

impl ApiKeyConfig {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Derive a deterministic key ID from the key value.
    pub fn key_id(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.key.as_bytes());
        let hash = hasher.finalize();
        format!("key-{}", hex_encode(&hash[..8]))
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ── Key ring (client-side key management) ────────────────────

#[derive(Debug, Clone)]
pub struct KeyRing {
    keys: Vec<ApiKeyConfig>,
    active_index: usize,
}

impl KeyRing {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            active_index: 0,
        }
    }

    pub fn add_key(&mut self, key: ApiKeyConfig) {
        self.keys.push(key);
    }

    pub fn set_active(&mut self, index: usize) -> bool {
        if index < self.keys.len() {
            self.active_index = index;
            true
        } else {
            false
        }
    }

    pub fn active_key(&self) -> Option<&ApiKeyConfig> {
        self.keys.get(self.active_index)
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn remove_key(&mut self, index: usize) -> Option<ApiKeyConfig> {
        if index < self.keys.len() {
            let removed = self.keys.remove(index);
            if self.active_index >= self.keys.len() && !self.keys.is_empty() {
                self.active_index = self.keys.len() - 1;
            }
            Some(removed)
        } else {
            None
        }
    }

    pub fn list_keys(&self) -> &[ApiKeyConfig] {
        &self.keys
    }
}

// ── Gateway Client ───────────────────────────────────────────

pub struct GatewayClient<T: GatewayTransport> {
    transport: T,
    key_ring: KeyRing,
    base_url: String,
}

impl<T: GatewayTransport> GatewayClient<T> {
    pub fn new(transport: T, base_url: impl Into<String>) -> Self {
        Self {
            transport,
            key_ring: KeyRing::new(),
            base_url: base_url.into(),
        }
    }

    pub fn add_key(&mut self, key: ApiKeyConfig) {
        self.key_ring.add_key(key);
    }

    pub fn set_active_key(&mut self, index: usize) -> bool {
        self.key_ring.set_active(index)
    }

    pub fn key_ring(&self) -> &KeyRing {
        &self.key_ring
    }

    pub fn key_ring_mut(&mut self) -> &mut KeyRing {
        &mut self.key_ring
    }

    fn active_api_key(&self) -> Result<&str, GatewayError> {
        self.key_ring
            .active_key()
            .map(|k| k.key.as_str())
            .ok_or(GatewayError::Unauthorized)
    }

    fn do_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<TransportResponse, GatewayError> {
        let key = self.active_api_key()?;
        Ok(self.transport.request(method, path, key, body))
    }

    fn parse_response(&self, resp: TransportResponse) -> Result<String, GatewayError> {
        match resp.status {
            200 | 201 => Ok(resp.body),
            400 => Err(GatewayError::BadRequest(extract_error(&resp.body))),
            401 => Err(GatewayError::Unauthorized),
            403 => Err(GatewayError::Forbidden(extract_error(&resp.body))),
            404 => Err(GatewayError::NotFound),
            429 => {
                let retry = resp
                    .headers
                    .get("Retry-After")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60);
                Err(GatewayError::RateLimited {
                    retry_after_secs: retry,
                })
            }
            s => Err(GatewayError::ServerError(format!(
                "status {}: {}",
                s, resp.body
            ))),
        }
    }

    // ── Inference operations ─────────────────────────────────

    pub fn submit_inference(
        &self,
        req: &InferenceRequestBuilder,
    ) -> Result<SubmitResult, GatewayError> {
        let resp = self.do_request("POST", "/v1/inference", Some(&req.to_json()))?;
        let body = self.parse_response(resp)?;
        let job_id = extract_field(&body, "job_id")
            .ok_or(GatewayError::ParseError("missing job_id".into()))?;
        let status_str = extract_field(&body, "status").unwrap_or_else(|| "queued".into());
        let model = extract_field(&body, "model").unwrap_or_default();
        Ok(SubmitResult {
            job_id,
            status: JobStatus::from_str(&status_str).unwrap_or(JobStatus::Queued),
            model,
        })
    }

    pub fn get_job(&self, job_id: &str) -> Result<JobResult, GatewayError> {
        let path = format!("/v1/inference/{}", job_id);
        let resp = self.do_request("GET", &path, None)?;
        let body = self.parse_response(resp)?;
        let id = extract_field(&body, "job_id").unwrap_or_else(|| job_id.into());
        let status_str = extract_field(&body, "status").unwrap_or_default();
        let output = extract_field(&body, "output").filter(|s| !s.is_empty());
        Ok(JobResult {
            job_id: id,
            status: JobStatus::from_str(&status_str).unwrap_or(JobStatus::Queued),
            output,
        })
    }

    pub fn cancel_job(&self, job_id: &str) -> Result<JobResult, GatewayError> {
        let path = format!("/v1/inference/{}", job_id);
        let resp = self.do_request("DELETE", &path, None)?;
        let body = self.parse_response(resp)?;
        Ok(JobResult {
            job_id: job_id.into(),
            status: JobStatus::Cancelled,
            output: None,
        })
    }

    /// Poll a job until it reaches a terminal state, up to max_polls attempts.
    pub fn poll_until_done(
        &self,
        job_id: &str,
        max_polls: usize,
    ) -> Result<JobResult, GatewayError> {
        for _ in 0..max_polls {
            let result = self.get_job(job_id)?;
            if result.status.is_terminal() {
                return Ok(result);
            }
        }
        // Return last status (non-terminal)
        self.get_job(job_id)
    }

    // ── Model operations ─────────────────────────────────────

    pub fn list_models(&self) -> Result<ModelList, GatewayError> {
        let resp = self.do_request("GET", "/v1/models", None)?;
        let body = self.parse_response(resp)?;
        // Parse models array from {"models":["a","b"]}
        let models = extract_string_array(&body, "models");
        Ok(ModelList { models })
    }

    // ── Health ───────────────────────────────────────────────

    pub fn health(&self) -> Result<bool, GatewayError> {
        let resp = self.do_request("GET", "/v1/health", None)?;
        let body = self.parse_response(resp)?;
        Ok(body.contains("healthy"))
    }

    // ── Batch operations ─────────────────────────────────────

    /// Submit multiple inference requests, returning results for each.
    pub fn submit_batch(
        &self,
        requests: &[InferenceRequestBuilder],
    ) -> Vec<Result<SubmitResult, GatewayError>> {
        requests.iter().map(|r| self.submit_inference(r)).collect()
    }

    /// Cancel multiple jobs.
    pub fn cancel_batch(&self, job_ids: &[&str]) -> Vec<Result<JobResult, GatewayError>> {
        job_ids.iter().map(|id| self.cancel_job(id)).collect()
    }
}

// ── JSON helpers (no serde dependency) ───────────────────────

fn extract_field(body: &str, field: &str) -> Option<String> {
    let pattern = format!(r#""{}":""#, field);
    if let Some(start) = body.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = body[value_start..].find('"') {
            return Some(body[value_start..value_start + end].to_string());
        }
    }
    None
}

fn extract_error(body: &str) -> String {
    extract_field(body, "error").unwrap_or_else(|| body.to_string())
}

fn extract_string_array(body: &str, field: &str) -> Vec<String> {
    let pattern = format!(r#""{}":["#, field);
    if let Some(start) = body.find(&pattern) {
        let arr_start = start + pattern.len();
        if let Some(arr_end) = body[arr_start..].find(']') {
            let arr_str = &body[arr_start..arr_start + arr_end];
            return arr_str
                .split('"')
                .filter(|s| !s.is_empty() && *s != ",")
                .map(|s| s.to_string())
                .collect();
        }
    }
    Vec::new()
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: create a mock transport that dispatches to a simple in-memory gateway
    fn mock_transport(
    ) -> InProcessGatewayTransport<impl Fn(&str, &str, &str, Option<&str>) -> TransportResponse>
    {
        use std::sync::{Arc, Mutex};
        let jobs: Arc<Mutex<HashMap<String, (String, String)>>> =
            Arc::new(Mutex::new(HashMap::new())); // job_id -> (status, output)
        let counter: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
        let req_count: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

        let jobs_c = jobs.clone();
        let counter_c = counter.clone();
        let req_count_c = req_count.clone();

        InProcessGatewayTransport::new(
            move |method: &str, path: &str, api_key: &str, body: Option<&str>| {
                // Auth
                if api_key != "valid-key" && api_key != "readonly-key" {
                    return TransportResponse {
                        status: 401,
                        body: r#"{"error":"unauthorized"}"#.into(),
                        headers: HashMap::new(),
                    };
                }

                // Track requests for rate limit testing
                let mut rc = req_count_c.lock().unwrap();
                *rc += 1;

                match (method, path) {
                    ("GET", "/v1/health") => TransportResponse {
                        status: 200,
                        body: r#"{"status":"healthy"}"#.into(),
                        headers: HashMap::new(),
                    },
                    ("GET", "/v1/models") => TransportResponse {
                        status: 200,
                        body: r#"{"models":["llama-7b","mistral-7b","phi-3"]}"#.into(),
                        headers: HashMap::new(),
                    },
                    ("POST", "/v1/inference") => {
                        if api_key == "readonly-key" {
                            return TransportResponse {
                                status: 403,
                                body: r#"{"error":"missing permission"}"#.into(),
                                headers: HashMap::new(),
                            };
                        }
                        let body = body.unwrap_or("");
                        let model = extract_field(body, "model_id").unwrap_or_default();
                        if model != "llama-7b" && model != "mistral-7b" && model != "phi-3" {
                            return TransportResponse {
                                status: 400,
                                body: r#"{"error":"unknown model"}"#.into(),
                                headers: HashMap::new(),
                            };
                        }
                        let mut c = counter_c.lock().unwrap();
                        *c += 1;
                        let job_id = format!("job-{:06}", *c);
                        jobs_c
                            .lock()
                            .unwrap()
                            .insert(job_id.clone(), ("queued".into(), "".into()));
                        TransportResponse {
                            status: 201,
                            body: format!(
                                r#"{{"job_id":"{}","status":"queued","model":"{}"}}"#,
                                job_id, model
                            ),
                            headers: HashMap::new(),
                        }
                    }
                    ("GET", p) if p.starts_with("/v1/inference/") => {
                        let job_id = p.strip_prefix("/v1/inference/").unwrap();
                        let jobs = jobs_c.lock().unwrap();
                        match jobs.get(job_id) {
                            Some((status, output)) => TransportResponse {
                                status: 200,
                                body: format!(
                                    r#"{{"job_id":"{}","status":"{}","output":"{}"}}"#,
                                    job_id, status, output
                                ),
                                headers: HashMap::new(),
                            },
                            None => TransportResponse {
                                status: 404,
                                body: r#"{"error":"not found"}"#.into(),
                                headers: HashMap::new(),
                            },
                        }
                    }
                    ("DELETE", p) if p.starts_with("/v1/inference/") => {
                        let job_id = p.strip_prefix("/v1/inference/").unwrap();
                        let mut jobs = jobs_c.lock().unwrap();
                        match jobs.get_mut(job_id) {
                            Some((status, _)) if status == "queued" || status == "running" => {
                                *status = "cancelled".into();
                                TransportResponse {
                                    status: 200,
                                    body: format!(
                                        r#"{{"job_id":"{}","status":"cancelled"}}"#,
                                        job_id
                                    ),
                                    headers: HashMap::new(),
                                }
                            }
                            Some(_) => TransportResponse {
                                status: 400,
                                body: r#"{"error":"job already terminal"}"#.into(),
                                headers: HashMap::new(),
                            },
                            None => TransportResponse {
                                status: 404,
                                body: r#"{"error":"not found"}"#.into(),
                                headers: HashMap::new(),
                            },
                        }
                    }
                    _ => TransportResponse {
                        status: 404,
                        body: r#"{"error":"not found"}"#.into(),
                        headers: HashMap::new(),
                    },
                }
            },
        )
    }

    fn make_client() -> GatewayClient<
        InProcessGatewayTransport<impl Fn(&str, &str, &str, Option<&str>) -> TransportResponse>,
    > {
        let mut client = GatewayClient::new(mock_transport(), "http://localhost:8080");
        client.add_key(ApiKeyConfig::new("valid-key").with_label("primary"));
        client
    }

    #[test]
    fn test_no_key_returns_unauthorized() {
        let client: GatewayClient<_> =
            GatewayClient::new(mock_transport(), "http://localhost:8080");
        let err = client.health().unwrap_err();
        assert_eq!(err, GatewayError::Unauthorized);
    }

    #[test]
    fn test_health_check() {
        let client = make_client();
        assert!(client.health().unwrap());
    }

    #[test]
    fn test_list_models() {
        let client = make_client();
        let models = client.list_models().unwrap();
        assert_eq!(models.models.len(), 3);
        assert!(models.models.contains(&"llama-7b".to_string()));
        assert!(models.models.contains(&"mistral-7b".to_string()));
        assert!(models.models.contains(&"phi-3".to_string()));
    }

    #[test]
    fn test_submit_inference() {
        let client = make_client();
        let req = InferenceRequestBuilder::new("llama-7b", "Hello world");
        let result = client.submit_inference(&req).unwrap();
        assert_eq!(result.job_id, "job-000001");
        assert_eq!(result.status, JobStatus::Queued);
        assert_eq!(result.model, "llama-7b");
    }

    #[test]
    fn test_submit_with_options() {
        let client = make_client();
        let req = InferenceRequestBuilder::new("mistral-7b", "Translate this")
            .max_tokens(512)
            .callback_url("https://example.com/webhook");
        let result = client.submit_inference(&req).unwrap();
        assert_eq!(result.model, "mistral-7b");
    }

    #[test]
    fn test_submit_unknown_model() {
        let client = make_client();
        let req = InferenceRequestBuilder::new("gpt-4", "test");
        let err = client.submit_inference(&req).unwrap_err();
        assert!(matches!(err, GatewayError::BadRequest(_)));
    }

    #[test]
    fn test_get_job_status() {
        let client = make_client();
        let req = InferenceRequestBuilder::new("llama-7b", "test");
        let submit = client.submit_inference(&req).unwrap();
        let result = client.get_job(&submit.job_id).unwrap();
        assert_eq!(result.job_id, "job-000001");
        assert_eq!(result.status, JobStatus::Queued);
    }

    #[test]
    fn test_get_nonexistent_job() {
        let client = make_client();
        let err = client.get_job("job-999999").unwrap_err();
        assert_eq!(err, GatewayError::NotFound);
    }

    #[test]
    fn test_cancel_job() {
        let client = make_client();
        let req = InferenceRequestBuilder::new("llama-7b", "test");
        let submit = client.submit_inference(&req).unwrap();
        let result = client.cancel_job(&submit.job_id).unwrap();
        assert_eq!(result.status, JobStatus::Cancelled);

        // Verify it's now cancelled
        let status = client.get_job(&submit.job_id).unwrap();
        assert_eq!(status.status, JobStatus::Cancelled);
    }

    #[test]
    fn test_cancel_nonexistent() {
        let client = make_client();
        let err = client.cancel_job("job-999").unwrap_err();
        assert_eq!(err, GatewayError::NotFound);
    }

    #[test]
    fn test_readonly_key_forbidden() {
        let mut client = GatewayClient::new(mock_transport(), "http://localhost:8080");
        client.add_key(ApiKeyConfig::new("readonly-key"));
        let req = InferenceRequestBuilder::new("llama-7b", "test");
        let err = client.submit_inference(&req).unwrap_err();
        assert!(matches!(err, GatewayError::Forbidden(_)));
    }

    #[test]
    fn test_key_ring_management() {
        let mut ring = KeyRing::new();
        assert!(ring.is_empty());

        ring.add_key(ApiKeyConfig::new("key-1").with_label("primary"));
        ring.add_key(ApiKeyConfig::new("key-2").with_label("backup"));
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.active_key().unwrap().key, "key-1");

        assert!(ring.set_active(1));
        assert_eq!(ring.active_key().unwrap().key, "key-2");

        assert!(!ring.set_active(5)); // out of bounds

        let removed = ring.remove_key(0).unwrap();
        assert_eq!(removed.key, "key-1");
        assert_eq!(ring.len(), 1);
        assert_eq!(ring.active_key().unwrap().key, "key-2");
    }

    #[test]
    fn test_key_id_derivation() {
        let k1 = ApiKeyConfig::new("test-key-abc");
        let k2 = ApiKeyConfig::new("test-key-abc");
        let k3 = ApiKeyConfig::new("different-key");
        assert_eq!(k1.key_id(), k2.key_id()); // deterministic
        assert_ne!(k1.key_id(), k3.key_id()); // different keys, different IDs
        assert!(k1.key_id().starts_with("key-"));
    }

    #[test]
    fn test_switch_active_key() {
        let mut client = GatewayClient::new(mock_transport(), "http://localhost:8080");
        client.add_key(ApiKeyConfig::new("invalid-key"));
        client.add_key(ApiKeyConfig::new("valid-key"));

        // First key is invalid
        assert!(client.health().is_err());

        // Switch to valid key
        assert!(client.set_active_key(1));
        assert!(client.health().unwrap());
    }

    #[test]
    fn test_batch_submit() {
        let client = make_client();
        let reqs = vec![
            InferenceRequestBuilder::new("llama-7b", "hello"),
            InferenceRequestBuilder::new("mistral-7b", "world"),
            InferenceRequestBuilder::new("gpt-4", "invalid"), // should fail
        ];
        let results = client.submit_batch(&reqs);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
    }

    #[test]
    fn test_batch_cancel() {
        let client = make_client();
        let r1 = InferenceRequestBuilder::new("llama-7b", "a");
        let r2 = InferenceRequestBuilder::new("llama-7b", "b");
        let j1 = client.submit_inference(&r1).unwrap().job_id;
        let j2 = client.submit_inference(&r2).unwrap().job_id;

        let results = client.cancel_batch(&[&j1, &j2, "job-nonexistent"]);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
    }

    #[test]
    fn test_poll_until_done_already_terminal() {
        let client = make_client();
        let req = InferenceRequestBuilder::new("llama-7b", "test");
        let submit = client.submit_inference(&req).unwrap();
        client.cancel_job(&submit.job_id).unwrap(); // make it terminal

        let result = client.poll_until_done(&submit.job_id, 5).unwrap();
        assert_eq!(result.status, JobStatus::Cancelled);
    }

    #[test]
    fn test_poll_until_done_stays_queued() {
        let client = make_client();
        let req = InferenceRequestBuilder::new("llama-7b", "test");
        let submit = client.submit_inference(&req).unwrap();

        // Will exhaust polls and return non-terminal
        let result = client.poll_until_done(&submit.job_id, 3).unwrap();
        assert_eq!(result.status, JobStatus::Queued); // still queued
    }

    #[test]
    fn test_job_status_is_terminal() {
        assert!(!JobStatus::Queued.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_inference_request_builder_json() {
        let req = InferenceRequestBuilder::new("llama-7b", "hello")
            .max_tokens(128)
            .callback_url("https://example.com/cb");
        let json = req.to_json();
        assert!(json.contains(r#""model_id":"llama-7b""#));
        assert!(json.contains(r#""input":"hello""#));
        assert!(json.contains(r#""max_tokens":"128""#));
        assert!(json.contains(r#""callback_url":"https://example.com/cb""#));
    }

    #[test]
    fn test_error_display() {
        assert_eq!(format!("{}", GatewayError::Unauthorized), "unauthorized");
        assert_eq!(format!("{}", GatewayError::NotFound), "not found");
        assert!(format!(
            "{}",
            GatewayError::RateLimited {
                retry_after_secs: 30
            }
        )
        .contains("30s"));
    }

    #[test]
    fn test_extract_string_array() {
        let body = r#"{"models":["a","b","c"]}"#;
        let arr = extract_string_array(body, "models");
        assert_eq!(arr, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_extract_string_array_empty() {
        let body = r#"{"models":[]}"#;
        let arr = extract_string_array(body, "models");
        assert!(arr.is_empty());
    }
}

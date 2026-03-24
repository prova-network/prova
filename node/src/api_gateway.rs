// node/src/api_gateway.rs — HTTP API Gateway for external inference requests
//
// Routes external HTTP requests to the internal scheduler, handles auth,
// rate limiting, request validation, and response formatting.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// API key with associated permissions and rate limits.
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub key: String,
    pub owner: String,
    pub permissions: Vec<Permission>,
    pub rate_limit: RateLimit,
    pub created_at: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    SubmitInference,
    QueryStatus,
    ListModels,
    CancelJob,
    Admin,
}

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub max_requests: u64,
    pub window_secs: u64,
}

/// Tracks request counts per key per window.
#[derive(Debug, Clone)]
struct RateLimitState {
    count: u64,
    window_start: u64,
}

/// An incoming API request.
#[derive(Debug, Clone)]
pub struct ApiRequest {
    pub method: HttpMethod,
    pub path: String,
    pub api_key: Option<String>,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Delete,
}

/// API response.
#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl ApiResponse {
    pub fn ok(body: &str) -> Self {
        Self {
            status: 200,
            body: body.to_string(),
            headers: Self::json_headers(),
        }
    }
    pub fn created(body: &str) -> Self {
        Self {
            status: 201,
            body: body.to_string(),
            headers: Self::json_headers(),
        }
    }
    pub fn bad_request(msg: &str) -> Self {
        Self {
            status: 400,
            body: format!(r#"{{"error":"{}"}}"#, msg),
            headers: Self::json_headers(),
        }
    }
    pub fn unauthorized() -> Self {
        Self {
            status: 401,
            body: r#"{"error":"unauthorized"}"#.to_string(),
            headers: Self::json_headers(),
        }
    }
    pub fn forbidden(msg: &str) -> Self {
        Self {
            status: 403,
            body: format!(r#"{{"error":"{}"}}"#, msg),
            headers: Self::json_headers(),
        }
    }
    pub fn not_found() -> Self {
        Self {
            status: 404,
            body: r#"{"error":"not found"}"#.to_string(),
            headers: Self::json_headers(),
        }
    }
    pub fn rate_limited(retry_after: u64) -> Self {
        let mut h = Self::json_headers();
        h.insert("Retry-After".into(), retry_after.to_string());
        Self {
            status: 429,
            body: r#"{"error":"rate limit exceeded"}"#.to_string(),
            headers: h,
        }
    }
    fn json_headers() -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("Content-Type".into(), "application/json".into());
        h
    }
}

/// Inference submission via the API.
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    pub model_id: String,
    pub input: String,
    pub max_tokens: u64,
    pub callback_url: Option<String>,
}

/// Result of a submitted inference job.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub job_id: String,
    pub status: JobStatus,
    pub output: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Webhook registration for async job completion notifications.
#[derive(Debug, Clone)]
pub struct Webhook {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: String,
    pub active: bool,
}

/// The API Gateway.
pub struct ApiGateway {
    keys: HashMap<String, ApiKey>,
    rate_state: Arc<Mutex<HashMap<String, RateLimitState>>>,
    jobs: Arc<Mutex<HashMap<String, InferenceResult>>>,
    webhooks: Arc<Mutex<Vec<Webhook>>>,
    models: Vec<String>,
    next_job_id: Arc<Mutex<u64>>,
    webhook_deliveries: Arc<Mutex<Vec<(String, String)>>>, // (webhook_id, job_id)
}

impl ApiGateway {
    pub fn new(models: Vec<String>) -> Self {
        Self {
            keys: HashMap::new(),
            rate_state: Arc::new(Mutex::new(HashMap::new())),
            jobs: Arc::new(Mutex::new(HashMap::new())),
            webhooks: Arc::new(Mutex::new(Vec::new())),
            models,
            next_job_id: Arc::new(Mutex::new(1)),
            webhook_deliveries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register_key(&mut self, key: ApiKey) {
        self.keys.insert(key.key.clone(), key);
    }

    pub fn register_webhook(&self, webhook: Webhook) {
        self.webhooks.lock().unwrap().push(webhook);
    }

    /// Main request router.
    pub fn handle(&self, req: &ApiRequest) -> ApiResponse {
        // Auth check
        let key = match &req.api_key {
            Some(k) => match self.keys.get(k) {
                Some(key) if key.enabled => key,
                Some(_) => return ApiResponse::forbidden("api key disabled"),
                None => return ApiResponse::unauthorized(),
            },
            None => return ApiResponse::unauthorized(),
        };

        // Rate limit check
        if let Some(resp) = self.check_rate_limit(key, req.timestamp) {
            return resp;
        }

        // Route
        match (&req.method, req.path.as_str()) {
            (HttpMethod::Post, "/v1/inference") => self.submit_inference(key, req),
            (HttpMethod::Get, p) if p.starts_with("/v1/inference/") => {
                let job_id = p.strip_prefix("/v1/inference/").unwrap();
                self.get_job(key, job_id)
            }
            (HttpMethod::Delete, p) if p.starts_with("/v1/inference/") => {
                let job_id = p.strip_prefix("/v1/inference/").unwrap();
                self.cancel_job(key, job_id)
            }
            (HttpMethod::Get, "/v1/models") => self.list_models(key),
            (HttpMethod::Get, "/v1/health") => ApiResponse::ok(r#"{"status":"healthy"}"#),
            _ => ApiResponse::not_found(),
        }
    }

    fn check_rate_limit(&self, key: &ApiKey, now: u64) -> Option<ApiResponse> {
        let mut state = self.rate_state.lock().unwrap();
        let entry = state.entry(key.key.clone()).or_insert(RateLimitState {
            count: 0,
            window_start: now,
        });
        if now - entry.window_start >= key.rate_limit.window_secs {
            entry.count = 0;
            entry.window_start = now;
        }
        entry.count += 1;
        if entry.count > key.rate_limit.max_requests {
            let retry_after = key.rate_limit.window_secs - (now - entry.window_start);
            return Some(ApiResponse::rate_limited(retry_after));
        }
        None
    }

    fn submit_inference(&self, key: &ApiKey, req: &ApiRequest) -> ApiResponse {
        if !key.permissions.contains(&Permission::SubmitInference) {
            return ApiResponse::forbidden("missing permission: SubmitInference");
        }
        let body = match &req.body {
            Some(b) => b,
            None => return ApiResponse::bad_request("missing request body"),
        };

        // Parse simple JSON-like body (model_id, input, max_tokens)
        let parsed = match self.parse_inference_body(body) {
            Some(p) => p,
            None => return ApiResponse::bad_request("invalid request body"),
        };

        if !self.models.contains(&parsed.model_id) {
            return ApiResponse::bad_request("unknown model");
        }

        let mut id_counter = self.next_job_id.lock().unwrap();
        let job_id = format!("job-{:06}", *id_counter);
        *id_counter += 1;

        let result = InferenceResult {
            job_id: job_id.clone(),
            status: JobStatus::Queued,
            output: None,
        };
        self.jobs.lock().unwrap().insert(job_id.clone(), result);

        ApiResponse::created(&format!(
            r#"{{"job_id":"{}","status":"queued","model":"{}"}}"#,
            job_id, parsed.model_id
        ))
    }

    fn get_job(&self, key: &ApiKey, job_id: &str) -> ApiResponse {
        if !key.permissions.contains(&Permission::QueryStatus) {
            return ApiResponse::forbidden("missing permission: QueryStatus");
        }
        let jobs = self.jobs.lock().unwrap();
        match jobs.get(job_id) {
            Some(j) => {
                let status = match j.status {
                    JobStatus::Queued => "queued",
                    JobStatus::Running => "running",
                    JobStatus::Completed => "completed",
                    JobStatus::Failed => "failed",
                    JobStatus::Cancelled => "cancelled",
                };
                let output = j.output.as_deref().unwrap_or("");
                ApiResponse::ok(&format!(
                    r#"{{"job_id":"{}","status":"{}","output":"{}"}}"#,
                    j.job_id, status, output
                ))
            }
            None => ApiResponse::not_found(),
        }
    }

    fn cancel_job(&self, key: &ApiKey, job_id: &str) -> ApiResponse {
        if !key.permissions.contains(&Permission::CancelJob) {
            return ApiResponse::forbidden("missing permission: CancelJob");
        }
        let mut jobs = self.jobs.lock().unwrap();
        match jobs.get_mut(job_id) {
            Some(j) if j.status == JobStatus::Queued || j.status == JobStatus::Running => {
                j.status = JobStatus::Cancelled;
                ApiResponse::ok(&format!(
                    r#"{{"job_id":"{}","status":"cancelled"}}"#,
                    job_id
                ))
            }
            Some(_) => ApiResponse::bad_request("job already terminal"),
            None => ApiResponse::not_found(),
        }
    }

    fn list_models(&self, key: &ApiKey) -> ApiResponse {
        if !key.permissions.contains(&Permission::ListModels) {
            return ApiResponse::forbidden("missing permission: ListModels");
        }
        let models: Vec<String> = self.models.iter().map(|m| format!(r#""{}""#, m)).collect();
        ApiResponse::ok(&format!(r#"{{"models":[{}]}}"#, models.join(",")))
    }

    /// Complete a job (called by internal scheduler callback).
    pub fn complete_job(&self, job_id: &str, output: &str) -> bool {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(j) = jobs.get_mut(job_id) {
            j.status = JobStatus::Completed;
            j.output = Some(output.to_string());
            // Fire webhooks
            let webhooks = self.webhooks.lock().unwrap();
            let mut deliveries = self.webhook_deliveries.lock().unwrap();
            for wh in webhooks.iter() {
                if wh.active && wh.events.contains(&"job.completed".to_string()) {
                    deliveries.push((wh.id.clone(), job_id.to_string()));
                }
            }
            true
        } else {
            false
        }
    }

    /// Fail a job.
    pub fn fail_job(&self, job_id: &str) -> bool {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(j) = jobs.get_mut(job_id) {
            j.status = JobStatus::Failed;
            true
        } else {
            false
        }
    }

    pub fn webhook_deliveries(&self) -> Vec<(String, String)> {
        self.webhook_deliveries.lock().unwrap().clone()
    }

    pub fn job_count(&self) -> usize {
        self.jobs.lock().unwrap().len()
    }

    fn parse_inference_body(&self, body: &str) -> Option<InferenceRequest> {
        // Simple key-value parsing (no serde dependency)
        let model_id = Self::extract_field(body, "model_id")?;
        let input = Self::extract_field(body, "input")?;
        let max_tokens = Self::extract_field(body, "max_tokens")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(256);
        let callback_url = Self::extract_field(body, "callback_url");
        Some(InferenceRequest {
            model_id,
            input,
            max_tokens,
            callback_url,
        })
    }

    fn extract_field(body: &str, field: &str) -> Option<String> {
        let pattern = format!(r#""{}":""#, field);
        if let Some(start) = body.find(&pattern) {
            let value_start = start + pattern.len();
            if let Some(end) = body[value_start..].find('"') {
                return Some(body[value_start..value_start + end].to_string());
            }
        }
        // Try numeric values
        let pattern_num = format!(r#""{}":"#, field);
        if let Some(start) = body.find(&pattern_num) {
            let value_start = start + pattern_num.len();
            let rest = &body[value_start..];
            let end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            if end > 0 {
                return Some(rest[..end].to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key(perms: Vec<Permission>) -> ApiKey {
        ApiKey {
            key: "test-key-1".into(),
            owner: "alice".into(),
            permissions: perms,
            rate_limit: RateLimit {
                max_requests: 10,
                window_secs: 60,
            },
            created_at: 1000,
            enabled: true,
        }
    }

    fn test_gateway() -> ApiGateway {
        let mut gw = ApiGateway::new(vec!["llama-7b".into(), "mistral-7b".into()]);
        gw.register_key(test_key(vec![
            Permission::SubmitInference,
            Permission::QueryStatus,
            Permission::ListModels,
            Permission::CancelJob,
        ]));
        gw
    }

    fn req(method: HttpMethod, path: &str, key: Option<&str>, body: Option<&str>) -> ApiRequest {
        ApiRequest {
            method,
            path: path.into(),
            api_key: key.map(|s| s.into()),
            body: body.map(|s| s.into()),
            headers: HashMap::new(),
            timestamp: 1000,
        }
    }

    #[test]
    fn test_unauthorized_no_key() {
        let gw = test_gateway();
        let r = gw.handle(&req(HttpMethod::Get, "/v1/health", None, None));
        assert_eq!(r.status, 401);
    }

    #[test]
    fn test_unauthorized_bad_key() {
        let gw = test_gateway();
        let r = gw.handle(&req(HttpMethod::Get, "/v1/health", Some("wrong"), None));
        assert_eq!(r.status, 401);
    }

    #[test]
    fn test_disabled_key() {
        let mut gw = test_gateway();
        let mut k = test_key(vec![Permission::ListModels]);
        k.key = "disabled-key".into();
        k.enabled = false;
        gw.register_key(k);
        let r = gw.handle(&req(
            HttpMethod::Get,
            "/v1/models",
            Some("disabled-key"),
            None,
        ));
        assert_eq!(r.status, 403);
    }

    #[test]
    fn test_health_check() {
        let gw = test_gateway();
        let r = gw.handle(&req(
            HttpMethod::Get,
            "/v1/health",
            Some("test-key-1"),
            None,
        ));
        assert_eq!(r.status, 200);
        assert!(r.body.contains("healthy"));
    }

    #[test]
    fn test_list_models() {
        let gw = test_gateway();
        let r = gw.handle(&req(
            HttpMethod::Get,
            "/v1/models",
            Some("test-key-1"),
            None,
        ));
        assert_eq!(r.status, 200);
        assert!(r.body.contains("llama-7b"));
        assert!(r.body.contains("mistral-7b"));
    }

    #[test]
    fn test_submit_inference() {
        let gw = test_gateway();
        let body = r#"{"model_id":"llama-7b","input":"hello world","max_tokens":128}"#;
        let r = gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        assert_eq!(r.status, 201);
        assert!(r.body.contains("job-000001"));
        assert!(r.body.contains("queued"));
        assert_eq!(gw.job_count(), 1);
    }

    #[test]
    fn test_submit_unknown_model() {
        let gw = test_gateway();
        let body = r#"{"model_id":"gpt-4","input":"hello"}"#;
        let r = gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        assert_eq!(r.status, 400);
        assert!(r.body.contains("unknown model"));
    }

    #[test]
    fn test_get_job_status() {
        let gw = test_gateway();
        let body = r#"{"model_id":"llama-7b","input":"test"}"#;
        gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        let r = gw.handle(&req(
            HttpMethod::Get,
            "/v1/inference/job-000001",
            Some("test-key-1"),
            None,
        ));
        assert_eq!(r.status, 200);
        assert!(r.body.contains("queued"));
    }

    #[test]
    fn test_get_nonexistent_job() {
        let gw = test_gateway();
        let r = gw.handle(&req(
            HttpMethod::Get,
            "/v1/inference/job-999",
            Some("test-key-1"),
            None,
        ));
        assert_eq!(r.status, 404);
    }

    #[test]
    fn test_cancel_job() {
        let gw = test_gateway();
        let body = r#"{"model_id":"llama-7b","input":"test"}"#;
        gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        let r = gw.handle(&req(
            HttpMethod::Delete,
            "/v1/inference/job-000001",
            Some("test-key-1"),
            None,
        ));
        assert_eq!(r.status, 200);
        assert!(r.body.contains("cancelled"));
    }

    #[test]
    fn test_cancel_completed_job() {
        let gw = test_gateway();
        let body = r#"{"model_id":"llama-7b","input":"test"}"#;
        gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        gw.complete_job("job-000001", "result");
        let r = gw.handle(&req(
            HttpMethod::Delete,
            "/v1/inference/job-000001",
            Some("test-key-1"),
            None,
        ));
        assert_eq!(r.status, 400);
        assert!(r.body.contains("already terminal"));
    }

    #[test]
    fn test_complete_job() {
        let gw = test_gateway();
        let body = r#"{"model_id":"llama-7b","input":"test"}"#;
        gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        assert!(gw.complete_job("job-000001", "generated text"));
        let r = gw.handle(&req(
            HttpMethod::Get,
            "/v1/inference/job-000001",
            Some("test-key-1"),
            None,
        ));
        assert!(r.body.contains("completed"));
        assert!(r.body.contains("generated text"));
    }

    #[test]
    fn test_fail_job() {
        let gw = test_gateway();
        let body = r#"{"model_id":"llama-7b","input":"test"}"#;
        gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        assert!(gw.fail_job("job-000001"));
        let r = gw.handle(&req(
            HttpMethod::Get,
            "/v1/inference/job-000001",
            Some("test-key-1"),
            None,
        ));
        assert!(r.body.contains("failed"));
    }

    #[test]
    fn test_rate_limiting() {
        let mut gw = ApiGateway::new(vec!["llama-7b".into()]);
        let mut key = test_key(vec![Permission::ListModels]);
        key.rate_limit = RateLimit {
            max_requests: 3,
            window_secs: 60,
        };
        gw.register_key(key);

        for i in 0..3 {
            let r = gw.handle(&req(
                HttpMethod::Get,
                "/v1/models",
                Some("test-key-1"),
                None,
            ));
            assert_eq!(r.status, 200, "request {} should succeed", i);
        }
        let r = gw.handle(&req(
            HttpMethod::Get,
            "/v1/models",
            Some("test-key-1"),
            None,
        ));
        assert_eq!(r.status, 429);
        assert!(r.headers.contains_key("Retry-After"));
    }

    #[test]
    fn test_rate_limit_window_reset() {
        let mut gw = ApiGateway::new(vec!["llama-7b".into()]);
        let mut key = test_key(vec![Permission::ListModels]);
        key.rate_limit = RateLimit {
            max_requests: 2,
            window_secs: 10,
        };
        gw.register_key(key);

        let mut r = ApiRequest {
            method: HttpMethod::Get,
            path: "/v1/models".into(),
            api_key: Some("test-key-1".into()),
            body: None,
            headers: HashMap::new(),
            timestamp: 1000,
        };
        assert_eq!(gw.handle(&r).status, 200);
        assert_eq!(gw.handle(&r).status, 200);
        assert_eq!(gw.handle(&r).status, 429);

        // Advance past window
        r.timestamp = 1011;
        assert_eq!(gw.handle(&r).status, 200);
    }

    #[test]
    fn test_permission_denied() {
        let mut gw = ApiGateway::new(vec!["llama-7b".into()]);
        let mut key = test_key(vec![Permission::ListModels]); // no SubmitInference
        gw.register_key(key);
        let body = r#"{"model_id":"llama-7b","input":"test"}"#;
        let r = gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        assert_eq!(r.status, 403);
    }

    #[test]
    fn test_webhook_delivery_on_complete() {
        let gw = test_gateway();
        gw.register_webhook(Webhook {
            id: "wh-1".into(),
            url: "https://example.com/hook".into(),
            events: vec!["job.completed".into()],
            secret: "secret".into(),
            active: true,
        });
        let body = r#"{"model_id":"llama-7b","input":"test"}"#;
        gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        gw.complete_job("job-000001", "done");
        let deliveries = gw.webhook_deliveries();
        assert_eq!(deliveries.len(), 1);
        assert_eq!(
            deliveries[0],
            ("wh-1".to_string(), "job-000001".to_string())
        );
    }

    #[test]
    fn test_not_found_route() {
        let gw = test_gateway();
        let r = gw.handle(&req(
            HttpMethod::Get,
            "/v1/unknown",
            Some("test-key-1"),
            None,
        ));
        assert_eq!(r.status, 404);
    }

    #[test]
    fn test_multiple_jobs_sequential_ids() {
        let gw = test_gateway();
        let body = r#"{"model_id":"llama-7b","input":"a"}"#;
        let r1 = gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        let r2 = gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            Some(body),
        ));
        assert!(r1.body.contains("job-000001"));
        assert!(r2.body.contains("job-000002"));
        assert_eq!(gw.job_count(), 2);
    }

    #[test]
    fn test_missing_body() {
        let gw = test_gateway();
        let r = gw.handle(&req(
            HttpMethod::Post,
            "/v1/inference",
            Some("test-key-1"),
            None,
        ));
        assert_eq!(r.status, 400);
    }
}

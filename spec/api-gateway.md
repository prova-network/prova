# SPEC-024: API Gateway Specification

**Status:** Final
**Authors:** Capri (autonomous build)
**Created:** 2026-03-04

## 1. Overview

The Prova API Gateway provides an HTTP interface for external clients to submit inference requests, query job status, list available models, manage webhooks, and integrate with the Prova network without running a full node. It serves as the primary entry point for application developers.

## 2. Design Goals

- **Simplicity:** RESTful JSON API with minimal required fields
- **Security:** API key authentication with per-key permissions and rate limiting
- **Observability:** Webhook-based async notifications for job lifecycle events
- **Composability:** Thin translation layer between HTTP and the internal scheduler — no business logic duplication

## 3. Authentication

### 3.1 API Keys

All requests MUST include an API key via the `X-API-Key` header or `api_key` query parameter.

```
X-API-Key: prova_live_a1b2c3d4e5f6...
```

Keys have the following properties:

| Field | Type | Description |
|-------|------|-------------|
| `key` | string | Opaque token, prefix `prova_live_` (production) or `prova_test_` (testnet) |
| `owner` | string | Account address of the key holder |
| `permissions` | string[] | Subset of: `submit_inference`, `query_status`, `list_models`, `cancel_job`, `admin` |
| `rate_limit` | object | `{ max_requests: u64, window_secs: u64 }` |
| `created_at` | u64 | Unix epoch seconds |
| `enabled` | bool | Disabled keys receive HTTP 403 |

### 3.2 Permission Model

| Permission | Grants |
|------------|--------|
| `submit_inference` | POST /v1/inference |
| `query_status` | GET /v1/inference/{job_id} |
| `list_models` | GET /v1/models |
| `cancel_job` | DELETE /v1/inference/{job_id} |
| `admin` | All endpoints + key management |

Requests to endpoints without the required permission return `403 Forbidden`.

### 3.3 Rate Limiting

Each API key has an independent sliding window rate limiter:

- Window: configurable per key (default 60s)
- Max requests: configurable per key (default 100)
- On exceeded: HTTP 429 with `Retry-After` header (seconds until window resets)
- Window resets when `now - window_start >= window_secs`

Rate limit headers on every response:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 73
X-RateLimit-Reset: 1709553600
```

## 4. Endpoints

Base URL: `https://{gateway-host}/v1/`

### 4.1 Health Check

```
GET /v1/health
```

Response `200`:
```json
{ "status": "healthy" }
```

No authentication required in permissive mode; requires valid key in strict mode (default).

### 4.2 List Models

```
GET /v1/models
```

Response `200`:
```json
{
  "models": [
    {
      "id": "llama-7b",
      "name": "LLaMA 7B",
      "providers": 12,
      "avg_latency_ms": 340,
      "price_per_token": "0.00001"
    }
  ]
}
```

### 4.3 Submit Inference

```
POST /v1/inference
Content-Type: application/json
```

Request body:
```json
{
  "model_id": "llama-7b",
  "input": "What is the capital of France?",
  "max_tokens": 256,
  "callback_url": "https://example.com/webhook"  // optional
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `model_id` | yes | Must match a registered model |
| `input` | yes | UTF-8 input text |
| `max_tokens` | no | Default 256, max 4096 |
| `callback_url` | no | Per-request webhook URL for completion |

Response `201`:
```json
{
  "job_id": "job-000001",
  "status": "queued",
  "model": "llama-7b"
}
```

Errors:
- `400` — missing body, invalid JSON, unknown model, max_tokens out of range
- `403` — missing `submit_inference` permission
- `429` — rate limited

### 4.4 Query Job Status

```
GET /v1/inference/{job_id}
```

Response `200`:
```json
{
  "job_id": "job-000001",
  "status": "completed",
  "output": "The capital of France is Paris.",
  "model": "llama-7b",
  "created_at": 1709553000,
  "completed_at": 1709553002,
  "tokens_used": 12
}
```

Job statuses: `queued` → `running` → `completed` | `failed` | `cancelled`

### 4.5 Cancel Job

```
DELETE /v1/inference/{job_id}
```

Response `200`:
```json
{ "job_id": "job-000001", "status": "cancelled" }
```

Errors:
- `400` — job already in terminal state (`completed`, `failed`, `cancelled`)
- `404` — job not found

### 4.6 Not Found

All unmatched routes return:
```json
{ "error": "not found" }
```

## 5. Webhooks

### 5.1 Registration

Webhooks are registered via the admin API or node configuration.

```json
{
  "id": "wh-abc123",
  "url": "https://example.com/prova-events",
  "events": ["job.completed", "job.failed"],
  "secret": "whsec_...",
  "active": true
}
```

### 5.2 Delivery

On matching events, the gateway POSTs to the webhook URL:

```json
{
  "event": "job.completed",
  "job_id": "job-000001",
  "timestamp": 1709553002,
  "data": {
    "output": "The capital of France is Paris.",
    "tokens_used": 12
  }
}
```

Headers:
```
Content-Type: application/json
X-Prova-Signature: sha256=<HMAC-SHA256(secret, body)>
X-Prova-Event: job.completed
X-Prova-Delivery: <uuid>
```

### 5.3 Retry Policy

- 3 attempts with exponential backoff: 1s, 5s, 25s
- Non-2xx responses trigger retry
- After 3 failures, webhook is marked `failed` (not disabled)
- Signature verification: `HMAC-SHA256(webhook.secret, raw_body)`

## 6. Error Format

All errors use a consistent JSON envelope:

```json
{
  "error": "description of what went wrong",
  "code": "RATE_LIMITED",
  "retry_after": 42
}
```

Error codes: `UNAUTHORIZED`, `FORBIDDEN`, `NOT_FOUND`, `RATE_LIMITED`, `BAD_REQUEST`, `INTERNAL_ERROR`

## 7. Internal Architecture

```
Client → [HTTP] → API Gateway → [internal] → Scheduler → Provider Network
                       ↓
                  Rate Limiter
                       ↓
                  Auth / Permissions
                       ↓
                  Route Dispatch
                       ↓
                  Job Store (in-memory, persisted via state trie)
                       ↓
                  Webhook Delivery Engine
```

The gateway is **stateless** except for:
1. Rate limit counters (ephemeral, can be lost on restart)
2. Job store (persisted to chain state for durability)

Multiple gateway instances can run behind a load balancer with shared job store.

## 8. Security Considerations

- **Key rotation:** Keys should be rotatable without downtime; old keys remain valid for a grace period
- **TLS required:** Gateway MUST only accept HTTPS in production
- **Input validation:** `model_id` validated against registry; `input` length bounded; `max_tokens` capped
- **No credential forwarding:** API keys are gateway-scoped, never forwarded to providers
- **Webhook HMAC:** Prevents spoofed delivery; clients MUST verify `X-Prova-Signature`

## 9. Future Extensions

- **Streaming responses** via SSE (`GET /v1/inference/{job_id}/stream`)
- **Batch inference** (`POST /v1/inference/batch`)
- **API key self-service** via signed on-chain transactions
- **Usage metering** and billing integration with payment channels

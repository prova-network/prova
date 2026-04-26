# 4.1 Network protocol

How provers talk to each other and to clients off-chain. This section is **draft** because we expect the conventions to evolve as we run the testnet at scale.

## 4.1.1 Transport

All Prova network traffic is **HTTPS over TCP**, no exceptions. We do not use libp2p, gRPC, or QUIC at v1. Provers MUST present a valid TLS 1.2+ certificate from a publicly-trusted CA. Self-signed certificates are not accepted.

Rationale: Prova clients are commodity software (browsers, curl, the CLI). HTTPS is the only transport every client speaks. We do not need pubsub or peer discovery; the on-chain registry IS the discovery layer.

Provers SHOULD enable HSTS:

```
strict-transport-security: max-age=31536000; includeSubDomains
```

Provers MUST NOT serve `/piece/*` over plain HTTP. A request to `http://{endpoint}/piece/{cid}` MUST be rejected with `301` redirect to the HTTPS equivalent.

## 4.1.2 Endpoint registration

A prover MUST register an HTTPS endpoint via `ProverRegistry.register(endpoint, features, capacity, region, attestation)`. The endpoint MUST:

- Resolve to a hostname under the prover's control
- Serve TLS 1.2 or 1.3
- Be reachable from at least one third-party probe (we run a small probe network and publish results in `RetrievabilityRegistry`)
- Respond to `GET /healthz` with `200 {"ok": true, "endpoint": "<endpoint>", "now": <unix>}` within 5 seconds

A prover MAY register multiple endpoints under the same registry entry by using comma-separated URLs in the `endpoint` field. The first reachable URL is used by retrieval clients; the others are tried in order.

### 4.1.2.1 Endpoint update

A prover MAY update the endpoint via `ProverRegistry.updateEndpoint(newEndpoint)`. The update is subject to a 1-epoch (24h) staleness window, during which clients MAY use either the old or the new endpoint. Provers MUST keep both endpoints serving correct bytes during this window.

### 4.1.2.2 Endpoint health

A health probe failing the 5-second budget for `numUnhealthyEpochs` (proposed: 7) consecutive epochs SHOULD trigger automatic deregistration via `ProverRegistry.markStale(prover)`, callable by anyone. A deregistered prover MAY re-register but loses any in-flight deal acceptances scheduled in the stale window.

## 4.1.3 Retrieval

```
GET https://{prover-endpoint}/piece/{cid}
```

The prover MUST respond with the raw bytes of the piece, with these headers:

| Header | Value |
| --- | --- |
| `content-type` | as committed in the deal's metadata, defaulting to `application/octet-stream` |
| `content-length` | piece size in bytes |
| `x-prova-piece-cid` | the requested CID |
| `x-prova-verified` | `1` if the prover (origin, not CDN) recomputed the CID at intake; `0` otherwise |
| `x-prova-prover` | the prover's Ethereum address |
| `x-prova-epoch` | the current epoch number |
| `cache-control` | `public, max-age=3600, immutable` |
| `content-security-policy` | `default-src 'none'; sandbox` (for non-image/audio/video MIME types) |
| `x-content-type-options` | `nosniff` |
| `referrer-policy` | `no-referrer` |
| `content-disposition` | `attachment; filename="{cid}"` for non-renderable types |
| `access-control-allow-origin` | `*` |
| `access-control-expose-headers` | `x-prova-piece-cid, x-prova-prover, x-prova-epoch, x-prova-verified` |

The `immutable` cache directive is correct: piece-CIDs are content-addressed, so the bytes for a given CID never change. CDNs can cache aggressively.

`HEAD /piece/{cid}` MUST return the same headers without a body and MUST NOT trigger any disk I/O beyond an index lookup. A prover that needs to fault-in the bytes for `HEAD` is misconfigured.

Rate limiting MAY be applied per source IP. Provers SHOULD return `429` with a `Retry-After` header when rate-limited rather than dropping the connection. Recommended limit: 10 req/s per source IP, with bursts up to 50.

### 4.1.3.1 Status code semantics

| Code | When to return |
| --- | --- |
| `200` | Bytes follow, full piece. |
| `206` | Partial Content (range request). |
| `301` | Redirect from `http://` to `https://`. |
| `400` | Malformed CID in the URL path. |
| `404` | The prover does not currently hold this piece (no deal, or deal terminated). |
| `410` | The prover used to hold this piece but the deal was slashed/cancelled. Includes `x-prova-replacement-prover` if known. |
| `429` | Rate limited. Include `Retry-After`. |
| `500` | Internal error. SHOULD include `x-prova-request-id` for correlation. |
| `503` | Service Unavailable (maintenance). Include `Retry-After` and the `x-prova-maintenance-window` header (start unix, end unix). |

Provers MUST NOT use any code outside this set for `/piece/{cid}` responses.

## 4.1.4 Range requests

Retrieval MUST support HTTP range requests:

```
Range: bytes=0-1048575
```

The prover MUST respond with `206 Partial Content`, `Content-Range: bytes 0-1048575/{total}`, and the requested byte range.

The prover MUST support:

- Single-range requests (`bytes=0-1048575`)
- Open-ended ranges (`bytes=1048576-`)
- Suffix ranges (`bytes=-65536`)

The prover MAY refuse multi-range requests (`bytes=0-100,200-300`) with `416 Range Not Satisfiable`.

Range requests are how SDKs stream large files without buffering the whole piece in memory, and how the SDK's progressive verification (§4.1.5) works.

## 4.1.5 Verification at the client

A retrieval client SHOULD recompute the piece-CID over the received bytes and compare to the requested CID. The CLI's `prova get` does this by default; the SDK exposes `verify: true` as a config option (default `true`).

For range requests, the client MAY perform **progressive verification**: each range maps to a specific subtree of the piece's Merkle tree (since the piece-CID is computed by the same scheme as §2.1 PDP). The client recomputes the subtree root and compares against a fragment of the on-chain root. This lets a SDK reject bad bytes after the first chunk instead of after the whole download.

### 4.1.5.1 Verification failure handling

If the recomputed CID does not match, the client MUST treat the response as invalid. The client MAY:

1. Retry against another prover holding the same piece (use `ContentRegistry.getProversForPiece(cid)`).
2. Submit a `markRetrievabilityFault` call once the off-chain dispute window opens (§2.3).
3. If the response is structurally valid (correct length, correct headers) but the bytes are wrong, the client MAY report a `provider_returned_wrong_bytes` event to the API gateway for analytics.

A retry against the same prover within `retryBackoff` (proposed: 60s) MUST NOT count as a separate retrievability sample; the sampler protocol coalesces these.

## 4.1.6 Prover-to-prover replication

When a deal is replicated across multiple provers (deal redundancy parameter > 1), one prover MAY pull the bytes from another prover holding the same piece, rather than requiring the client to upload N copies.

The pull request format:

```
GET https://{source-endpoint}/piece/{cid}?replicate-for={destination-prover-address}&deal-id={dealId}
```

The source prover MAY honor or refuse this request based on its own policy. There is NO protocol-level requirement to honor it; it's a courtesy that helps the network bootstrap.

A source prover that honors replication SHOULD:

- Verify the `destination-prover-address` is registered in `ProverRegistry`
- Verify the `deal-id` exists in `StorageMarketplace` and lists the destination as a participant
- Apply a separate (typically more generous) rate limit for replication vs. client retrieval
- Set `x-prova-replication: 1` and `x-prova-source-prover: {own-address}` in the response

A destination prover MUST NOT count replicated bytes toward its own `x-prova-verified: 1` claim unless it independently recomputes the CID.

## 4.1.7 Sponsored upload path

For the protocol's sponsored / free-tier uploads (browser drag-drop), the upload flow uses the centralized stage server at `p.prova.network`. The stage server's role is documented in [§4.2 API gateway](/api-gateway).

The stage server is not a prover; it does not stake. It accepts uploads from clients without a token, computes the piece-CID, and queues the piece for a real prover via `StorageMarketplace.proposeSponsoredDeal()`. Once a real prover accepts, the bytes are transferred and the stage server retains them only as a fallback for `retentionWindowDays` (proposed: 7).

Sponsored uploads are bounded per source IP per day; the limit is published at `https://p.prova.network/limits` and currently sits at 100 MiB/day per IP.

## 4.1.8 Open questions

- **CDN integration**: provers SHOULD be free to put a CDN in front of `/piece/{cid}` for retrieval performance. We have not specified how the CDN bypass affects `x-prova-verified` (the CDN won't have recomputed the CID). *Current guidance*: set `x-prova-verified: 1` only at the origin, and let the CDN propagate it. Better: a future amendment will define `x-prova-verifier-chain: <prover-address>,<cdn-host>` so clients can reason about who saw the bytes when.
- **WebTransport / HTTP/3**: a future amendment may permit HTTP/3 for retrieval. Not required at v1. The `x-prova-transport: h3` header SHOULD be set when used, so probes can record transport diversity.
- **Reciprocal sampling protocol**: see [§2.3 Data availability](/data-availability).
- **`HEAD` budget for cold pieces**: §4.1.3 says `HEAD` MUST NOT trigger fault-in beyond an index lookup. For provers using cold storage tiers (S3 Glacier, tape), this is not always feasible. A future amendment may define a `HEAD` extension that returns `202 Accepted` with `x-prova-warm-eta: <seconds>` for pieces that need rehydration.
- **Range across pieces**: clients sometimes want to fetch a contiguous range that spans multiple pieces (large dataset). Currently this is the SDK's job. A future API extension may add `GET /pieces/{cid1}..{cidN}/range/{from}-{to}` directly.

This section will be promoted from Draft to Reliable when:

1. At least 20 independent provers are running production-grade endpoints on testnet for ≥ 30 days with no protocol-level changes.
2. A formal interoperability test exists in `prover/` that exercises every status code and header.
3. The CDN guidance has been validated against at least 3 distinct CDN providers (Cloudflare, Fastly, BunnyCDN).

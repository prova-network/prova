# 4.1 Network protocol

How provers talk to each other and to clients off-chain. This section is **draft** because we expect the conventions to evolve as we run the testnet at scale.

## 4.1.1 Transport

All Prova network traffic is **HTTPS over TCP**, no exceptions. We do not use libp2p, gRPC, or QUIC at v1. Provers MUST present a valid TLS 1.2+ certificate from a publicly-trusted CA. Self-signed certificates are not accepted.

Rationale: Prova clients are commodity software (browsers, curl, the CLI). HTTPS is the only transport every client speaks. We do not need pubsub or peer discovery; the on-chain registry IS the discovery layer.

## 4.1.2 Endpoint registration

A prover MUST register an HTTPS endpoint via `ProverRegistry.register(endpoint, features, capacity, region, attestation)`. The endpoint MUST:

- Resolve to a hostname under the prover's control
- Serve TLS 1.2 or 1.3
- Be reachable from at least one third-party probe (we run a small probe network and publish results)
- Respond to `GET /healthz` with `200 {"ok": true}` within 5 seconds

A prover MAY register multiple endpoints under the same registry entry by using comma-separated URLs. The first reachable URL is used by retrieval clients.

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
| `x-prova-verified` | `1` if the prover recomputed the CID at intake; `0` otherwise |
| `cache-control` | `public, max-age=3600` |
| `content-security-policy` | `default-src 'none'; sandbox` (for non-image/audio/video MIME types) |
| `x-content-type-options` | `nosniff` |
| `content-disposition` | `attachment; filename="{cid}"` for non-renderable types |
| `access-control-allow-origin` | `*` |

`HEAD /piece/{cid}` MUST return the same headers without a body.

Rate limiting MAY be applied per source IP. Provers SHOULD return `429` with a `Retry-After` header when rate-limited rather than dropping the connection.

## 4.1.4 Range requests

Retrieval MUST support HTTP range requests:

```
Range: bytes=0-1048575
```

The prover MUST respond with `206 Partial Content`, `Content-Range: bytes 0-1048575/{total}`, and the requested byte range.

Range requests are how SDKs stream large files without buffering the whole piece in memory.

## 4.1.5 Verification at the client

A retrieval client SHOULD recompute the piece-CID over the received bytes and compare to the requested CID. The CLI's `prova get` does this by default; the SDK exposes `verify: true` as a config option (default `true`).

If the recomputed CID does not match, the client MUST treat the response as invalid. The client MAY:

- Retry against another prover holding the same piece
- Submit a `markRetrievabilityFault` call once the off-chain dispute window opens

## 4.1.6 Prover-to-prover replication

When a deal is replicated across multiple provers (deal redundancy parameter > 1), one prover MAY pull the bytes from another prover holding the same piece, rather than requiring the client to upload N copies.

The pull request format:

```
GET https://{source-endpoint}/piece/{cid}?replicate-for={destination-prover-address}
```

The source prover MAY honor or refuse this request based on its own policy. There is NO protocol-level requirement to honor it; it's a courtesy that helps the network bootstrap.

## 4.1.7 Sponsored upload path

For the protocol's sponsored / free-tier uploads (browser drag-drop), the upload flow uses the centralized stage server at `p.prova.network`. The stage server's role is documented in [§4.2 API gateway](/api-gateway).

## 4.1.8 Open questions

- **CDN integration**: provers SHOULD be free to put a CDN in front of `/piece/{cid}` for retrieval performance. We have not specified how the CDN bypass affects `x-prova-verified` (the CDN won't have recomputed the CID). Currently we recommend setting `x-prova-verified: 1` only when the origin proxy verified.
- **WebTransport / HTTP/3**: a future amendment may permit HTTP/3 for retrieval. Not required at v1.
- **Reciprocal sampling protocol**: see [§2.3 Data availability](/data-availability).

// POST /api/upload?cid=<piece-cid>
// Stages bytes to R2. Two auth modes:
//   - Bearer pk_live_… token (CLI / SDK / app)
//   - No auth (browser drag-drop) -> sponsored tier with IP rate limit

import { authenticateRequest, checkQuota, recordUsage, type TokenEnv } from '../_shared/tokens';

interface Env extends TokenEnv {
  PROVA_PIECES?: R2Bucket;
  PROVA_RATE?: KVNamespace;
  PROVA_FILES?: KVNamespace;
  PROVA_TOKEN_SECRET?: string;

  // Stage server (Hetzner) used as a temporary R2 substitute.
  // When PROVA_STAGE_URL is set we PUT bytes there instead of (or in addition to) R2.
  PROVA_STAGE_URL?: string;
  PROVA_STAGE_KEY?: string;
}

const SPONSORED_FILE_LIMIT  = 100 * 1024 * 1024;   // 100 MB / file
const SPONSORED_DAILY_LIMIT = 200 * 1024 * 1024;   // 200 MB / IP / 24h
const AUTHED_FILE_LIMIT     = 5 * 1024 * 1024 * 1024;  // 5 GiB / file with token
const RETENTION_DAYS = 30;

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const { request: req, env } = ctx;
  if (req.method !== 'POST') return j({ error: 'method_not_allowed' }, 405);

  // Storage path: prefer R2 if bound, fall back to Hetzner stage server.
  if (!env.PROVA_PIECES && !env.PROVA_STAGE_URL) {
    return j({
      error: 'storage_offline',
      detail: 'Sponsored upload backend is being provisioned. Please try again in a few minutes, or use the SDK / CLI route.',
    }, 503);
  }

  const url = new URL(req.url);
  const cid = url.searchParams.get('cid');
  if (!cid || !/^[a-z0-9]{8,80}$/i.test(cid)) {
    return j({ error: 'invalid_cid', detail: 'cid query param is required' }, 400);
  }

  // Try bearer auth. If no token, fall back to sponsored mode.
  // F-02: only honor Authorization header; query-param tokens are gone.
  let authedUser: { id: string; email: string; quotaMb: number } | null = null;
  if (req.headers.get('authorization')) {
    const auth = await authenticateRequest(req, env);
    if (!auth.ok) return j({ error: auth.error, detail: auth.detail }, auth.status);
    if (!auth.payload.scopes.includes('put')) {
      return j({ error: 'insufficient_scope', detail: 'Token lacks put scope.' }, 403);
    }
    authedUser = { id: auth.payload.sub, email: auth.payload.email, quotaMb: auth.payload.quotaMb };
  }

  // Size cap depending on tier
  const limit = authedUser ? AUTHED_FILE_LIMIT : SPONSORED_FILE_LIMIT;
  const contentLength = parseInt(req.headers.get('content-length') || '0', 10);
  if (contentLength > limit) {
    return j({ error: 'too_large', detail: `Capped at ${limit / 1024 / 1024} MB on this tier.` }, 413);
  }

  if (!req.body) return j({ error: 'no_body' }, 400);

  const filename = req.headers.get('x-filename') || cid;
  const contentType = req.headers.get('content-type') || 'application/octet-stream';

  // Read and enforce hard size limit
  const buf = await readWithLimit(req.body, limit + 1);
  if (buf.byteLength > limit) {
    return j({ error: 'too_large', detail: `Capped at ${limit / 1024 / 1024} MB on this tier.` }, 413);
  }
  if (buf.byteLength === 0) return j({ error: 'empty_body' }, 400);

  // Daily quota check
  if (authedUser) {
    const auth = await authenticateRequest(req, env);
    if (auth.ok) {
      const q = await checkQuota(auth.payload, buf.byteLength, env);
      if (!q.ok) {
        return j({
          error: 'quota_exceeded',
          detail: `Daily quota of ${authedUser.quotaMb} MB reached.`,
          used: q.used,
          limit: q.limit,
        }, 429);
      }
    }
  } else {
    // Sponsored: per-IP daily cap
    const ip = req.headers.get('cf-connecting-ip') || 'unknown';
    const rateKey = `r:${ip}:${todayUTC()}`;
    if (env.PROVA_RATE) {
      const used = parseInt((await env.PROVA_RATE.get(rateKey)) || '0', 10);
      if (used + buf.byteLength > SPONSORED_DAILY_LIMIT) {
        return j({
          error: 'rate_limited',
          detail: `Sponsored tier capped at ${SPONSORED_DAILY_LIMIT / 1024 / 1024} MB / IP / 24h. Sign up for a free token to lift the cap.`,
        }, 429);
      }
      await env.PROVA_RATE.put(rateKey, String(used + buf.byteLength), { expirationTtl: 60 * 60 * 26 });
    }
  }

  // Stash bytes. R2 if bound, otherwise to Hetzner stage server.
  const safeName = sanitizeFilename(filename);
  if (env.PROVA_PIECES) {
    await env.PROVA_PIECES.put(`pieces/${cid}`, buf, {
      httpMetadata: {
        contentType,
        contentDisposition: `inline; filename="${safeName}"`,
      },
      customMetadata: {
        uploadedAt: new Date().toISOString(),
        ownerId: authedUser?.id || 'anon',
        ownerEmail: authedUser?.email || '',
        filename: safeName,
      },
    });
  } else if (env.PROVA_STAGE_URL && env.PROVA_STAGE_KEY) {
    const target = `${env.PROVA_STAGE_URL.replace(/\/+$/, '')}/pieces/${cid}`;
    const stageRes = await fetch(target, {
      method: 'PUT',
      headers: {
        'authorization': `Bearer ${env.PROVA_STAGE_KEY}`,
        'content-type': contentType,
        'content-length': String(buf.byteLength),
        'x-filename': safeName,
      },
      body: buf,
    });
    if (!stageRes.ok) {
      const text = await stageRes.text().catch(() => '');
      return j({
        error: 'stage_failed',
        detail: `stage server rejected upload (${stageRes.status}): ${text || stageRes.statusText}`,
      }, 502);
    }
  } else {
    return j({ error: 'storage_offline', detail: 'No storage backend bound.' }, 503);
  }

  // Record file ownership for /api/files listing
  if (authedUser && env.PROVA_FILES) {
    await env.PROVA_FILES.put(
      `f:${authedUser.id}:${cid}`,
      JSON.stringify({
        cid,
        size: buf.byteLength,
        filename: sanitizeFilename(filename),
        contentType,
        uploadedAt: new Date().toISOString(),
        term: `${RETENTION_DAYS} days`,
        sponsored: false,
      }),
      { expirationTtl: 60 * 60 * 24 * (RETENTION_DAYS + 1) }
    );
  }

  // Record per-user usage
  if (authedUser) {
    const auth = await authenticateRequest(req, env);
    if (auth.ok) await recordUsage(auth.payload, buf.byteLength, env);
  }

  const dealId = synthesizeDealId(cid, buf.byteLength);
  const retrievalUrl = `${url.origin}/p/${cid}`;

  return j({
    cid,
    dealId,
    size: buf.byteLength,
    retrievalUrl,
    term: `${RETENTION_DAYS} days`,
    sponsored: !authedUser,
    owner: authedUser?.email || null,
  });
};

function j(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });
}

function todayUTC(): string {
  return new Date().toISOString().slice(0, 10);
}

function sanitizeFilename(name: string): string {
  return decodeURIComponent(name).replace(/[^\w.\-]+/g, '_').slice(0, 120);
}

async function readWithLimit(body: ReadableStream<Uint8Array>, limit: number): Promise<Uint8Array> {
  const reader = body.getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;
  while (true) {
    const { value, done } = await reader.read();
    if (done) break;
    if (!value) continue;
    total += value.byteLength;
    if (total > limit) {
      try { await reader.cancel(); } catch { /* ignore */ }
      throw new Error('body too large');
    }
    chunks.push(value);
  }
  const out = new Uint8Array(total);
  let off = 0;
  for (const c of chunks) { out.set(c, off); off += c.byteLength; }
  return out;
}

function synthesizeDealId(cid: string, size: number): string {
  const hash = (cid + ':' + size).split('').reduce((a, c) => ((a << 5) - a + c.charCodeAt(0)) | 0, 0);
  return 'd-0x' + Math.abs(hash).toString(16).padStart(8, '0').slice(0, 8);
}

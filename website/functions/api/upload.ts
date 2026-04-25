// Cloudflare Pages Function: POST /api/upload?cid=<piece-cid>
// Body is the file bytes. Stages to R2, returns synthetic deal info.

interface Env {
  PROVA_PIECES?: R2Bucket;
  PROVA_RATE?: KVNamespace;
}

const FREE_LIMIT = 100 * 1024 * 1024;
const DAILY_LIMIT_PER_IP = 200 * 1024 * 1024;
const RETENTION_DAYS = 30;

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const { request: req, env } = ctx;
  if (req.method !== 'POST') return j({ error: 'method_not_allowed' }, 405);

  // R2 enablement gate. Until R2 is enabled on the account and bound,
  // we cannot store bytes. Surface a helpful message rather than 500ing.
  if (!env.PROVA_PIECES) {
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

  const ip = req.headers.get('cf-connecting-ip') || 'unknown';
  const rateKey = `r:${ip}:${todayUTC()}`;
  const used = env.PROVA_RATE
    ? parseInt((await env.PROVA_RATE.get(rateKey)) || '0', 10)
    : 0;
  if (used >= DAILY_LIMIT_PER_IP) {
    return j({
      error: 'rate_limited',
      detail: `Daily free quota of ${DAILY_LIMIT_PER_IP / 1024 / 1024} MB used. Try again tomorrow.`,
    }, 429);
  }

  const contentLength = parseInt(req.headers.get('content-length') || '0', 10);
  if (contentLength > FREE_LIMIT) {
    return j({ error: 'too_large', detail: `Free tier capped at ${FREE_LIMIT / 1024 / 1024} MB.` }, 413);
  }

  if (!req.body) return j({ error: 'no_body' }, 400);

  const filename = req.headers.get('x-filename') || cid;
  const contentType = req.headers.get('content-type') || 'application/octet-stream';

  const buf = await readWithLimit(req.body, FREE_LIMIT + 1);
  if (buf.byteLength > FREE_LIMIT) {
    return j({ error: 'too_large', detail: `Free tier capped at ${FREE_LIMIT / 1024 / 1024} MB.` }, 413);
  }
  if (buf.byteLength === 0) return j({ error: 'empty_body' }, 400);

  await env.PROVA_PIECES.put(`pieces/${cid}`, buf, {
    httpMetadata: {
      contentType,
      contentDisposition: `inline; filename="${sanitizeFilename(filename)}"`,
    },
    customMetadata: {
      uploadedAt: new Date().toISOString(),
      ip,
      filename: sanitizeFilename(filename),
    },
  });

  if (env.PROVA_RATE) {
    await env.PROVA_RATE.put(rateKey, String(used + buf.byteLength), { expirationTtl: 60 * 60 * 26 });
  }

  const dealId = synthesizeDealId(cid, buf.byteLength);
  const retrievalUrl = `${url.origin}/p/${cid}`;

  return j({
    cid,
    dealId,
    size: buf.byteLength,
    retrievalUrl,
    term: `${RETENTION_DAYS} days`,
    sponsored: true,
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

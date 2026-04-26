// Shared token helpers for Prova API.
//
// Design:
//   - User signs in with email (magic link) -> we mint a long-lived API token
//   - Tokens are HMAC-signed (HS256) with a server secret
//   - Token payload: { sub: userId, email, scopes, quotaMb, exp, jti }
//   - Tokens are also recorded in KV under tokens:<jti> for fast revocation
//   - Quota usage is tracked in KV under usage:<userId>:<yyyymmdd>
//
// For local/dev (PROVA_TOKEN_SECRET unset), we accept any token starting
// with `pk_test_` and grant a 100 MB / day demo quota.

export interface TokenPayload {
  sub: string;       // user id (uuid)
  email: string;
  scopes: string[];  // e.g. ['put', 'get', 'list']
  quotaMb: number;   // daily quota in MB
  iat: number;
  exp: number;
  jti: string;       // unique id, used for revocation
}

export interface TokenEnv {
  PROVA_RATE?: KVNamespace;
  PROVA_TOKENS?: KVNamespace;
  PROVA_TOKEN_SECRET?: string;
}

const TOKEN_PREFIX = 'pk_';

// ── Sign / verify (HS256, no external deps) ─────────────────────────────────
async function importHmacKey(secret: string): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign', 'verify']
  );
}

function b64urlEncode(bytes: Uint8Array): string {
  let s = btoa(String.fromCharCode(...bytes));
  return s.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function b64urlDecode(s: string): Uint8Array {
  const pad = '='.repeat((4 - (s.length % 4)) % 4);
  const bin = atob(s.replace(/-/g, '+').replace(/_/g, '/') + pad);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

export async function signToken(payload: TokenPayload, secret: string): Promise<string> {
  const header = { alg: 'HS256', typ: 'JWT' };
  const encHeader = b64urlEncode(new TextEncoder().encode(JSON.stringify(header)));
  const encPayload = b64urlEncode(new TextEncoder().encode(JSON.stringify(payload)));
  const data = `${encHeader}.${encPayload}`;
  const key = await importHmacKey(secret);
  const sig = new Uint8Array(await crypto.subtle.sign('HMAC', key, new TextEncoder().encode(data)));
  return `${TOKEN_PREFIX}live_${data}.${b64urlEncode(sig)}`;
}

export async function verifyToken(token: string, secret: string): Promise<TokenPayload | null> {
  if (!token.startsWith(TOKEN_PREFIX)) return null;
  // strip pk_live_ or pk_test_ prefix, but not the actual JWT
  const stripped = token.replace(/^pk_(live|test)_/, '');
  const parts = stripped.split('.');
  if (parts.length !== 3) return null;
  const [h, p, s] = parts;
  const data = `${h}.${p}`;
  try {
    const key = await importHmacKey(secret);
    const ok = await crypto.subtle.verify('HMAC', key, b64urlDecode(s), new TextEncoder().encode(data));
    if (!ok) return null;
    const payload = JSON.parse(new TextDecoder().decode(b64urlDecode(p))) as TokenPayload;
    if (payload.exp < Math.floor(Date.now() / 1000)) return null;
    return payload;
  } catch {
    return null;
  }
}

// ── Public: extract + validate the bearer token from a request ──────────────
export async function authenticateRequest(req: Request, env: TokenEnv): Promise<{
  ok: true; payload: TokenPayload;
} | {
  ok: false; status: number; error: string; detail: string;
}> {
  // F-02 fix: bearer header only. The old `?token=` query fallback leaked
  // tokens into Referer headers, server logs, browser history, and was a
  // CSRF amplifier. We deliberately do NOT consume url.searchParams.token
  // any more.
  const authHeader = req.headers.get('authorization') || '';
  const raw = authHeader.replace(/^Bearer\s+/i, '').trim();
  if (!raw) {
    return { ok: false, status: 401, error: 'no_token', detail: 'Authorization: Bearer pk_live_… header required.' };
  }

  // Test-mode token (used in CI / local dev when secret isn't set)
  if (raw.startsWith('pk_test_') && !env.PROVA_TOKEN_SECRET) {
    return {
      ok: true,
      payload: {
        sub: 'test-user',
        email: 'test@local',
        scopes: ['put', 'get', 'list'],
        quotaMb: 100,
        iat: Math.floor(Date.now() / 1000) - 10,
        exp: Math.floor(Date.now() / 1000) + 86400,
        jti: 'test-' + crypto.randomUUID(),
      },
    };
  }

  if (!env.PROVA_TOKEN_SECRET) {
    return { ok: false, status: 503, error: 'auth_offline', detail: 'Token signing key not configured.' };
  }
  const payload = await verifyToken(raw, env.PROVA_TOKEN_SECRET);
  if (!payload) {
    return { ok: false, status: 401, error: 'invalid_token', detail: 'Token is invalid, malformed, or expired.' };
  }

  // Check revocation
  if (env.PROVA_TOKENS) {
    const status = await env.PROVA_TOKENS.get(`tokens:${payload.jti}`);
    if (status === 'revoked') {
      return { ok: false, status: 401, error: 'revoked_token', detail: 'Token has been revoked.' };
    }
  }
  return { ok: true, payload };
}

// ── Quota helpers ───────────────────────────────────────────────────────────
export async function checkQuota(payload: TokenPayload, sizeBytes: number, env: TokenEnv): Promise<{
  ok: true;
} | {
  ok: false; used: number; limit: number;
}> {
  if (!env.PROVA_RATE) return { ok: true };
  const day = new Date().toISOString().slice(0, 10);
  const key = `usage:${payload.sub}:${day}`;
  const used = parseInt((await env.PROVA_RATE.get(key)) || '0', 10);
  const limitBytes = payload.quotaMb * 1024 * 1024;
  if (used + sizeBytes > limitBytes) {
    return { ok: false, used, limit: limitBytes };
  }
  return { ok: true };
}

export async function recordUsage(payload: TokenPayload, sizeBytes: number, env: TokenEnv): Promise<void> {
  if (!env.PROVA_RATE) return;
  const day = new Date().toISOString().slice(0, 10);
  const key = `usage:${payload.sub}:${day}`;
  const used = parseInt((await env.PROVA_RATE.get(key)) || '0', 10);
  await env.PROVA_RATE.put(key, String(used + sizeBytes), { expirationTtl: 60 * 60 * 26 * 7 }); // 7 days
}

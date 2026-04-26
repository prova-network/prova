// POST /api/auth/verify  { challenge?, email?, code?, label? }
//
// Step 2 of magic-link auth. Two paths land here:
//   - Browser link click: { challenge }
//   - CLI paste:          { email, code }
// On success, deletes the challenge from KV (one-time use), records the
// user, and mints a 1-year JWT. This is the API token the user uses
// from now on.

import { signToken, type TokenEnv, type TokenPayload } from '../../_shared/tokens';

interface Env extends TokenEnv {
  PROVA_USERS?: KVNamespace;
  PROVA_TOKEN_SECRET?: string;
}

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const { request: req, env } = ctx;
  if (req.method !== 'POST') return j({ error: 'method_not_allowed' }, 405);

  // Allow same-site OR no-origin (CLI calls). Browsers always send Origin.
  const origin = req.headers.get('origin');
  if (origin && !isProvaOrigin(origin)) {
    return j({ error: 'forbidden_origin' }, 403);
  }

  if (!env.PROVA_TOKEN_SECRET) {
    return j({ error: 'auth_offline', detail: 'PROVA_TOKEN_SECRET not configured.' }, 503);
  }

  let body: { challenge?: string; email?: string; code?: string; label?: string };
  try { body = await req.json(); } catch { return j({ error: 'invalid_json' }, 400); }

  let challengeKey = '';
  let entry: MagicEntry | null = null;

  if (body.challenge && /^[a-f0-9]{64}$/.test(body.challenge)) {
    challengeKey = `magic:${body.challenge}`;
    entry = await readEntry(env, challengeKey);
  } else if (body.email && body.code) {
    const email = body.email.trim().toLowerCase();
    const code = String(body.code).trim();
    if (!/^[0-9]{6}$/.test(code)) return j({ error: 'invalid_code' }, 400);
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) return j({ error: 'invalid_email' }, 400);
    if (!env.PROVA_USERS) return j({ error: 'storage_offline' }, 503);
    const challenge = await env.PROVA_USERS.get(`magic:e:${await sha256Hex(email)}:${code}`);
    if (!challenge) return j({ error: 'invalid_code' }, 401);
    challengeKey = `magic:${challenge}`;
    entry = await readEntry(env, challengeKey);
  } else {
    return j({ error: 'invalid_request' }, 400);
  }

  if (!entry) return j({ error: 'expired_or_unknown' }, 401);

  // Bump attempt counter; reject after 5 incorrect tries
  if (entry.attempts >= 5) {
    if (env.PROVA_USERS) await env.PROVA_USERS.delete(challengeKey);
    return j({ error: 'too_many_attempts' }, 401);
  }

  // Burn the challenge — single use
  if (env.PROVA_USERS) {
    await env.PROVA_USERS.delete(challengeKey);
    await env.PROVA_USERS.delete(`magic:e:${await sha256Hex(entry.email)}:${entry.code}`);
  }

  // Mint the long-lived API token
  const userId = (await sha256Hex(`prova:${entry.email}`)).slice(0, 16);
  const jti = crypto.randomUUID();
  const now = Math.floor(Date.now() / 1000);
  const payload: TokenPayload = {
    sub: userId,
    email: entry.email,
    scopes: ['put', 'get', 'list', 'pin'],
    quotaMb: 1024,
    iat: now,
    exp: now + 60 * 60 * 24 * 365,
    jti,
  };
  const token = await signToken(payload, env.PROVA_TOKEN_SECRET);

  // Persist user + token
  if (env.PROVA_USERS) {
    const existing = await env.PROVA_USERS.get(`u:${userId}`);
    if (!existing) {
      await env.PROVA_USERS.put(`u:${userId}`, JSON.stringify({
        email: entry.email,
        createdAt: new Date().toISOString(),
        verifiedAt: new Date().toISOString(),
      }));
    } else {
      const u = JSON.parse(existing);
      if (!u.verifiedAt) {
        u.verifiedAt = new Date().toISOString();
        await env.PROVA_USERS.put(`u:${userId}`, JSON.stringify(u));
      }
    }
    const label = sanitizeLabel(body.label || entry.label || 'web');
    await env.PROVA_USERS.put(`u:${userId}:t:${jti}`, JSON.stringify({
      label,
      createdAt: new Date().toISOString(),
      scopes: payload.scopes,
      quotaMb: payload.quotaMb,
      expiresAt: new Date(payload.exp * 1000).toISOString(),
    }));
  }

  return j({
    token,
    userId,
    email: entry.email,
    scopes: payload.scopes,
    quotaMb: payload.quotaMb,
    expiresAt: new Date(payload.exp * 1000).toISOString(),
    returnUrl: entry.returnUrl || '',
  });
};

interface MagicEntry {
  email: string;
  code: string;
  label: string;
  returnUrl: string;
  attempts: number;
  createdAt: number;
}

async function readEntry(env: Env, key: string): Promise<MagicEntry | null> {
  if (!env.PROVA_USERS) return null;
  const raw = await env.PROVA_USERS.get(key);
  if (!raw) return null;
  try {
    const e = JSON.parse(raw) as MagicEntry;
    return e;
  } catch { return null; }
}

function j(d: unknown, s = 200) {
  return new Response(JSON.stringify(d), {
    status: s,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });
}

function isProvaOrigin(s: string) {
  try {
    const u = new URL(s);
    return u.protocol === 'https:' && (u.hostname === 'prova.network' || u.hostname === 'www.prova.network' || u.hostname.endsWith('.prova-network.pages.dev'));
  } catch { return false; }
}

function sanitizeLabel(s: string) {
  return s.replace(/[^\w\-. ]/g, '').slice(0, 64) || 'web';
}

async function sha256Hex(s: string) {
  const h = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(s));
  return [...new Uint8Array(h)].map(b => b.toString(16).padStart(2, '0')).join('');
}

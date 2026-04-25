// POST /api/auth/signup  { email }
// Issues a Prova API token tied to the email. Pre-launch: no email verify
// step; we trust the address and let the token live for 1 year. After mainnet
// we'll add a magic-link verify before issuing.

import { signToken, type TokenEnv, type TokenPayload } from '../../_shared/tokens';

interface Env extends TokenEnv {
  PROVA_USERS?: KVNamespace;
  PROVA_TOKEN_SECRET?: string;
}

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const { request: req, env } = ctx;
  if (req.method !== 'POST') return j({ error: 'method_not_allowed' }, 405);

  if (!env.PROVA_TOKEN_SECRET) {
    return j({ error: 'auth_offline', detail: 'PROVA_TOKEN_SECRET not configured on Pages.' }, 503);
  }

  let body: { email?: string; label?: string };
  try {
    body = await req.json();
  } catch {
    return j({ error: 'invalid_json' }, 400);
  }
  const email = (body.email || '').trim().toLowerCase();
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
    return j({ error: 'invalid_email' }, 400);
  }

  // userId is a stable hash of the email (so re-signups land on the same record)
  const userId = await sha256Hex(`prova:${email}`);
  const jti = crypto.randomUUID();
  const now = Math.floor(Date.now() / 1000);
  const payload: TokenPayload = {
    sub: userId,
    email,
    scopes: ['put', 'get', 'list', 'pin'],
    quotaMb: 1024, // 1 GiB / day on free tier (we eat the cost as a loss-leader)
    iat: now,
    exp: now + 60 * 60 * 24 * 365, // 1 year
    jti,
  };

  const token = await signToken(payload, env.PROVA_TOKEN_SECRET);

  // Record user + token (best-effort)
  if (env.PROVA_USERS) {
    const existing = await env.PROVA_USERS.get(`u:${userId}`);
    if (!existing) {
      await env.PROVA_USERS.put(`u:${userId}`, JSON.stringify({
        email,
        createdAt: new Date().toISOString(),
      }));
    }
    const label = (body.label || 'cli').slice(0, 64);
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
    email,
    scopes: payload.scopes,
    quotaMb: payload.quotaMb,
    expiresAt: new Date(payload.exp * 1000).toISOString(),
  });
};

function j(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });
}

async function sha256Hex(s: string): Promise<string> {
  const h = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(s));
  return [...new Uint8Array(h)].map(b => b.toString(16).padStart(2, '0')).join('').slice(0, 16);
}

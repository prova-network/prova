// POST /api/tokens/revoke   { jti }   (auth: Bearer pk_live_…)
// Marks a token's jti as revoked. The auth middleware refuses revoked jtis.

import { authenticateRequest, type TokenEnv } from '../../_shared/tokens';

interface Env extends TokenEnv {
  PROVA_TOKENS?: KVNamespace;
  PROVA_USERS?: KVNamespace;
}

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const auth = await authenticateRequest(ctx.request, ctx.env);
  if (!auth.ok) return j({ error: auth.error, detail: auth.detail }, auth.status);

  if (ctx.request.method !== 'POST') return j({ error: 'method_not_allowed' }, 405);

  let body: { jti?: string };
  try {
    body = await ctx.request.json();
  } catch {
    return j({ error: 'invalid_json' }, 400);
  }
  const jti = (body.jti || '').trim();
  if (!jti) return j({ error: 'invalid_jti' }, 400);

  // Only allow revoking tokens you own
  if (ctx.env.PROVA_USERS) {
    const meta = await ctx.env.PROVA_USERS.get(`u:${auth.payload.sub}:t:${jti}`);
    if (!meta) return j({ error: 'not_found', detail: 'Token does not belong to this user.' }, 404);
  }

  if (!ctx.env.PROVA_TOKENS) {
    return j({ error: 'storage_offline', detail: 'PROVA_TOKENS not bound.' }, 503);
  }

  await ctx.env.PROVA_TOKENS.put(`tokens:${jti}`, 'revoked', {
    expirationTtl: 60 * 60 * 24 * 366,
  });
  return j({ jti, revoked: true });
};

function j(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });
}

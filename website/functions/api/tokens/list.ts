// GET /api/tokens/list   (auth: Bearer pk_live_…)
// Returns all tokens belonging to the authenticated user.

import { authenticateRequest, type TokenEnv } from '../../_shared/tokens';

interface Env extends TokenEnv {
  PROVA_USERS?: KVNamespace;
}

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const auth = await authenticateRequest(ctx.request, ctx.env);
  if (!auth.ok) return j({ error: auth.error, detail: auth.detail }, auth.status);
  const userId = auth.payload.sub;

  if (!ctx.env.PROVA_USERS) {
    return j({ tokens: [], note: 'PROVA_USERS not bound; token listing offline.' });
  }

  const list = await ctx.env.PROVA_USERS.list({ prefix: `u:${userId}:t:` });
  const tokens = [];
  for (const k of list.keys) {
    const raw = await ctx.env.PROVA_USERS.get(k.name);
    if (raw) {
      try {
        const meta = JSON.parse(raw);
        tokens.push({
          jti: k.name.split(':').pop(),
          ...meta,
          isCurrent: k.name.endsWith(`:${auth.payload.jti}`),
        });
      } catch { /* skip malformed */ }
    }
  }
  return j({ userId, email: auth.payload.email, tokens });
};

function j(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });
}

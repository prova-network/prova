// GET /api/files   (auth: Bearer pk_live_…)
// Lists all files owned by the authenticated user.

import { authenticateRequest, type TokenEnv } from '../_shared/tokens';

interface Env extends TokenEnv {
  PROVA_FILES?: KVNamespace;
}

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const auth = await authenticateRequest(ctx.request, ctx.env);
  if (!auth.ok) return j({ error: auth.error, detail: auth.detail }, auth.status);
  if (!auth.payload.scopes.includes('list')) {
    return j({ error: 'insufficient_scope', detail: 'Token lacks list scope.' }, 403);
  }

  if (!ctx.env.PROVA_FILES) {
    return j({ files: [], note: 'PROVA_FILES not bound; file index offline.' });
  }

  const list = await ctx.env.PROVA_FILES.list({ prefix: `f:${auth.payload.sub}:` });
  const files = [];
  for (const k of list.keys) {
    const raw = await ctx.env.PROVA_FILES.get(k.name);
    if (raw) {
      try {
        files.push(JSON.parse(raw));
      } catch { /* skip */ }
    }
  }
  return j({
    userId: auth.payload.sub,
    email: auth.payload.email,
    count: files.length,
    files: files.sort((a, b) => (b.uploadedAt || '').localeCompare(a.uploadedAt || '')),
  });
};

function j(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });
}

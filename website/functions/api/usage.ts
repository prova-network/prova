// GET /api/usage   (auth: Bearer pk_live_…)
// Returns the user's storage usage for today + last 7 days.

import { authenticateRequest, type TokenEnv } from '../_shared/tokens';

interface Env extends TokenEnv {
  PROVA_RATE?: KVNamespace;
}

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const auth = await authenticateRequest(ctx.request, ctx.env);
  if (!auth.ok) return j({ error: auth.error, detail: auth.detail }, auth.status);

  const userId = auth.payload.sub;
  const days: { date: string; bytes: number }[] = [];
  let totalBytes = 0;

  if (ctx.env.PROVA_RATE) {
    for (let i = 0; i < 7; i++) {
      const d = new Date();
      d.setUTCDate(d.getUTCDate() - i);
      const day = d.toISOString().slice(0, 10);
      const used = parseInt((await ctx.env.PROVA_RATE.get(`usage:${userId}:${day}`)) || '0', 10);
      days.push({ date: day, bytes: used });
      totalBytes += used;
    }
  }

  return j({
    userId,
    email: auth.payload.email,
    quotaMb: auth.payload.quotaMb,
    quotaBytes: auth.payload.quotaMb * 1024 * 1024,
    today: days[0],
    last7Days: days.reverse(),
    last7DaysTotalBytes: totalBytes,
  });
};

function j(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });
}

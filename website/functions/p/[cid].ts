// Cloudflare Pages Function: GET/HEAD /p/:cid
// Streams the stored piece bytes from R2.

interface Env {
  PROVA_PIECES?: R2Bucket;
  PROVA_STAGE_URL?: string;
}

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const { request: req, env, params } = ctx;
  const cid = params.cid as string;
  if (!cid || !/^[a-z0-9]{8,80}$/i.test(cid)) {
    return new Response('Invalid cid', { status: 400 });
  }
  // R2 path
  if (env.PROVA_PIECES) {
    const obj = await env.PROVA_PIECES.get(`pieces/${cid}`);
    if (!obj) return new Response('Not found', { status: 404 });
    return serveR2(req, obj, cid);
  }

  // Hetzner stage fallback
  if (env.PROVA_STAGE_URL) {
    const target = `${env.PROVA_STAGE_URL.replace(/\/+$/, '')}/p/${encodeURIComponent(cid)}`;
    const upstream = await fetch(target, {
      method: req.method,
      headers: { 'user-agent': 'prova-pages-fn/1' },
    });
    if (!upstream.ok) {
      return new Response(await upstream.text().catch(() => 'Upstream error'), {
        status: upstream.status,
      });
    }
    const headers = new Headers();
    for (const k of ['content-type', 'content-length', 'content-disposition', 'cache-control', 'x-prova-piece-cid']) {
      const v = upstream.headers.get(k);
      if (v) headers.set(k, v);
    }
    headers.set('x-prova-source', 'stage');
    return new Response(upstream.body, { headers });
  }

  return new Response('Storage offline (provisioning)', { status: 503 });

};

function serveR2(req: Request, obj: R2ObjectBody, cid: string): Response {
  if (req.method === 'HEAD') {
    const h = new Headers();
    h.set('content-length', String(obj.size));
    if (obj.httpMetadata?.contentType) h.set('content-type', obj.httpMetadata.contentType);
    return new Response(null, { headers: h });
  }
  const headers = new Headers();
  if (obj.httpMetadata?.contentType) headers.set('content-type', obj.httpMetadata.contentType);
  if (obj.httpMetadata?.contentDisposition) headers.set('content-disposition', obj.httpMetadata.contentDisposition);
  headers.set('cache-control', 'public, max-age=3600');
  headers.set('x-prova-piece-cid', cid);
  headers.set('x-prova-source', 'r2');
  return new Response(obj.body, { headers });
}

// Cloudflare Pages Function: GET/HEAD /p/:cid
// Streams the stored piece bytes from R2.

interface Env {
  PROVA_PIECES?: R2Bucket;
}

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const { request: req, env, params } = ctx;
  const cid = params.cid as string;
  if (!cid || !/^[a-z0-9]{8,80}$/i.test(cid)) {
    return new Response('Invalid cid', { status: 400 });
  }
  if (!env.PROVA_PIECES) {
    return new Response('Storage offline (provisioning)', { status: 503 });
  }
  const obj = await env.PROVA_PIECES.get(`pieces/${cid}`);
  if (!obj) return new Response('Not found', { status: 404 });

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
  return new Response(obj.body, { headers });
};

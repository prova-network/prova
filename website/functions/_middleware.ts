// Confidentiality lockdown. Re-enable per-host or per-path after Nicklas
// is clear of his Curio/PL/FilOz contract.

interface Env {}

const PUBLIC_PATHS = new Set<string>([
  // Marketing only. The hero/earth/diagrams ship was already public yesterday.
  '/',
  '/index.html',
  '/whitepaper.html',
  '/specs.html',
  '/styles.css',
  '/earth.js',
  '/lab.js',
  '/diagrams.js',
  '/upload.css',
  // Brand + assets needed to render the marketing pages.
]);

const PUBLIC_PREFIXES = ['/brand/', '/images/', '/models/', '/screenshots/'];

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const url = new URL(ctx.request.url);
  const host = url.hostname;
  const path = url.pathname;

  // Kill get.prova.network entirely.
  if (host === 'get.prova.network') {
    return text(503, '# Offline.\n');
  }

  // Block everything new we shipped today: /upload, /app, /api, /p/*.
  // The marketing site stays as it was yesterday.
  if (
    path.startsWith('/upload') ||
    path.startsWith('/app') ||
    path.startsWith('/api') ||
    path.startsWith('/p/')
  ) {
    return text(503, JSON.stringify({
      error: 'offline',
      detail: 'Endpoint paused for confidentiality. Will be re-enabled.',
    }, null, 2), 'application/json');
  }

  return ctx.next();
};

function text(status: number, body: string, ct = 'text/plain; charset=utf-8') {
  return new Response(body, {
    status,
    headers: { 'content-type': ct, 'cache-control': 'no-store' },
  });
}

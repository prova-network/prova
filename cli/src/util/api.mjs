// Tiny fetch wrapper for the Prova API.

import { DEFAULT_API } from './config.mjs';

export async function api(path, opts = {}, baseUrl = DEFAULT_API) {
  const headers = new Headers(opts.headers || {});
  if (opts.token) headers.set('authorization', `Bearer ${opts.token}`);
  if (opts.json && !headers.has('content-type')) {
    headers.set('content-type', 'application/json');
  }

  const res = await fetch(baseUrl + path, {
    method: opts.method || 'GET',
    headers,
    body: opts.json ? JSON.stringify(opts.json) : opts.body,
  });

  if (opts.raw) return res;

  const contentType = res.headers.get('content-type') || '';
  let body;
  if (contentType.includes('application/json')) {
    body = await res.json();
  } else {
    body = await res.text();
  }

  if (!res.ok) {
    const detail = (body && body.detail) || (body && body.error) || res.statusText;
    const err = new Error(`API ${path} -> ${res.status}: ${detail}`);
    err.status = res.status;
    err.body = body;
    throw err;
  }
  return body;
}

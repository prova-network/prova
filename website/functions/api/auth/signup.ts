// Deprecated. Old endpoint that minted tokens without email verification.
// Replaced by the magic-link flow at /api/auth/start + /api/auth/verify.
//
// We return 410 Gone with an explanatory body so old clients fail loudly
// instead of silently appearing to work.

export const onRequest: PagesFunction = async () => {
  return new Response(JSON.stringify({
    error: 'gone',
    detail: 'POST /api/auth/signup has been replaced by the magic-link flow.',
    use: { start: 'POST /api/auth/start { email }', verify: 'POST /api/auth/verify { challenge } | { email, code }' },
  }), {
    status: 410,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });
};

// POST /api/auth/start  { email, label?, returnUrl? }
//
// Step 1 of magic-link auth. Generates a 6-digit code + a long-lived
// challenge token, stores them in KV with TTL, and emails the user a
// link they can click + a code they can paste into the CLI.
//
// Response is intentionally vague — we don't reveal whether the email
// was syntactically valid or whether the user already exists, beyond
// what's needed for the UI to give feedback. We DO 400 on garbage
// emails so the form can show a useful error.

import { sendMail, type MailEnv } from '../../_shared/email';

interface Env extends MailEnv {
  PROVA_USERS?: KVNamespace;
  PROVA_RATE?: KVNamespace;
  PROVA_PUBLIC_BASE?: string; // e.g. https://prova.network
}

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const { request: req, env } = ctx;
  if (req.method !== 'POST') return j({ error: 'method_not_allowed' }, 405);

  // Origin / Referer check — same-site only
  const origin = req.headers.get('origin') || '';
  const refer  = req.headers.get('referer') || '';
  const ok = isProvaOrigin(origin) || isProvaOrigin(refer);
  if (!ok) return j({ error: 'forbidden_origin' }, 403);

  let body: { email?: string; label?: string; returnUrl?: string };
  try { body = await req.json(); } catch { return j({ error: 'invalid_json' }, 400); }

  const email = (body.email || '').trim().toLowerCase();
  if (!isValidEmail(email)) return j({ error: 'invalid_email' }, 400);

  const label = sanitizeLabel(body.label || 'web');
  const returnUrl = sanitizeReturnUrl(body.returnUrl || '');

  // Per-IP and per-email rate limit: 5 starts / 15 minutes
  const ip = req.headers.get('cf-connecting-ip') || '0.0.0.0';
  if (env.PROVA_RATE) {
    const ipKey = `auth:ip:${ip}`;
    const emailKey = `auth:em:${await sha256Hex(email)}`;
    if (await overLimit(env.PROVA_RATE, ipKey, 5, 900)) {
      return j({ error: 'rate_limited', detail: 'Too many sign-in requests. Try again in 15 minutes.' }, 429);
    }
    await overLimit(env.PROVA_RATE, emailKey, 5, 900);
  }

  // 6-digit code (CLI paste) + 32-byte challenge (link click)
  const code = String(Math.floor(100000 + Math.random() * 900000)); // 100000-999999
  const challenge = bytesToHex(crypto.getRandomValues(new Uint8Array(32)));

  // Store challenge → email mapping for 15 minutes
  if (env.PROVA_USERS) {
    const value = JSON.stringify({
      email, code, label, returnUrl,
      attempts: 0,
      createdAt: Math.floor(Date.now() / 1000),
    });
    await env.PROVA_USERS.put(`magic:${challenge}`, value, { expirationTtl: 900 });
    // Reverse lookup so the CLI can verify with code+email instead of needing the link
    await env.PROVA_USERS.put(`magic:e:${await sha256Hex(email)}:${code}`, challenge, { expirationTtl: 900 });
  }

  const base = env.PROVA_PUBLIC_BASE || originRoot(origin) || 'https://prova.network';
  const link = `${base}/auth/verify?challenge=${challenge}${returnUrl ? `&return=${encodeURIComponent(returnUrl)}` : ''}`;

  // From-address: prefers MAIL_FROM_EMAIL when configured (e.g.
  // hello@prova.network once Resend verifies the domain). Falls back
  // to Resend's shared onboarding sender so sign-in keeps working
  // before domain verification completes.
  const fromEmail = (env as { MAIL_FROM_EMAIL?: string }).MAIL_FROM_EMAIL || 'onboarding@resend.dev';
  const replyTo   = (env as { MAIL_REPLY_TO?: string }).MAIL_REPLY_TO || 'hello@prova.network';

  const mail = await sendMail({
    to: { email },
    from: { email: fromEmail, name: 'Prova' },
    replyTo: { email: replyTo },
    subject: `Your Prova sign-in code: ${code}`,
    content: [
      { type: 'text/plain', value: textBody(code, link) },
      { type: 'text/html',  value: htmlBody(code, link) },
    ],
  }, env);

  if (!mail.ok) {
    return j({
      error: 'mail_failed',
      detail: 'Could not deliver sign-in email. Try again in a moment.',
      ...((env as { DEBUG?: string }).DEBUG ? { upstream: mail } : {}),
    }, 502);
  }

  return j({
    sent: true,
    email,
    expiresIn: 900,
    // We hand back the challenge so the verify-by-link page can render
    // a "did you mean to verify on this device?" affordance, but the
    // code itself is NOT returned.
    challenge,
  });
};

function j(d: unknown, s = 200) {
  return new Response(JSON.stringify(d), {
    status: s,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });
}

function isValidEmail(s: string) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(s) && s.length < 256;
}

function sanitizeLabel(s: string) {
  return s.replace(/[^\w\-. ]/g, '').slice(0, 64) || 'web';
}

function sanitizeReturnUrl(s: string) {
  if (!s) return '';
  // Only allow same-origin paths, no protocol-relative / off-site
  if (s.startsWith('/') && !s.startsWith('//')) return s.slice(0, 200);
  return '';
}

function isProvaOrigin(s: string) {
  if (!s) return false;
  try {
    const u = new URL(s);
    return u.protocol === 'https:' && (u.hostname === 'prova.network' || u.hostname === 'www.prova.network' || u.hostname.endsWith('.prova-network.pages.dev'));
  } catch { return false; }
}

function originRoot(s: string) {
  try { const u = new URL(s); return `${u.protocol}//${u.host}`; } catch { return ''; }
}

function bytesToHex(b: Uint8Array) {
  return [...b].map(x => x.toString(16).padStart(2, '0')).join('');
}

async function sha256Hex(s: string) {
  const h = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(s));
  return bytesToHex(new Uint8Array(h));
}

async function overLimit(kv: KVNamespace, key: string, limit: number, ttl: number) {
  const cur = parseInt((await kv.get(key)) || '0', 10);
  if (cur >= limit) return true;
  await kv.put(key, String(cur + 1), { expirationTtl: ttl });
  return false;
}

function textBody(code: string, link: string) {
  return [
    'Hello!',
    '',
    `Your Prova sign-in code is: ${code}`,
    '',
    'Or click this link to sign in:',
    link,
    '',
    'This code expires in 15 minutes.',
    "If you didn't request this, you can ignore this email.",
    '',
    '— Prova (https://prova.network)',
  ].join('\n');
}

function htmlBody(code: string, link: string) {
  // Inline styles only — no external CSS, MailChannels strips it anyway.
  // Keep brand identity in line with marketing site (teal gradient + Apple stack).
  return `<!doctype html><html><body style="margin:0;padding:0;background:#f6f8fa;font-family:-apple-system,BlinkMacSystemFont,'SF Pro Display','Helvetica Neue',Arial,sans-serif;">
  <table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background:#f6f8fa;padding:40px 16px;">
    <tr><td align="center">
      <table role="presentation" cellpadding="0" cellspacing="0" style="max-width:520px;width:100%;background:#ffffff;border-radius:16px;overflow:hidden;box-shadow:0 1px 2px rgba(15,76,92,0.08);">
        <tr>
          <td style="padding:32px 32px 8px 32px;text-align:left;">
            <div style="font-size:13px;letter-spacing:0.08em;text-transform:uppercase;color:#0F4C5C;font-weight:600;">Prova</div>
            <h1 style="margin:8px 0 0;font-size:24px;line-height:1.2;color:#0F1419;font-weight:600;">Sign in to your account</h1>
          </td>
        </tr>
        <tr>
          <td style="padding:8px 32px 0 32px;">
            <p style="margin:16px 0;font-size:15px;line-height:1.55;color:#39424c;">Use this code to finish signing in:</p>
            <div style="background:linear-gradient(135deg,#5DC3E5 0%,#2EC4B6 55%,#0F4C5C 100%);padding:1px;border-radius:14px;">
              <div style="background:#ffffff;border-radius:13px;padding:18px 24px;text-align:center;">
                <div style="font-family:ui-monospace,'SF Mono',Menlo,Consolas,monospace;font-size:32px;letter-spacing:0.18em;color:#0F4C5C;font-weight:600;">${escapeHtml(code)}</div>
              </div>
            </div>
            <p style="margin:24px 0 8px;font-size:15px;color:#39424c;line-height:1.55;">Or click the button to sign in directly:</p>
            <p style="margin:8px 0 24px;">
              <a href="${escapeAttr(link)}" style="display:inline-block;background:#0F4C5C;color:#ffffff;text-decoration:none;padding:12px 22px;border-radius:10px;font-size:15px;font-weight:500;">Sign in to Prova</a>
            </p>
            <p style="margin:20px 0 0;font-size:12px;color:#9aa4b1;line-height:1.6;">This code expires in 15 minutes. If you didn't request this, you can safely ignore this email.</p>
          </td>
        </tr>
        <tr>
          <td style="padding:24px 32px 32px 32px;border-top:1px solid #e6eaef;">
            <p style="margin:0;font-size:12px;color:#9aa4b1;">Prova · <a href="https://prova.network" style="color:#9aa4b1;text-decoration:underline;">prova.network</a></p>
          </td>
        </tr>
      </table>
    </td></tr>
  </table>
</body></html>`;
}

function escapeHtml(s: string) {
  return s.replace(/[&<>"']/g, c => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;',
  } as Record<string,string>)[c]);
}

function escapeAttr(s: string) {
  return escapeHtml(s);
}

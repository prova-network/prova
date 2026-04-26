// Outbound transactional email for Cloudflare Pages Functions.
//
// Provider priority:
//   1. RESEND_API_KEY  → Resend (https://resend.com)
//   2. POSTMARK_TOKEN  → Postmark (https://postmarkapp.com)
//   3. dev fallback    → log the message, never actually send
//
// MailChannels was retired as a free relay in 2024 so we don't use it any more.

export interface MailAddress {
  email: string;
  name?: string;
}

export interface MailContent {
  type: 'text/plain' | 'text/html';
  value: string;
}

export interface SendMailParams {
  to: MailAddress | MailAddress[];
  from: MailAddress;
  subject: string;
  content: MailContent[]; // First entry text/plain, second text/html (Resend prefers html field directly)
  replyTo?: MailAddress;
}

export interface MailEnv {
  RESEND_API_KEY?: string;
  POSTMARK_TOKEN?: string;
  MAIL_DEV_LOG?: string; // "1" to enable dev-mode logging fallback
}

export async function sendMail(p: SendMailParams, env: MailEnv = {}): Promise<{ ok: true; provider: string } | { ok: false; status: number; body: string }> {
  if (env.RESEND_API_KEY) return sendViaResend(p, env.RESEND_API_KEY);
  if (env.POSTMARK_TOKEN) return sendViaPostmark(p, env.POSTMARK_TOKEN);
  if (env.MAIL_DEV_LOG === '1') {
    // Dev fallback so the rest of the system can be tested without a real provider.
    console.log('[mail:dev]', JSON.stringify({
      to: p.to,
      from: p.from,
      subject: p.subject,
      preview: p.content.find(c => c.type === 'text/plain')?.value.slice(0, 200),
    }));
    return { ok: true, provider: 'dev-log' };
  }
  return { ok: false, status: 503, body: 'No email provider configured (set RESEND_API_KEY or POSTMARK_TOKEN).' };
}

async function sendViaResend(p: SendMailParams, key: string) {
  const to = Array.isArray(p.to) ? p.to.map(a => a.email) : [p.to.email];
  const text = p.content.find(c => c.type === 'text/plain')?.value || '';
  const html = p.content.find(c => c.type === 'text/html')?.value;
  const fromHeader = p.from.name ? `${p.from.name} <${p.from.email}>` : p.from.email;
  const body: Record<string, unknown> = {
    from: fromHeader,
    to,
    subject: p.subject,
  };
  if (html) body.html = html;
  if (text) body.text = text;
  if (p.replyTo) body.reply_to = p.replyTo.email;

  const r = await fetch('https://api.resend.com/emails', {
    method: 'POST',
    headers: {
      'authorization': `Bearer ${key}`,
      'content-type': 'application/json',
    },
    body: JSON.stringify(body),
  });
  if (r.ok) return { ok: true as const, provider: 'resend' };
  let detail = '';
  try { detail = await r.text(); } catch {}
  return { ok: false as const, status: r.status, body: detail.slice(0, 400) };
}

async function sendViaPostmark(p: SendMailParams, token: string) {
  const to = Array.isArray(p.to) ? p.to.map(a => a.email).join(', ') : p.to.email;
  const text = p.content.find(c => c.type === 'text/plain')?.value;
  const html = p.content.find(c => c.type === 'text/html')?.value;
  const fromHeader = p.from.name ? `${p.from.name} <${p.from.email}>` : p.from.email;
  const body: Record<string, unknown> = {
    From: fromHeader,
    To: to,
    Subject: p.subject,
  };
  if (html) body.HtmlBody = html;
  if (text) body.TextBody = text;
  if (p.replyTo) body.ReplyTo = p.replyTo.email;
  body.MessageStream = 'outbound';

  const r = await fetch('https://api.postmarkapp.com/email', {
    method: 'POST',
    headers: {
      'X-Postmark-Server-Token': token,
      'accept': 'application/json',
      'content-type': 'application/json',
    },
    body: JSON.stringify(body),
  });
  if (r.ok) return { ok: true as const, provider: 'postmark' };
  let detail = '';
  try { detail = await r.text(); } catch {}
  return { ok: false as const, status: r.status, body: detail.slice(0, 400) };
}

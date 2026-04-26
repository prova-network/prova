// POST /api/abuse/report
//
// Public, unauthenticated endpoint for reporting abusive content (e.g.
// CSAM, malware, copyright violations). Anyone can file a report; we
// log it and queue it for review. We do NOT auto-ban based on a single
// report.
//
// Body: { cid?, url?, reason, contact? }
//
// All reports get an HMAC-derived ID returned so reporters can refer
// to their submission later.

interface Env {
  PROVA_RATE?: KVNamespace;
  PROVA_FILES?: KVNamespace;
  ABUSE_FORWARD_TO?: string; // optional Resend mail target
  RESEND_API_KEY?: string;
}

const MAX_REASON_LEN = 2000;

export const onRequest: PagesFunction<Env> = async (ctx) => {
  const { request: req, env } = ctx;
  if (req.method !== 'POST') {
    return j({ error: 'method_not_allowed' }, 405);
  }

  const ip = req.headers.get('cf-connecting-ip') || 'unknown';

  // Rate-limit reports per IP: 10/hour.
  if (env.PROVA_RATE) {
    const k = `abuse:ip:${ip}:${Math.floor(Date.now() / 3600000)}`;
    const cur = parseInt((await env.PROVA_RATE.get(k)) || '0', 10);
    if (cur >= 10) {
      return j({ error: 'rate_limited', detail: 'Too many reports from this address. Email hello@prova.network.' }, 429);
    }
    await env.PROVA_RATE.put(k, String(cur + 1), { expirationTtl: 7200 });
  }

  let body: { cid?: string; url?: string; reason?: string; contact?: string };
  try { body = await req.json(); } catch { return j({ error: 'invalid_json' }, 400); }

  const cid    = (body.cid    || '').trim().slice(0, 100);
  const target = (body.url    || '').trim().slice(0, 500);
  const reason = (body.reason || '').trim().slice(0, MAX_REASON_LEN);
  const contact = (body.contact || '').trim().slice(0, 200);
  if (!reason || reason.length < 20) {
    return j({ error: 'reason_required', detail: 'Please describe what is being reported (20+ characters).' }, 400);
  }
  if (!cid && !target) {
    return j({ error: 'target_required', detail: 'Specify either cid or url.' }, 400);
  }
  // Validate CID shape if present
  if (cid && !/^baga[a-z0-9]{4,76}$/i.test(cid) && !/^bafy[a-z0-9]{4,76}$/i.test(cid)) {
    return j({ error: 'invalid_cid' }, 400);
  }

  const reportId = await sha256Hex(`${Date.now()}:${ip}:${cid}:${target}`).then(s => s.slice(0, 16));
  const now = new Date().toISOString();
  const record = {
    reportId,
    submittedAt: now,
    reporterIp: ip,
    contact,
    cid: cid || null,
    url: target || null,
    reason,
    status: 'pending',
  };

  if (env.PROVA_FILES) {
    await env.PROVA_FILES.put(`abuse:${reportId}`, JSON.stringify(record), {
      expirationTtl: 60 * 60 * 24 * 365,
    });
  }

  // Forward to abuse@ inbox if Resend is configured
  if (env.RESEND_API_KEY) {
    const to = env.ABUSE_FORWARD_TO || 'hello@prova.network';
    try {
      await fetch('https://api.resend.com/emails', {
        method: 'POST',
        headers: {
          'authorization': `Bearer ${env.RESEND_API_KEY}`,
          'content-type': 'application/json',
        },
        body: JSON.stringify({
          from: 'Prova Abuse <hello@prova.network>',
          to: [to],
          subject: `[abuse-report] ${reportId} ${cid || target}`,
          text: [
            `Report ID: ${reportId}`,
            `Submitted: ${now}`,
            `Reporter IP: ${ip}`,
            `Reporter contact: ${contact || '(none)'}`,
            '',
            `CID: ${cid || '(n/a)'}`,
            `URL: ${target || '(n/a)'}`,
            '',
            'Reason:',
            reason,
          ].join('\n'),
        }),
      });
    } catch {
      // Don't fail the request if email forwarding fails
    }
  }

  return j({
    received: true,
    reportId,
    detail: 'Thanks. A human will review this. Severe issues are typically actioned within 24 hours.',
  });
};

function j(d: unknown, s = 200) {
  return new Response(JSON.stringify(d), {
    status: s,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });
}

async function sha256Hex(s: string) {
  const h = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(s));
  return [...new Uint8Array(h)].map(b => b.toString(16).padStart(2, '0')).join('');
}

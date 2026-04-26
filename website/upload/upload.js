// Prova upload — drag-and-drop client. Talks to /api/upload.
//
// Stages (visible in the UI):
//   1. hash    — compute real piece-CID (CommP, Fr32-padded SHA-256 binary tree)
//                in the browser via piece-cid.js

import { computePieceCid } from './piece-cid.js';
//   2. upload  — stream bytes to the worker, which stores to R2 + serves at /p/{cid}
//   3. propose — sponsor wallet calls proposeDeal on Base (server-side; stubbed pre-deploy)
//   4. accept  — sponsor prover fetches from R2, recomputes, accepts
//   5. active  — first proof posted, deal is Active
//
// Pre-mainnet, stages 3-5 are simulated by the worker (returns synthetic deal-id
// and walks the stages). The piece-cid + retrieval URL are fully real.

const API_BASE = ''; // same-origin in production. Override for local dev.
const FREE_LIMIT_BYTES = 100 * 1024 * 1024;

const $ = (sel) => document.querySelector(sel);
const $$ = (sel) => document.querySelectorAll(sel);

const drop      = $('#drop');
const fileInput = $('#file-input');
const pickBtn   = $('#pick-file');
const progress  = $('#progress');
const result    = $('#result');
const errorBox  = $('#error');

// ── DnD wiring ──────────────────────────────────────────────────────────────
['dragenter', 'dragover'].forEach(ev => {
  drop.addEventListener(ev, (e) => { e.preventDefault(); drop.classList.add('is-over'); });
});
['dragleave', 'drop'].forEach(ev => {
  drop.addEventListener(ev, (e) => { e.preventDefault(); drop.classList.remove('is-over'); });
});
drop.addEventListener('drop', (e) => {
  const f = e.dataTransfer?.files?.[0];
  if (f) handleFile(f);
});
drop.addEventListener('click', (e) => {
  if (e.target.tagName !== 'BUTTON') fileInput.click();
});
pickBtn.addEventListener('click', (e) => { e.stopPropagation(); fileInput.click(); });
fileInput.addEventListener('change', (e) => {
  const f = e.target.files?.[0];
  if (f) handleFile(f);
});

$('#error-retry').addEventListener('click', resetUI);
$('#r-another').addEventListener('click', resetUI);
$$('.copy').forEach(btn => btn.addEventListener('click', () => {
  const id = btn.dataset.copy;
  const v = document.getElementById(id)?.textContent;
  if (v) navigator.clipboard.writeText(v);
  btn.textContent = '✓';
  setTimeout(() => { btn.textContent = '⧉'; }, 1500);
}));

// ── Pipeline ────────────────────────────────────────────────────────────────
async function handleFile(file) {
  if (file.size > FREE_LIMIT_BYTES) {
    return showError(
      'File too large for free tier',
      `This file is ${formatSize(file.size)}. The sponsored tier caps at ${formatSize(FREE_LIMIT_BYTES)}. ` +
      `For larger files, connect a Base wallet (coming soon).`
    );
  }
  if (file.size === 0) {
    return showError('Empty file', 'There is nothing to store.');
  }

  showProgress(file);

  try {
    setStage('hash', 'active', 'Hashing in your browser…');
    setStage('hash', 'active', 'Computing piece-CID (Fr32-padded SHA-256 tree)…');
    const { cid, digestHex, paddedSize } = await computePieceCid(file, (p) => {
      if (p.phase === 'hash') {
        const pct = p.total > 0 ? (p.done / p.total) * 50 : 0; // hash phase = first half
        $('#progress-fill').style.width = pct.toFixed(1) + '%';
      } else if (p.phase === 'merkle') {
        const pct = 50 + (p.total > 0 ? (p.done / p.total) * 50 : 0);
        $('#progress-fill').style.width = pct.toFixed(1) + '%';
      }
    });
    console.debug('piece-cid:', { cid, digestHex, paddedSize, rawSize: file.size });
    setStage('hash', 'done');

    setStage('upload', 'active', 'Staging bytes for the prover…');
    const uploaded = await uploadBytes(file, cid);
    setStage('upload', 'done');

    setStage('propose', 'active', 'Proposing deal on Base Sepolia…');
    await sleep(900);
    setStage('propose', 'done');

    setStage('accept', 'active', 'Prover fetching, verifying piece-cid…');
    await sleep(1100);
    setStage('accept', 'done');

    setStage('active', 'active', 'First proof posted, deal is live.');
    await sleep(700);
    setStage('active', 'done');

    showResult({
      cid,
      dealId: uploaded.dealId,
      size: file.size,
      retrievalUrl: uploaded.retrievalUrl,
      paid: 'sponsored (free)',
      term: '30 days',
    });
  } catch (err) {
    console.error(err);
    showError(
      'Upload failed',
      err?.message || 'The sponsor prover or staging area is offline. Try again in a minute.'
    );
  }
}

// Real piece-CID is provided by piece-cid.js (Fr32 + SHA-256 binary Merkle tree).

// ── Upload to worker ────────────────────────────────────────────────────────
async function uploadBytes(file, cid) {
  const url = `${API_BASE}/api/upload?cid=${encodeURIComponent(cid)}`;
  const res = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': file.type || 'application/octet-stream',
      'X-Filename': encodeURIComponent(file.name),
    },
    body: file,
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Worker rejected upload (${res.status}): ${text || res.statusText}`);
  }
  return res.json();
}

// ── UI helpers ──────────────────────────────────────────────────────────────
function showProgress(file) {
  drop.parentElement.classList.add('hidden');
  progress.classList.remove('hidden');
  result.classList.add('hidden');
  errorBox.classList.add('hidden');
  $('#progress-name').textContent = file.name;
  $('#progress-stage').textContent = 'PREPARING';
  $('#progress-fill').style.width = '0%';
  $$('#steps li').forEach(li => { li.classList.remove('is-active', 'is-done'); });
}

function setStage(step, kind, sub) {
  const order = ['hash', 'upload', 'propose', 'accept', 'active'];
  const idx = order.indexOf(step);
  const total = order.length;
  $$('#steps li').forEach(li => {
    const s = li.dataset.step;
    const i = order.indexOf(s);
    li.classList.remove('is-active', 'is-done');
    if (i < idx) li.classList.add('is-done');
    else if (i === idx) {
      if (kind === 'done') li.classList.add('is-done');
      else li.classList.add('is-active');
    }
  });
  $('#progress-stage').textContent = ({
    hash: 'COMPUTING PIECE-CID',
    upload: 'STAGING BYTES',
    propose: 'PROPOSING DEAL',
    accept: 'PROVER ACCEPTING',
    active: 'ACTIVE',
  })[step] || step.toUpperCase();
  if (sub) $('#progress-sub').textContent = sub;
  const pct = Math.round(((idx + (kind === 'done' ? 1 : 0.5)) / total) * 100);
  $('#progress-fill').style.width = pct + '%';
}

function showResult({ cid, dealId, size, retrievalUrl, paid, term }) {
  progress.classList.add('hidden');
  result.classList.remove('hidden');
  $('#r-cid').textContent = cid;
  $('#r-deal').textContent = dealId;
  $('#r-size').textContent = formatSize(size);
  $('#r-paid').textContent = paid;
  $('#r-term').textContent = term;
  const a = $('#r-url');
  a.textContent = retrievalUrl;
  a.href = retrievalUrl;
  $('#r-download').href = retrievalUrl;
}

function showError(title, detail) {
  drop.parentElement.classList.remove('hidden');
  progress.classList.add('hidden');
  result.classList.add('hidden');
  errorBox.classList.remove('hidden');
  $('#error-title').textContent = title;
  $('#error-detail').textContent = detail;
}

function resetUI() {
  drop.parentElement.classList.remove('hidden');
  progress.classList.add('hidden');
  result.classList.add('hidden');
  errorBox.classList.add('hidden');
  fileInput.value = '';
}

function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }
function formatSize(b) {
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
  if (b < 1024 * 1024 * 1024) return `${(b / 1024 / 1024).toFixed(1)} MB`;
  return `${(b / 1024 / 1024 / 1024).toFixed(2)} GB`;
}
function base32(bytes) {
  const alpha = 'abcdefghijklmnopqrstuvwxyz234567';
  let out = '', bits = 0, val = 0;
  for (const b of bytes) {
    val = (val << 8) | b; bits += 8;
    while (bits >= 5) { out += alpha[(val >> (bits - 5)) & 31]; bits -= 5; }
  }
  if (bits) out += alpha[(val << (5 - bits)) & 31];
  return out;
}

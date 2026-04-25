// Prova app dashboard. Pure HTML+JS, no framework.

const $ = (s) => document.querySelector(s);
const $$ = (s) => document.querySelectorAll(s);

const STORAGE_KEY = 'prova.token';

function getToken() { return localStorage.getItem(STORAGE_KEY); }
function setToken(t) { localStorage.setItem(STORAGE_KEY, t); }
function clearToken() { localStorage.removeItem(STORAGE_KEY); }

function showSignin() {
  $('#signin').classList.remove('hidden');
  $('#app-body').classList.add('hidden');
  $('#who-email').textContent = '';
  $('#logout-btn').classList.add('hidden');
}

function showApp(email) {
  $('#signin').classList.add('hidden');
  $('#app-body').classList.remove('hidden');
  $('#who-email').textContent = email;
  $('#logout-btn').classList.remove('hidden');
}

async function api(path, opts = {}) {
  const token = getToken();
  const headers = new Headers(opts.headers || {});
  if (token) headers.set('authorization', 'Bearer ' + token);
  if (opts.json) headers.set('content-type', 'application/json');
  const res = await fetch(path, {
    method: opts.method || 'GET',
    headers,
    body: opts.json ? JSON.stringify(opts.json) : opts.body,
  });
  const ct = res.headers.get('content-type') || '';
  const body = ct.includes('application/json') ? await res.json() : await res.text();
  if (!res.ok) {
    const err = new Error((body && body.detail) || (body && body.error) || res.statusText);
    err.status = res.status;
    err.body = body;
    throw err;
  }
  return body;
}

// ── Sign-in flow ────────────────────────────────────────────────────────────
$('#signin-form').addEventListener('submit', async (e) => {
  e.preventDefault();
  const email = e.target.email.value.trim().toLowerCase();
  if (!email) return;
  const btn = e.target.querySelector('button[type=submit]');
  btn.disabled = true;
  btn.textContent = 'Minting…';
  try {
    const res = await api('/api/auth/signup', {
      method: 'POST',
      json: { email, label: 'web-app' },
    });
    setToken(res.token);

    // Show the freshly-minted token once
    $('#full-token').textContent = res.token;
    $('#full-token-snippet').textContent = res.token;
    $('#current-token-card').classList.remove('hidden');

    showApp(res.email);
    await refreshAll();
  } catch (err) {
    btn.disabled = false;
    btn.textContent = 'Mint my token';
    alert('Sign-in failed: ' + err.message);
  }
});

$('#logout-btn').addEventListener('click', () => {
  if (!confirm('Sign out? This clears the token from this browser. Your tokens stay valid until revoked.')) return;
  clearToken();
  location.reload();
});

$('#copy-token-btn').addEventListener('click', () => {
  const v = $('#full-token').textContent;
  navigator.clipboard.writeText(v);
  $('#copy-token-btn').textContent = '✓ Copied';
  setTimeout(() => { $('#copy-token-btn').textContent = 'Copy'; }, 1500);
});

// ── Data refresh ────────────────────────────────────────────────────────────
async function refreshAll() {
  await Promise.all([refreshUsage(), refreshFiles(), refreshTokens()]);
}

async function refreshUsage() {
  try {
    const u = await api('/api/usage');
    const today = u.today?.bytes || 0;
    const quota = u.quotaBytes;
    $('#stat-today').textContent = `${formatSize(today)} / ${formatSize(quota)}`;
    $('#quota-fill').style.width = Math.min(100, (today / quota) * 100) + '%';

    // Render last 7 days as bar chart
    const max = Math.max(...u.last7Days.map(d => d.bytes), 1);
    const chart = $('#usage-chart');
    chart.innerHTML = '';
    for (const d of u.last7Days) {
      const pct = (d.bytes / max) * 100;
      const day = new Date(d.date).toLocaleDateString(undefined, { weekday: 'short' });
      const div = document.createElement('div');
      div.className = 'usage-bar';
      div.innerHTML = `
        <div class="usage-bar-val">${d.bytes ? formatSize(d.bytes) : '—'}</div>
        <div class="usage-bar-fill" style="height: ${pct}%"></div>
        <div class="usage-bar-day">${day}</div>
      `;
      chart.appendChild(div);
    }
  } catch (err) {
    console.error('usage failed:', err);
  }
}

async function refreshFiles() {
  try {
    const r = await api('/api/files');
    $('#stat-files').textContent = r.count;
    const list = $('#files-list');
    list.innerHTML = '';
    if (!r.files.length) {
      list.innerHTML = '<p class="empty">No files yet. <a href="../upload/" style="color:var(--accent);">Upload one</a>.</p>';
      return;
    }
    for (const f of r.files) {
      const row = document.createElement('div');
      row.className = 'file-row';
      row.innerHTML = `
        <span class="file-name" title="${escapeHtml(f.filename)}">${escapeHtml(f.filename)}</span>
        <span class="file-cid" title="${f.cid}">${f.cid.slice(0, 14)}…${f.cid.slice(-4)}</span>
        <span class="file-size">${formatSize(f.size)}</span>
        <span class="file-actions">
          <a href="/p/${f.cid}" target="_blank" rel="noopener">retrieve</a>
        </span>
      `;
      list.appendChild(row);
    }
  } catch (err) {
    console.error('files failed:', err);
  }
}

async function refreshTokens() {
  try {
    const r = await api('/api/tokens/list');
    $('#stat-tokens').textContent = r.tokens.length;
    const list = $('#tokens-list');
    list.innerHTML = '';
    if (!r.tokens.length) {
      list.innerHTML = '<p class="empty">No tokens yet.</p>';
      return;
    }
    for (const t of r.tokens) {
      const row = document.createElement('div');
      row.className = 'token-row';
      const created = new Date(t.createdAt).toLocaleDateString();
      const expires = new Date(t.expiresAt).toLocaleDateString();
      row.innerHTML = `
        <span class="token-label">${escapeHtml(t.label || 'unnamed')}</span>
        <span class="token-meta">created ${created} · expires ${expires}</span>
        <span class="token-actions">
          ${t.isCurrent
            ? '<button class="is-current" disabled>current</button>'
            : `<button data-jti="${t.jti}" class="revoke-btn">Revoke</button>`}
        </span>
      `;
      list.appendChild(row);
    }
    $$('.revoke-btn').forEach(b => b.addEventListener('click', async (e) => {
      const jti = e.target.dataset.jti;
      if (!confirm('Revoke this token? Apps using it will be signed out.')) return;
      e.target.disabled = true; e.target.textContent = '…';
      try {
        await api('/api/tokens/revoke', { method: 'POST', json: { jti } });
        await refreshTokens();
      } catch (err) {
        alert('Revoke failed: ' + err.message);
        e.target.disabled = false; e.target.textContent = 'Revoke';
      }
    }));
  } catch (err) {
    console.error('tokens failed:', err);
  }
}

$('#new-token-btn').addEventListener('click', async () => {
  const email = $('#who-email').textContent;
  if (!email) return alert('Sign in first.');
  const label = prompt('Label for the new token (e.g. "ci", "laptop"):', 'cli') || 'unlabeled';
  try {
    const res = await api('/api/auth/signup', {
      method: 'POST',
      json: { email, label },
    });
    $('#full-token').textContent = res.token;
    $('#full-token-snippet').textContent = res.token;
    $('#current-token-card').classList.remove('hidden');
    await refreshTokens();
  } catch (err) {
    alert('Token mint failed: ' + err.message);
  }
});

// ── Boot ────────────────────────────────────────────────────────────────────
async function boot() {
  if (!getToken()) {
    showSignin();
    return;
  }
  try {
    const u = await api('/api/usage');
    showApp(u.email);
    await refreshAll();
  } catch (err) {
    if (err.status === 401) {
      clearToken();
      showSignin();
    } else {
      alert('Could not load app: ' + err.message);
    }
  }
}
boot();

// ── Helpers ─────────────────────────────────────────────────────────────────
function formatSize(b) {
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
  if (b < 1024 * 1024 * 1024) return `${(b / 1024 / 1024).toFixed(1)} MB`;
  return `${(b / 1024 / 1024 / 1024).toFixed(2)} GB`;
}
function escapeHtml(s) {
  return String(s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
}

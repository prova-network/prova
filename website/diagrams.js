// Prova — animated, narrated diagrams.
// Replace the static brand SVGs with inline SVG that walks the user through
// the actual flow: a deal as a packet, moving through the system.

(() => {
  const $ = (s, root = document) => root.querySelector(s);
  const ACCENT_DARK  = '#5dc3e5';
  const ACCENT_LIGHT = '#2c6f8a';
  const accent = () => document.body.dataset.theme === 'light' ? ACCENT_LIGHT : ACCENT_DARK;
  const ink    = () => getComputedStyle(document.body).getPropertyValue('--type-loud').trim();
  const inkSoft= () => getComputedStyle(document.body).getPropertyValue('--type-soft').trim();
  const stroke = () => getComputedStyle(document.body).getPropertyValue('--diag-stroke').trim();
  const bg     = () => getComputedStyle(document.body).getPropertyValue('--diag-bg').trim();

  // ════════════════════════════════════════════════════════════════════════
  // ARCHITECTURE — Client → Prover → Ethereum, live
  // Walks: 1) client uploads piece → 2) prover stores → 3) prover proves →
  //        4) Ethereum verifies → 5) USDC streams back to prover.
  // ════════════════════════════════════════════════════════════════════════
  function renderArchitecture() {
    const host = $('#diag-architecture');
    if (!host) return;
    host.innerHTML = `
      <svg viewBox="0 0 760 360" preserveAspectRatio="xMidYMid meet" role="img"
           aria-label="Animated Prova architecture: client uploads, prover stores and proves, Ethereum settles">
        <defs>
          <marker id="ar-head" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="8" markerHeight="8" orient="auto">
            <path d="M0 0 L10 5 L0 10 Z" fill="${inkSoft()}" opacity="0.85"/>
          </marker>
          <radialGradient id="ar-pulse" cx="0.5" cy="0.5" r="0.5">
            <stop offset="0%"  stop-color="${accent()}" stop-opacity="0.95"/>
            <stop offset="60%" stop-color="${accent()}" stop-opacity="0.35"/>
            <stop offset="100%" stop-color="${accent()}" stop-opacity="0"/>
          </radialGradient>
        </defs>

        <!-- ── Boxes ─────────────────────────────────────────────── -->
        <g id="ar-boxes">
          <g class="ar-box" transform="translate(20, 60)">
            <rect width="180" height="120" rx="14" fill="none" stroke="${stroke()}" stroke-width="1.2"/>
            <text x="90" y="46" text-anchor="middle" fill="${ink()}" font-family="-apple-system,sans-serif" font-size="18" font-weight="500">Client</text>
            <text x="90" y="72" text-anchor="middle" fill="${inkSoft()}" font-family="ui-monospace,Menlo,monospace" font-size="11" letter-spacing="0.08em">UPLOAD · PAY · VERIFY</text>
            <text x="90" y="100" text-anchor="middle" fill="${inkSoft()}" font-family="-apple-system,sans-serif" font-size="12">.eth website · dataset</text>
          </g>
          <g class="ar-box" transform="translate(290, 60)">
            <rect width="180" height="120" rx="14" fill="none" stroke="${stroke()}" stroke-width="1.2"/>
            <text x="90" y="46" text-anchor="middle" fill="${ink()}" font-family="-apple-system,sans-serif" font-size="18" font-weight="500">Prover</text>
            <text x="90" y="72" text-anchor="middle" fill="${inkSoft()}" font-family="ui-monospace,Menlo,monospace" font-size="11" letter-spacing="0.08em">STAKE · STORE · PROVE</text>
            <text x="90" y="100" text-anchor="middle" fill="${inkSoft()}" font-family="-apple-system,sans-serif" font-size="12">disk + bandwidth + bond</text>
          </g>
          <g class="ar-box" transform="translate(560, 60)">
            <rect width="180" height="120" rx="14" fill="none" stroke="${stroke()}" stroke-width="1.2"/>
            <text x="90" y="46" text-anchor="middle" fill="${ink()}" font-family="-apple-system,sans-serif" font-size="18" font-weight="500">Ethereum</text>
            <text x="90" y="72" text-anchor="middle" fill="${inkSoft()}" font-family="ui-monospace,Menlo,monospace" font-size="11" letter-spacing="0.08em">VERIFY · SLASH · PAY</text>
            <text x="90" y="100" text-anchor="middle" fill="${inkSoft()}" font-family="-apple-system,sans-serif" font-size="12">Base L2 settlement</text>
          </g>
        </g>

        <!-- ── Static skeleton arrows ────────────────────────────── -->
        <path d="M200 120 L290 120" stroke="${stroke()}" stroke-width="1.4" fill="none" marker-end="url(#ar-head)" opacity="0.5"/>
        <path d="M470 120 L560 120" stroke="${stroke()}" stroke-width="1.4" fill="none" marker-end="url(#ar-head)" opacity="0.5"/>
        <path d="M650 180 Q650 240 470 240 L290 240 Q200 240 200 180" stroke="${stroke()}" stroke-width="1.4" fill="none" stroke-dasharray="3 4" opacity="0.45"/>

        <!-- ── Step labels ───────────────────────────────────────── -->
        <g font-family="ui-monospace,Menlo,monospace" font-size="10" letter-spacing="0.16em" fill="${inkSoft()}">
          <text x="245" y="106" text-anchor="middle">1. UPLOAD</text>
          <text x="515" y="106" text-anchor="middle">2. PROOF</text>
          <text x="380" y="232" text-anchor="middle">3. SETTLEMENT · USDC PAYOUT</text>
        </g>

        <!-- ── Live packet (animated along the path) ─────────────── -->
        <g id="ar-packet">
          <circle r="22" fill="url(#ar-pulse)"/>
          <circle r="6" fill="${accent()}"/>
          <text y="-22" text-anchor="middle" fill="${ink()}" font-family="ui-monospace,Menlo,monospace" font-size="10" letter-spacing="0.12em" id="ar-packet-label">PIECE</text>
        </g>
      </svg>
    `;

    const packet = host.querySelector('#ar-packet');
    const label  = host.querySelector('#ar-packet-label');

    // Define the path as a series of (x, y, label) waypoints for the packet
    const STAGES = [
      { from: [200, 120], to: [290, 120], dur: 1500, label: 'PIECE' },         // upload
      { from: [290, 120], to: [380, 120], dur: 800,  label: 'STORE' },         // settle in prover
      { from: [380, 120], to: [470, 120], dur: 800,  label: 'STORE' },
      { from: [470, 120], to: [560, 120], dur: 1500, label: 'PROOF' },         // prove
      { from: [560, 180], to: [650, 240], dur: 800,  label: 'VERIFY' },        // verify (Ethereum down arc)
      { from: [650, 240], to: [380, 240], dur: 1500, label: 'SETTLE' },        // settle along bottom
      { from: [380, 240], to: [200, 240], dur: 1500, label: 'USDC' },          // payout
      { from: [200, 240], to: [200, 180], dur: 600,  label: '' },              // tuck back into client
    ];

    let stageIdx = 0;
    let stageStart = performance.now();
    let raf;

    function tick(now) {
      const stage = STAGES[stageIdx];
      const t = Math.min(1, (now - stageStart) / stage.dur);
      const eased = t < 0.5 ? 2*t*t : 1 - Math.pow(-2*t+2, 2)/2;
      const x = stage.from[0] + (stage.to[0] - stage.from[0]) * eased;
      const y = stage.from[1] + (stage.to[1] - stage.from[1]) * eased;
      packet.setAttribute('transform', `translate(${x}, ${y})`);
      if (stage.label) label.textContent = stage.label;
      packet.style.opacity = stage.label === '' ? Math.max(0, 1 - t * 1.4) : 1;

      if (t >= 1) {
        stageIdx = (stageIdx + 1) % STAGES.length;
        stageStart = now + (stageIdx === 0 ? 600 : 80);
        if (stageIdx === 0) packet.style.opacity = 1;
      }
      raf = requestAnimationFrame(tick);
    }
    cancelAnimationFrame(raf);
    raf = requestAnimationFrame(tick);
  }

  // ════════════════════════════════════════════════════════════════════════
  // LIFECYCLE — deal moves through 5 states; current state pulses
  // ════════════════════════════════════════════════════════════════════════
  function renderLifecycle() {
    const host = $('#diag-lifecycle');
    if (!host) return;

    const STATES = [
      { key: 'proposed',    label: 'Proposed',    sub: 'on-chain' },
      { key: 'downloading', label: 'Downloading', sub: 'prover pulls' },
      { key: 'storing',     label: 'Storing',     sub: 'piece-cid' },
      { key: 'active',      label: 'Active',      sub: 'proof every 30 s' },
      { key: 'settled',     label: 'Settled',     sub: 'finalized' },
    ];
    const W = 760, H = 200;
    const boxW = 132, boxH = 80;
    const gap = (W - STATES.length * boxW) / (STATES.length + 1);

    let svg = `
      <svg viewBox="0 0 ${W} ${H}" preserveAspectRatio="xMidYMid meet" role="img"
           aria-label="Deal lifecycle moving from Proposed to Settled">
        <defs>
          <marker id="lc-head" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="8" markerHeight="8" orient="auto">
            <path d="M0 0 L10 5 L0 10 Z" fill="${inkSoft()}" opacity="0.85"/>
          </marker>
        </defs>
    `;
    for (let i = 0; i < STATES.length; i++) {
      const x = gap + i * (boxW + gap);
      const cy = H / 2;
      svg += `
        <g class="lc-box" data-key="${STATES[i].key}" transform="translate(${x}, ${cy - boxH/2})">
          <rect class="lc-box-bg" width="${boxW}" height="${boxH}" rx="12"
                fill="none" stroke="${stroke()}" stroke-width="1.2"/>
          <text x="${boxW/2}" y="34" text-anchor="middle"
                fill="${ink()}" font-family="-apple-system,sans-serif" font-size="16" font-weight="500">${STATES[i].label}</text>
          <text x="${boxW/2}" y="56" text-anchor="middle"
                fill="${inkSoft()}" font-family="ui-monospace,Menlo,monospace" font-size="11" letter-spacing="0.08em">${STATES[i].sub.toUpperCase()}</text>
        </g>
      `;
      if (i < STATES.length - 1) {
        const ax = x + boxW;
        const bx = x + boxW + gap;
        svg += `<path d="M${ax} ${cy} L${bx - 2} ${cy}" stroke="${stroke()}" stroke-width="1.4" fill="none" marker-end="url(#lc-head)" opacity="0.65"/>`;
      }
    }
    svg += `</svg>`;
    host.innerHTML = svg;

    let active = 0;
    function step() {
      host.querySelectorAll('.lc-box-bg').forEach((rect, i) => {
        if (i === active) {
          rect.setAttribute('stroke', accent());
          rect.setAttribute('stroke-width', '2.2');
          rect.setAttribute('fill', 'rgba(93,195,229,0.06)');
        } else {
          rect.setAttribute('stroke', stroke());
          rect.setAttribute('stroke-width', '1.2');
          rect.setAttribute('fill', 'none');
        }
      });
      active = (active + 1) % STATES.length;
      setTimeout(step, 1400);
    }
    step();
  }

  // ════════════════════════════════════════════════════════════════════════
  // CLIENT FLOWS — three small animated cards: .eth website, dataset, AI corpus
  // ════════════════════════════════════════════════════════════════════════
  function renderClientFlows() {
    const host = $('#client-flows');
    if (!host) return;
    host.innerHTML = `
      <div class="cf-card">
        <div class="cf-icon">
          <svg viewBox="0 0 64 64" fill="none">
            <rect x="6" y="14" width="52" height="40" rx="3" stroke="currentColor" stroke-width="2"/>
            <circle cx="14" cy="22" r="1.5" fill="currentColor"/>
            <circle cx="20" cy="22" r="1.5" fill="currentColor"/>
            <circle cx="26" cy="22" r="1.5" fill="currentColor"/>
            <line x1="6" y1="28" x2="58" y2="28" stroke="currentColor" stroke-width="1.2"/>
            <text x="32" y="46" text-anchor="middle" fill="currentColor" font-family="-apple-system,sans-serif" font-size="12" font-weight="500">vitalik.eth</text>
          </svg>
        </div>
        <div class="cf-meta">.eth website</div>
        <h4>Host a censorship-resistant site</h4>
        <p>Build a static site, drop the bundle into the Prova CLI, get back a <code>piece-cid</code>. Point your ENS contenthash at it. Now your site lives across Prova provers, served over HTTPS, proven daily.</p>
        <pre><code>npx prova upload ./dist
ens set-content vitalik.eth ipfs://&lt;cid&gt;</code></pre>
      </div>

      <div class="cf-card">
        <div class="cf-icon">
          <svg viewBox="0 0 64 64" fill="none">
            <ellipse cx="32" cy="14" rx="22" ry="6" stroke="currentColor" stroke-width="2"/>
            <path d="M10 14 L10 50 Q10 56 32 56 Q54 56 54 50 L54 14" stroke="currentColor" stroke-width="2" fill="none"/>
            <path d="M10 26 Q10 32 32 32 Q54 32 54 26" stroke="currentColor" stroke-width="1.4" fill="none" opacity="0.65"/>
            <path d="M10 38 Q10 44 32 44 Q54 44 54 38" stroke="currentColor" stroke-width="1.4" fill="none" opacity="0.45"/>
          </svg>
        </div>
        <div class="cf-meta">Datasets</div>
        <h4>Pin scientific or research data</h4>
        <p>Upload a 200&nbsp;GB CSV, a Parquet shard, an open-data archive. Prova handles chunking, redundancy, and proof. Pay per TiB-month in USDC. Retrieve over HTTPS or libp2p.</p>
        <pre><code>prova put ./genome.parquet \\
  --redundancy 4 --term 2y</code></pre>
      </div>

      <div class="cf-card">
        <div class="cf-icon">
          <svg viewBox="0 0 64 64" fill="none">
            <circle cx="32" cy="32" r="22" stroke="currentColor" stroke-width="2"/>
            <circle cx="32" cy="32" r="3" fill="currentColor"/>
            <path d="M14 32 L24 32 M40 32 L50 32 M32 14 L32 24 M32 40 L32 50" stroke="currentColor" stroke-width="1.4"/>
            <path d="M22 22 L26 26 M38 38 L42 42 M22 42 L26 38 M38 26 L42 22" stroke="currentColor" stroke-width="1.2" opacity="0.65"/>
          </svg>
        </div>
        <div class="cf-meta">AI corpora</div>
        <h4>Anchor model weights and training data</h4>
        <p>Make the corpus auditable. Anyone can verify the file you trained on is the file you said. Useful for open weights, dataset provenance, and reproducibility.</p>
        <pre><code>prova put ./weights.safetensors \\
  --label "llama-derivative-v3"</code></pre>
      </div>
    `;
  }

  // ════════════════════════════════════════════════════════════════════════
  // Boot + theme reactivity
  // ════════════════════════════════════════════════════════════════════════
  function renderAll() {
    renderArchitecture();
    renderLifecycle();
    renderClientFlows();
  }
  renderAll();
  window.__renderDiagrams = renderAll;
})();

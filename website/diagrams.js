// Prova - animated narrated diagrams.
// Inline SVG that walks the user through the actual flow:
// a deal as a packet, moving through the system.

(() => {
  const $ = (s, root = document) => root.querySelector(s);
  const ACCENT_DARK  = '#5dc3e5';
  const ACCENT_LIGHT = '#2c6f8a';
  const accent = () => document.body.dataset.theme === 'light' ? ACCENT_LIGHT : ACCENT_DARK;
  const ink    = () => getComputedStyle(document.body).getPropertyValue('--type-loud').trim();
  const inkSoft= () => getComputedStyle(document.body).getPropertyValue('--type-soft').trim();
  const stroke = () => getComputedStyle(document.body).getPropertyValue('--diag-stroke').trim();
  const bg     = () => getComputedStyle(document.body).getPropertyValue('--bg').trim();

  // ============================================================
  // ARCHITECTURE: Client -> Prover -> Ethereum, live packet
  // ============================================================
  function renderArchitecture() {
    const host = $('#diag-architecture');
    if (!host) return;
    host.innerHTML = `
      <svg viewBox="0 0 760 320" preserveAspectRatio="xMidYMid meet" role="img"
           aria-label="Animated Prova architecture: client uploads, prover proves, Ethereum settles">
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

        <!-- Three role boxes -->
        <g class="ar-box" transform="translate(20, 60)">
          <rect width="180" height="120" rx="14" fill="none" stroke="${stroke()}" stroke-width="1.2"/>
          <text x="90" y="46" text-anchor="middle" fill="${ink()}" font-family="-apple-system,sans-serif" font-size="18" font-weight="500">Client</text>
          <text x="90" y="72" text-anchor="middle" fill="${inkSoft()}" font-family="ui-monospace,Menlo,monospace" font-size="11" letter-spacing="0.08em">UPLOAD &#183; PAY &#183; VERIFY</text>
          <text x="90" y="100" text-anchor="middle" fill="${inkSoft()}" font-family="-apple-system,sans-serif" font-size="12">.eth website &#183; dataset</text>
        </g>
        <g class="ar-box" transform="translate(290, 60)">
          <rect width="180" height="120" rx="14" fill="none" stroke="${stroke()}" stroke-width="1.2"/>
          <text x="90" y="46" text-anchor="middle" fill="${ink()}" font-family="-apple-system,sans-serif" font-size="18" font-weight="500">Prover</text>
          <text x="90" y="72" text-anchor="middle" fill="${inkSoft()}" font-family="ui-monospace,Menlo,monospace" font-size="11" letter-spacing="0.08em">STAKE &#183; STORE &#183; PROVE</text>
          <text x="90" y="100" text-anchor="middle" fill="${inkSoft()}" font-family="-apple-system,sans-serif" font-size="12">disk + bandwidth + bond</text>
        </g>
        <g class="ar-box" transform="translate(560, 60)">
          <rect width="180" height="120" rx="14" fill="none" stroke="${stroke()}" stroke-width="1.2"/>
          <text x="90" y="46" text-anchor="middle" fill="${ink()}" font-family="-apple-system,sans-serif" font-size="18" font-weight="500">Ethereum</text>
          <text x="90" y="72" text-anchor="middle" fill="${inkSoft()}" font-family="ui-monospace,Menlo,monospace" font-size="11" letter-spacing="0.08em">VERIFY &#183; SLASH &#183; PAY</text>
          <text x="90" y="100" text-anchor="middle" fill="${inkSoft()}" font-family="-apple-system,sans-serif" font-size="12">Base L2 settlement</text>
        </g>

        <!-- Static skeleton arrows -->
        <path d="M200 120 L290 120" stroke="${stroke()}" stroke-width="1.4" fill="none" marker-end="url(#ar-head)" opacity="0.5"/>
        <path d="M470 120 L560 120" stroke="${stroke()}" stroke-width="1.4" fill="none" marker-end="url(#ar-head)" opacity="0.5"/>
        <path d="M650 180 Q650 240 470 240 L290 240 Q200 240 200 180" stroke="${stroke()}" stroke-width="1.4" fill="none" stroke-dasharray="3 4" opacity="0.45"/>

        <!-- Subtle stage label only on the bottom return arc (where the
             pill is hidden because the packet sits below center) -->
        <g font-family="ui-monospace,Menlo,monospace" font-size="10" letter-spacing="0.16em" fill="${inkSoft()}" opacity="0.6">
          <text x="380" y="282" text-anchor="middle">SETTLEMENT &#183; USDC PAYOUT</text>
        </g>

        <!-- Live packet -->
        <g id="ar-packet">
          <circle r="22" fill="url(#ar-pulse)"/>
          <circle r="6" fill="${accent()}"/>
          <g id="ar-packet-pill" transform="translate(0,-26)">
            <rect x="-34" y="-10" width="68" height="18" rx="9" fill="${bg()}" stroke="${stroke()}" stroke-width="1"/>
            <text x="0" y="3" text-anchor="middle" fill="${ink()}" font-family="ui-monospace,Menlo,monospace" font-size="9.5" letter-spacing="0.14em" id="ar-packet-label">PIECE</text>
          </g>
        </g>
      </svg>
    `;

    const packet = host.querySelector('#ar-packet');
    const pill   = host.querySelector('#ar-packet-pill');
    const label  = host.querySelector('#ar-packet-label');

    // Single smooth path. Packet travels from Client to Prover to Ethereum
    // along the top, then settles back along the bottom return arc.
    // Stages are only the OUTSIDE-of-box segments. Inside the box the packet
    // would just sit there, so we skip it via setTimeout gap.
    // Bezier-shaped path described by control point sets per stage.
    // Stage 1+2: straight horizontal hops on top connectors.
    // Stage 3: full curved descent + traversal along the bottom return arc.
    // Stage 4: traversal on bottom + curve back up into the Client.
    const STAGES = [
      { type: 'line',  from: [200, 120], to: [290, 120], dur: 1400, label: 'PIECE'  },
      { type: 'line',  from: [470, 120], to: [560, 120], dur: 1400, label: 'PROOF'  },
      { type: 'curve',
        // Match the static dashed path: M650 180 Q650 240 470 240 L290 240 Q200 240 200 180
        // Animate Eth box bottom-edge -> down -> along bottom past Prover
        points: [[650, 180], [650, 240], [470, 240], [380, 240]],
        dur: 1700, label: 'VERIFY' },
      { type: 'curve',
        // Continue along bottom -> curve up into Client box
        points: [[380, 240], [290, 240], [200, 240], [200, 180]],
        dur: 1500, label: 'USDC' },
    ];

    function lerp(a, b, t) { return a + (b - a) * t; }
    function bezierAt(pts, t) {
      // Cubic-ish: if 4 points use cubic; if 2 use linear
      if (pts.length === 2) {
        return [lerp(pts[0][0], pts[1][0], t), lerp(pts[0][1], pts[1][1], t)];
      }
      // Cubic Bezier (4 control points)
      const u = 1 - t;
      const x = u*u*u*pts[0][0] + 3*u*u*t*pts[1][0] + 3*u*t*t*pts[2][0] + t*t*t*pts[3][0];
      const y = u*u*u*pts[0][1] + 3*u*u*t*pts[1][1] + 3*u*t*t*pts[2][1] + t*t*t*pts[3][1];
      return [x, y];
    }

    let stageIdx = 0;
    let stageStart = performance.now();
    let raf;

    function tick(now) {
      const stage = STAGES[stageIdx];
      const t = Math.min(1, Math.max(0, (now - stageStart) / stage.dur));
      const eased = t < 0.5 ? 2*t*t : 1 - Math.pow(-2*t+2, 2)/2;
      let x, y;
      if (stage.type === 'curve') {
        [x, y] = bezierAt(stage.points, eased);
      } else {
        x = stage.from[0] + (stage.to[0] - stage.from[0]) * eased;
        y = stage.from[1] + (stage.to[1] - stage.from[1]) * eased;
      }
      packet.setAttribute('transform', `translate(${x}, ${y})`);

      // Hide the pill when packet enters a box (avoids overlap with box text)
      const overBox =
        (x >= 20  && x <= 200 && y < 180) ||
        (x >= 290 && x <= 470 && y < 180) ||
        (x >= 560 && x <= 740 && y < 180);
      pill.style.opacity = overBox ? '0' : '1';
      pill.style.transition = 'opacity 0.25s ease';

      if (t >= 1) {
        stageIdx = (stageIdx + 1) % STAGES.length;
        stageStart = now + 350;
        label.textContent = STAGES[stageIdx].label;
      }
      raf = requestAnimationFrame(tick);
    }
    cancelAnimationFrame(raf);
    label.textContent = STAGES[0].label;
    raf = requestAnimationFrame(tick);
  }

  // ============================================================
  // LIFECYCLE: deal walks through 5 states, current pulses
  // ============================================================
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

  // ============================================================
  // CLIENT FLOW CARDS
  // ============================================================
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
        <p>Build a static site, drop the bundle into the Prova CLI, get back a piece-cid. Point your ENS contenthash at it. Now your site lives across Prova provers, served over HTTPS, proven daily.</p>
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

  function renderAll() {
    renderArchitecture();
    renderLifecycle();
    renderClientFlows();
  }
  renderAll();
  window.__renderDiagrams = renderAll;
})();

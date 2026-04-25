// Prova — Redundancy lab + earnings calculator
(() => {
  const $ = (sel) => document.querySelector(sel);

  // ── Redundancy lab ────────────────────────────────────────────────────────
  const canvas = $('#redundancyCanvas');
  if (canvas) {
    const N = 6;
    const W = 800, H = 400;
    const cx = W / 2, cy = H / 2;
    const r = 150;
    let alive = Array(N).fill(true);

    function isLight() {
      return document.body.dataset.theme === 'light';
    }

    function buildSvg() {
      const accent = isLight() ? '#2c6f8a' : '#5dc3e5';
      const ink = isLight() ? '#0f141a' : '#e8edf1';
      const inkSoft = isLight() ? '#5d6873' : '#aab3ba';
      const hair = isLight() ? 'rgba(31,58,82,0.22)' : 'rgba(180,192,202,0.22)';
      const bg = isLight() ? '#e3e7ec' : '#0a0c0f';

      let out = `<svg viewBox="0 0 ${W} ${H}" preserveAspectRatio="xMidYMid meet">`;
      out += '<defs>';
      out += '<filter id="pulse-glow"><feGaussianBlur stdDeviation="4" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>';
      out += `<radialGradient id="center-grad"><stop offset="0%" stop-color="${accent}" stop-opacity="0.5"/><stop offset="100%" stop-color="${accent}" stop-opacity="0"/></radialGradient>`;
      out += '</defs>';

      // Lines from center to provers
      for (let i = 0; i < N; i++) {
        const a = (i / N) * Math.PI * 2 - Math.PI / 2;
        const px = cx + Math.cos(a) * r;
        const py = cy + Math.sin(a) * r;
        const col = alive[i] ? accent : '#e06464';
        const colOp = alive[i] ? '0.55' : '0.25';
        const dash = alive[i] ? '' : 'stroke-dasharray="4 4"';
        out += `<line x1="${cx}" y1="${cy}" x2="${px}" y2="${py}" stroke="${col}" stroke-opacity="${colOp}" stroke-width="1.5" ${dash}/>`;
      }

      // Center: client
      out += `<circle cx="${cx}" cy="${cy}" r="70" fill="url(#center-grad)"/>`;
      out += `<circle cx="${cx}" cy="${cy}" r="36" fill="${bg}" stroke="${accent}" stroke-width="2"/>`;
      out += `<text x="${cx}" y="${cy - 2}" text-anchor="middle" fill="${ink}" font-family="-apple-system,sans-serif" font-size="15" font-weight="500">client</text>`;
      out += `<text x="${cx}" y="${cy + 14}" text-anchor="middle" fill="${accent}" opacity="0.85" font-family="ui-monospace,Menlo,monospace" font-size="9" letter-spacing="0.1em">YOUR FILE</text>`;

      // Provers around the ring
      for (let i = 0; i < N; i++) {
        const a = (i / N) * Math.PI * 2 - Math.PI / 2;
        const px = cx + Math.cos(a) * r;
        const py = cy + Math.sin(a) * r;
        const isAlive = alive[i];
        const strokeColor = isAlive ? accent : '#e06464';
        const fillOp = isAlive ? '0.15' : '0.08';
        const glow = isAlive ? 'filter="url(#pulse-glow)"' : '';
        out += `<g class="prover-node" data-idx="${i}" style="cursor:pointer">`;
        out += `<circle cx="${px}" cy="${py}" r="32" fill="${strokeColor}" fill-opacity="${fillOp}" stroke="${strokeColor}" stroke-width="2" ${glow}/>`;
        out += `<text x="${px}" y="${py - 2}" text-anchor="middle" fill="${isAlive ? ink : '#e06464'}" font-family="ui-monospace,Menlo,monospace" font-size="12" font-weight="600">P${i + 1}</text>`;
        out += `<text x="${px}" y="${py + 14}" text-anchor="middle" fill="${strokeColor}" font-family="ui-monospace,Menlo,monospace" font-size="9">${isAlive ? 'alive' : 'down'}</text>`;
        out += '</g>';
      }
      out += '</svg>';
      return out;
    }

    function updateStats() {
      const aliveCount = alive.filter(Boolean).length;
      const down = N - aliveCount;
      $('#statTotal').textContent = N;
      $('#statAlive').textContent = aliveCount;
      $('#statDown').textContent = down;
      const st = $('#statStatus');
      if (aliveCount > 0) {
        st.textContent = aliveCount === N ? 'ONLINE' : 'ONLINE (via P' + (alive.findIndex(Boolean) + 1) + ')';
        st.classList.add('alive'); st.classList.remove('dead');
      } else {
        st.textContent = 'OFFLINE';
        st.classList.remove('alive'); st.classList.add('dead');
      }
    }

    function render() {
      canvas.innerHTML = buildSvg();
      canvas.querySelectorAll('.prover-node').forEach(g => {
        g.addEventListener('click', () => {
          const idx = parseInt(g.dataset.idx);
          alive[idx] = !alive[idx];
          render();
          updateStats();
        });
      });
      updateStats();
    }

    render();
    window.__renderLab = render;

    $('#labReset').addEventListener('click', () => {
      alive = Array(N).fill(true);
      render();
    });
  }

  // ── Earnings calculator ───────────────────────────────────────────────────
  const sliders = {
    tb: $('#sliderTb'),
    util: $('#sliderUtil'),
    price: $('#sliderPrice'),
    uptime: $('#sliderUptime'),
  };
  if (!sliders.tb) return;

  const out = {
    valTb: $('#valTb'), valUtil: $('#valUtil'), valPrice: $('#valPrice'), valUptime: $('#valUptime'),
    monthly: $('#earnMonthly'), gross: $('#earnGross'), fee: $('#earnFee'), drag: $('#earnDrag'),
    year: $('#earnYear'), breakEven: $('#earnBreak'),
  };

  function updRange(input) {
    const min = parseFloat(input.min), max = parseFloat(input.max);
    const pct = ((parseFloat(input.value) - min) / (max - min)) * 100;
    input.style.setProperty('--fill', pct + '%');
  }

  function fmt(n, d = 0) {
    if (n >= 10000) return n.toLocaleString('en-US', { maximumFractionDigits: 0 });
    return n.toLocaleString('en-US', { minimumFractionDigits: d, maximumFractionDigits: d });
  }

  function recalc() {
    const tb = parseFloat(sliders.tb.value);
    const util = parseFloat(sliders.util.value) / 100;
    const price = parseFloat(sliders.price.value);
    const uptime = parseFloat(sliders.uptime.value) / 100;

    out.valTb.textContent = tb.toFixed(1);
    out.valUtil.textContent = Math.round(util * 100);
    out.valPrice.textContent = price.toFixed(2);
    out.valUptime.textContent = uptime === 1 ? '100.0' : (uptime * 100).toFixed(1);

    const tibCommitted = tb * 0.9094;
    const tibActive = tibCommitted * util;
    const gross = tibActive * price;
    const feeAmt = gross * 0.01;
    const drag = gross * (1 - uptime);
    const net = gross - feeAmt - drag;
    const yearly = net * 12;
    const breakEvenTb = 50 / Math.max(0.01, price * 0.9094 * util * uptime * 0.99);

    out.gross.textContent = fmt(gross, gross < 100 ? 2 : 0);
    out.fee.textContent = fmt(feeAmt, 2);
    out.drag.textContent = fmt(drag, 2);
    out.monthly.textContent = fmt(net, net < 100 ? 2 : 0);
    out.year.textContent = fmt(yearly);
    out.breakEven.textContent = fmt(breakEvenTb, 1) + ' TB';

    Object.values(sliders).forEach(updRange);
  }

  Object.values(sliders).forEach(input => {
    input.addEventListener('input', recalc);
    updRange(input);
  });
  recalc();
})();

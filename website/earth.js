// Prova — Live WebGL Earth
// A real animated planet, not a still. Custom shader earth + atmosphere,
// archive nodes, great-circle arcs, client→node pings, and proof rings.
//
// Motion model:
//   - constant slow auto-rotate (one revolution ≈ 180s)
//   - subtle camera bob — reads as breath, not shake
//   - light direction drifts slowly across the surface (terminator sweep)
//   - scroll position rotates the planet an additional 0.6 turns over 2000px
//
// Themes:
//   - dark   : graphite ocean, ash land, cold cyan rim       (planetary archive at night)
//   - light  : mist ocean, slate land, soft steel-blue glow  (mineral, daylit, NOT saas)
//
// Expects THREE on window (loaded by index.html).

(() => {
  const T = window.THREE;
  const canvas = document.getElementById('earth-canvas');
  if (!T || !canvas) return;

  // ── Themes ────────────────────────────────────────────────────────────────
  const THEMES = {
    dark: {
      bg:           [0.040, 0.047, 0.058],
      ocean_deep:   [0.038, 0.055, 0.075],
      ocean_shore:  [0.082, 0.108, 0.140],
      land_low:     [0.180, 0.205, 0.232],
      land_high:    [0.460, 0.498, 0.532],
      atmo:         [0.20,  0.30,  0.42 ],
      rim:          [0.32,  0.50,  0.66 ],
      sun:          [0.96,  0.97,  1.00 ],
      ambient:      0.14,
      atmoStrength: 0.42,
      rimStrength:  0.55,
      pulseHex:     0xc8d6df,
      arcHex:       0x9ab2c2,
      arcOpacity:   0.20,
      nodeHex:      0xd8e1e8,
      clientHex:    0xa5b3c0,
      pingHex:      0xe6edf3,
    },
    light: {
      bg:           [0.892, 0.910, 0.928],
      ocean_deep:   [0.380, 0.510, 0.640],
      ocean_shore:  [0.530, 0.640, 0.745],
      land_low:     [0.745, 0.760, 0.780],
      land_high:    [0.910, 0.918, 0.925],
      atmo:         [0.55,  0.68,  0.84 ],
      rim:          [0.62,  0.74,  0.88 ],
      sun:          [1.00,  0.99,  0.96 ],
      ambient:      0.45,
      atmoStrength: 0.55,
      rimStrength:  0.42,
      pulseHex:     0x4a6f8c,
      arcHex:       0x5c7d96,
      arcOpacity:   0.32,
      nodeHex:      0x36506b,
      clientHex:    0x2a3f55,
      pingHex:      0x1e3148,
    },
  };

  let activeTheme = 'dark';

  // ── Renderer ──────────────────────────────────────────────────────────────
  const renderer = new T.WebGLRenderer({ canvas, antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.outputColorSpace = T.SRGBColorSpace;
  renderer.toneMapping = T.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.0;

  const scene  = new T.Scene();
  const camera = new T.PerspectiveCamera(28, 1, 0.1, 50);
  camera.position.set(0, 0, 4.6);

  // ── Textures ──────────────────────────────────────────────────────────────
  const loader = new T.TextureLoader();
  const bumpTex = loader.load('images/globe_texture_bump.jpg');
  bumpTex.colorSpace = T.NoColorSpace;
  bumpTex.wrapS = bumpTex.wrapT = T.RepeatWrapping;
  bumpTex.anisotropy = renderer.capabilities.getMaxAnisotropy();

  // ── Earth shader ──────────────────────────────────────────────────────────
  const earthMat = new T.ShaderMaterial({
    uniforms: {
      uBump:        { value: bumpTex },
      uTime:        { value: 0 },
      uSunDir:      { value: new T.Vector3(0.6, 0.25, 0.75).normalize() },
      uOceanDeep:   { value: new T.Color() },
      uOceanShore:  { value: new T.Color() },
      uLandLow:     { value: new T.Color() },
      uLandHigh:    { value: new T.Color() },
      uRim:         { value: new T.Color() },
      uSun:         { value: new T.Color() },
      uAmbient:     { value: 0.18 },
      uRimStrength: { value: 0.72 },
      uNightDim:    { value: 0.55 },
    },
    vertexShader: `
      varying vec2 vUv;
      varying vec3 vNormal;
      varying vec3 vWorldNormal;
      varying vec3 vViewDir;
      void main() {
        vUv = uv;
        vNormal = normalize(normalMatrix * normal);
        vWorldNormal = normalize(mat3(modelMatrix) * normal);
        vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
        vViewDir = normalize(-mvPos.xyz);
        gl_Position = projectionMatrix * mvPos;
      }
    `,
    fragmentShader: `
      precision highp float;
      uniform sampler2D uBump;
      uniform vec3 uSunDir;
      uniform vec3 uOceanDeep, uOceanShore, uLandLow, uLandHigh;
      uniform vec3 uRim, uSun;
      uniform float uAmbient, uRimStrength;
      varying vec2 vUv;
      varying vec3 vNormal;
      varying vec3 vWorldNormal;
      varying vec3 vViewDir;

      void main() {
        float h = texture2D(uBump, vUv).r;
        float h2 = (
          texture2D(uBump, vUv + vec2( 0.0008, 0.0)).r +
          texture2D(uBump, vUv + vec2(-0.0008, 0.0)).r +
          texture2D(uBump, vUv + vec2(0.0,  0.0008)).r +
          texture2D(uBump, vUv + vec2(0.0, -0.0008)).r
        ) * 0.25;
        h = mix(h, h2, 0.35);
        float landMask = smoothstep(0.035, 0.085, h);
        vec3 ocean = mix(uOceanDeep, uOceanShore, smoothstep(0.0, 0.06, h));
        vec3 land  = mix(uLandLow, uLandHigh, smoothstep(0.06, 0.55, h));
        float coast = smoothstep(0.03, 0.06, h) * (1.0 - smoothstep(0.06, 0.10, h));
        vec3 base = mix(ocean, land, landMask);
        base = mix(base, base + uRim * 0.18, coast * 0.6);

        vec3 N = normalize(vWorldNormal);
        float NdotL = dot(N, normalize(uSunDir));
        float wrap  = max(0.0, (NdotL + 0.18) / 1.18);
        float diffuse = pow(wrap, 1.05);
        vec3 H = normalize(normalize(uSunDir) + vViewDir);
        float spec = pow(max(dot(N, H), 0.0), 60.0) * (1.0 - landMask) * 0.35;
        float fres = pow(1.0 - max(dot(vNormal, vViewDir), 0.0), 3.0);
        vec3 rim = uRim * fres * uRimStrength;
        vec3 lit = base * (uAmbient + diffuse) * uSun + spec * uSun + rim;
        float night = smoothstep(0.05, -0.15, NdotL);
        lit = mix(lit, lit * 0.55 + uOceanDeep * 0.35, night * uNightDim);
        gl_FragColor = vec4(lit, 1.0);
      }
    `,
  });

  // ── Atmosphere (back sphere, additive) ────────────────────────────────────
  const atmoMat = new T.ShaderMaterial({
    uniforms: {
      uAtmo:     { value: new T.Color() },
      uStrength: { value: 0.95 },
    },
    vertexShader: `
      varying vec3 vNormal;
      varying vec3 vViewDir;
      void main() {
        vNormal = normalize(normalMatrix * normal);
        vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
        vViewDir = normalize(-mvPos.xyz);
        gl_Position = projectionMatrix * mvPos;
      }
    `,
    fragmentShader: `
      precision highp float;
      uniform vec3 uAtmo;
      uniform float uStrength;
      varying vec3 vNormal;
      varying vec3 vViewDir;
      void main() {
        float f = pow(1.0 - max(dot(vNormal, vViewDir), 0.0), 2.6);
        gl_FragColor = vec4(uAtmo, f * uStrength);
      }
    `,
    transparent: true,
    side: T.BackSide,
    blending: T.AdditiveBlending,
    depthWrite: false,
  });

  // Outer haze
  const hazeMat = atmoMat.clone();
  hazeMat.uniforms = {
    uAtmo: atmoMat.uniforms.uAtmo,
    uStrength: { value: 0.18 },
  };

  // ── Geometry assemblies ───────────────────────────────────────────────────
  const earth = new T.Mesh(new T.SphereGeometry(1, 128, 96), earthMat);
  const atmo  = new T.Mesh(new T.SphereGeometry(1.10, 96, 64), atmoMat);
  const haze  = new T.Mesh(new T.SphereGeometry(1.34, 64, 48), hazeMat);
  scene.add(atmo); scene.add(haze);

  const planet = new T.Group();
  planet.add(earth);
  scene.add(planet);

  // ── Archive nodes (storage providers) ─────────────────────────────────────
  const NODES = [
    { name: 'Reykjavík',     lat:  64.13, lon: -21.95 },
    { name: 'Stockholm',     lat:  59.33, lon:  18.07 },
    { name: 'Dublin',        lat:  53.35, lon:  -6.26 },
    { name: 'Ashburn',       lat:  39.04, lon: -77.49 },
    { name: 'Montréal',      lat:  45.50, lon: -73.57 },
    { name: 'San Francisco', lat:  37.77, lon:-122.42 },
    { name: 'Querétaro',     lat:  20.58, lon:-100.39 },
    { name: 'São Paulo',     lat: -23.55, lon: -46.63 },
    { name: 'Cape Town',     lat: -33.92, lon:  18.42 },
    { name: 'Lagos',         lat:   6.52, lon:   3.38 },
    { name: 'Tokyo',         lat:  35.68, lon: 139.69 },
    { name: 'Singapore',     lat:   1.35, lon: 103.82 },
    { name: 'Sydney',        lat: -33.87, lon: 151.21 },
    { name: 'Mumbai',        lat:  19.08, lon:  72.88 },
    { name: 'Dubai',         lat:  25.20, lon:  55.27 },
  ];

  // ── Clients (smaller, dispersed, asymmetric) ──────────────────────────────
  const CLIENTS = [
    { lat:  51.51, lon:  -0.13 },  // London
    { lat:  40.71, lon: -74.00 },  // NYC
    { lat:  48.85, lon:   2.35 },  // Paris
    { lat:  52.52, lon:  13.40 },  // Berlin
    { lat:  37.56, lon: 126.97 },  // Seoul
    { lat:  22.32, lon: 114.17 },  // Hong Kong
    { lat: -34.60, lon: -58.45 },  // Buenos Aires
    { lat:  19.43, lon: -99.13 },  // Mexico City
    { lat:  30.04, lon:  31.23 },  // Cairo
    { lat:  41.01, lon:  28.97 },  // Istanbul
    { lat:  59.91, lon:  10.75 },  // Oslo
    { lat:  35.69, lon:  51.39 },  // Tehran
  ];

  function latLonToVec(lat, lon, r = 1) {
    const phi = (90 - lat) * Math.PI / 180;
    const lam = (lon + 180) * Math.PI / 180;
    return new T.Vector3(
      -r * Math.sin(phi) * Math.cos(lam),
       r * Math.cos(phi),
       r * Math.sin(phi) * Math.sin(lam)
    );
  }

  // Soft sprite texture (reused)
  const dotTex = (() => {
    const c = document.createElement('canvas'); c.width = c.height = 128;
    const g = c.getContext('2d');
    const grad = g.createRadialGradient(64, 64, 0, 64, 64, 64);
    grad.addColorStop(0,    'rgba(255,255,255,1)');
    grad.addColorStop(0.30, 'rgba(255,255,255,0.55)');
    grad.addColorStop(1,    'rgba(255,255,255,0)');
    g.fillStyle = grad; g.fillRect(0, 0, 128, 128);
    const tex = new T.CanvasTexture(c);
    tex.colorSpace = T.SRGBColorSpace;
    return tex;
  })();

  // Archive node sprites
  const nodes = NODES.map(n => {
    const pos = latLonToVec(n.lat, n.lon, 1.005);
    const sprite = new T.Sprite(new T.SpriteMaterial({
      map: dotTex,
      color: 0xd8e1e8,
      transparent: true,
      depthWrite: false,
      blending: T.AdditiveBlending,
      opacity: 0.85,
    }));
    sprite.position.copy(pos);
    sprite.scale.setScalar(0.026);
    sprite.userData = { node: n, pos, baseScale: 0.026, flash: 0 };
    planet.add(sprite);
    return sprite;
  });

  // Smaller client sprites
  const clients = CLIENTS.map(c => {
    const pos = latLonToVec(c.lat, c.lon, 1.003);
    const sprite = new T.Sprite(new T.SpriteMaterial({
      map: dotTex,
      color: 0xa5b3c0,
      transparent: true,
      depthWrite: false,
      blending: T.AdditiveBlending,
      opacity: 0.55,
    }));
    sprite.position.copy(pos);
    sprite.scale.setScalar(0.014);
    sprite.userData = { client: c, pos, baseScale: 0.014, flash: 0 };
    planet.add(sprite);
    return sprite;
  });

  // ── Background great-circle arcs (constant relations) ─────────────────────
  function greatCircle(a, b, segs = 80, lift = 0.18) {
    const va = latLonToVec(a.lat, a.lon, 1);
    const vb = latLonToVec(b.lat, b.lon, 1);
    const angle = va.angleTo(vb);
    const axis = new T.Vector3().crossVectors(va, vb).normalize();
    const pts = [];
    for (let i = 0; i <= segs; i++) {
      const t = i / segs;
      const v = va.clone().applyAxisAngle(axis, angle * t);
      v.multiplyScalar(1 + Math.sin(t * Math.PI) * lift);
      pts.push(v);
    }
    return pts;
  }

  const ARC_PAIRS = [
    [0, 3], [4, 5], [3, 7], [2, 0], [5, 6],
    [10, 11], [10, 12], [13, 14], [8, 9], [3, 14],
    [11, 13], [1, 10],
  ];
  const arcGroup = new T.Group();
  planet.add(arcGroup);
  ARC_PAIRS.forEach(([i, j]) => {
    const pts = greatCircle(NODES[i], NODES[j], 80, 0.13);
    const geo = new T.BufferGeometry().setFromPoints(pts);
    const mat = new T.LineBasicMaterial({
      color: 0x9ab2c2,
      transparent: true,
      opacity: 0.18,
      depthWrite: false,
    });
    arcGroup.add(new T.Line(geo, mat));
  });

  // ── Animated ping (client → node → client) ───────────────────────────────
  // A ping is a small head + a fading trail traveling along a great-circle arc.
  // Implemented as a moving point sprite + an arc line whose opacity follows
  // a kernel centered on the head — so it reads as "energy flowing across".

  const pings = [];

  function spawnPing(client, node, returning = false) {
    const a = returning ? node : client;
    const b = returning ? client : node;
    const liftAmt = 0.16 + Math.random() * 0.05;
    const arcPts = greatCircle(a, b, 96, liftAmt);

    const arcGeo = new T.BufferGeometry().setFromPoints(arcPts);
    // Per-vertex colors to drive the trail kernel via a basic line.
    const colors = new Float32Array(arcPts.length * 3);
    arcGeo.setAttribute('color', new T.BufferAttribute(colors, 3));

    const arcMat = new T.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.95,
      depthWrite: false,
      blending: T.AdditiveBlending,
    });
    const arcLine = new T.Line(arcGeo, arcMat);
    planet.add(arcLine);

    // Head sprite
    const head = new T.Sprite(new T.SpriteMaterial({
      map: dotTex,
      color: returning ? 0xffffff : THEMES[activeTheme].pingHex,
      transparent: true,
      depthWrite: false,
      blending: T.AdditiveBlending,
      opacity: 1.0,
    }));
    head.scale.setScalar(0.018);
    head.position.copy(arcPts[0]);
    planet.add(head);

    pings.push({
      arcLine,
      arcGeo,
      arcMat,
      head,
      pts: arcPts,
      colors,
      t0: performance.now(),
      life: 1400 + Math.random() * 300,
      returning,
      client,
      node,
    });
  }

  // ── Proof rings (occasional, on archive nodes) ────────────────────────────
  const ringGeo = new T.RingGeometry(0.6, 0.62, 64);
  const rings = [];
  function spawnProofRing(node) {
    const pos = latLonToVec(node.lat, node.lon, 1.002);
    const mat = new T.MeshBasicMaterial({
      color: THEMES[activeTheme].pulseHex,
      transparent: true,
      opacity: 0.8,
      depthWrite: false,
      blending: T.AdditiveBlending,
      side: T.DoubleSide,
    });
    const ring = new T.Mesh(ringGeo, mat);
    ring.position.copy(pos);
    ring.lookAt(new T.Vector3(0, 0, 0));
    ring.rotateY(Math.PI);
    ring.scale.setScalar(0.001);
    ring.userData = { t0: performance.now(), life: 1900 };
    planet.add(ring);
    rings.push(ring);

    const sprite = nodes.find(s => s.userData.node === node);
    if (sprite) sprite.userData.flash = performance.now();
  }

  // ── Theme application ─────────────────────────────────────────────────────
  function applyTheme(name) {
    activeTheme = name;
    const t = THEMES[name];
    earthMat.uniforms.uOceanDeep.value.fromArray(t.ocean_deep);
    earthMat.uniforms.uOceanShore.value.fromArray(t.ocean_shore);
    earthMat.uniforms.uLandLow.value.fromArray(t.land_low);
    earthMat.uniforms.uLandHigh.value.fromArray(t.land_high);
    earthMat.uniforms.uRim.value.fromArray(t.rim);
    earthMat.uniforms.uSun.value.fromArray(t.sun);
    earthMat.uniforms.uAmbient.value = t.ambient;
    earthMat.uniforms.uRimStrength.value = t.rimStrength;
    earthMat.uniforms.uNightDim.value = name === 'light' ? 0.18 : 0.55;
    atmoMat.uniforms.uAtmo.value.fromArray(t.atmo);
    atmoMat.uniforms.uStrength.value = t.atmoStrength;
    // Switch blending: additive on dark (glow), normal-alpha on light (no clip)
    const blend = name === 'dark' ? T.AdditiveBlending : T.NormalBlending;
    atmoMat.blending = blend;
    hazeMat.blending = blend;
    atmoMat.needsUpdate = true;
    hazeMat.needsUpdate = true;
    document.body.dataset.theme = name;
    document.body.style.background = `rgb(${t.bg.map(v => Math.round(v*255)).join(',')})`;
    nodes.forEach(s => s.material.color.setHex(t.nodeHex));
    clients.forEach(s => s.material.color.setHex(t.clientHex));
    arcGroup.children.forEach(l => {
      l.material.color.setHex(t.arcHex);
      l.material.opacity = t.arcOpacity;
    });
  }
  applyTheme('dark');
  window.__applyTheme = applyTheme;

  // ── Resize ────────────────────────────────────────────────────────────────
  function resize() {
    // Use the parent .hero size, not viewport
    const parent = canvas.parentElement;
    const rect = parent.getBoundingClientRect();
    const w = rect.width || window.innerWidth;
    const h = rect.height || window.innerHeight;
    renderer.setSize(w, h, false);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  }
  resize();
  window.addEventListener('resize', resize);

  // ── Cadence: client → node pings + occasional proof rings ────────────────
  function pingLoop() {
    // Pause work when hero is fully scrolled past
    const r = canvas.getBoundingClientRect();
    if (r.bottom < 0) {
      setTimeout(pingLoop, 1500);
      return;
    }
    const c = CLIENTS[Math.floor(Math.random() * CLIENTS.length)];
    const n = NODES[Math.floor(Math.random() * NODES.length)];
    spawnPing(c, n, false);
    setTimeout(() => {
      spawnPing(c, n, true);
      if (Math.random() < 0.55) spawnProofRing(n);
    }, 700 + Math.random() * 250);
    setTimeout(pingLoop, 900 + Math.random() * 1100);
  }
  setTimeout(pingLoop, 600);

  // ── Render loop ──────────────────────────────────────────────────────────
  const start = performance.now();
  function frame() {
    const now = (performance.now() - start) / 1000;
    earthMat.uniforms.uTime.value = now;

    // Constant slow auto-rotate, no scroll dependency
    const baseRot = (now / 180) * Math.PI * 2;
    planet.rotation.y = baseRot;
    planet.rotation.z = 0.41 + Math.sin(now / 28) * 0.01;
    planet.rotation.x = Math.sin(now / 20) * 0.03;

    // Sun direction drifts slowly (terminator sweep)
    const sa = now / 90;
    earthMat.uniforms.uSunDir.value.set(
      Math.cos(sa) * 0.85,
      0.22 + Math.sin(now / 47) * 0.05,
      Math.sin(sa) * 0.85
    ).normalize();

    // Camera breath only — hero is contained, scroll doesn't drive the scene
    camera.position.y = Math.sin(now / 14) * 0.04;
    camera.position.x = Math.cos(now / 19) * 0.02;
    camera.position.z = 4.6;
    camera.lookAt(0, 0, 0);

    // ── Pings: animate head along arc, fade trail ──────────────────────────
    const tn = performance.now();
    const headColor = new T.Color(activeTheme === 'dark' ? 0xe6edf3 : 0x1e3148);
    for (let i = pings.length - 1; i >= 0; i--) {
      const p = pings[i];
      const k = (tn - p.t0) / p.life;
      if (k >= 1) {
        planet.remove(p.head);
        planet.remove(p.arcLine);
        p.head.material.dispose();
        p.arcMat.dispose();
        p.arcGeo.dispose();
        pings.splice(i, 1);
        continue;
      }
      const eased = 1 - Math.pow(1 - k, 2.4);
      const idxF = eased * (p.pts.length - 1);
      const headIdx = Math.min(p.pts.length - 1, Math.floor(idxF));
      p.head.position.copy(p.pts[headIdx]);

      // Update vertex colors for trail kernel: bright near head, fade behind
      const trailLen = 22; // segments visible
      for (let v = 0; v < p.pts.length; v++) {
        const dist = headIdx - v;
        let intensity = 0;
        if (dist >= 0 && dist <= trailLen) {
          intensity = (1 - (dist / trailLen)) * (1 - k * 0.4);
        }
        p.colors[v * 3 + 0] = headColor.r * intensity;
        p.colors[v * 3 + 1] = headColor.g * intensity;
        p.colors[v * 3 + 2] = headColor.b * intensity;
      }
      p.arcGeo.attributes.color.needsUpdate = true;
      p.head.material.opacity = (1 - k * k);
    }

    // ── Proof rings: scale + fade ──────────────────────────────────────────
    for (let i = rings.length - 1; i >= 0; i--) {
      const r = rings[i];
      const k = (tn - r.userData.t0) / r.userData.life;
      if (k >= 1) {
        planet.remove(r);
        r.material.dispose();
        rings.splice(i, 1);
        continue;
      }
      const eased = 1 - Math.pow(1 - k, 3);
      r.scale.setScalar(0.002 + eased * 0.20);
      r.material.opacity = (1 - k) * 0.85;
    }

    // ── Node flash ─────────────────────────────────────────────────────────
    nodes.forEach(s => {
      const f = s.userData.flash;
      if (f) {
        const k = (tn - f) / 1100;
        if (k >= 1) {
          s.userData.flash = 0;
          s.material.opacity = 0.85;
          s.scale.setScalar(s.userData.baseScale);
        } else {
          const e = 1 - Math.pow(1 - k, 2);
          s.material.opacity = 0.85 + (1 - e) * 0.15;
          s.scale.setScalar(s.userData.baseScale + (1 - e) * 0.018);
        }
      }
    });

    renderer.render(scene, camera);
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
})();

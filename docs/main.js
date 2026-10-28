// upz GitHub Pages landing page — spatial field + live 64B LitanyWireFrame & SPQT calculator.
(() => {
  "use strict";

  const canvas = document.getElementById("field");
  const ctx = canvas.getContext("2d", { alpha: false });

  const state = {
    w: 0,
    h: 0,
    dpr: 1,
    t: 0,
    mode: "ultra",
    intensity: 1,
    bg: null,
    particles: [],
    pulses: [],
    pointer: { x: 0.5, y: 0.5, down: false, active: false },
  };

  const PALETTES = {
    ultra: ["#dff6ff", "#8fd0ff", "#4b7bff", "#c9d6ff", "#ffffff"],
    zeno: ["#ffe6a6", "#e0b6ff", "#8a5cff", "#ff8ad8", "#ffffff"],
  };

  const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  function palette() {
    return PALETTES[state.mode] || PALETTES.ultra;
  }

  function resize() {
    state.dpr = Math.min(window.devicePixelRatio || 1, 2);
    state.w = canvas.clientWidth;
    state.h = canvas.clientHeight;
    canvas.width = Math.floor(state.w * state.dpr);
    canvas.height = Math.floor(state.h * state.dpr);
    ctx.setTransform(state.dpr, 0, 0, state.dpr, 0, 0);

    const target = Math.min(750, Math.round((state.w * state.h) / 2800));
    const n = Math.max(120, target);
    state.bg = null;
    state.particles = new Array(n);
    for (let i = 0; i < n; i++) {
      state.particles[i] = {
        x: Math.random() * state.w,
        y: Math.random() * state.h,
        vx: 0,
        vy: 0,
        sp: 0.4 + Math.random() * 0.9,
      };
    }
  }

  function flowAngle(x, y, t) {
    const s = 0.0016;
    return (
      Math.sin(x * s * state.intensity + t * 0.6) * 1.4 +
      Math.cos(y * s * 1.3 - t * 0.45) * 1.2 +
      Math.sin((x + y) * s * 0.7 + t * 0.25) * 0.9
    );
  }

  function step(dt) {
    const { w, h, particles, pointer } = state;
    const px = pointer.x * w;
    const py = pointer.y * h;
    const force = 26 * state.intensity;

    for (const p of particles) {
      const a = flowAngle(p.x, p.y, state.t);
      p.vx += Math.cos(a) * force * dt * p.sp;
      p.vy += Math.sin(a) * force * dt * p.sp;

      const dx = px - p.x;
      const dy = py - p.y;
      const d2 = dx * dx + dy * dy + 4000;
      const pull = (pointer.down ? 900 : 260) / d2;
      p.vx += dx * pull * dt * 60;
      p.vy += dy * pull * dt * 60;

      p.vx *= 0.965;
      p.vy *= 0.965;
      p.x += p.vx * dt * 60 * 0.05;
      p.y += p.vy * dt * 60 * 0.05;

      if (p.x < -20) p.x = w + 20;
      if (p.x > w + 20) p.x = -20;
      if (p.y < -20) p.y = h + 20;
      if (p.y > h + 20) p.y = -20;
    }
  }

  function drawBackground() {
    const g = ctx.createLinearGradient(0, 0, 0, state.h);
    if (state.mode === "zeno") {
      g.addColorStop(0, "#0b0714");
      g.addColorStop(0.55, "#120a1e");
      g.addColorStop(1, "#050308");
    } else {
      g.addColorStop(0, "#030811");
      g.addColorStop(0.55, "#04101d");
      g.addColorStop(1, "#01030a");
    }
    ctx.globalCompositeOperation = "source-over";
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, state.w, state.h);
    state.bg = g;
  }

  function drawHorizon() {
    const { w, h, pointer } = state;
    const horizon = h * 0.66;
    const vx = pointer.x * w;
    const pal = palette();
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    ctx.strokeStyle = pal[2];
    ctx.globalAlpha = 0.16;
    ctx.lineWidth = 1;

    const cols = Math.floor(w / 48);
    for (let i = 0; i <= cols; i++) {
      const xTop = (i / cols) * w;
      const xBot = vx + (xTop - vx) * 3.4;
      ctx.beginPath();
      ctx.moveTo(xTop, horizon);
      ctx.lineTo(xBot, h + 40);
      ctx.stroke();
    }
    ctx.restore();
  }

  function drawParticles() {
    const { particles, pointer } = state;
    const pal = palette();
    const px = pointer.x * state.w;
    const py = pointer.y * state.h;
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    ctx.lineCap = "round";
    for (let i = 0; i < particles.length; i++) {
      const p = particles[i];
      const dx = p.x - px;
      const dy = p.y - py;
      const d = Math.sqrt(dx * dx + dy * dy);
      const near = Math.max(0, 1 - d / 260);
      ctx.strokeStyle = pal[(i + Math.floor(state.t * 2)) % pal.length];
      ctx.globalAlpha = 0.12 + near * 0.58;
      ctx.lineWidth = 0.6 + near * 1.7;
      ctx.beginPath();
      ctx.moveTo(p.x, p.y);
      ctx.lineTo(p.x - p.vx * 0.9, p.y - p.vy * 0.9);
      ctx.stroke();
    }
    ctx.restore();
  }

  function drawPulses(dt) {
    const pal = palette();
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    for (let i = state.pulses.length - 1; i >= 0; i--) {
      const s = state.pulses[i];
      s.r += 260 * dt;
      s.life -= dt * 0.9;
      if (s.life <= 0) {
        state.pulses.splice(i, 1);
        continue;
      }
      ctx.strokeStyle = pal[0];
      ctx.globalAlpha = Math.max(0, s.life) * 0.6;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2);
      ctx.stroke();
    }
    ctx.restore();
  }

  // ── Pure BigInt 64B LitanyWireFrame + RationalPQ + SPQT base-243 ──────────
  const MASK64 = 0xffffffffffffffffn;
  function gcdBig(a, b) {
    let x = a < 0n ? -a : a;
    let y = b < 0n ? -b : b;
    while (y !== 0n) {
      const r = x % y;
      x = y;
      y = r;
    }
    return x === 0n ? 1n : x;
  }

  function packSpqt3B(word15) {
    const clean = Number(BigInt(word15) & 0x7fffn);
    const out = [0, 0, 0];
    for (let g = 0; g < 3; g++) {
      let acc = 0;
      let pow3 = 1;
      for (let j = 0; j < 5; j++) {
        const bit = (clean >> (g * 5 + j)) & 1;
        acc += (bit === 1 ? 2 : 1) * pow3;
        pow3 *= 3;
      }
      out[g] = acc;
    }
    return out;
  }

  function trit(x) {
    if (x >= 0.33) return 1;
    if (x <= -0.33) return -1;
    return 0;
  }

  function glyph(t) {
    return t > 0 ? "+" : t < 0 ? "-" : "0";
  }

  function wireInspector() {
    const rawEl = document.getElementById("w-raw");
    const packEl = document.getElementById("w-pack");
    const axes = ["v", "a", "d"].map((id) => document.getElementById("q-" + id));
    const out = document.getElementById("q-out");
    const glyphs = document.getElementById("q-glyphs");
    const wireHex = document.getElementById("wire-hex");
    if (!rawEl || !packEl || !out || !wireHex) return;

    const render = () => {
      const rawLen = BigInt(rawEl.value || 3761);
      const packLen = BigInt(packEl.value || 3314);
      document.getElementById("w-raw-v").textContent = String(rawLen);
      document.getElementById("w-pack-v").textContent = String(packLen);

      const vad = axes.map((el) => parseFloat(el.value));
      for (const id of ["q-v", "q-a", "q-d"]) {
        const el = document.getElementById(id);
        const vOut = document.getElementById(id + "-v");
        if (el && vOut) {
          vOut.textContent = (el.value >= 0 ? "+" : "") + parseFloat(el.value).toFixed(2);
        }
      }
      const trits = vad.map(trit);
      let qtByte = 0;
      for (let j = 0; j < 3; j++) {
        qtByte |= ((trits[j] + 1) & 3) << (2 * j);
      }
      qtByte |= 0x40; // anchor non-zero mask
      const g = gcdBig(rawLen, packLen);
      const rp = rawLen / g;
      const rq = packLen / g;

      const rawFold = (0x48031a0d4c054e69n ^ (rawLen * 0x9e3779b97f4a7c15n)) & MASK64;
      const packFold = (0x206f642074737557n ^ (packLen * 0xbf58476d1ce4e5b9n)) & MASK64;
      const compFold = rawFold ^ packFold;
      const mask64 = (BigInt(0x55) * 0x0101010101010101n) & MASK64;
      const bfpPos = compFold & mask64;
      const bfpNeg = compFold & (~mask64 & MASK64);
      const spqt = packSpqt3B(compFold & 0x7fffn);
      const seal16 = Number((rawFold ^ packFold ^ rp ^ rq ^ 0x0101n ^ 0x62n) & 0xffffn);

      if (glyphs) glyphs.textContent = trits.map(glyph).join(" ");
      out.textContent =
        "qt:" +
        qtByte.toString(16).padStart(2, "0") +
        " · p/q = " +
        rp.toString() +
        "/" +
        rq.toString() +
        " · SPQT(243) = [" +
        spqt.join(", ") +
        "]";

      wireHex.textContent =
        "[00..16] raw_fold   = 0x" + rawFold.toString(16).padStart(16, "0") +
        "  packed_fold = 0x" + packFold.toString(16).padStart(16, "0") + "\n" +
        "[16..32] raw_len    = " + rawLen.toString().padStart(10, " ") +
        "          packed_len  = " + packLen.toString().padStart(10, " ") +
        "  gate = 0x62  abi = 0x01\n" +
        "[32..48] bfp64_pos  = 0x" + bfpPos.toString(16).padStart(16, "0") +
        "  bfp64_neg   = 0x" + bfpNeg.toString(16).padStart(16, "0") + "\n" +
        "[48..64] ratio_pq   = " + rp.toString() + "/" + rq.toString() +
        "  spqt_root = [" + spqt.join(", ") + "]  qt = 0x55  lanes = 0x0101  seal16 = 0x" +
        seal16.toString(16).padStart(4, "0");
    };

    rawEl.addEventListener("input", render);
    packEl.addEventListener("input", render);
    for (const el of axes) el.addEventListener("input", render);
    for (const btn of document.querySelectorAll("[data-preset]")) {
      btn.addEventListener("click", () => {
        const [r, p] = btn.getAttribute("data-preset").split(",");
        rawEl.value = r;
        packEl.value = p;
        render();
      });
    }
    render();
  }

  function wireModes() {
    for (const id of ["ultra", "zeno"]) {
      const el = document.getElementById("mode-" + id);
      if (el) {
        el.addEventListener("click", () => {
          state.mode = id;
          state.bg = null;
          document.body.dataset.mode = id;
          for (const other of ["ultra", "zeno"]) {
            const o = document.getElementById("mode-" + other);
            if (o) o.setAttribute("aria-pressed", String(other === id));
          }
        });
      }
    }
    const intensity = document.getElementById("intensity");
    if (intensity) {
      intensity.addEventListener("input", () => {
        state.intensity = parseFloat(intensity.value);
      });
    }
  }

  window.addEventListener("pointermove", (e) => {
    state.pointer.x = e.clientX / state.w;
    state.pointer.y = e.clientY / state.h;
  });
  window.addEventListener("pointerdown", (e) => {
    state.pointer.down = true;
    state.pulses.push({ x: e.clientX, y: e.clientY, r: 6, life: 1 });
  });
  window.addEventListener("pointerup", () => {
    state.pointer.down = false;
  });

  let last = performance.now();
  function frame(now) {
    const dt = Math.min(0.05, (now - last) / 1000);
    last = now;
    state.t += dt;

    ctx.globalCompositeOperation = "source-over";
    if (!state.bg) drawBackground();
    ctx.globalAlpha = 0.16;
    ctx.fillStyle = state.bg;
    ctx.fillRect(0, 0, state.w, state.h);
    ctx.globalAlpha = 1;

    step(dt);
    drawHorizon();
    drawParticles();
    drawPulses(dt);

    requestAnimationFrame(frame);
  }

  window.addEventListener("resize", resize);
  resize();
  wireInspector();
  wireModes();
  if (!reduced) {
    requestAnimationFrame(frame);
  } else {
    drawBackground();
    drawHorizon();
    drawParticles();
  }
})();

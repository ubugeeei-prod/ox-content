// Shared WebGL2 runtime for the skins that render a live hero backdrop.
//
// Liquid Glass deliberately has none. A caustic field behind the page makes the
// *page* the glass, and the material belongs to the controls sitting on it —
// the refraction there is geometric, done with an SVG displacement map on the
// backdrop filter rather than painted underneath.
//
// The SSG concatenates a theme's `js` into one classic inline <script>, so this
// cannot import anything — hence a hand-rolled runtime rather than a library.
// It is strictly progressive enhancement: the CSS backdrop is the real design,
// and the canvas only ever layers on top of it. Every bail-out path below
// leaves the page exactly as it renders without JavaScript.

function octcGL(fragmentSource, options) {
  const opts = options || {};
  const host = document.querySelector(opts.target || ".hero");
  if (!host || typeof WebGL2RenderingContext === "undefined") return;
  if (matchMedia("(prefers-reduced-motion: reduce)").matches) return;

  const canvas = document.createElement("canvas");
  canvas.setAttribute("aria-hidden", "true");
  canvas.style.cssText =
    "position:absolute;inset:0;width:100%;height:100%;z-index:0;pointer-events:none;" +
    "opacity:0;transition:opacity 900ms ease";
  const gl = canvas.getContext("webgl2", {
    alpha: true,
    antialias: false,
    depth: false,
    stencil: false,
    powerPreference: "low-power",
  });
  if (!gl) return;

  // A fullscreen triangle from gl_VertexID needs no buffers at all.
  const VERT =
    "#version 300 es\nvoid main(){vec2 p=vec2(float((gl_VertexID<<1)&2),float(gl_VertexID&2));" +
    "gl_Position=vec4(p*2.0-1.0,0.0,1.0);}";

  function compile(type, src) {
    const sh = gl.createShader(type);
    gl.shaderSource(sh, src);
    gl.compileShader(sh);
    if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
      gl.deleteShader(sh);
      return null;
    }
    return sh;
  }

  const vs = compile(gl.VERTEX_SHADER, VERT);
  const fs = compile(gl.FRAGMENT_SHADER, fragmentSource);
  if (!vs || !fs) return;

  const program = gl.createProgram();
  gl.attachShader(program, vs);
  gl.attachShader(program, fs);
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) return;
  gl.useProgram(program);

  const uRes = gl.getUniformLocation(program, "u_res");
  const uTime = gl.getUniformLocation(program, "u_time");
  const uColors = [0, 1, 2, 3].map((i) => gl.getUniformLocation(program, "u_c" + i));

  // Palette values are read through a probe element rather than parsed from the
  // custom property, because a scheme may define one as color-mix(), which only
  // resolves to concrete channels once it is used.
  const probe = document.createElement("span");
  probe.style.cssText = "position:absolute;width:0;height:0;opacity:0;pointer-events:none";
  host.appendChild(probe);

  function readColor(name, fallback) {
    probe.style.color = fallback;
    probe.style.color = "var(" + name + "," + fallback + ")";
    const value = getComputedStyle(probe).color;
    const parts = value.match(/[\d.]+/g);
    if (!parts) return [0, 0, 0];
    // `color-mix()` serialises as `color(srgb r g b)` with 0-1 channels.
    const scale = value.indexOf("color(") === 0 ? 1 : 255;
    return [parts[0] / scale, parts[1] / scale, parts[2] / scale];
  }

  const SLOTS = opts.colors || [
    "--octc-accent-a",
    "--octc-accent-b",
    "--octc-accent-c",
    "--octc-color-bg",
  ];

  function pushColors() {
    for (let i = 0; i < uColors.length; i++) {
      if (!uColors[i]) continue;
      const c = readColor(SLOTS[i], "#888888");
      gl.uniform3f(uColors[i], c[0], c[1], c[2]);
    }
  }

  // Resolution is capped rather than tied to devicePixelRatio: this is a soft,
  // out-of-focus backdrop, so a retina-sharp buffer costs fill rate for nothing.
  let width = 0;
  let height = 0;
  function resize() {
    const dpr = Math.min(devicePixelRatio || 1, 1.5);
    const w = Math.max(1, Math.round(host.clientWidth * dpr));
    const h = Math.max(1, Math.round(host.clientHeight * dpr));
    if (w === width && h === height) return;
    width = canvas.width = w;
    height = canvas.height = h;
    gl.viewport(0, 0, w, h);
    gl.uniform2f(uRes, w, h);
  }

  let running = false;
  let visible = false;
  let frame = 0;
  const start = performance.now();

  function tick(now) {
    if (!running) return;
    resize();
    gl.uniform1f(uTime, (now - start) / 1000);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    frame = requestAnimationFrame(tick);
  }

  function setRunning(next) {
    if (next === running) return;
    running = next;
    if (running) frame = requestAnimationFrame(tick);
    else cancelAnimationFrame(frame);
  }

  // Scrolled past or backgrounded means no work: an ambient effect must never
  // keep a tab busy once it is off screen.
  new IntersectionObserver((entries) => {
    visible = entries[0].isIntersecting;
    setRunning(visible && !document.hidden);
  }).observe(host);
  document.addEventListener("visibilitychange", () => {
    setRunning(visible && !document.hidden);
  });
  addEventListener("resize", resize, { passive: true });

  // The scheme changes under us whenever the reader flips the theme toggle.
  new MutationObserver(pushColors).observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme"],
  });

  host.insertBefore(canvas, host.firstChild);
  pushColors();
  resize();
  // Set directly rather than from a frame callback: anything that throws
  // between mount and that callback leaves the canvas permanently invisible,
  // which is indistinguishable from the effect simply not existing.
  canvas.style.opacity = opts.opacity == null ? "1" : String(opts.opacity);
}

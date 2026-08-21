// Liquid Glass — real refraction, done the way a compositor does it.
//
// The math is a direct port of ShojiWM's liquid-glass fragment shader
// (github.com/bea4dev/ShojiWM, packages/config/src/liquid-glass.frag): a
// rounded-rect SDF locates each pixel relative to the panel, the middle of the
// panel passes the backdrop through untouched, and an edge band bends the
// sample toward the panel centre along a circular profile —
// `1 - sqrt(1 - d^2)` is the silhouette of a spherical bevel, which is why the
// result reads as a ground glass edge rather than a smear. Red is sampled a
// little deeper than blue, so the rim carries the chromatic fringe a real lens
// would.
//
// CSS cannot do any of this — `backdrop-filter: url(...)` is unsupported in
// Safari and unstable in Chromium — so the canvas draws the hero wallpaper
// itself and refracts it under the real DOM panels, whose rects are fed in as
// uniforms each frame. The panels' own CSS keeps only a thin tint and a faint
// blur, so what you see through them is this shader.
octcGL(
  `#version 300 es
precision highp float;
uniform vec2 u_res; uniform float u_time;
uniform vec3 u_c0, u_c1, u_c2, u_c3;
uniform vec4 u_rects[8]; uniform float u_rads[8]; uniform int u_count;
out vec4 outColor;

float blob(vec2 p, vec2 c, float r){
  return 1.0 - smoothstep(0.0, r, distance(p, c));
}

// The wallpaper needs structure — a rim bending a flat gradient is invisible —
// so soft accent pools sit over slow, low-contrast bands of light.
vec3 wallpaper(vec2 q){
  float aspect = u_res.x / max(u_res.y, 1.0);
  vec2 p = vec2(q.x * aspect, q.y);
  float t = u_time * 0.05;
  vec3 col = u_c3;
  col = mix(col, u_c0, 0.42 * blob(p, vec2(0.16 * aspect + 0.04 * sin(t), 0.22), 0.62));
  col = mix(col, u_c2, 0.36 * blob(p, vec2(0.86 * aspect, 0.18 + 0.05 * cos(t * 0.8)), 0.55));
  col = mix(col, u_c1, 0.30 * blob(p, vec2(0.62 * aspect, 0.92), 0.68));
  float bands = sin((q.x * 2.6 - q.y * 1.8 + t * 0.9) * 6.2831);
  float fine = sin((q.x * 7.0 + q.y * 4.4 - t * 0.6) * 6.2831);
  col += 0.03 * bands + 0.016 * fine;
  return col;
}

float sdRoundRect(vec2 p, vec2 b, float r){
  vec2 d = abs(p) - b + vec2(r);
  return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - r;
}

void main(){
  vec2 frag = vec2(gl_FragCoord.x, u_res.y - gl_FragCoord.y);
  vec3 col = vec3(0.0);
  bool inside = false;
  for (int i = 0; i < 8; i++){
    if (i >= u_count) break;
    vec2 half_ = u_rects[i].zw;
    vec2 p = frag - u_rects[i].xy;
    float s = sdRoundRect(p, half_, u_rads[i]);
    if (s >= 0.0) continue;
    inside = true;
    float size = 2.0 * min(half_.x, half_.y);
    float inv = -s / size;
    float d = 1.0 - clamp(inv / 0.34, 0.0, 1.0);
    float distortion = 1.0 - sqrt(max(1.0 - d * d, 0.0));
    vec2 dir = p / max(length(p), 0.0001);
    vec2 off = distortion * dir * half_ * 0.62;
    float edge = smoothstep(0.0, 0.02, inv);
    vec2 shift = dir * edge * 3.0;
    vec2 sp = frag - off;
    col = vec3(
      wallpaper((sp - shift) / u_res).r,
      wallpaper(sp / u_res).g,
      wallpaper((sp + shift) / u_res).b
    );
    // A lens gathers light where its curvature is highest.
    col = col * 0.985 + distortion * 0.085;
    break;
  }
  if (!inside) col = wallpaper(frag / u_res);
  outColor = vec4(col, 1.0);
}`,
  {
    colors: ["--octc-accent-a", "--octc-accent-b", "--octc-accent-c", "--octc-color-bg"],
    setup(gl, program, host) {
      const uRects = gl.getUniformLocation(program, "u_rects");
      const uRads = gl.getUniformLocation(program, "u_rads");
      const uCount = gl.getUniformLocation(program, "u_count");
      if (!uRects || !uCount) return null;
      // The header capsule floats over the hero, the actions and notice sit in
      // it. Radii are static per element, so they are read once.
      const els = Array.from(
        document.querySelectorAll(".header, .hero-action, .hero-notice"),
      ).slice(0, 8);
      const radii = els.map((el) => parseFloat(getComputedStyle(el).borderTopLeftRadius) || 0);
      const rects = new Float32Array(32);
      const rads = new Float32Array(8);
      return function frame(now, width, height) {
        const hb = host.getBoundingClientRect();
        if (hb.width === 0 || hb.height === 0) return;
        const sx = width / hb.width;
        const sy = height / hb.height;
        let n = 0;
        for (let i = 0; i < els.length && n < 8; i++) {
          const r = els[i].getBoundingClientRect();
          if (r.width === 0 || r.bottom <= hb.top || r.top >= hb.bottom) continue;
          rects[n * 4] = (r.left + r.width / 2 - hb.left) * sx;
          rects[n * 4 + 1] = (r.top + r.height / 2 - hb.top) * sy;
          rects[n * 4 + 2] = (r.width / 2) * sx;
          rects[n * 4 + 3] = (r.height / 2) * sy;
          rads[n] = Math.min(radii[i], r.height / 2) * sx;
          n++;
        }
        gl.uniform4fv(uRects, rects);
        gl.uniform1fv(uRads, rads);
        gl.uniform1i(uCount, n);
      };
    },
  },
);

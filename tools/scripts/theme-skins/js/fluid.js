// Fluid — dye carried by a curl-noise velocity field. Tracing the flow backwards
// a few steps and sampling there is semi-Lagrangian advection, the same idea a
// full fluid solver uses, minus the pressure solve it would need to be physical.
// The result reads as liquid at a fraction of the cost.
octcGL(
  `#version 300 es
precision highp float;
uniform vec2 u_res; uniform float u_time;
uniform vec3 u_c0, u_c1, u_c2, u_c3;
out vec4 outColor;

float hash(vec2 p){ return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }

float noise(vec2 p){
  vec2 i = floor(p), f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  return mix(mix(hash(i), hash(i + vec2(1, 0)), u.x),
             mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), u.x), u.y);
}

float fbm(vec2 p){
  float v = 0.0, a = 0.5;
  for (int i = 0; i < 4; i++){ v += a * noise(p); p *= 2.03; a *= 0.5; }
  return v;
}

// Curl of a scalar field is divergence-free by construction, which is exactly
// the property that makes the motion look like an incompressible fluid.
vec2 curl(vec2 p, float t){
  float e = 0.04;
  float a = fbm(p + vec2(0.0, e) + t);
  float b = fbm(p - vec2(0.0, e) + t);
  float c = fbm(p + vec2(e, 0.0) + t);
  float d = fbm(p - vec2(e, 0.0) + t);
  return vec2(a - b, d - c) / (2.0 * e);
}

void main(){
  vec2 uv = gl_FragCoord.xy / u_res;
  vec2 p = uv * vec2(u_res.x / u_res.y, 1.0);
  float t = u_time * 0.05;

  // Walk backwards along the velocity field to find where this dye came from.
  vec2 q = p;
  for (int i = 0; i < 4; i++) q -= curl(q * 1.5, t) * 0.022;

  float d0 = fbm(q * 1.9 + t * 0.6);
  float d1 = fbm(q * 2.4 - t * 0.45 + 9.1);

  vec3 col = u_c3;
  col = mix(col, u_c0, smoothstep(0.35, 0.85, d0) * 0.6);
  col = mix(col, u_c1, smoothstep(0.4, 0.9, d1) * 0.5);
  col = mix(col, u_c2, smoothstep(0.55, 0.95, d0 * d1) * 0.45);

  float edge = smoothstep(0.0, 0.25, uv.y) * smoothstep(1.0, 0.7, uv.y);
  outColor = vec4(col, edge * 0.8);
}`,
  { opacity: 0.85 },
);

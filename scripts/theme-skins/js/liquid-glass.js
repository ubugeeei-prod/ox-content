// Liquid Glass — a refractive slab rendered for real rather than faked with a
// blur. A domain-warped height field becomes a normal map; that normal bends
// the lookup into a procedural backdrop, which is what refraction physically
// is. Fresnel adds the bright grazing edge that sells the material.
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
  for (int i = 0; i < 5; i++){ v += a * noise(p); p *= 2.02; a *= 0.5; }
  return v;
}

// The backdrop the slab refracts: a soft three-stop wash built from the scheme.
vec3 backdrop(vec2 uv){
  float d0 = smoothstep(1.1, 0.0, distance(uv, vec2(0.22, 0.18)));
  float d1 = smoothstep(1.2, 0.0, distance(uv, vec2(0.84, 0.32)));
  float d2 = smoothstep(1.3, 0.0, distance(uv, vec2(0.5, 0.95)));
  vec3 c = u_c3;
  c = mix(c, u_c0, d0 * 0.55);
  c = mix(c, u_c2, d1 * 0.45);
  c = mix(c, u_c1, d2 * 0.35);
  return c;
}

void main(){
  vec2 uv = gl_FragCoord.xy / u_res;
  vec2 p = uv * vec2(u_res.x / u_res.y, 1.0);
  float t = u_time * 0.055;

  // Warp the sampling domain with a second noise field so the surface flows
  // like a thick liquid instead of scrolling like a texture.
  vec2 warp = vec2(fbm(p * 2.1 + t), fbm(p * 2.1 - t + 4.7));
  float h = fbm(p * 2.6 + warp * 1.6);

  // Normal from the height field's gradient, by finite difference.
  float e = 1.6 / u_res.y;
  float hx = fbm((p + vec2(e, 0.0)) * 2.6 + warp * 1.6);
  float hy = fbm((p + vec2(0.0, e)) * 2.6 + warp * 1.6);
  vec3 n = normalize(vec3((hx - h) / e, (hy - h) / e, 1.6));

  vec3 refracted = backdrop(uv + n.xy * 0.07);

  // Fresnel: grazing angles reflect, head-on angles transmit.
  float fres = pow(1.0 - clamp(n.z, 0.0, 1.0), 2.4);
  vec3 spec = vec3(1.0) * pow(max(dot(n, normalize(vec3(0.45, 0.7, 0.6))), 0.0), 42.0);

  vec3 col = refracted + spec * 0.5 + fres * 0.16;

  // Fade at the frame edges so the canvas never announces its own rectangle.
  float edge = smoothstep(0.0, 0.22, uv.y) * smoothstep(1.0, 0.72, uv.y);
  outColor = vec4(col, edge * 0.85);
}`,
  { opacity: 0.9 },
);

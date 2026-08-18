// Fabric — actual shading rather than a printed texture. The weave is built as a
// height field of interleaved warp and weft threads; that height gives a normal,
// and the normal is lit by a slowly orbiting key light. Threads catch the light
// along their own direction, which is what makes woven cloth read as cloth.
octcGL(
  `#version 300 es
precision highp float;
uniform vec2 u_res; uniform float u_time;
uniform vec3 u_c0, u_c1, u_c2, u_c3;
out vec4 outColor;

float hash(vec2 p){ return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }

// One cell holds a warp thread crossing a weft thread. Which one passes over
// alternates per cell, exactly as in a plain weave.
float weave(vec2 p){
  vec2 cell = floor(p);
  vec2 f = fract(p);
  bool warpOver = mod(cell.x + cell.y, 2.0) < 1.0;
  float warp = sin(f.x * 3.14159);
  float weft = sin(f.y * 3.14159);
  float over = warpOver ? warp : weft;
  float under = warpOver ? weft : warp;
  // Slight per-thread variation stops the weave reading as a printed grid.
  float slub = 0.9 + 0.2 * hash(warpOver ? vec2(cell.x, 0.0) : vec2(0.0, cell.y));
  return (over * 0.75 + under * 0.25) * slub;
}

void main(){
  vec2 uv = gl_FragCoord.xy / u_res;
  vec2 p = uv * vec2(u_res.x / u_res.y, 1.0) * 190.0;

  float h = weave(p);
  float e = 0.6;
  vec3 n = normalize(vec3(
    (weave(p + vec2(e, 0.0)) - h) / e,
    (weave(p + vec2(0.0, e)) - h) / e,
    1.0
  ));

  // A key light that drifts slowly, so the cloth catches the light as you read.
  float a = u_time * 0.12;
  vec3 light = normalize(vec3(cos(a) * 0.6, sin(a) * 0.6, 0.75));
  float lambert = max(dot(n, light), 0.0);
  float sheen = pow(max(dot(reflect(-light, n), vec3(0.0, 0.0, 1.0)), 0.0), 18.0);

  vec3 base = mix(u_c3, u_c0, 0.10);
  vec3 col = base * (0.82 + lambert * 0.34) + sheen * 0.10;

  float edge = smoothstep(0.0, 0.2, uv.y) * smoothstep(1.0, 0.78, uv.y);
  outColor = vec4(col, edge * 0.55);
}`,
  { opacity: 0.7 },
);

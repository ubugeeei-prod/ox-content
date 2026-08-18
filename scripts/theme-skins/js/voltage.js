// Voltage — a charged field rather than literal lightning bolts. Ridged noise
// gives filaments their characteristic sharp crease (a bolt is a crease in a
// field, not a line), and the ridges drift on two different time bases so the
// pattern never visibly repeats.
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

// Ridged noise: folding the field about its midpoint turns smooth hills into
// sharp creases, which is what reads as a filament.
float ridge(vec2 p){
  float v = 0.0, a = 0.5;
  for (int i = 0; i < 5; i++){
    v += a * (1.0 - abs(noise(p) * 2.0 - 1.0));
    p *= 2.05; a *= 0.5;
  }
  return v;
}

void main(){
  vec2 uv = gl_FragCoord.xy / u_res;
  vec2 p = uv * vec2(u_res.x / u_res.y, 1.0);
  float t = u_time * 0.04;

  float f = ridge(p * 2.3 + vec2(t, -t * 0.7));
  float g = ridge(p * 3.7 - vec2(t * 0.6, t * 1.1) + 5.3);

  // Only the top of the ridge lights up, so filaments stay thin.
  float arc = pow(smoothstep(0.72, 1.0, f), 3.0);
  float haze = pow(smoothstep(0.5, 1.0, g), 2.0) * 0.35;

  vec3 col = u_c3;
  col = mix(col, u_c0, haze);
  col = mix(col, u_c1, arc * 0.75);
  col += u_c2 * arc * 0.25;

  float edge = smoothstep(0.0, 0.3, uv.y) * smoothstep(1.0, 0.66, uv.y);
  outColor = vec4(col, edge * 0.7);
}`,
  { opacity: 0.75 },
);

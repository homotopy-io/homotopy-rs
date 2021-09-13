#version 300 es

precision highp float;

in vec4 position_start;
in vec4 position_end;

uniform float t;

out float hidden;
out vec3 frag_pos;
out vec3 frag_normal;

uniform mat4 mvp;

void main() {
  float relative_t = (t - position_start.w) / (position_end.w - position_start.w);

  hidden = (relative_t < 0. || relative_t > 1.) ? 1. : 0.;

  float lerp = clamp(relative_t, 0., 1.);

  frag_pos = position_start.xyz + lerp * (position_end.xyz - position_start.xyz);
  // frag_normal = normal_start + lerp * (normal_end - normal_start);
  frag_normal = vec3(1., 0., 0.);

  gl_Position = mvp * vec4(frag_pos, 1.);
}

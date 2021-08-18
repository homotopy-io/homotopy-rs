#version 300 es

precision highp float;

in vec3 out_normal;

out vec4 frag_color;

void main() {
  frag_color = vec4((out_normal.rgb + vec3(1.)) / 2., 1.);
}

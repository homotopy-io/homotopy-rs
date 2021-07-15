#version 300 es

precision highp float;

in vec3 out_normal;

out vec4 frag_color;

void main() {
  frag_color = vec4(abs(out_normal.rgb), 1.);
}

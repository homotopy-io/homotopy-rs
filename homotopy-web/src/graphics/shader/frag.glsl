#version 300 es

precision highp float;

in vec3 out_color;

out vec4 frag_color;

void main() {
  frag_color = vec4(out_color, 1.0);
}

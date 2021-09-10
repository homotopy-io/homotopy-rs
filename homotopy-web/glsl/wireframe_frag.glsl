#version 300 es

precision highp float;

in vec3 frag_diffuse;

out vec4 frag_color;

void main() {
  frag_color = vec4(frag_diffuse, 1.);
}

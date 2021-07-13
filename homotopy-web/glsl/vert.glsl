#version 300 es

precision highp float;

in vec4 position;
in vec3 in_color;

out vec3 out_color;

void main() {
  out_color = in_color;
  gl_Position = position;
}

#version 300 es

precision highp float;

in vec4 position;
in vec3 in_color;

uniform mat4 transform;

out vec3 out_color;

void main() {
  out_color = in_color;
  gl_Position = transform * position;
}

#version 300 es

precision highp float;

in vec3 position;
in vec3 color;

out vec3 frag_diffuse;

uniform mat4 mvp;

void main() {
  frag_diffuse = color;
  gl_Position = mvp * vec4(position, 1.);
}

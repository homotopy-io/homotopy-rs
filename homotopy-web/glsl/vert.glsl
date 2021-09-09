#version 300 es

precision highp float;

in vec3 position;
in vec3 normal;

out vec3 frag_pos;
out vec3 frag_normal;

uniform mat4 mvp;

void main() {
  frag_pos = position;
  frag_normal = normal;
  
  gl_Position = mvp * vec4(position, 1.);
}

#version 300 es

precision highp float;

in vec3 position;
in vec3 in_normal;

uniform mat4 mvp;
uniform mat4 m_inv;

out vec3 out_normal;

void main() {
  gl_Position = mvp * vec4(position, 1.0);

  out_normal = normalize(mat3(m_inv) * in_normal);
}

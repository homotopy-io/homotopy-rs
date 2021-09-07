#version 300 es

precision highp float;

in vec3 position;
in vec3 normal;

out vec3 frag_pos;
out vec3 frag_normal;

uniform mat4 m;
uniform mat4 m_inv;
uniform mat4 mvp;

void main() {
  vec4 frag_pos_hom = m * vec4(position, 1.);
  frag_pos = frag_pos_hom.xyz / frag_pos_hom.w;
  frag_normal = vec3(transpose(m_inv) * vec4(normal, 0.));
  
  gl_Position = mvp * vec4(position, 1.);
}

#version 300 es

precision highp float;

in vec3 position;
in vec3 normal;

out float hidden;
out vec3 frag_pos;
out vec3 frag_normal;

uniform float t;
uniform mat4 mvp;

void main() {
    frag_pos = position;
    frag_normal = normal;
    hidden = 0.;
    // use `t` here so it isn't optimised out
    gl_Position = mvp * vec4(position, max(t, 1.));
}

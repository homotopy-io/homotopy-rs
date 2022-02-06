#version 300 es

precision highp float;

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;

out float hidden;
out vec3 frag_pos;
out vec3 frag_normal;

uniform float t;
uniform mat4 mv;
uniform mat4 p;

void main() {
    vec4 transformed_normal = mv * vec4(normal, 0.);
    vec4 transformed_position = mv * vec4(position, max(t, 1.));

    frag_pos = transformed_position.xyz / transformed_position.w;
    frag_normal = normalize(transformed_normal.xyz);
    hidden = 0.;
    // use `t` here so it isn't optimised out
    gl_Position = p * transformed_position;
}

#version 300 es

precision highp float;

layout (location = 0) in vec4 position_start;
layout (location = 1) in vec4 position_end;

layout (location = 2) in vec4 normal_start;
layout (location = 3) in vec4 normal_end;

uniform float t;

out float hidden;
out vec3 frag_pos;
out vec3 frag_normal;

uniform mat4 mv;
uniform mat4 p;

void main() {
    float lerp = (t - position_start.w) / (position_end.w - position_start.w);

    if (lerp < 0. || lerp > 1.) {
        hidden = 1.;
        gl_Position = vec4(0., 0., 0., 1.);
        return;
    }

    vec3 lerp_normal = normal_start.xyz + lerp * (normal_end.xyz - normal_start.xyz);
    vec3 lerp_position = position_start.xyz + lerp * (position_end.xyz - position_start.xyz);

    vec4 transformed_normal = mv * vec4(lerp_normal, 0.);
    vec4 transformed_position = mv * vec4(lerp_position, 1.);

    frag_pos = transformed_position.xyz / transformed_position.w;
    frag_normal = normalize(transformed_normal.xyz);
    gl_Position = p * transformed_position;
}

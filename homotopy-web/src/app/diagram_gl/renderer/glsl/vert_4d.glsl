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

uniform mat4 mvp;

void main() {
    float lerp = (t - position_start.w) / (position_end.w - position_start.w);

    if (lerp < 0. || lerp > 1.) {
        hidden = 1.;
        gl_Position = vec4(0., 0., 0., 1.);
        return;
    }

    frag_pos = position_start.xyz + lerp * (position_end.xyz - position_start.xyz);
    frag_normal = normal_start.xyz + lerp * (normal_end.xyz - normal_start.xyz);
    gl_Position = mvp * vec4(frag_pos, 1.);
}

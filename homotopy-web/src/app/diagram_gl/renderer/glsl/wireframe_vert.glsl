#version 300 es

precision highp float;

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 albedo;

out vec3 frag_albedo;

uniform mat4 mv;
uniform mat4 p;

void main() {
    frag_albedo = albedo;
    gl_Position = p * mv * vec4(position, 1.);
}

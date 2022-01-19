#version 300 es

precision highp float;

in vec3 frag_albedo;

out vec4 frag_color;

void main() {
    frag_color = vec4(frag_albedo, 1.);
}

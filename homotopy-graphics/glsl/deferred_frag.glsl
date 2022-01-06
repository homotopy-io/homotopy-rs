#version 300 es

precision highp float;

in vec2 frag_tex_coords;

out vec4 frag_color;

void main() { 
    frag_color = vec4(0., 0., 1., 1.);
}

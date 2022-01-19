#version 300 es

precision highp float;

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 tex_coords;

out vec2 frag_tex_coords;

void main() {
    frag_tex_coords = tex_coords;
    gl_Position = vec4(position.x, position.y, 0., 1.); 
}

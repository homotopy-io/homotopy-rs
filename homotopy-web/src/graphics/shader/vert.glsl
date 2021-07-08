#version 300 es

precision mediump float;

in vec4 position;

out vec4 frag_coord;

void main() {
    frag_coord = position;
}

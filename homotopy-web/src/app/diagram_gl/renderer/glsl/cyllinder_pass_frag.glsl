#version 300 es

precision highp float;

in vec2 frag_tex_coords;

uniform sampler2D in_position;
uniform sampler2D in_albedo;

layout (location = 0) out vec3 out_position;
layout (location = 1) out vec3 out_normal;
layout (location = 2) out vec4 out_albedo;

const int RADIUS = 5;

void main() {
    vec3 position = vec3(0.);

    int nearest = RADIUS * RADIUS + 1;

    for (int i = -RADIUS; i <= RADIUS; ++i) {
        for (int j = -RADIUS; j <= RADIUS; ++j) {

            int distance = i * i + j * j;
            vec2 uv = frag_tex_coords + 0.0005 * vec2(i, j);

            if (texture(in_albedo, uv).rgb != vec3(0.) && distance <= nearest) {
                nearest = distance;
                position += texture(in_position, uv).rgb;
            }
        }
    }

    out_position = position;
    out_normal = vec3(1., 0., 0.);
    out_albedo = vec4(0., 0., 1., 1.);

    if (nearest >= RADIUS * RADIUS) {
        discard;
    }
}

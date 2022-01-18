#version 300 es

precision highp float;

in vec2 frag_tex_coords;

uniform sampler2D in_position;
uniform sampler2D in_albedo;

layout (location = 0) out vec3 out_position;
layout (location = 1) out vec3 out_normal;
layout (location = 2) out vec4 out_albedo;

void main() {
    vec3 position = vec3(0.);

    float valence = 0.;

    for (int i = -5; i <= 5; ++i) {
        for (int j = -5; j <= 5; ++j) {
            if (i * i + j * j >= 24) {
                continue;
            }

            vec2 uv = frag_tex_coords + 0.001 * vec2(i, j);

            if (texture(in_albedo, uv).rgb != vec3(0.)) {
                valence += 1.;
                position += texture(in_position, uv).rgb;
            }
        }
    }

    if (valence <= 0.) {
        discard;
    }

    out_position = position / valence;
    out_normal = vec3(1., 0., 0.);
    out_albedo = vec4(1., 1., 1., 0.);
}

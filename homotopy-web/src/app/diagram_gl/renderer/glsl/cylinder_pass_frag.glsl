#version 300 es

precision highp float;

in vec2 frag_tex_coords;

uniform sampler2D in_position;
uniform sampler2D in_albedo;

uniform mat4 p;

layout (location = 0) out vec3 out_position;
layout (location = 1) out vec3 out_normal;
layout (location = 2) out vec4 out_albedo;

const int KERNEL_RADIUS = 4;
const float DELTA = 0.05;
const float TUBE_RADIUS = 0.05;

void main() {
    vec2 texel = 1. / vec2(textureSize(in_albedo, 0));

    vec3 position = vec3(0., 0., -1e10);
    vec3 albedo = vec3(0.);
    vec3 normal = vec3(0.);

    float nearest = 1.1;

    for (int i = -KERNEL_RADIUS; i <= KERNEL_RADIUS; ++i) {
        for (int j = -KERNEL_RADIUS; j <= KERNEL_RADIUS; ++j) {
            ivec2 x = ivec2(i, j);
            vec2 n = vec2(x) / float(KERNEL_RADIUS);
            float distance = dot(n, n);

            vec2 uv = frag_tex_coords + texel * vec2(x);
            vec4 local_albedo = texture(in_albedo, uv);

            if (local_albedo.a != 0. && distance <= 1.) {
                vec3 local_normal = vec3(n.x, n.y, sqrt(1. - distance));
                vec3 local_position = texture(in_position, uv).rgb + normal * TUBE_RADIUS;

                if (local_position.z - position.z > DELTA
                      || (abs(local_position.z - position.z) < DELTA && distance < nearest)) {
                    nearest = distance;
                    normal = local_normal;
                    position = local_position;
                    albedo = local_albedo.rgb;
                }
            }
        }
    }

    out_position = position;
    out_normal = normal;
    out_albedo = vec4(albedo, 1.);

    vec4 clip_pos = p * vec4(out_position, 1.);
    float ndc_depth = clip_pos.z / clip_pos.w;
    gl_FragDepth = ((gl_DepthRange.diff * ndc_depth) + gl_DepthRange.near + gl_DepthRange.far) / 2.;

    if (nearest > 1.) {
        discard;
    }
}

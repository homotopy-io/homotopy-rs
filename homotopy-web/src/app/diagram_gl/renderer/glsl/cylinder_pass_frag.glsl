#version 300 es

precision highp float;

in vec2 frag_tex_coords;

uniform sampler2D in_position;
uniform sampler2D in_albedo;

uniform mat4 p;

layout (location = 0) out vec3 out_position;
layout (location = 1) out vec3 out_normal;
layout (location = 2) out vec4 out_albedo;

const int KERNEL_RADIUS = 3;
const int OCCLUSION_GAP = KERNEL_RADIUS * 2;
const float DELTA = 0.05;
const float TUBE_RADIUS = 0.05;
const float OCCLUSION_DELTA = 0.25;

void main() {
    vec2 texel = 1. / vec2(textureSize(in_albedo, 0));

    vec3 position = vec3(0., 0., -1e10);
    vec3 albedo = vec3(0.);
    vec3 normal = vec3(0.);

    int inner_hits = 0;
    int outer_hits = 0;
    float inner_nearest_z = -1e10;
    float outer_nearest_z = -1e10;

    for (int k = 0; k <= 1; ++k) {
        int r = KERNEL_RADIUS + OCCLUSION_GAP * k;
        
        int hits = 0;
        float nearest_z = -1e10;

        int i = -r;
        int j = -r;
        int di = 1;
        int dj = 0;
        for (int s = 0; s < 4; ++s) {
            for (int t = 2 * r * s; t < 2 * r * (s + 1); ++t) {
                ivec2 x = ivec2(i, j);
                vec2 n = vec2(x) / float(r);

                vec2 uv = frag_tex_coords + texel * vec2(x);
                vec4 local_albedo = texture(in_albedo, uv);

                if (local_albedo.a != 0.) {
                    hits++;

                    vec3 local_normal = vec3(n.x, n.y, 0.);
                    vec3 local_position = texture(in_position, uv).rgb + normal * TUBE_RADIUS;

                    if (local_position.z - nearest_z > DELTA) {
                        nearest_z = local_position.z;
                    }
                    if (k == 0 && local_position.z - position.z > DELTA) {
                        normal = local_normal;
                        position = local_position;
                        albedo = local_albedo.rgb;
                    }
                }

                i += di;
                j += dj;
            }

            int temp = di;
            di = -dj;
            dj = temp;
        }

        if (k == 0) {
            inner_hits = hits;
            inner_nearest_z = nearest_z;
        } else {
            outer_hits = hits;
            outer_nearest_z = nearest_z;
        }
    }

    out_position = position;
    out_normal = normal;
    out_albedo = vec4(albedo, 1.);

    vec4 clip_pos = p * vec4(out_position, 1.);
    float ndc_depth = clip_pos.z / clip_pos.w;
    gl_FragDepth = ((gl_DepthRange.diff * ndc_depth) + gl_DepthRange.near + gl_DepthRange.far) / 2.;

    if (inner_hits == 0 || outer_nearest_z - inner_nearest_z > OCCLUSION_DELTA && outer_hits > inner_hits) {
        discard;
    }
}

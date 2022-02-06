#version 300 es

precision highp float;

in vec2 frag_tex_coords;

uniform sampler2D in_position;
uniform sampler2D in_albedo;

uniform mat4 p;

uniform int scr_width;
uniform int scr_height;

layout (location = 0) out vec3 out_position;
layout (location = 1) out vec3 out_normal;
layout (location = 2) out vec4 out_albedo;

const int SAMPLES = 10;
const int KERNEL_RADIUS = SAMPLES / 2;
const float DELTA = 0.005;

void main() {
    float step_x = .5 / float(scr_width);
    float step_y = .5 / float(scr_height);

    vec3 position = vec3(0., 0., -1e10);
    vec3 albedo = vec3(0.);
    vec3 normal = vec3(0.);

    float nearest = 1.1;

    for (int i = 0; i < SAMPLES; ++i) {
        for (int j = 0; j <= SAMPLES; ++j) {
            int x = i - KERNEL_RADIUS;
            int y = j - KERNEL_RADIUS;
            float nx = float(x) / float(KERNEL_RADIUS);
            float ny = float(y) / float(KERNEL_RADIUS);
            float distance = nx * nx + ny + ny;

            vec2 uv = frag_tex_coords + vec2(step_x * float(x), step_y * float(y));
            vec4 local_albedo = texture(in_albedo, uv);

            if (local_albedo.a != 0. && distance <= 1.) {
                vec3 local_normal = normalize(vec3(nx, ny, sqrt(1. - distance)));
                vec3 local_position = texture(in_position, uv).rgb + normal * DELTA;

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

    vec4 clip_pos = p * vec4(out_position + DELTA * vec3(0., 0., 1.), 1.);
    float ndc_depth = clip_pos.z / clip_pos.w;
    gl_FragDepth = ((gl_DepthRange.diff * ndc_depth) + gl_DepthRange.near + gl_DepthRange.far) / 2.;

    if (nearest > 1.) {
        discard;
    }
}

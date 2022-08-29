#version 300 es

precision highp float;

in vec2 frag_tex_coords;

uniform sampler2D in_position;
uniform sampler2D in_albedo;

uniform mat4 p;

layout (location = 0) out vec3 out_position;
layout (location = 1) out vec3 out_normal;
layout (location = 2) out vec4 out_albedo;

const float TUBE_RADIUS = .05;
const float NORMAL_MOD = 0.8;

const int[] SAMPLES_X = int[69](
          -2, -1,  0,  1,  2,
      -3, -2, -1,  0,  1,  2,  3,
  -4, -3, -2, -1,  0,  1,  2,  3,  4,
  -4, -3, -2, -1,  0,  1,  2,  3,  4,
  -4, -3, -2, -1,  0,  1,  2,  3,  4,
  -4, -3, -2, -1,  0,  1,  2,  3,  4,
  -4, -3, -2, -1,  0,  1,  2,  3,  4,
      -3, -2, -1,  0,  1,  2,  3,
          -2, -1,  0,  1,  2
);
const int[] SAMPLES_Y = int[69](
          -4, -4, -4, -4, -4,
      -3, -3, -3, -3, -3, -3, -3,
  -2, -2, -2, -2, -2, -2, -2, -2, -2,
  -1, -1, -1, -1, -1, -1, -1, -1, -1,
   0,  0,  0,  0,  0,  0,  0,  0,  0,
   1,  1,  1,  1,  1,  1,  1,  1,  1,
   2,  2,  2,  2,  2,  2,  2,  2,  2,
       3,  3,  3,  3,  3,  3,  3,
           4,  4,  4,  4,  4
);
const int SAMPLES = 69;

const int[] MASK_SAMPLES_X = int[](
                      -2, -1,  0,  1,  2,
              -4, -3, -2, -1,  0,  1,  2,  3,  4,
          -5, -4, -3, -2, -1,  0,  1,  2,  3,  4,  5,
      -6, -5, -4, -3,                      3,  4,  5,  6,
      -6, -5, -4,                              4,  5,  6,
  -7, -6, -5,                                      5,  6,  7,
  -7, -6, -5,                                      5,  6,  7,
  -7, -6, -5,                                      5,  6,  7,
  -7, -6, -5,                                      5,  6,  7,
  -7, -6, -5,                                      5,  6,  7,
      -6, -5, -4,                              4,  5,  6,
      -6, -5, -4, -3,                      3,  4,  5,  6,
          -5, -4, -3, -2, -1,  0,  1,  2,  3,  4,  5,
              -4, -3, -2, -1,  0,  1,  2,  3,  4,
                      -2, -1,  0,  1,  2
);
const int[] MASK_SAMPLES_Y = int[108](
                      -7, -7, -7, -7, -7,
              -6, -6, -6, -6, -6, -6, -6, -6, -6,
          -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5,
      -4, -4, -4, -4,                     -4, -4, -4, -4,
      -3, -3, -3,                             -3, -3, -3,
  -2, -2, -2,                                     -2, -2, -2,
  -1, -1, -1,                                     -1, -1, -1,
   0,  0,  0,                                      0,  0,  0,
   1,  1,  1,                                      1,  1,  1,
   2,  2,  2,                                      2,  2,  2,
       3,  3,  3,                              3,  3,  3,
       4,  4,  4,  4,                      4,  4,  4,  4,
           5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,
               6,  6,  6,  6,  6,  6,  6,  6,  6,
                       7,  7,  7,  7,  7
);
const int MASK_SAMPLES = 108;

const int HACK_RATIO_INNER = 3;
const int HACK_RATIO_OUTER = 2;
const float HACK_TUBE_SCALE = 32.;

void main() {
    vec2 texel = 1. / vec2(textureSize(in_albedo, 0));

    float nearest_distance = 1e10;
    float nearest_z = -1e10;
    int nearest_idx = 0;
    bool drop = true;

    int inner_hits = 0;
    int outer_hits = 0;

    for (int i = 0; i < SAMPLES; ++i) {
          vec2 offset = vec2(SAMPLES_X[i], SAMPLES_Y[i]);
          vec2 n = NORMAL_MOD * normalize(offset);
          vec2 uv = frag_tex_coords + texel * offset;

          float local_distance = sqrt(1. - dot(n, n));
          float local_z = texture(in_position, uv).z + TUBE_RADIUS * local_distance;
          vec4 local_albedo = texture(in_albedo, uv);

          float delta = local_z - nearest_z;
          float abs_delta = abs(delta);

          if (local_albedo.a != 0.
                  && (delta > TUBE_RADIUS || (abs_delta < TUBE_RADIUS && local_distance < nearest_distance))) {
              nearest_idx = i;
              nearest_distance = local_distance;
              nearest_z = local_z;
              ++inner_hits;
          }
    }

    for (int i = 0; i < MASK_SAMPLES; ++i) {
        vec2 offset = vec2(MASK_SAMPLES_X[i], MASK_SAMPLES_Y[i]);
        vec2 n = NORMAL_MOD * normalize(offset);
        vec2 uv = frag_tex_coords + texel * offset;

        float local_distance = sqrt(1. - dot(n, n));
        float local_z = texture(in_position, uv).z + TUBE_RADIUS * local_distance;
        vec4 local_albedo = texture(in_albedo, uv);

        if (local_albedo.a != 0. && local_z - nearest_z > HACK_TUBE_SCALE * TUBE_RADIUS) {
            ++outer_hits;
        }
    }

    ivec2 final_offset = ivec2(SAMPLES_X[nearest_idx], SAMPLES_Y[nearest_idx]);
    vec2 final_uv = frag_tex_coords + texel * vec2(final_offset);

    vec2 n = NORMAL_MOD * normalize(vec2(final_offset));
    vec3 normal = vec3(n, sqrt(1. - dot(n, n)));

    out_position = texture(in_position, final_uv).xyz + normal * TUBE_RADIUS;
    out_albedo = vec4(texture(in_albedo, final_uv).rgb, 1.);
    out_normal = normal;

    vec4 clip_pos = p * vec4(out_position, 1.);
    float ndc_depth = clip_pos.z / clip_pos.w;
    gl_FragDepth = ((gl_DepthRange.diff * ndc_depth) + gl_DepthRange.near + gl_DepthRange.far) / 2.;

    if (HACK_RATIO_INNER * inner_hits - HACK_RATIO_OUTER * outer_hits <= 0) {
        discard;
    }
}

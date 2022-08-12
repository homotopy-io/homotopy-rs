#version 300 es

precision highp float;

in vec2 frag_tex_coords;

uniform sampler2D g_position;
uniform sampler2D g_normal;
uniform sampler2D g_albedo;

uniform bool disable_lighting;
uniform bool debug_normals;

uniform float alpha;
uniform float spec;
uniform float gamma;

uniform vec3 camera_pos;

const float a = 0.5;
const int LIGHTS = 5;
const vec3 light_offsets[LIGHTS] = vec3[LIGHTS](
  vec3(0., 0., .25 * sqrt(3.)),
  vec3(-.5, 0., -.25 * sqrt(3.)),
  vec3(.5, 0., -.25 * sqrt(3.)),
  vec3(0., 1., 0.),
  vec3(0., -1., 0.)
);

out vec4 frag_color;

void main() {
    vec4 raw_albedo = texture(g_albedo, frag_tex_coords);

    float lighting_enable = raw_albedo.a;
    vec3 albedo = raw_albedo.rgb;
    vec3 normal = normalize(texture(g_normal, frag_tex_coords).rgb);
    vec3 frag_pos = texture(g_position, frag_tex_coords).rgb;

    float camera_distance = length(camera_pos);

    if (lighting_enable == 0.) {
        discard;
    } else if (debug_normals) {
        frag_color = vec4(.5 * normal + vec3(.5), 1.);
    } else if (disable_lighting) {
        frag_color = vec4(pow(albedo, vec3(1. / gamma)), 1.);
    } else if (lighting_enable == 1.) {
        vec3 color = a * albedo;
        for (int i = 0; i < LIGHTS; i++) {
            vec3 s = vec3(spec);
            vec3 d = 300. * light_offsets[i] + vec3(0., 0., -camera_distance) - frag_pos;
            vec3 l = normalize(d);

            float lambertian = max(dot(l, normal), 0.);
            float specular = 0.;

            if (lambertian > 0.) {
                vec3 view = normalize(-frag_pos);
                vec3 halfway = normalize(l + view);
                float theta = max(dot(halfway, normal), 0.);
                specular = pow(theta, alpha);
            }

            color += 0.4 * ((albedo + 0.01) * lambertian + s * specular);
        }

        frag_color = vec4(pow(color, vec3(1. / gamma)), 1.);
    } else {
        frag_color = vec4(1., 0., 1., 1.);
    }
}

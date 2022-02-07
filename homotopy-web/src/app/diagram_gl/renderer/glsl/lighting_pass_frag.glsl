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

const float a = 0.3;
const vec3 light_offset = vec3(-.75, .25, -.5);

out vec4 frag_color;

void main() {
    vec4 raw_albedo = texture(g_albedo, frag_tex_coords);

    float lighting_enable = raw_albedo.a;
    vec3 albedo = normalize(raw_albedo.rgb); 
    vec3 normal = normalize(texture(g_normal, frag_tex_coords).rgb);
    vec3 frag_pos = texture(g_position, frag_tex_coords).rgb;

    if (lighting_enable == 0.) {
        discard;
    } else if (debug_normals) {
        frag_color = vec4(.5 * normal + vec3(.5), 1.);
    } else if (disable_lighting) {
        frag_color = vec4(albedo, 1.);
    } else if (lighting_enable == 1.) {
        vec3 s = vec3(spec);
        vec3 l = normalize(light_offset - frag_pos);

        float lambertian = max(dot(l, normal), 0.);
        float specular = 0.;

        if (lambertian > 0.) {
            vec3 view = normalize(-frag_pos);
            vec3 halfway = normalize(l + view);
            float theta = max(dot(halfway, normal), 0.);
            specular = pow(theta, alpha);
        }

        vec3 color = albedo * (a + lambertian) + .25 * s * specular;
        frag_color = vec4(pow(color, vec3(1. / gamma)), 1.);
    } else {
        frag_color = vec4(1., 0., 1., 1.);
    }
}

#version 300 es

precision highp float;

in vec2 frag_tex_coords;

uniform sampler2D g_position;
uniform sampler2D g_normal;
uniform sampler2D g_albedo;

uniform vec3 camera_pos;

const float a = 0.3;
const vec3 s = vec3(1., 1., 1.);
const vec3 light_offset = vec3(-.75, .25, -.5);
const float alpha = 64.;

out vec4 frag_color;

void main() {
    vec3 frag_pos = texture(g_position, frag_tex_coords).rgb;
    vec3 normal = normalize(texture(g_normal, frag_tex_coords).rgb);
    vec3 albedo = normalize(texture(g_albedo, frag_tex_coords).rgb); 

    if (normal == vec3(0.)) {
        discard;
    }

    vec3 l = normalize(camera_pos + light_offset - frag_pos);

    float lambertian = max(dot(l, normal), 0.);
    float specular = 0.;

    if (lambertian > 0.) {
        vec3 view = normalize(camera_pos - frag_pos);
        vec3 halfway = normalize(l + view);
        float theta = max(dot(halfway, normal), 0.);
        specular = pow(theta, alpha);
    }

    vec3 color = albedo * (a + lambertian) + .25 * s * specular;
    frag_color = vec4(color, 1.);
}

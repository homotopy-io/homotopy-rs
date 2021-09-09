#version 300 es

precision highp float;

uniform bool lighting;
uniform vec3 light_pos;

in vec3 frag_pos;
in vec3 frag_normal;

out vec4 frag_color;

const float a = 0.05;
const vec3 d = vec3(.161, .502, .725);
const vec3 s = vec3(1., 1., 1.);

const float alpha = 48.;
const float gamma = 2.2;

void main() {
  if (lighting) {
    vec3 normal = normalize(frag_normal);
    vec3 l = normalize(light_pos - frag_pos);

    if (!gl_FrontFacing) {
        normal = -normal;  
    }

    float lambertian = max(dot(l, normal), 0.);
    float specular = 0.;

    if (lambertian > 0.) {
      vec3 view = normalize(-frag_pos);
      vec3 halfway = normalize(l + view);
      float theta = max(dot(halfway, normal), 0.);
      specular = pow(theta, alpha);
    }

    vec3 color = d * (a + lambertian) + s * specular;
    frag_color = vec4(pow(color, vec3(1. / gamma)), 1.);
  } else {
    frag_color = vec4(pow(d, vec3(1. / gamma)), 1.);
  }
}

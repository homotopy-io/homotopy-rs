#version 300 es

precision highp float;

in vec3 frag_pos;
in vec3 frag_normal;

out vec4 frag_color;

const vec3 l = normalize(vec3(.7, 1.2, 1.));

const vec3 a = vec3(.1, 0., 0.);
const vec3 d = vec3(.5, 0., 0.);
const vec3 s = vec3(1., 1., 1.);

const float alpha = 32.;
const float gamma = 2.2;

void main() {
  vec3 normal = normalize(frag_normal);

  float lambertian = max(dot(l, normal), 0.);
  float specular = 0.;

  if (lambertian > 0.) {
    vec3 view = normalize(-frag_pos);
    vec3 halfway = normalize(l + view);
    float theta = max(dot(halfway, normal), 0.);
    specular = pow(theta, alpha);
  }

  vec3 color = a + d * lambertian + s * specular;
  frag_color = vec4(pow(color, vec3(1. / gamma)), 1.);
}

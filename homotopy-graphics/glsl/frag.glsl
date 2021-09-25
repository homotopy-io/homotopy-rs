#version 300 es

precision highp float;

uniform bool debug_normals;
uniform vec3 camera_pos;

in float hidden;
in vec3 frag_pos;
in vec3 frag_normal;

out vec4 frag_color;

const float a = 0.05;
const vec3 d = vec3(.161, .502, .725);
const vec3 s = vec3(1., 1., 1.);

const vec3 light_offset = vec3(-.75, .25, -.5);

const float alpha = 64.;
const float gamma = 2.2;

void main() {
  if (hidden != 0.) {
    discard;
  }

  vec3 normal = normalize(frag_normal);

  if (!gl_FrontFacing) {
      normal = -normal;
  }

  if (debug_normals) {
    frag_color = vec4(0.5 * (normal + vec3(1.)), 1.);
  } else {
    vec3 l = normalize(camera_pos + light_offset - frag_pos);

    float lambertian = max(dot(l, normal), 0.);
    float specular = 0.;

    if (lambertian > 0.) {
      vec3 view = normalize(camera_pos - frag_pos);
      vec3 halfway = normalize(l + view);
      float theta = max(dot(halfway, normal), 0.);
      specular = pow(theta, alpha);
    }

    vec3 color = d * (a + lambertian) + .25 * s * specular;
    frag_color = vec4(pow(color, vec3(1. / gamma)), 1.);
  }
}

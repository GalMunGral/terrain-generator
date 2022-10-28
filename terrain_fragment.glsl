#version 300 es

precision highp float;

const vec3 light = vec3(1, -0.5, 1);
const vec3 lightColor = vec3(1, 1, 1);
const float phong_exp = 80.0;

uniform sampler2D terrainTexture;
uniform vec3 eye;

in vec3 vNormal;
in vec2 vTexCoord;

out vec4 fragColor;

void main() { 
    vec3 normal = normalize(vNormal);
    vec4 material = texture(terrainTexture, vTexCoord);

    vec3 l = normalize(light);
    float lambert = clamp(dot(l, normal), 0.0, 1.0);
    vec3 diffuse = lambert * material.rgb;

    vec3 r = 2.0 * dot(normal, l) * normal - l;
    vec3 e = normalize(eye);
    vec3 specular = pow(clamp(dot(r, e), 0.0, 1.0), phong_exp) * lightColor;
    
    fragColor = vec4(diffuse + specular, material.a);
}

#version 300 es

precision highp float;

const vec3 light = vec3(1, -1, 1);
const vec3 lightColor = vec3(1, 1, 1);
const vec4 fogColor = vec4(0.5, 0.5, 0.5, 0.5);
const float phong_exp = 100.0;

uniform bool fogEnabled;
uniform sampler2D terrainTexture;
uniform vec3 eye;

in vec3 vNormal;
in vec2 vTexCoord;
in float vDepth;

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

    vec4 objectColor = vec4(diffuse + specular, material.a);
    float f = exp(-0.002 * vDepth);
    fragColor = fogEnabled ? f * objectColor + (1.0 - f) * fogColor : objectColor;
}

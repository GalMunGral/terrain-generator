#version 300 es

precision highp float;

const vec3 light = vec3(0, 1, 1);
const vec3 lightColor = vec3(1, 1, 1);
const vec3 rockColor = vec3(0.3, 0.3, 0.3);

uniform vec3 eye;

in vec3 vNormal;
in vec3 vColor;

out vec4 fragColor;

void main() { 

    vec3 normal = normalize(vNormal);
    bool steep = abs(dot(normal, vec3(0, 0, 1))) < 0.3;

    vec3 material = float(steep) * rockColor + float(!steep) * vColor;
    float phong_exp = float(steep) * 1.0 + float(!steep) * 80.0;

    vec3 l = normalize(light);
    float lambert = clamp(dot(l, normal), 0.0, 1.0);
    vec3 diffuse = lambert * material;

    vec3 r = 2.0 * dot(normal, l) * normal - l;
    vec3 e = normalize(eye);
    vec3 specular = pow(clamp(dot(r, e), 0.0, 1.0), phong_exp) * lightColor;
    
    fragColor = vec4(diffuse + specular, 1);
}

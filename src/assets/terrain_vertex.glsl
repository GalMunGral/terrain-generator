#version 300 es

uniform mat4 m;
uniform mat4 v;
uniform mat4 p;

in vec4 position;
in vec3 normal;
in vec2 texCoord;

out vec3 vNormal;
out vec2 vTexCoord;
out float vDepth;

void main() {
    gl_Position = p * v * m * position;
    vNormal = normal;
    vTexCoord = texCoord;
    vDepth = -(v * m * position).z;
}

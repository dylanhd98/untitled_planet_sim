#version 140
in vec3 normal;
out vec4 color;

void main() {
    color = vec4(vec3(normal), 1.0);
}
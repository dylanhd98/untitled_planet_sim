 #version 140

in vec3 position;

uniform mat4 world;
uniform mat4 view;
uniform mat4 perspective;

out vec3 normal;

void main() {
    normal = position;
    gl_Position = perspective*view*world*vec4(position, 1.0);
}
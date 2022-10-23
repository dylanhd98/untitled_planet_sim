 #version 140

in vec3 position;

uniform mat4 world;
uniform mat4 view;
uniform mat4 perspective;

void main() {
    gl_Position = world*view*perspective*vec4(position, 1.0);
}
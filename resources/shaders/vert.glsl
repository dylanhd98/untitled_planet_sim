 #version 140

in vec3 position;
in vec2 tex_coords;
out vec2 v_tex_coords;

uniform mat4 world;
uniform mat4 view;
uniform mat4 perspective;

void main() {
    v_tex_coords = tex_coords;
    gl_Position = perspective*view*world*vec4(position, 1.0);
}
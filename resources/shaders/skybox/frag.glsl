#version 330

//coordinate for cubemap
in vec3 v_tex_coords;

out vec4 color;

//cubemap sampler
uniform samplerCube cube;

void main() {
    color = texture(cube, v_tex_coords);
}
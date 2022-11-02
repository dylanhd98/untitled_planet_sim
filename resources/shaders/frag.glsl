#version 140

in vec3 v_normal;
in vec2 v_tex_coords;

out vec4 color;

uniform sampler2D tex;
uniform vec3 to_light;

void main() {
    color = texture(tex, v_tex_coords)*dot(to_light,v_normal);
}
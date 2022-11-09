 #version 330

//shape data in
in vec3 position;

//data out for texture cube pos 
out vec3 v_tex_coords;

//uniforms
uniform mat4 view;
uniform mat4 perspective;

void main() {
    //get new pos
    vec4 pos = perspective*view*vec4(position, 1.0);
    //asign pos to tex_coords for cube map
    v_tex_coords = position;
    //translate vertices to screen, no z value
    gl_Position = pos.xyww; 
}
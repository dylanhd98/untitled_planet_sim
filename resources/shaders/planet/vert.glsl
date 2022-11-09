 #version 330

//shape data in
in vec3 position;
in vec3 normal;

//cell data in
in float height;
in float humidity;
in float temperature;

//data for frag
out vec3 v_normal;
out vec2 v_tex_coords;
out float v_height;

//uniforms
//uniform mat4 world;
uniform mat4 view;
uniform mat4 perspective;

void main() {
    v_tex_coords = vec2((1.0+humidity)*0.5,temperature);
    v_height = height;

    vec3 new_pos = position;
    if(height>0.0){
        new_pos *= (1+(height/15));
    }
    //v_normal = transpose(inverse(mat3(world))) * normal;
    v_normal = normal;

    gl_Position = (perspective*view*vec4(new_pos, 1.0)); 
}
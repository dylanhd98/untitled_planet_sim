 #version 330

//shape data in
in vec3 position;

//cell data in
in float height;
in float humidity;
in float temperature;

//data for geometry shader
out VS_OUT {
    vec3 pos;
    vec2 tex_coords;
    float height;
} vs_out;


//uniforms
//uniform mat4 world;
uniform mat4 view;
uniform mat4 perspective;

void main() {
    vs_out.tex_coords = vec2((1.0+humidity)*0.5,temperature);
    vs_out.height = height;

    vec3 new_pos = position;
    if(height>0.0){
        new_pos *= (1+(height*0.05));
    }
    vs_out.pos = new_pos;

    //v_normal = transpose(inverse(mat3(world))) * normal;
    //v_normal = normal;

    gl_Position = (perspective*view*vec4(new_pos, 1.0)); 
}
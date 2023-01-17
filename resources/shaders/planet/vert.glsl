 #version 330 core

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
    //float humidity;
    //float temperature;
    float height;
} vs_out;

//uniforms
uniform mat4 view;
uniform mat4 perspective;
uniform float terra_scale;

void main() {
    vs_out.tex_coords = vec2((1.0+humidity)*0.5,temperature);
    vs_out.height = height;

    vec3 new_pos = position;
    if(height>0.0){
        new_pos *= (1+(height*terra_scale));
    }
    vs_out.pos = new_pos;

    gl_Position = (perspective*view*vec4(new_pos, 1.0)); 
}
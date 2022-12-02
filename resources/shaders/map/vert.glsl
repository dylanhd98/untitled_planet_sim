#version 330

//vertex shader to transform 3d points on sphere into flat  one

//shape data in
in vec3 position;

//cell data in
in float height;
in float humidity;
in float temperature;

//out for frag shader
out vec3 v_normal;
out vec2 v_tex_coords;
out float v_height;

void main() {
    //normal stuff for frag shader, not map related
    v_tex_coords = vec2((1.0+humidity)*0.5,temperature);
    v_height = height;
    v_normal = vec3(1.0,0.0,0.0);//TEMPORARY, BE RID OF THIS

    //mercator projection
    //float u = 0.5+(atan2(position.z,position.y)/6.28318530718);
    float u = position.x;
    float v = (position.y);

    gl_Position = vec4(u,v,0.0,1.0); 
}
#version 330 core

//in from geometry shader
in vec3 v_normal;
in vec2 v_tex_coords;
in float v_height;

//out to whatever this renders too
out vec4 color;

//uniforms
uniform sampler2D tex;
uniform vec3 to_light;

void main() {
    float brightness = max(dot(to_light,v_normal),0.1);

    if(v_height>0.0){
        color = vec4(vec3(texture(tex, v_tex_coords)*brightness),1.0);
    }
    else{
        color = vec4(vec3(mix(vec4(0.0,0.02,0.15,1.0),vec4(0.0,0.0,0.10,1.0),abs(v_height*0.5))* brightness),1.0);
    }
    //color = vec4(vec3((1+v_height)/2),1.0);
    //color = vec4(1.0);
}
#version 150

in vec3 v_normal;
in vec2 v_tex_coords;

in float v_height;

out vec4 color;

uniform sampler2D tex;
uniform vec3 to_light;

void main() {
    float brightness = max(0.1,dot(to_light,v_normal));

    if(v_height<0.0){
        color = vec4(0.0,0.0,0.15,1.0)* brightness;
    }
    else{
        color = texture(tex, v_tex_coords)*brightness;
    }
    //color = vec4(vec3(v_height),1.0);
    //color = texture(tex, v_tex_coords)*dot(to_light,v_normal);
}
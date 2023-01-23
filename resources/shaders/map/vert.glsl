#version 330 core

#define PI 3.141592653589793

//shape data in
in vec3 position;

//cell data in
in float height;
in float humidity;
in float temperature;

//out for frag shader
out vec3 v_normal;
out float v_humidity;
out float v_temperature;
out float v_height;

float angle(vec2 pos){
    if(!(abs(pos.x)>0)) return 0.0;
    return sign(pos.x)*acos(pos.y/length(pos.xy));
}

vec3 cart_to_sphere(vec3 pos){
    //y axis is the zenith
    float rad = 1.0;
    //issue with this?
    float azimuth = angle(pos.xz);//think longitude-> east-west measurement (-pi -> pi)
    float polar = acos(pos.y/rad);// think latitude-> north-south measurement (0 -> pi)
    return vec3(rad,azimuth,polar);//standard order layout for these coords use in maths
}

vec2 proj_to_screen(vec2 proj){
    //y: (0 -> pi)->(-1 -> 1)
    //x: (-pi -> pi)->(-1 -> 1)
    return vec2(proj.x/PI,-(((proj.y*2)-PI)/PI));
}

//projections
vec2 cylindrical_equal_area (vec3 sphere){
    float x = sphere.y;
    float y = sin(sphere.z);
    return vec2(x,y);
}

vec2 mercator(vec3 sphere){
    float x = sphere.x*(sphere.y);
    float y = sphere.x*log(tan((PI/4)+(sphere.z/2)));
    
    return vec2(x,y);
}

vec2 equirect(vec3 sphere){
    float x = (sphere.x*(sphere.y));
    float y = (sphere.x*(sphere.z));
    return vec2(x, y);
}

void main() {
    //normal stuff for frag shader, not map related
    v_humidity = humidity;
    v_temperature = temperature;
    v_height = height;
    v_normal = position;//TEMPORARY, BE RID OF THIS

    vec3 sphere = cart_to_sphere(position);
    vec2 proj = equirect(sphere);
    
    gl_Position = vec4(proj_to_screen(proj),0.0,1.0); 
}


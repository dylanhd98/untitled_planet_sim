#version 330

#define PI 3.1415926538
#define MERCATOR_GL_PI 3.1415926
#define MERCATOR_GL_TILE_SCALE 512.0/(MERCATOR_GL_PI*2.0)
#define mercator_gl_angleDerivatives vec3(1.0,0.0,0.0)
#define DEGREES_TO_RADIANS 3.1415926 / 180

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

vec3 cart_to_sphere(vec3 pos){
    //y axis is the zenith
    float rad = length(pos);
    //issue with this?
    float azimuth = sign(pos.x)*acos(pos.z/length(vec2(pos.x,pos.z)));//think longitude-> east-west measurement
    float polar = acos(pos.y/rad);// think latitude-> north-south measurement
    return vec3(rad,azimuth,polar);//standard order layout for these coords use in mathstextbooks
}

//vec2 stereographic (vec3 sphere){}

vec2 equirect(vec3 sphere){
    float x = (sphere.x*(sphere.y)*cos(0.0));
    float y = (sphere.x*(sphere.z));
    return vec2(x,y);
}

vec2 mercator(vec3 sphere){
    float x = sphere.x*(sphere.y);
    float y = sphere.x*log(tan((PI/4)+(sphere.z/2)));
    
    return vec2(x,y);
}


void main() {
    //normal stuff for frag shader, not map related
    v_tex_coords = vec2((1.0+humidity)*0.5,temperature);
    v_height = height;
    v_normal = position;//TEMPORARY, BE RID OF THIS

    vec3 sphere = cart_to_sphere(position);
    vec2 proj = equirect(sphere);
    
    gl_Position = vec4(sphere.y, position.y,0.0,1.0); 
}


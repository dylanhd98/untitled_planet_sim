#version 330 core
layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

//in from vertex
in VS_OUT {
    vec3 pos;
    float humidity;
    float temperature;
    float height;
    float water;
} gs_in[];

//out for frag
out vec3 v_normal;
out float v_humidity;
out float v_temperature;
out float v_height;
out float v_water;


void main() {
    //stores positions of vertices
    vec3 v0 = gs_in[0].pos.xyz;
    vec3 v1 = gs_in[1].pos.xyz;
    vec3 v2 = gs_in[2].pos.xyz;
    
    vec3 normal = normalize(cross((v0 - v1),(v0 - v2)));
    if (dot(normal,v0)<0)
    {
        normal*=-1;
    }

    v_normal = normal;

    for(int i=0;i<3;i++){
        gl_Position = gl_in[i].gl_Position;
        v_humidity = gs_in[i].humidity;
        v_temperature = gs_in[i].temperature;
        v_height = gs_in[i].height;
        v_water = gs_in[i].water;
        EmitVertex();
    }
    EndPrimitive();
}  
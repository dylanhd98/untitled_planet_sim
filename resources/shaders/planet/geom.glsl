#version 330 core
layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

//in from vertex
in VS_OUT {
    vec3 pos;
    vec2 tex_coords;
    float height;
} gs_in[];

//out for frag
out vec3 v_normal;
out vec2 v_tex_coords;
out float v_height;


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

    gl_Position = gl_in[0].gl_Position;
    v_tex_coords = gs_in[0].tex_coords;
    v_height = gs_in[0].height;
    EmitVertex();

    gl_Position = gl_in[1].gl_Position;
    v_tex_coords = gs_in[1].tex_coords;
    v_height = gs_in[1].height;
    EmitVertex();
    
    gl_Position = gl_in[2].gl_Position;
    v_tex_coords = gs_in[2].tex_coords;
    v_height = gs_in[2].height;
    EmitVertex();

    EndPrimitive();
}  
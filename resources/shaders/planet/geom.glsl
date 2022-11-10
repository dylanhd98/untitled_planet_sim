#version 330 core
layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;


out vec3 v_normal;
out vec2 v_tex_coords;
out float v_height;

in VS_OUT {
    vec2 tex_coords;
    float height;
} gs_in[];

void main() {    
    vec3 v0 = gl_in[0].gl_Position.xyz;
    vec3 v1 = gl_in[1].gl_Position.xyz;
    vec3 v2 = gl_in[2].gl_Position.xyz;
    
    vec3 normal = normalize(cross((v0 - v1),(v0 - v2)));
    if (dot(normal,v0)<0)
    {
        normal*=-1;
    }

    gl_Position = gl_in[0].gl_Position;
    v_tex_coords = gs_in[0].tex_coords;
    v_height = gs_in[0].height;
    v_normal = normal;
    EmitVertex();
    gl_Position = gl_in[1].gl_Position;
    v_tex_coords = gs_in[1].tex_coords;
    v_height = gs_in[1].height;
    v_normal = normal;
    EmitVertex();
    gl_Position = gl_in[2].gl_Position;
    v_tex_coords = gs_in[2].tex_coords;
    v_height = gs_in[2].height;
    v_normal = normal;
    EmitVertex();
    EndPrimitive();
}  
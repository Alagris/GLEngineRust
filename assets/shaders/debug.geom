#version 330 core
layout (triangles) in;
layout (line_strip, max_vertices = 6) out;
uniform mat4 MVP;
uniform mat4 V;
uniform mat4 P;
uniform mat4 M;
//uniform float normalLength;
in VS_OUT {
    vec3 normal;
} gs_in[];
void main() {

    int i;
    for(i=0; i<gl_in.length(); i++)
    {
        vec4 Pos = gl_in[i].gl_Position;
        vec3 N = gs_in[i].normal;

        gl_Position = MVP * (Pos + vec4(normalize(N)*0.8 , 1.0));
        EmitVertex();

        gl_Position = MVP * Pos;
        EmitVertex();

        EndPrimitive();

//        gl_Position = gl_in[i].gl_Position;
//        EmitVertex();
//        EndPrimitive();
    }
}
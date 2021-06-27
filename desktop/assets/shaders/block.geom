#version 330 core
layout (triangles) in;
layout (line_strip, max_vertices = 18) out;
uniform mat4 MVP;
uniform mat4 V;
uniform mat4 P;
uniform mat4 M;
out vec3 color;
//uniform float normalLength;
in VS_OUT {
    vec3 normal;
    vec3 tangent;
    vec3 bitangent;
} gs_in[];
void main() {

    int i;
    for(i=0; i<gl_in.length(); i++)
    {
        vec4 Pos = gl_in[i].gl_Position;
        vec3 N = gs_in[i].normal;
        vec3 T = gs_in[i].tangent;
        vec3 B = gs_in[i].bitangent;
        vec4 MVPPos = MVP * Pos;


        color = vec3(1,0,0);
        gl_Position = MVP * (Pos + vec4(N*0.1, 0.0));
        EmitVertex();

        gl_Position = MVPPos;
        EmitVertex();

        EndPrimitive();


        color = vec3(0,1,0);
        gl_Position = MVP * (Pos + vec4(normalize(T)*0.1, 0.0));
        EmitVertex();

        gl_Position = MVPPos;
        EmitVertex();

        EndPrimitive();


        color = vec3(0,0,1);
        gl_Position = MVP * (Pos + vec4(normalize(B)*0.1, 0.0));
        EmitVertex();

        gl_Position = MVPPos;
        EmitVertex();

        EndPrimitive();

//        gl_Position = gl_in[i].gl_Position;
//        EmitVertex();
//        EndPrimitive();
    }
}
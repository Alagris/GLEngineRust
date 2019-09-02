#version 330 core
layout (location = 0) in vec3 vertexPosition_modelspace;
layout (location = 2) in vec2 vertexUV;
layout (location = 3) in vec3 vertexNormal_modelspace;
layout (location = 4) in vec3 vertexTangent_modelspace;
layout (location = 5) in vec3 vertexBitangent_modelspace;
out VS_OUT {
    vec3 normal;
    vec3 tangent;
    vec3 bitangent;
} vs_out;

void main()
{
    gl_Position =  vec4(vertexPosition_modelspace, 1.0);
    vs_out.normal =  vertexNormal_modelspace;
    vs_out.tangent =  vertexTangent_modelspace;
    vs_out.bitangent =  vertexBitangent_modelspace;

}

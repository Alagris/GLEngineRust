#version 330 core
layout (location = 0) in vec3 vertexPosition_modelspace;
layout (location = 2) in vec2 vertexUV;
layout (location = 3) in vec3 vertexNormal_modelspace;

out VS_OUT {
    vec3 normal;
} vs_out;

void main()
{
    gl_Position =  vec4(vertexPosition_modelspace, 1.0);
    vs_out.normal =  vertexNormal_modelspace;
}

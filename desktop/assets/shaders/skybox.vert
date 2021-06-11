#version 330 core

layout (location = 0) in vec3 position;

uniform mat4 VP;

out vec3 UV;


void main()
{
    UV = position;
    gl_Position = VP * vec4(position,1.0);
}
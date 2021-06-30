
#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 vertexUV;

out vec2 UV;

uniform vec2 offsets[100];
uniform mat4 MVP;

void main()
{
    vec2 offset = offsets[gl_InstanceID];
    gl_Position = MVP * vec4(aPos + offset, 0.0, 1.0);
    UV = vertexUV;
}  
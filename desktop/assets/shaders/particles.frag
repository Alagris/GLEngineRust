
#version 330 core
out vec4 FragColor;
in vec3 color;
uniform sampler2D myTextureSampler;
void main()
{
    vec3 MaterialDiffuseColor = texture( myTextureSampler, texCoords ).rgb;
    FragColor = vec4(MaterialDiffuseColor, 1.0);
}
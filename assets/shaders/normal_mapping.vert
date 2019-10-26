#version 330 core

layout (location = 0) in vec3 vertexPosition_modelspace;
//layout (location = 1) in vec3 Color;
layout (location = 2) in vec2 vertexUV;
layout (location = 3) in vec3 vertexNormal_modelspace;
layout (location = 4) in vec3 vertexTangent_modelspace;
layout (location = 5) in vec3 vertexBitangent_modelspace;

uniform mat4 MVP;
uniform mat3 MV3x3;
//uniform mat4 MV;
uniform mat4 V;
uniform mat4 M;
//x,y,z = position of light source
uniform vec3 lightSource;


out vec2 UV;
out vec3 Position_worldspace;
out vec3 Normal_cameraspace;
out vec3 EyeDirection_cameraspace;
out vec3 LightDirection_cameraspace;

out vec3 LightDirection_tangentspace;
out vec3 EyeDirection_tangentspace;
out vec3 Position_tangentspace;

void main()
{


    mat4 MV = V * M;
    // Output position of the vertex, in clip space : MVP * position
    gl_Position =  MVP * vec4(vertexPosition_modelspace,1);

    // Position of the vertex, in worldspace : M * position
    Position_worldspace = (M * vec4(vertexPosition_modelspace,1)).xyz;

    // Vector that goes from the vertex to the camera, in camera space.
    // In camera space, the camera is at the origin (0,0,0).
    vec3 vertexPosition_cameraspace = ( MV * vec4(vertexPosition_modelspace,1)).xyz;
    EyeDirection_cameraspace = vec3(0,0,0) - vertexPosition_cameraspace;

    // Vector that goes from the vertex to the light, in camera space. M is ommited because it's identity.
    vec3 LightPosition_cameraspace = ( V * vec4(lightSource,1)).xyz;
    LightDirection_cameraspace = LightPosition_cameraspace + EyeDirection_cameraspace;

    // Normal of the the vertex, in camera space
    vec3 vertexNormal_cameraspace = MV3x3 * normalize(vertexNormal_modelspace);
    vec3 vertexTangent_cameraspace = MV3x3 * normalize(vertexTangent_modelspace);
    vec3 vertexBitangent_cameraspace = MV3x3 * normalize(vertexBitangent_modelspace);
    mat3 TBN = transpose(mat3(
        vertexTangent_cameraspace,
        vertexBitangent_cameraspace,
        vertexNormal_cameraspace
    ));
    LightDirection_tangentspace = TBN * LightDirection_cameraspace;
    EyeDirection_tangentspace =  TBN * EyeDirection_cameraspace;
    Position_tangentspace = TBN * Position_worldspace;
    // UV of the vertex. No special space for this one.
    UV = vertexUV;


//    gl_Position = MVP * vec4(Position, 1.0);
//    OUT.FragPosView = vec3(MV * vec4(Position, 1.0));
//    OUT.FragPos = vec3(M * vec4(Position, 1.0));
//    OUT.Normal = Normal;
//    OUT.UV = TextureUV;
}
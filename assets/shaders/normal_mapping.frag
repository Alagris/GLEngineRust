#version 330 core


uniform sampler2D myTextureSampler;
uniform sampler2D normalMap;
uniform sampler2D depthMap;
uniform vec3 lightSource;
uniform vec4 lightColor;
uniform mat4 MV;

in vec2 UV;
in vec3 Position_worldspace;
in vec3 Normal_cameraspace;
in vec3 EyeDirection_cameraspace;
in vec3 LightDirection_cameraspace;


in vec3 LightDirection_tangentspace;
in vec3 EyeDirection_tangentspace;
in vec3 Position_tangentspace;
// Ouput data
out vec3 color;

float height_scale = 0.05;

vec2 ParallaxMapping(vec2 texCoords, vec3 viewDir)
{
    float height =  texture(depthMap, texCoords).r;
    vec2 p = viewDir.xy / viewDir.z * (height * height_scale);
    return texCoords - p;
}

vec2 SteepParallaxMapping(vec2 texCoords, vec3 viewDir)
{
    // number of depth layers
    const float numLayers = 10;
    // calculate the size of each layer
    float layerDepth = 1.0 / numLayers;
    // depth of current layer
    float currentLayerDepth = 0.0;
    // the amount to shift the texture coordinates per layer (from vector P)
    vec2 P = viewDir.xy * height_scale;
    vec2 deltaTexCoords = P / numLayers;
    vec2  currentTexCoords     = texCoords;
    float currentDepthMapValue = texture(depthMap, currentTexCoords).r;

    while(currentLayerDepth < currentDepthMapValue)
    {
        // shift texture coordinates along direction of P
        currentTexCoords -= deltaTexCoords;
        // get depthmap value at current texture coordinates
        currentDepthMapValue = texture(depthMap, currentTexCoords).r;
        // get depth of next layer
        currentLayerDepth += layerDepth;
    }

    return currentTexCoords;
}

void main()
{
    // offset texture coordinates with Parallax Mapping
    vec3 viewDir   = normalize(EyeDirection_tangentspace - Position_tangentspace);
    vec2 texCoords = SteepParallaxMapping(UV,  viewDir);
    if(texCoords.x > 1.0 || texCoords.y > 1.0 || texCoords.x < 0.0 || texCoords.y < 0.0)
        discard;
    vec3 TextureNormal_tangentspace = normalize(texture( normalMap, texCoords ).rgb*2.0 - 1.0);

    // Light emission properties
    // You probably want to put them as uniforms
    vec3 LightColor = vec3(lightColor);
    float LightPower = lightColor.w;

    // Material properties
    vec3 MaterialDiffuseColor = texture( myTextureSampler, texCoords ).rgb;
    vec3 MaterialAmbientColor = vec3(0.1,0.1,0.1) * MaterialDiffuseColor;
    vec3 MaterialSpecularColor = vec3(0.3,0.3,0.3);

    // Distance to the light
    float distance = length( lightSource - Position_worldspace ) / LightPower;

    // Normal of the computed fragment, in camera space
    vec3 n = TextureNormal_tangentspace;
    // Direction of the light (from the fragment to the light)
    vec3 l = normalize( LightDirection_tangentspace );
    // Cosine of the angle between the normal and the light direction,
    // clamped above 0
    //  - light is at the vertical of the triangle -> 1
    //  - light is perpendicular to the triangle -> 0
    //  - light is behind the triangle -> 0
    float cosTheta = abs( dot( n,l ));

    // Eye vector (towards the camera)
    vec3 E = normalize(EyeDirection_tangentspace);
    // Direction in which the triangle reflects the light
    vec3 R = reflect(-l,n);
    // Cosine of the angle between the Eye vector and the Reflect vector,
    // clamped to 0
    //  - Looking into the reflection -> 1
    //  - Looking elsewhere -> < 1
    float cosAlpha = clamp( dot( E,R ), 0,1 );

    color =
        // Ambient : simulates indirect lighting
        MaterialAmbientColor +
        // Diffuse : "color" of the object
        MaterialDiffuseColor * LightColor  * cosTheta / (distance*distance) +
        // Specular : reflective highlight, like a mirror
        MaterialSpecularColor * LightColor * pow(cosAlpha,32) / (distance*distance);


}
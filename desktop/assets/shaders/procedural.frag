#version 330 core

uniform vec3 lightSource;
uniform vec4 lightColor;
uniform mat4 MV;

in vec2 UV;
in vec3 Position_worldspace;
in vec3 Normal_cameraspace;
in vec3 EyeDirection_cameraspace;
in vec3 LightDirection_cameraspace;

// Ouput data
out vec3 color;

vec3 c(float h){
    vec3 bottom = vec3(0., 0., 153./255.) ;
    vec3 surface = vec3(102./255., 204./255., 1.);
    vec3 shore = vec3(1., 1, 204./255.);
    vec3 plain = vec3(0., 153./255., 51./255.);
    float sea_lvl = 0.5;
    float grass = 1.;
    float top = 3.;
    if(h > sea_lvl){
        if(h > grass){
            if( h> top){
                return mix(plain,vec3(1.,1.,1.),(h-top)/(top));
            }
            return mix(shore,plain,(h-grass)/(top-grass));
        }
        return mix(surface,shore,(h-sea_lvl)/(grass-sea_lvl));
    }
    return mix(bottom,surface,(h)/(sea_lvl));
}

void main()
{

    // Light emission properties
    // You probably want to put them as uniforms
    vec3 LightColor = vec3(lightColor);
    float LightPower = lightColor.w;


    // Material properties
    vec3 MaterialDiffuseColor = c(Position_worldspace.y);
    vec3 MaterialAmbientColor = vec3(0.1,0.1,0.1) * MaterialDiffuseColor;
    vec3 MaterialSpecularColor = vec3(0.3,0.3,0.3);

    // Distance to the light
    float distance = length( lightSource - Position_worldspace ) / LightPower;

    // Normal of the computed fragment, in camera space
    vec3 n = normalize( Normal_cameraspace );
    // Direction of the light (from the fragment to the light)
    vec3 l = normalize( LightDirection_cameraspace );
    // Cosine of the angle between the normal and the light direction,
    // clamped above 0
    //  - light is at the vertical of the triangle -> 1
    //  - light is perpendicular to the triangle -> 0
    //  - light is behind the triangle -> 0
    float cosTheta = abs( dot( n,l ));

    // Eye vector (towards the camera)
    vec3 E = normalize(EyeDirection_cameraspace);
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
        MaterialDiffuseColor * LightColor  * cosTheta / (distance) +
        // Specular : reflective highlight, like a mirror
        MaterialSpecularColor * LightColor * pow(cosAlpha,32) / (distance*distance);


//    float ambientStrength = 0.1;
//    float lightStrnegth = lightColor.w;
//    vec3 ambient = ambientStrength * vec3(lightColor);
//
//    vec3 norm = normalize(IN.Normal);
//    vec3 lightDir = normalize(lightSource - IN.FragPos);
//    float diff = abs(dot(norm, lightDir));
//    vec3 diffuse = diff * vec3(lightColor);
//
//    float specularStrength = 0.5;
//    vec3 viewDir = normalize(-IN.FragPosView);
//    vec3 reflectDir = reflect(-lightDir, norm);
//    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
//    vec3 specular = specularStrength * spec * vec3(lightColor);
//
//
//    float lightDistance =  distance(lightSource,IN.FragPos)/ lightColor.w ;
//    vec4 objectColor = texture( myTextureSampler, IN.UV ).rgba;
//    vec4 result = vec4(ambient + diffuse / (lightDistance*lightDistance ),1.0) * objectColor ;
//    Color = result;
}
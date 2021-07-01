#version 330 core
layout (location = 11) in uint block_face;
out vec2 UV;
uniform mat4 MVP;

void main()
{

    const vec3 A = vec3(0,0,0);// left bottom front
    const vec3 B = vec3(1,0,0);// right bottom front
    const vec3 C = vec3(1,0,1);// right bottom back
    const vec3 D = vec3(0,0,1);// left bottom back
    const vec3 E = vec3(0,1,0);// left top front
    const vec3 F = vec3(1,1,0);// right top front
    const vec3 G = vec3(1,1,1);// right top back
    const vec3 H = vec3(0,1,1);// left top back

    const vec3[6*6] vertices = vec3[6*6](
    // YPlus ortientation = block's top face
    G, F, E, G, E, H,
     // YMinus ortientation = block's bottom face
    C, A, B, C, D, A,
     // XPlus ortientation = block's right face
    G, B, F, B, G, C,
    // XMinus ortientation = block's left face
    A, D, H, A, H, E,
    // ZPlus ortientation = block's back face
    H, D, C, G, H, C,
    // ZMinus ortientation = block's front face
    F, B, A, F, A, E
    );

    const vec2 K = vec2(0,0);// left bottom
    const vec2 L = vec2(1,0);// right bottom
    const vec2 M = vec2(1,1);// right top
    const vec2 N = vec2(0,1);// left top

    const vec2[6*6] texture_uv = vec2[6*6](
        // YPlus ortientation = block's top face
        M, L, K, M, K, N,
        // YMinus ortientation = block's bottom face
        M, K, L, M, N, K,
        // XPlus ortientation = block's right face
        M, K, N, K, M, L,
        // XMinus ortientation = block's left face
        L, K, N, L, N, M,
        // ZPlus ortientation = block's back face
        M, L, K, N, M, K,
        // ZMinus ortientation = block's front face
        M, L, K, M, K, N
    );
    uint max_byte = uint(255);
    uint x = block_face & max_byte;
    uint y = (block_face>>8) & max_byte;
    uint z = (block_face>>16) & max_byte;
    uint orientation = block_face>>24;
    vec3 block_position = vec3(float(x),float(y),float(z));
    vec3 vertex_pos = vertices[orientation*uint(6) + uint(gl_VertexID)];
    gl_Position = MVP * vec4(vertex_pos+block_position, 1.0);
    UV = texture_uv[orientation*uint(6) + uint(gl_VertexID)];
}

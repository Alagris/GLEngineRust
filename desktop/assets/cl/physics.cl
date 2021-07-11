
// Entity position is defined as position of its eyes
// Hitbox is defined by two vectors. First one specifies the lower left front corner (hitbox_vector1 + position = hitbox_from).
// Second one specifies the upper right back corner (hitbox_vector2 + position = hitbox_to). Hitboxes are always rectangular cuboids.
static const float3 hitbox_per_entity_type[1][2] = {
    {float3(-0.4, -1.5, -0.4), float3(0.4, 0.3, 0.4)}
};

struct Entity{
    float3 position,
    float3 rotation,
    uint type,
}

__kernel void main(__global uint * world, size_t world_width, size_t world_depth, __global Entity * entities){
    buffer[get_global_id(0)] = 1.53;
}
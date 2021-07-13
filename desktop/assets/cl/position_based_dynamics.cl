
#define chunk_height 256
#define chunk_width 16
#define chunk_depth 16
#define collision_slots_per_block_side 8
//because a single block is 16x16x16 pixels, this means that every particles in 2x2x2 pixels.
#define collision_slots_per_block collision_slots_per_block_side*collision_slots_per_block_side*collision_slots_per_block_side
#define collision_slots_width chunk_width*collision_slots_per_block_side+4
//the slots at the boundaries of chunks will need to be checked twice in order to ensure that collision detection works properly
//at chunk borders. Hence +4 is added (+2 slots on the left side of chunk and +2 slots on the right side)
#define collision_slots_depth chunk_depth*collision_slots_per_block_side+4
#define collision_slots_height chunk_height*collision_slots_per_block_side
#define world_height chunk_height
#define collision_slots_side_len 1./collision_slots_per_block_side
#define chunk_border_size (float3)(2., 0, 2.)*collision_slots_side_len
#define collision_slots_size (float3)(collision_slots_width, collision_slots_height, collision_slots_depth)*collision_slots_side_len
#define diameter sqrt(3*collision_slots_side_len*collision_slots_side_len)
#define radius diameter/2.
// this is the smallest possible
// radius that still guarantees at most one particle will fall into each collision slot. However, note that
// collision_slots_side_len/2 is the largest possible radius which could guarantee that a single particle can reach no further
// that its direct neighbouring collision cells. Unfortunately that collision_slots_side_len/2 is less than
// sqrt(3*collision_slots_side_len*collision_slots_side_len)/2 , which means that we need to check more than 27 cells
// for possible collisions. This is fine, because the alternative would be storing up to 4 particles in a single cell, which
// in worst case scenario might result in the same number of particle distance comparisons. However, building a parallel algorithm for
// storing multiple particles in a single cell is tricky and requires careful planning to avoid data races.

int3 collision_slot_position(float3 position_in_chunk){
    return convert_int3( (position_in_chunk+chunk_border_size)/collision_slots_side_len);
}
int collision_slot_idx(int3 slot_position){
    return slot_position.x  + (slot_position.z + slot_position.y * collision_slots_depth) * collision_slots_width;
}
__kernel void collision_detection(
                   __global uint * world,
                   __global unsigned short * chunk_collision_slots, // this assumes the precondition that chunk_collision_slots is filled with -1 (which is equal ot maximum value for uint)
                   int world_width,
                   int world_depth,
                   __global float * particle_inv_masses,
                   __global float3 * particle_positions,
                   __global float3 * particle_collision_impulses,
                   __global float3 * particle_velocities) {
    int how_many_chunks_in_x_axis = world_width/chunk_width; //it is assumed that world_width is a multiple of chunk_width
    int how_many_chunks_in_z_axis = world_depth/chunk_depth; //it is assumed that world_depth is a multiple of chunk_depth
    for(int chunk_x = 0; chunk_x < how_many_chunks_in_x_axis ; chunk_x++){
        for(int chunk_z = 0; chunk_z < how_many_chunks_in_z_axis ; chunk_z++){
            float3 chunk_offset = (float3)(chunk_x*chunk_width,0,chunk_z*chunk_depth);
            float3 position = particle_positions[get_global_id(0)];
            float3 velocity = particle_velocities[get_global_id(0)];
            float3 particle_collision_impulse = (float3)(0,0,0);
            float3 position_in_chunk = position - chunk_offset;
            if(all(islessequal(-chunk_border_size, position_in_chunk))&&all(islessequal(position_in_chunk, collision_slots_size-chunk_border_size))){
                int3 particle_slot_position = collision_slot_position(position_in_chunk);
                int particle_slot_idx = collision_slot_idx(particle_slot_position);
                chunk_collision_slots[particle_slot_idx] = get_global_id(0);
                float3 position_in_slot = position_in_chunk - convert_float3(particle_slot_position)*collision_slots_side_len;
                int2 x_range = particle_slot_position.x + (int2)(-1,2) - 1*(position_in_slot.x<0.5);
                int2 y_range = particle_slot_position.y + (int2)(-1,2) - 1*(position_in_slot.y<0.5);
                int2 z_range = particle_slot_position.z + (int2)(-1,2) - 1*(position_in_slot.z<0.5);
                for(int y=y_range.x;y<=y_range.y;y++){
                    if (0 <= y && y < chunk_height) {
                        for(int x=x_range.x;x<=x_range.y;x++) {
                            for(int z=z_range.x;z<=z_range.y;z++) {
                                int neighbour_slot_idx = collision_slot_idx((int3)(x,y,z));
                                int neighbour_idx = (int)chunk_collision_slots[neighbour_slot_idx];
                                if(neighbour_idx != -1){
                                    float3 neighbour_position = particle_positions[neighbour_idx];
                                    if(distance(neighbour_position, position)<=diameter){
                                        //collision detected!
                                        float3 neighbour_velocity = particle_velocities[neighbour_idx];
                                        float3 collision_normal = normalize(position - neighbour_position);
                                        float3 collision_reflected_velocity = velocity - 2*dot(position,collision_normal)*collision_normal;
                                        particle_collision_impulse += collision_reflected_velocity;
                                    }
                                }
                            }
                        }
                    }
                }
                particle_collision_impulses[get_global_id(0)] = particle_collision_impulse;
                chunk_collision_slots[particle_slot_idx] = -1; // cleanup
            }
        }
    }
}

struct __attribute__ ((packed)) VertexSizeAlphaClr
{
    float3 pos;
    float size;
    float3 clr;
};

__kernel void test(__global struct VertexSizeAlphaClr * particles) {
    particles[get_global_id(0)].pos.y += 0.001;
}
/*
__kernel void project_constraints(
                   __global float * particle_inv_masses,
                   __global float3 * particle_positions,
                   __global float3 * particle_collision_impulses,
                   __global float3 * particle_velocities) {
    
}
*/
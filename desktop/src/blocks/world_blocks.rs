use crate::render_gl::data::{u8_u8_u8_u8};
use crate::render_gl::data::VertexAttribPointers;
use crate::render_gl::data::VertexAttrib;
use crate::render_gl::util::init_array;
use crate::blocks::block_properties::{BLOCKS, STONE};
use std::fmt::{Display, Formatter};
use crate::render_gl::buffer::{BufferDynamicDraw, DynamicBuffer};
use crate::render_gl::instanced_logical_model::InstancedLogicalModel;
use crate::render_gl::Program;
use crate::render_gl::shader::UniformVec3fv;
use crate::blocks::block::Block;
use crate::blocks::face_orientation::FaceOrientation;
use crate::blocks::chunk_faces::ChunkFaces;
use crate::blocks::world_size::WorldSize;


pub struct WorldBlocks{
    blocks: Vec<Block>,
    size:WorldSize
}


impl WorldBlocks {
    pub fn size(&self)->&WorldSize{
        &self.size
    }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.blocks[self.size().block_pos_into_world_idx(x,y,z)]
    }
    pub fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        let idx = self.size().block_pos_into_world_idx(x,y,z);
        &mut self.blocks[idx]
    }

    pub fn new(size: WorldSize) -> Self {
        Self { size, blocks:vec![Block::air();size.world_volume()] }
    }

    pub fn no_update_remove_block(&mut self, x: usize, y: usize, z: usize) {
        self.no_update_set_block(x, y, z, Block::air())
    }
    pub fn no_update_set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        *self.get_block_mut(x, y, z) = block
    }
    pub fn no_update_fill(&mut self, from_x: usize, from_y: usize, from_z: usize, width: usize, height: usize, depth: usize, block: Block) {
        for x in from_x..(from_x + width) {
            for y in from_y..(from_y + height) {
                for z in from_z..(from_z + depth) {
                    self.no_update_set_block(x, y, z, block);
                }
            }
        }
    }
    pub fn no_update_outline(&mut self, from_x: usize, from_y: usize, from_z: usize, width: usize, height: usize, depth: usize, block: Block) {
        self.no_update_fill(from_x, from_y, from_z,width, 1, depth, block);
        if height > 1 {
            self.no_update_fill(from_x, from_y + height - 1, from_z , width, 1, depth, block);
            if height > 2 {
                for y in from_y+1..(from_y + height-1) {
                    if width>0 {
                        for z in from_z..(from_z + depth) {
                            self.no_update_set_block(from_x, y, z, block);
                            self.no_update_set_block(from_x + width - 1, y, z, block);
                        }
                    }
                    if depth>2 {
                        for x in from_x..(from_x + width) {
                            self.no_update_set_block(x, y, from_z, block);
                            self.no_update_set_block(x, y, from_z+depth-1, block);
                        }
                    }
                }
            }
        }

    }
    pub fn no_update_fill_level(&mut self, from_y: usize, height: usize, block: Block) {
        self.no_update_fill(0, from_y, 0, self.size().world_width(), height, self.size().world_depth(), block)
    }
    pub fn no_update_replace(&mut self, from_x: usize, from_y: usize, from_z: usize, width: usize, height: usize, depth: usize, old_block: Block, new_block: Block) {
        for x in from_x..(from_x + width) {
            for y in from_y..(from_y + height) {
                for z in from_z..(from_z + depth) {
                    let b = self.get_block_mut(x, y, z);
                    if b == &old_block {
                        *b = new_block
                    }
                }
            }
        }
    }
    pub fn no_update_heightmap(&mut self, filler_block: Block, height_at: impl Fn(usize, usize) -> usize) {
        for x in 0..self.size().world_width() {
            for z in 0..self.size().world_depth() {
                for y in 0..height_at(x, z) {
                    self.no_update_set_block(x, y, z, filler_block)
                }
            }
        }
    }




}



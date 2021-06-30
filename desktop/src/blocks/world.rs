use crate::render_gl::buffer::Buffer;

pub struct Block{
    idx:u32,
}
pub const CHUNK_WIDTH:usize=16;
pub const CHUNK_DEPTH:usize=16;
pub const CHUNK_HEIGHT:usize=256;

pub struct Chunk{
    blocks:[[[Block;CHUNK_WIDTH];CHUNK_DEPTH];CHUNK_HEIGHT],
}

impl Chunk{

}

pub struct World{
    buffer:Vec<Chunk>
}

impl World{

}
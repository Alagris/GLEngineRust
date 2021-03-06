use crate::render_gl::data::u8_u8_u8_u8;
use crate::blocks::block::Block;
use crate::render_gl::data::VertexAttribPointers;
use crate::blocks::face_orientation::FaceOrientation;
use crate::blocks::world_size::{CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH};
use crate::render_gl::data::VertexAttrib;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct Face {
    #[location = 11]
    #[divisor = 1]
    coords: u8_u8_u8_u8,
    #[location = 13]
    #[divisor = 1]
    tex_id: u32,
}

impl Face {
    pub fn update_texture(&mut self, new_block: Block) {
        let ort = self.block_orientation();
        self.tex_id = new_block.texture_id(ort);
    }
    pub fn coords_and_ort(&self) -> u32 {
        self.coords.as_u32().clone()
    }
    pub fn x(&self) -> u8 {
        self.coords.d0
    }
    pub fn matches_coords(&self, x: u8, y: u8, z: u8) -> bool {
        self.x() == x && self.y() == y && self.z() == z
    }
    pub fn matches_block_coords(&self, x: usize, y: usize, z: usize) -> bool {
        self.block_x() == x && self.block_y() == y && self.block_z() == z
    }
    pub fn y(&self) -> u8 {
        self.coords.d1
    }
    pub fn z(&self) -> u8 {
        self.coords.d2
    }
    pub fn orientation(&self) -> u8 {
        self.coords.d3
    }
    pub fn texture_id(&self) -> u32 {
        self.tex_id
    }
    pub fn block_x(&self) -> usize {
        self.coords.d0 as usize
    }
    pub fn block_y(&self) -> usize {
        self.coords.d1 as usize
    }
    pub fn block_z(&self) -> usize {
        self.coords.d2 as usize
    }
    pub fn block_orientation(&self) -> FaceOrientation {
        num_traits::FromPrimitive::from_u8(self.coords.d3).unwrap()
    }
    pub fn encode_coords_and_ort(x: u8, y: u8, z: u8, orientation: FaceOrientation) -> u32 {
        assert!((x as usize) < CHUNK_WIDTH);
        assert!((y as usize) < CHUNK_HEIGHT);
        assert!((z as usize) < CHUNK_DEPTH);
        u8_u8_u8_u8::from((x, y, z, orientation as u8)).as_u32().clone()
    }
    pub fn from_coords_and_ort(x: u8, y: u8, z: u8, orientation: FaceOrientation, texture_id: u32) -> Self {
        assert!((x as usize) < CHUNK_WIDTH);
        assert!((y as usize) < CHUNK_HEIGHT);
        assert!((z as usize) < CHUNK_DEPTH);
        assert_eq!(
            std::mem::size_of::<FaceOrientation>(),
            std::mem::size_of::<u8>()
        );
        Self { coords: u8_u8_u8_u8::from((x, y, z, orientation as u8)), tex_id: texture_id }
    }
}
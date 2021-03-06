use std::fmt::{Display, Formatter};
use crate::blocks::block_properties::{BLOCKS, STONE};
use crate::render_gl::data::VertexAttribPointers;
use crate::blocks::face_orientation::FaceOrientation;
use crate::render_gl::data::VertexAttrib;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct Block {
    #[location = 11]
    #[divisor = 1]
    idx: u32,
}


impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Block {
    pub const fn air() -> Self {
        Self::new(0)
    }
    pub const fn new(idx: u32) -> Self {
        Self { idx }
    }
    pub fn weight(&self) -> u32 {
        (self.idx - 10).max(0)
    }
    pub fn is_solid(&self) -> bool {
        self.idx > 0
    }
    pub fn is_transparent(&self) -> bool {
        self.idx < STONE.idx
    }
    pub fn is_air(&self) -> bool {
        self.idx == 0
    }
    pub fn texture_id(&self, ort: FaceOrientation) -> u32 {
        BLOCKS[self.idx as usize].get_texture_id(ort)
    }
    pub fn name(&self) -> &'static str {
        BLOCKS[self.idx as usize].name()
    }
    pub fn show_neighboring_faces(&self) -> bool { self.is_transparent() }
    pub fn show_my_faces(&self) -> bool { !self.is_air() }
}

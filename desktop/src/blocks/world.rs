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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct Block {
    #[location = 11]
    #[divisor = 1]
    idx: u32,
}

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_DEPTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

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

#[derive(FromPrimitive, Copy, Clone, Eq, PartialEq, Debug)]
pub enum FaceOrientation {
    YPlus = 0,
    YMinus = 1,
    XPlus = 2,
    XMinus = 3,
    ZPlus = 4,
    ZMinus = 5,
}

impl FaceOrientation {
    pub fn is_side(&self) -> bool {
        (self.clone() as u8) > 1
    }
    pub fn opposite(&self) -> FaceOrientation {
        assert_eq!(std::mem::size_of::<Self>(), std::mem::size_of::<u8>());
        let m = self.clone() as u8;
        unsafe {
            if m % 2 == 0 {
                std::mem::transmute(m + 1)
            } else {
                std::mem::transmute(m - 1)
            }
        }
    }
}

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
    fn update_texture(&mut self, new_block: Block) {
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

pub struct Chunk {
    blocks: [[[Block; CHUNK_WIDTH]; CHUNK_DEPTH]; CHUNK_HEIGHT],
    opaque_faces: Vec<Face>,
    transparent_faces: Vec<Face>,
    has_opaque_faces_to_update: bool,
    has_transparent_faces_to_update: bool,
    opaque_faces_model: InstancedLogicalModel<Face, BufferDynamicDraw>,
    transparent_faces_model: InstancedLogicalModel<Face, BufferDynamicDraw>,
}

impl Chunk {
    pub fn gl_update_opaque(&mut self) -> bool{
        let Self {
            opaque_faces,
            has_opaque_faces_to_update,
            opaque_faces_model,..
        } = self;
        if *has_opaque_faces_to_update {
            *has_opaque_faces_to_update = false;
            opaque_faces_model.ibo_mut().update(opaque_faces.as_slice());
            assert_eq!(opaque_faces_model.ibo().len(),opaque_faces.len());
            true
        }else{false}

    }
    pub fn gl_update_transparent(&mut self) -> bool{
        let Self {
            transparent_faces,
            has_transparent_faces_to_update,
            transparent_faces_model, ..
        } = self;
        if *has_transparent_faces_to_update {
            *has_transparent_faces_to_update = false;
            transparent_faces_model.ibo_mut().update(transparent_faces.as_slice());
            assert_eq!(transparent_faces_model.ibo().len(),transparent_faces.len());
            true
        }else{false}
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }
    pub fn as_slice(&self) -> &[Block] {
        let len = CHUNK_WIDTH * CHUNK_DEPTH * CHUNK_HEIGHT;
        unsafe { std::slice::from_raw_parts(self.blocks.as_ptr() as *const Block, len) }
    }

    fn get_block(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.blocks[y][z % CHUNK_DEPTH][x % CHUNK_WIDTH]
    }
    fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        &mut self.blocks[y][z % CHUNK_DEPTH][x % CHUNK_WIDTH]
    }
    pub fn opaque_as_slice(&self) -> &[Face] {
        self.opaque_faces.as_slice()
    }
    pub fn transparent_as_slice(&self) -> &[Face] {
        self.transparent_faces.as_slice()
    }
    pub fn len_opaque(&self) -> usize {
        self.opaque_faces.len()
    }
    pub fn len_transparent(&self) -> usize {
        self.transparent_faces.len()
    }
    fn new(gl: &gl::Gl) -> Self {
        Self {
            blocks: [[[Block::air(); CHUNK_WIDTH]; CHUNK_DEPTH]; CHUNK_HEIGHT],
            opaque_faces: Vec::new(),
            transparent_faces: Vec::new(),
            has_opaque_faces_to_update: false,
            has_transparent_faces_to_update: false,
            opaque_faces_model: InstancedLogicalModel::new(DynamicBuffer::with_capacity(16, &gl), &gl),
            transparent_faces_model: InstancedLogicalModel::new(DynamicBuffer::with_capacity(16, &gl), &gl),
        }
    }
    fn push_block(&mut self, x: usize, y: usize, z: usize, ort: FaceOrientation, block: Block) {
        let (x, y, z) = ((x % CHUNK_WIDTH) as u8, y as u8, (z % CHUNK_DEPTH) as u8);
        self.push(x, y, z, ort, block)
    }
    fn push(&mut self, x: u8, y: u8, z: u8, ort: FaceOrientation, block: Block) {
        let face = Face::from_coords_and_ort(x, y, z, ort, block.texture_id(ort));
        assert!(self.find_opaque_by_coords_and_ort(face.coords_and_ort()).is_none());
        assert!(self.find_transparent_by_coords_and_ort(face.coords_and_ort()).is_none());
        if block.is_transparent() {
            self.transparent_faces.push(face);
            self.has_transparent_faces_to_update = true;
        } else {
            self.opaque_faces.push(face);
            self.has_opaque_faces_to_update = true;
        }
    }
    pub fn find_transparent_by_coords_and_ort(&self, coords: u32) -> Option<&Face> {
        self.transparent_faces.iter().find(|f| f.coords_and_ort() == coords)
    }
    pub fn find_opaque_by_coords_and_ort(&self, coords: u32) -> Option<&Face> {
        self.opaque_faces.iter().find(|f| f.coords_and_ort() == coords)
    }
    pub fn position_transparent_by_coords_and_ort(&self, coords: u32) -> Option<usize> {
        self.transparent_faces.iter().position(|f| f.coords_and_ort() == coords)
    }
    pub fn position_opaque_by_coords_and_ort(&self, coords: u32) -> Option<usize> {
        self.opaque_faces.iter().position(|f| f.coords_and_ort() == coords)
    }
    pub fn find_transparent(&self, x: u8, y: u8, z: u8) -> Option<&Face> {
        self.transparent_faces.iter().find(|f| f.matches_coords(x, y, z))
    }
    pub fn find_opaque(&self, x: u8, y: u8, z: u8) -> Option<&Face> {
        self.opaque_faces.iter().find(|f| f.matches_coords(x, y, z))
    }
    fn remove_block_transparent(&mut self, x: usize, y: usize, z: usize) {
        let (x, y, z) = ((x % CHUNK_WIDTH) as u8, y as u8, (z % CHUNK_DEPTH) as u8);
        self.remove_transparent(x, y, z);
    }

    fn remove_transparent(&mut self, x: u8, y: u8, z: u8) {
        let mut i = 0;
        assert!(self.find_opaque(x, y, z).is_none());
        assert!(self.find_transparent(x, y, z).is_some());
        while i < self.transparent_faces.len() {
            if self.transparent_faces[i].matches_coords(x, y, z) {
                self.remove_transparent_at(i);
            } else {
                i += 1;
            }
        }
    }
    fn remove_block_opaque(&mut self, x: usize, y: usize, z: usize) {
        let (x, y, z) = ((x % CHUNK_WIDTH) as u8, y as u8, (z % CHUNK_DEPTH) as u8);
        self.remove_opaque(x, y, z)
    }
    fn remove_opaque(&mut self, x: u8, y: u8, z: u8) {
        assert!(self.find_opaque(x, y, z).is_some());
        assert!(self.find_transparent(x, y, z).is_none());
        let mut i = 0;
        while i < self.opaque_faces.len() {
            if self.opaque_faces[i].matches_coords(x, y, z) {
                self.remove_opaque_at(i);
            } else {
                i += 1;
            }
        }
    }
    fn update_block_textures(&mut self, x: usize, y: usize, z: usize, new_block: Block) {
        let (x, y, z) = ((x % CHUNK_WIDTH) as u8, y as u8, (z % CHUNK_DEPTH) as u8);
        self.update_textures(x, y, z, new_block)
    }
    /**The transparency of old textures must be the same as that of new ones. If transparency can change, use change_textures instead*/
    fn update_textures(&mut self, x: u8, y: u8, z: u8, new_block: Block) {
        assert!(!new_block.is_air());
        let faces = if new_block.is_transparent() {
            assert!(self.find_opaque(x, y, z).is_none(), "Failed to update texture at {},{},{} to new block id {}", x, y, z, new_block);
            assert!(self.find_transparent(x, y, z).is_some(), "Failed to update texture at {},{},{} to new block id {}", x, y, z, new_block);
            self.has_transparent_faces_to_update = true;
            &mut self.transparent_faces
        } else {
            assert!(self.find_opaque(x, y, z).is_some(), "Failed to update texture at {},{},{} to new block id {}", x, y, z, new_block);
            assert!(self.find_transparent(x, y, z).is_none(), "Failed to update texture at {},{},{} to new block id {}", x, y, z, new_block);
            self.has_opaque_faces_to_update = true;
            &mut self.opaque_faces
        };

        for face in faces.iter_mut() {
            if face.matches_coords(x, y, z) {
                face.update_texture(new_block);
            }
        }
    }
    fn borrow_transparent_and_opaque_mut(&mut self) -> (&mut Vec<Face>, &mut Vec<Face>) {
        let Self { transparent_faces, opaque_faces, .. } = self;
        (transparent_faces, opaque_faces)
    }
    fn change_block_textures(&mut self, x: usize, y: usize, z: usize, new_block: Block) {
        let (x, y, z) = ((x % CHUNK_WIDTH) as u8, y as u8, (z % CHUNK_DEPTH) as u8);
        self.change_textures(x, y, z, new_block)
    }
    /**Changes textures on existing faces and assumes that the transparency is going to be switched. If transparency did not change, use update_textures instead*/
    fn change_textures(&mut self, x: u8, y: u8, z: u8, new_block: Block) {
        assert!(!new_block.is_air());
        let (from, to) = if new_block.is_transparent() {
            assert!(self.find_opaque(x, y, z).is_some(), "Failed to update texture at {},{},{} to new block id {}", x, y, z, new_block);
            assert!(self.find_transparent(x, y, z).is_none(), "Failed to update texture at {},{},{} to new block id {}", x, y, z, new_block);
            let (trans, opaq) = self.borrow_transparent_and_opaque_mut();
            (opaq, trans)
        } else {
            assert!(self.find_opaque(x, y, z).is_none(), "Failed to update texture at {},{},{} to new block id {}", x, y, z, new_block);
            assert!(self.find_transparent(x, y, z).is_some(), "Failed to update texture at {},{},{} to new block id {}", x, y, z, new_block);
            self.borrow_transparent_and_opaque_mut()
        };

        let mut i = 0;
        while i < from.len() {
            if from[i].matches_coords(x, y, z) {
                to.push(from.swap_remove(i))
            } else {
                i += 1;
            }
        }
    }
    fn remove_opaque_block_face(&mut self, x: usize, y: usize, z: usize, ort: FaceOrientation) {
        let (x, y, z) = ((x % CHUNK_WIDTH) as u8, y as u8, (z % CHUNK_DEPTH) as u8);
        self.remove_opaque_face(x, y, z, ort)
    }
    fn remove_opaque_face(&mut self, x: u8, y: u8, z: u8, ort: FaceOrientation) {
        let face = Face::encode_coords_and_ort(x, y, z, ort);
        self.remove_opaque_at(self.position_opaque_by_coords_and_ort(face).unwrap())
    }
    fn remove_transparent_block_face(&mut self, x: usize, y: usize, z: usize, ort: FaceOrientation) {
        let (x, y, z) = ((x % CHUNK_WIDTH) as u8, y as u8, (z % CHUNK_DEPTH) as u8);
        self.remove_transparent_face(x, y, z, ort)
    }
    fn remove_transparent_face(&mut self, x: u8, y: u8, z: u8, ort: FaceOrientation) {
        let face = Face::encode_coords_and_ort(x, y, z, ort);
        self.remove_transparent_at(self.position_transparent_by_coords_and_ort(face).unwrap())
    }
    fn update_texture(&mut self, idx: usize, new_block: Block) {
        assert!(!new_block.is_air());
        let face = if new_block.is_transparent() {
            self.has_transparent_faces_to_update = true;
            &mut self.transparent_faces[idx]
        } else {
            self.has_opaque_faces_to_update = true;
            &mut self.opaque_faces[idx]
        };
        face.update_texture(new_block)
    }
    fn remove_transparent_at(&mut self, idx: usize) {
        self.transparent_faces.swap_remove(idx);
        self.has_transparent_faces_to_update = true;
    }
    fn remove_opaque_at(&mut self, idx: usize) {
        self.opaque_faces.swap_remove(idx);
        self.has_opaque_faces_to_update = true;
    }
}


pub struct World {
    chunks: Vec<Chunk>,
    width: usize,
    depth: usize,
}

impl World {
    fn remove_block_transparent(&mut self, x: usize, y: usize, z: usize) {
        self.get_chunk_mut(x, y).remove_block_transparent(x, y, z)
    }
    fn remove_block_opaque(&mut self, x: usize, y: usize, z: usize) {
        self.get_chunk_mut(x, y).remove_block_opaque(x, y, z)
    }
    fn push_block(&mut self, x: usize, y: usize, z: usize, ort: FaceOrientation, block: Block) {
        self.get_chunk_mut(x, z).push_block(x, y, z, ort, block)
    }
    fn remove_transparent_block_face(&mut self, x: usize, y: usize, z: usize, ort: FaceOrientation) {
        self.get_chunk_mut(x, z).remove_transparent_block_face(x, y, z, ort)
    }
    fn remove_opaque_block_face(&mut self, x: usize, y: usize, z: usize, ort: FaceOrientation) {
        self.get_chunk_mut(x, z).remove_opaque_block_face(x, y, z, ort)
    }
    fn update_block_textures(&mut self, x: usize, y: usize, z: usize, new_block: Block) {
        self.get_chunk_mut(x, z).update_block_textures(x, y, z, new_block)
    }
    fn change_block_textures(&mut self, x: usize, y: usize, z: usize, new_block: Block) {
        self.get_chunk_mut(x, z).change_block_textures(x, y, z, new_block)
    }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> &Block {
        self.get_chunk(x, z).get_block(x, y, z)
    }
    pub fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        self.get_chunk_mut(x, z).get_block_mut(x, y, z)
    }
    pub fn chunk_idx_into_chunk_pos(&self, chunk_idx: usize) -> (usize, usize) {
        assert!(chunk_idx<self.chunks.len());
        (chunk_idx % self.width, chunk_idx / self.width)
    }
    pub fn chunk_pos_into_chunk_idx(&self, x: usize, z: usize) -> usize {
        assert!(x<self.width);
        assert!(z<self.depth);
        z * self.width + x
    }
    pub fn block_pos_into_chunk_idx(&self, x: usize, z: usize) -> usize {
        self.chunk_pos_into_chunk_idx(x / CHUNK_WIDTH, z / CHUNK_DEPTH)
    }
    pub fn get_chunk_mut(&mut self, x: usize, z: usize) -> &mut Chunk {
        let u = self.block_pos_into_chunk_idx(x, z);
        &mut self.chunks[u]
    }
    pub fn get_chunk(&self, x: usize, z: usize) -> &Chunk {
        &self.chunks[self.block_pos_into_chunk_idx(x, z)]
    }
}

impl World {
    pub fn new(width: usize, depth: usize, gl: &gl::Gl) -> Self {
        let chunks: Vec<Chunk> = std::iter::repeat_with(|| Chunk::new(gl)).take(width * depth).collect();
        Self { width, depth, chunks }
    }
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.update_block(x, y, z, move |b| {
            *b = block;
            true
        });
    }
    /**Returns true if previously there was no block at this position and the placement was carried out.
    If there was already a block, then placing a different one is impossible nad function returns false*/
    pub fn place_block(&mut self, x: usize, y: usize, z: usize, block: Block) -> bool {
        self.update_block(x, y, z, move |b| {
            if b.is_air() {
                *b = block;
                true
            } else {
                false
            }
        })
    }
    /**Returns true if previously there was block at this position and the removal was carried out.
    If there was no block, then no removal was necessary and function returns false*/
    pub fn remove_block(&mut self, x: usize, y: usize, z: usize) -> bool {
        self.update_block(x, y, z, move |b| {
            if !b.is_air() {
                *b = Block::air();
                true
            } else {
                false
            }
        })
    }
    /**Updates block according to custom policy. Function f should return true if a block was changed and face update is necessary.
    The result of this function is the same as the output of f.*/
    pub fn update_block<F: Fn(&mut Block) -> bool>(&mut self, x: usize, y: usize, z: usize, f: F) -> bool {
        let b = self.get_block_mut(x, y, z);
        let was_showing_neighboring_faces = b.show_neighboring_faces();
        let was_showing_my_faces = b.show_my_faces();
        let was_transparent = b.is_transparent();
        if f(b) {
            let is_showing_neighboring_faces = b.show_neighboring_faces();
            let is_showing_my_faces = b.show_my_faces();
            let is_transparent = b.is_transparent();
            let b = b.clone();//just to make borrow-checker happy
            if was_showing_my_faces {
                if is_showing_my_faces {
                    if was_transparent == is_transparent {
                        self.update_block_textures(x, y, z, b);
                    } else {
                        self.change_block_textures(x, y, z, b);
                    }
                } else {
                    if was_transparent {
                        self.remove_block_transparent(x, y, z);
                    } else {
                        self.remove_block_opaque(x, y, z);
                    }
                }
            }

            Self::for_each_neighbour(self.width, self.depth, x, y, z, |neighbour_x, neighbour_y, neighbour_z, my_face| {
                let &neighbour = self.get_block(neighbour_x, neighbour_y, neighbour_z);
                let neighbour_face = my_face.opposite();

                if was_showing_neighboring_faces && !is_showing_neighboring_faces && neighbour.show_my_faces() {
                    if neighbour.is_transparent() {
                        self.remove_transparent_block_face(neighbour_x, neighbour_y, neighbour_z, neighbour_face)
                    } else {
                        self.remove_opaque_block_face(neighbour_x, neighbour_y, neighbour_z, neighbour_face)
                    }
                }
                if !was_showing_neighboring_faces && is_showing_neighboring_faces && neighbour.show_my_faces() {
                    self.push_block(neighbour_x, neighbour_y, neighbour_z, neighbour_face, neighbour);
                }
                if !was_showing_my_faces && is_showing_my_faces && neighbour.show_neighboring_faces() {
                    self.push_block(x, y, z, my_face, b);
                }
            });
            true
        } else {
            false
        }
    }
    pub fn gl_update_all_chunks(&mut self) {
        for chunk in self.chunks.iter_mut() {
            chunk.gl_update_opaque();
            chunk.gl_update_transparent();
        }
    }
    pub fn gl_draw(&self, chunk_location_uniform: UniformVec3fv, shader: &Program) {
        for (chunk_idx, chunk) in self.chunks.iter().enumerate() {
            let (x, z) = self.chunk_idx_into_chunk_pos(chunk_idx);
            shader.set_uniform_vec3fv(chunk_location_uniform, &[(x * CHUNK_WIDTH) as f32, 0., (z * CHUNK_DEPTH) as f32]);
            chunk.opaque_faces_model.draw_instanced_triangles(0, /*one quad=2 triangles=6 vertices*/6, chunk.opaque_faces_model.ibo().len());
            chunk.transparent_faces_model.draw_instanced_triangles(0, /*one quad=2 triangles=6 vertices*/6, chunk.transparent_faces_model.ibo().len());
        }
    }
    pub fn is_position_in_bounds(&self, x: usize, y: usize, z: usize) -> bool {
        y < CHUNK_HEIGHT && x < self.width * CHUNK_WIDTH && z < self.depth * CHUNK_DEPTH
    }
    pub fn is_point_in_bounds(&self, x: f32, y: f32, z: f32) -> bool {
        0f32 <= x && 0f32 <= y && 0f32 <= z && y < CHUNK_HEIGHT as f32 && x < (self.width * CHUNK_WIDTH) as f32 && z < (self.depth * CHUNK_DEPTH) as f32
    }
    fn for_each_neighbour<F: FnMut(usize, usize, usize, FaceOrientation)>(
        world_width: usize,
        world_depth: usize,
        x: usize,
        y: usize,
        z: usize,
        mut f: F,
    ) {
        if y < CHUNK_HEIGHT - 1 {
            f(x, y + 1, z, FaceOrientation::YPlus)
        }
        if y >= 1 {
            f(x, y - 1, z, FaceOrientation::YMinus)
        }
        if x < world_width * CHUNK_WIDTH - 1 {
            f(x + 1, y, z, FaceOrientation::XPlus)
        }
        if x >= 1 {
            f(x - 1, y, z, FaceOrientation::XMinus)
        }
        if z < world_depth * CHUNK_DEPTH - 1 {
            f(x, y, z + 1, FaceOrientation::ZPlus)
        }
        if z >= 1 {
            f(x, y, z - 1, FaceOrientation::ZMinus)
        }
    }
    pub fn compute_faces(&mut self) {
        for x in 0..self.width * CHUNK_WIDTH {
            for z in 0..self.depth * CHUNK_DEPTH {
                for y in 0..CHUNK_HEIGHT {
                    let &block = self.get_block(x, y, z);
                    if block.show_my_faces() {
                        Self::for_each_neighbour(self.width, self.depth, x, y, z, |neighbour_x, neighbour_y, neighbour_z, ort| {
                            let neighbour = self.get_block(neighbour_x, neighbour_y, neighbour_z);
                            if neighbour.show_neighboring_faces() {
                                self.push_block(x, y, z, ort, block);
                            }
                        });
                    }
                }
            }
        }
    }
    pub fn point_to_block_position(point: &[f32]) -> (usize, usize, usize) {
        (point[0] as usize, point[1] as usize, point[2] as usize)
    }


    pub fn ray_cast_place_block(&mut self, start: &[f32], distance_and_direction: &[f32], block: Block) {
        ray_cast(start, distance_and_direction, |block_x, block_y, block_z, adjacent_x, adjacent_y, adjacent_z| {
            if self.is_point_in_bounds(block_x, block_y, block_z) && !self.get_block(block_x as usize, block_y as usize, block_z as usize).is_air() {
                if block_x != adjacent_x || block_y != adjacent_y || block_z != adjacent_z {
                    let adjacent_y = adjacent_y as usize;
                    if adjacent_y < CHUNK_HEIGHT {//we don't need to test other coordinates because
                        // normally it should be impossible for a player to reach them
                        self.place_block(adjacent_x as usize, adjacent_y, adjacent_z as usize, block);
                    }
                }
                Some(())
            } else {
                None
            }
        });
    }

    pub fn ray_cast_remove_block(&mut self, start: &[f32], distance_and_direction: &[f32]) {
        ray_cast(start, distance_and_direction, |block_x, block_y, block_z, adjacent_x, adjacent_y, adjacent_z| {
            if self.is_point_in_bounds(block_x, block_y, block_z) &&
                self.remove_block(block_x as usize, block_y as usize, block_z as usize) {
                Some(())
            } else {
                None
            }
        });
    }
}


pub fn ray_cast<T, F: FnMut(f32, f32, f32, f32, f32, f32) -> Option<T>>(start: &[f32], distance_and_direction: &[f32], mut f: F) -> Option<T> {
    //initial point A
    let (ax, ay, az) = (start[0], start[1], start[2]);
    //distance vector D
    let (dx, dy, dz) = (distance_and_direction[0], distance_and_direction[1], distance_and_direction[2]);
    //current voxel boundary
    let (mut vx, mut vy, mut vz) = (ax.floor(), ay.floor(), az.floor());
    let o = f(vx, vy, vz, vx, vy, vz);
    if o.is_some() {
        return o;
    }
    //final voxel boundary B
    let (bx, by, bz) = (ax + dx, ay + dy, az + dz);
    let bv = (bx.floor(), by.floor(), bz.floor());
    fn compute_step_and_initial_ray_length(d: f32, a: f32, v: f32) -> (f32, f32) {
        if d < 0. {
            (-1f32, (v - a) / d)//notice that the signs will cancel out and the result will be positive
        } else {
            (1f32, (1f32 + v - a) / d)
        }
    }
    let (step_x, mut t_max_x) = compute_step_and_initial_ray_length(dx, ax, vx);
    let (step_y, mut t_max_y) = compute_step_and_initial_ray_length(dy, ay, vy);
    let (step_z, mut t_max_z) = compute_step_and_initial_ray_length(dz, az, vz);
    let t_delta_x = step_x / dx;//notice that the signs will cancel out. Division by zero will yield +inf
    assert!(t_delta_x >= 0f32);
    let t_delta_y = step_y / dy;
    let t_delta_z = step_z / dz;

    while (vx, vy, vz) != bv {
        let o = if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                let new_vx = vx + step_x;
                let o = f(new_vx, vy, vz, vx, vy, vz);
                vx = new_vx;
                t_max_x += t_delta_x;
                o
            } else {
                let new_vz = vz + step_z;
                let o = f(vx, vy, new_vz, vx, vy, vz);
                vz = new_vz;
                t_max_z += t_delta_z;
                o
            }
        } else {
            if t_max_y < t_max_z {
                let new_vy = vy + step_y;
                let o = f(vx, new_vy, vz, vx, vy, vz);
                vy = new_vy;
                t_max_y += t_delta_y;
                o
            } else {
                let new_vz = vz + step_z;
                let o = f(vx, vy, new_vz, vx, vy, vz);
                vz = new_vz;
                t_max_z += t_delta_z;
                o
            }
        };

        if o.is_some() {
            return o;
        }
    }
    None
}

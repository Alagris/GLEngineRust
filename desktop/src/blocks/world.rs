use crate::render_gl::data::{u8_u8_u8_u8, u8_u8_u8_u8_u32};
use crate::render_gl::data::VertexAttribPointers;
use crate::render_gl::data::VertexAttrib;
use crate::render_gl::util::init_array;
use crate::blocks::block_properties::BLOCKS;
use std::fmt::{Display, Formatter};

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
impl Display for Block{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.name())
    }
}
impl Block {
    pub fn air() -> Self {
        Self { idx: 0 }
    }
    pub fn new(idx: u32) -> Self {
        Self { idx }
    }
    pub fn weight(&self) -> u32 {
        (self.idx - 10).max(0)
    }
    pub fn is_transparent(&self) -> bool {
        self.idx < 2
    }
    pub fn is_air(&self) -> bool {
        self.idx == 0
    }
    pub fn texture_id(&self, ort:FaceOrientation)->u32{
        BLOCKS[self.idx as usize].get_texture_id(ort)
    }
    pub fn name(&self)->&'static str{
        BLOCKS[self.idx as usize].name()
    }
    pub fn show_neighboring_faces(&self)->bool{ self.is_transparent()}
    pub fn show_my_faces(&self)->bool{!self.is_air()}
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
    pub fn is_side(&self) -> bool{
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
    coords: u8_u8_u8_u8_u32,
}

impl Face {
    pub fn as_u32(&self) -> (u32,u32) {
        self.coords.as_u32().clone()
    }
    pub fn x(&self) -> u8 {
        self.coords.d0
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
        self.coords.d4
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
    pub fn coords(&self) -> u32 {
        self.as_u32().0
    }
    pub fn encode_coords(x: usize, y: usize, z: usize, orientation: FaceOrientation) -> u32 {
        u8_u8_u8_u8::from((x as u8, y as u8, z as u8, orientation as u8)).as_u32().clone()
    }
    pub fn from_coords(x: usize, y: usize, z: usize, orientation: FaceOrientation, texture_id:u32) -> Self {
        assert!(x < CHUNK_WIDTH);
        assert!(y < CHUNK_HEIGHT);
        assert!(z < CHUNK_DEPTH);
        assert_eq!(
            std::mem::size_of::<FaceOrientation>(),
            std::mem::size_of::<u8>()
        );
        Self { coords: u8_u8_u8_u8_u32::from((x as u8, y as u8, z as u8, orientation as u8, texture_id)) }
    }
}

#[derive(Clone)]
pub struct ChunkFaces {
    faces: Vec<Face>,
}

impl ChunkFaces {
    pub fn as_slice(&self) -> &[Face] {
        self.faces.as_slice()
    }
    pub fn len(&self) -> usize {
        self.faces.len()
    }
    fn new() -> Self {
        Self { faces: Vec::new() }
    }
    fn push(&mut self, x: usize, y: usize, z: usize, ort: FaceOrientation, texture_id:u32) {
        let face = Face::from_coords(x, y, z, ort, texture_id);
        assert!(!self.faces.contains(&face));
        self.faces.push(face)
    }
    fn remove_block(&mut self, x: usize, y: usize, z: usize) {
        let mut i = 0;
        let (x, y, z) = ((x % CHUNK_WIDTH) as u8, y as u8, (z % CHUNK_DEPTH) as u8);
        while i < self.faces.len() {
            let face = self.faces[i];
            if face.x() == x && face.y() == y && face.z() == z {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }
    fn update_textures(&mut self, x: usize, y: usize, z: usize, new_block:&Block) {
        let mut i = 0;
        let (x, y, z) = ((x % CHUNK_WIDTH) as u8, y as u8, (z % CHUNK_DEPTH) as u8);
        while i < self.faces.len() {
            let face = self.faces[i];
            if face.x() == x && face.y() == y && face.z() == z {
                self.update_texture(i, new_block);
            } else {
                i += 1;
            }
        }
        assert!(false, "Failed to update texture at {},{},{} to new block id {}",x,y,z, new_block)
    }
    fn remove_face(&mut self, x: usize, y: usize, z: usize, ort: FaceOrientation) {
        let face = Face::encode_coords(x, y, z, ort);
        self.remove(
            self.faces
                .iter()
                .position(|x| x.coords() == face)
                .unwrap(),
        )
    }
    fn update_texture(&mut self, idx: usize, new_block:&Block) {
        let face = &mut self.faces[idx];
        let ort = face.block_orientation();
        face.coords.d4 = new_block.texture_id(ort);
    }
    fn remove(&mut self, idx: usize) {
        let last = self.faces.len() - 1;
        self.faces.swap(idx, last);
        self.faces.pop();
    }

}

#[derive(Clone)]
pub struct Chunk {
    blocks: [[[Block; CHUNK_WIDTH]; CHUNK_DEPTH]; CHUNK_HEIGHT],
}

impl Chunk {
    pub fn len(&self) -> usize {
        self.blocks.len()
    }
    pub fn as_slice(&self) -> &[Block] {
        let len = CHUNK_WIDTH * CHUNK_DEPTH * CHUNK_HEIGHT;
        unsafe { std::slice::from_raw_parts(self.blocks.as_ptr() as *const Block, len) }
    }
    fn new() -> Self {
        Self { blocks: [[[Block::air(); CHUNK_WIDTH]; CHUNK_DEPTH]; CHUNK_HEIGHT] }
    }
    fn get_block(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.blocks[y][z % CHUNK_DEPTH][x % CHUNK_WIDTH]
    }
    fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        &mut self.blocks[y][z % CHUNK_DEPTH][x % CHUNK_WIDTH]
    }
}

pub trait WorldChunks {
    fn get_chunk_mut(&mut self, x: usize, z: usize) -> &mut Chunk;
    fn get_chunk(&self, x: usize, z: usize) -> &Chunk;
    fn get_block(&self, x: usize, y: usize, z: usize) -> &Block {
        self.get_chunk(x, z).get_block(x, y, z)
    }
    fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        self.get_chunk_mut(x, z).get_block_mut(x, y, z)
    }
}

impl<const W: usize, const H: usize> WorldChunks for [[Chunk; W]; H] {
    fn get_chunk_mut(&mut self, x: usize, z: usize) -> &mut Chunk {
        &mut self[z / CHUNK_DEPTH][x / CHUNK_WIDTH]
    }
    fn get_chunk(&self, x: usize, z: usize) -> &Chunk {
        &self[z / CHUNK_DEPTH][x / CHUNK_WIDTH]
    }
}

pub trait WorldFaces {
    fn get_chunk_faces_mut(&mut self, x: usize, z: usize) -> &mut ChunkFaces;
    fn get_chunk_faces(&self, x: usize, z: usize) -> &ChunkFaces;
}

impl<const W: usize, const H: usize> WorldFaces for [[ChunkFaces; W]; H] {
    fn get_chunk_faces_mut(&mut self, x: usize, z: usize) -> &mut ChunkFaces {
        &mut self[z / CHUNK_DEPTH][x / CHUNK_WIDTH]
    }
    fn get_chunk_faces(&self, x: usize, z: usize) -> &ChunkFaces {
        &self[z / CHUNK_DEPTH][x / CHUNK_WIDTH]
    }
}

pub struct World<const W: usize, const H: usize> {
    blocks: [[Chunk; W]; H],
    faces: [[ChunkFaces; W]; H],
}

impl<const W: usize, const H: usize> WorldChunks for World<W, H> {
    fn get_chunk_mut(&mut self, x: usize, z: usize) -> &mut Chunk {
        self.blocks.get_chunk_mut(x, z)
    }

    fn get_chunk(&self, x: usize, z: usize) -> &Chunk {
        self.blocks.get_chunk(x, z)
    }
}

impl<const W: usize, const H: usize> WorldFaces for World<W, H> {
    fn get_chunk_faces_mut(&mut self, x: usize, z: usize) -> &mut ChunkFaces {
        self.faces.get_chunk_faces_mut(x, z)
    }

    fn get_chunk_faces(&self, x: usize, z: usize) -> &ChunkFaces {
        self.faces.get_chunk_faces(x, z)
    }
}

impl<const W: usize, const H: usize> World<W, H> {
    pub fn new() -> Self {
        let blocks: [[Chunk; W]; H] = init_array(|| init_array(|| Chunk::new()));
        let faces: [[ChunkFaces; W]; H] = init_array(|| init_array(|| ChunkFaces::new()));
        Self { blocks, faces }
    }
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.update_block(x, y, z, move |b| {
            *b = block;
            true
        });
    }
    /**Returns true if previously there was no block at this position and the placement was carried out.
    If there was already a block, then placing a different one is impossible nad function returns false*/
    pub fn place_block(&mut self, x: usize, y: usize, z: usize, block: Block) -> bool{
        self.update_block(x, y, z, move |b| {
            if b.is_air() {
                *b = block;
                true
            }else{
                false
            }
        })
    }
    /**Returns true if previously there was block at this position and the removal was carried out.
    If there was no block, then no removal was necessary and function returns false*/
    pub fn remove_block(&mut self, x: usize, y: usize, z: usize) -> bool{
        self.update_block(x, y, z, move |b| {
            if !b.is_air() {
                *b = Block::air();
                true
            }else{
                false
            }
        })
    }
    /**Updates block according to custom policy. Function f should return true if a block was changed and face update is necessary.
    The result of this function is the same as the output of f.*/
    pub fn update_block<F: Fn(&mut Block) -> bool>(&mut self, x: usize, y: usize, z: usize, f: F) -> bool{
        let (chunks, faces) = self.borrow_chunks_and_faces_mut();
        let b = chunks.get_block_mut(x, y, z);
        let was_showing_neighboring_faces = b.show_neighboring_faces();
        let was_showing_my_faces = b.show_my_faces();
        if f(b) {
            let is_showing_neighboring_faces = b.show_neighboring_faces();
            let is_showing_my_faces = b.show_my_faces();
            if was_showing_my_faces {
                if is_showing_my_faces {
                    faces.get_chunk_faces_mut(x, z).update_textures(x, y, z, &b);
                }else{
                    faces.get_chunk_faces_mut(x, z).remove_block(x, y, z);
                }
            }
            let b = b.clone();//just to make borrow-checker happy
            Self::for_each_neighbour(x, y, z, |neighbour_x, neighbour_y, neighbour_z, my_face| {
                let neighbour = chunks.get_block(neighbour_x, neighbour_y, neighbour_z);
                let neighbour_face = my_face.opposite();
                if was_showing_neighboring_faces && !is_showing_neighboring_faces && neighbour.show_my_faces(){
                    faces.get_chunk_faces_mut(neighbour_x, neighbour_y).remove_face(neighbour_x, neighbour_y, neighbour_z, neighbour_face)
                }
                if !was_showing_neighboring_faces && is_showing_neighboring_faces && neighbour.show_my_faces(){
                    faces.get_chunk_faces_mut(neighbour_x, neighbour_z).push(neighbour_x, neighbour_y, neighbour_z, neighbour_face,neighbour.texture_id(neighbour_face));
                }
                if !was_showing_my_faces && is_showing_my_faces && neighbour.show_neighboring_faces(){
                    faces.get_chunk_faces_mut(x, z).push(x, y, z, my_face, b.texture_id(my_face));
                }
            });
            true
        }else{
            false
        }
    }
    pub fn is_position_in_bounds(x: usize, y: usize, z: usize) -> bool {
        y < CHUNK_HEIGHT && x < W * CHUNK_WIDTH && z < H * CHUNK_DEPTH
    }
    pub fn is_point_in_bounds(x: f32, y: f32, z: f32) -> bool {
        0f32 <= x && 0f32 <= y && 0f32 <= z && y < CHUNK_HEIGHT as f32 && x < (W * CHUNK_WIDTH) as f32 && z < (H * CHUNK_DEPTH) as f32
    }
    fn for_each_neighbour<F: FnMut(usize, usize, usize, FaceOrientation)>(
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
        if x < W * CHUNK_WIDTH - 1 {
            f(x + 1, y, z, FaceOrientation::XPlus)
        }
        if x >= 1 {
            f(x - 1, y, z, FaceOrientation::XMinus)
        }
        if z < H * CHUNK_DEPTH - 1 {
            f(x, y, z + 1, FaceOrientation::ZPlus)
        }
        if z >= 1 {
            f(x, y, z - 1, FaceOrientation::ZMinus)
        }
    }
    pub fn borrow_chunks_and_faces_mut(
        &mut self,
    ) -> (&mut [[Chunk; W]; H], &mut [[ChunkFaces; W]; H]) {
        let Self { blocks, faces } = self;
        (blocks, faces)
    }
    pub fn compute_faces(&mut self) {
        for x in 0..W * CHUNK_WIDTH {
            for z in 0..H * CHUNK_DEPTH {
                let (chunks, faces) = self.borrow_chunks_and_faces_mut();
                let chunk = chunks.get_chunk(x, z);
                let faces = faces.get_chunk_faces_mut(x, z);
                for y in 0..CHUNK_HEIGHT {
                    let block = chunk.get_block(x, y, z);
                    if block.show_my_faces() {
                        Self::for_each_neighbour(x, y, z, |neighbour_x, neighbour_y, neighbour_z, ort| {
                            let neighbour = chunks.get_block(neighbour_x, neighbour_y, neighbour_z);
                            if neighbour.show_neighboring_faces() {
                                faces.push(x, y, z, ort, block.texture_id(ort));
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
            if Self::is_point_in_bounds(block_x , block_y , block_z ) && !self.get_block(block_x as usize, block_y as usize , block_z as usize).is_air() {
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
            if Self::is_point_in_bounds(block_x,block_y,block_z) &&
                self.remove_block(block_x as usize, block_y as usize, block_z as usize){
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
    let (bx,by,bz) = (ax + dx, ay + dy,az + dz);
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
                let new_vx = vx+step_x;
                let o = f(new_vx, vy, vz, vx, vy, vz);
                vx = new_vx;
                t_max_x += t_delta_x;
                o
            } else {
                let new_vz = vz+step_z;
                let o = f(vx, vy, new_vz, vx, vy, vz);
                vz = new_vz;
                t_max_z += t_delta_z;
                o
            }
        } else {
            if t_max_y < t_max_z {
                let new_vy = vy+step_y;
                let o = f(vx, new_vy, vz, vx, vy, vz);
                vy = new_vy;
                t_max_y += t_delta_y;
                o
            } else {
                let new_vz = vz+step_z;
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

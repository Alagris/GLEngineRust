use crate::render_gl::data::u8_u8_u8_u8;
use num_traits::FromPrimitive;
use crate::render_gl::data::VertexAttribPointers;
use crate::render_gl::data::VertexAttrib;
use std::mem::MaybeUninit;
use crate::render_gl::util::init_array;

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

impl Block {
    pub fn air() -> Self {
        Self { idx: 0 }
    }
    pub fn new(idx:u32) -> Self {
        Self { idx }
    }
    pub fn weight(&self) -> u32 {
        (self.idx - 10).max(0)
    }
    pub fn is_transparent(&self) -> bool {
        self.idx < 10
    }
    pub fn is_air(&self) -> bool {
        self.idx == 0
    }
}

#[derive(FromPrimitive, Copy, Clone,Eq, PartialEq,Debug)]
pub enum FaceOrientation {
    YPlus = 0,
    YMinus = 1,
    XPlus = 2,
    XMinus = 3,
    ZPlus = 4,
    ZMinus = 5,
}

impl FaceOrientation{
    pub fn opposite(&self)->FaceOrientation{
        assert_eq!(std::mem::size_of::<Self>(),std::mem::size_of::<u8>());
        let m = self.clone() as u8;
        unsafe{
            if m%2==0{
                std::mem::transmute(m+1)
            } else {
                std::mem::transmute(m-1)
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
}

impl Face {
    pub fn as_u32(&self) -> u32 {
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

    pub fn from_coords(x: usize, y: usize, z: usize, orientation: FaceOrientation) -> Self {
        assert!(x < CHUNK_WIDTH);
        assert!(y < CHUNK_HEIGHT);
        assert!(z < CHUNK_DEPTH);
        assert_eq!(
            std::mem::size_of::<FaceOrientation>(),
            std::mem::size_of::<u8>()
        );
        Self{coords:u8_u8_u8_u8::from((x as u8, y as u8, z as u8, orientation as u8))}
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
    fn push(&mut self, x: usize, y: usize, z: usize, ort: FaceOrientation) {
        let face = Face::from_coords(x, y, z, ort);
        assert!(!self.faces.contains(&face));
        self.faces.push(face)
    }
    fn remove_block(&mut self, x: usize, y: usize, z: usize) {
        let mut i = 0;
        let (x, y, z) = ((x%CHUNK_WIDTH) as u8, y as u8, (z%CHUNK_DEPTH) as u8);
        while i < self.faces.len() {
            let face = self.faces[i];
            if face.x() == x && face.y() == y && face.z() == z {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }
    fn remove_face(&mut self, x: usize, y: usize, z: usize, ort: FaceOrientation) {
        let face = Face::from_coords(x, y, z, ort).as_u32().clone();
        self.remove(
            self.faces
                .iter()
                .position(|x| x.as_u32().clone() == face)
                .unwrap(),
        )
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
        let len = CHUNK_WIDTH* CHUNK_DEPTH* CHUNK_HEIGHT;
        unsafe{std::slice::from_raw_parts(self.blocks.as_ptr() as *const Block, len)}
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
        let blocks:[[Chunk; W]; H] = init_array(||init_array(||Chunk::new()));
        let faces: [[ChunkFaces; W]; H] = init_array(||init_array(||ChunkFaces::new()));
        Self { blocks, faces}
    }
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        let b = self.get_block_mut(x, y, z);
        let was_transparent = b.is_transparent();
        let is_transparent = block.is_transparent();
        *b = block;
        if was_transparent {
            if !is_transparent{
                let (chunks, faces) = self.borrow_chunks_and_faces_mut();
                Self::for_each_neighbour(x, y, z, |neighbour_x, neighbour_y, neighbour_z, ort| {
                    if chunks.get_block(neighbour_x, neighbour_y, neighbour_z).is_transparent() {
                        faces.get_chunk_faces_mut(x,z).push(x, y, z, ort);
                    }else{
                        faces.get_chunk_faces_mut(neighbour_x, neighbour_y).remove_face(neighbour_x, neighbour_y, neighbour_z,ort.opposite())
                    }
                });
            }
        }else if is_transparent{
            self.get_chunk_faces_mut(x,z).remove_block(x,y,z);
            let (chunks, faces) = self.borrow_chunks_and_faces_mut();
            Self::for_each_neighbour(x, y, z, |neighbour_x, neighbour_y, neighbour_z, ort| {
                if !chunks.get_block(neighbour_x, neighbour_y, neighbour_z).is_transparent() {
                    faces.get_chunk_faces_mut(neighbour_x,neighbour_z).push(neighbour_x, neighbour_y, neighbour_z, ort.opposite());
                }
            });
        }
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
                    if !block.is_transparent() {
                        Self::for_each_neighbour(x, y, z, |neighbour_x, neighbour_y, neighbour_z, ort| {
                            if chunks.get_block(neighbour_x, neighbour_y, neighbour_z).is_transparent() {
                                faces.push(x, y, z, ort);
                            }
                        });
                    }
                }
            }
        }
    }
}

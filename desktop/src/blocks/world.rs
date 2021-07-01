use crate::render_gl::buffer::Buffer;
use crate::render_gl::data::{u8_u8_u8, u8_u8, u8_u8_u8_u8};
use num_traits::FromPrimitive;

pub struct Block {
    idx: u32,
}

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_DEPTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;

impl Block {
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


#[derive(FromPrimitive)]
pub enum FaceOrientation {
    YPlus = 0,
    YMinus = 1,
    XPlus = 2,
    XMinus = 3,
    ZPlus = 4,
    ZMinus = 5,
}

impl u8_u8_u8_u8 {

    pub fn block_x(&self) -> usize {
        self.d0 as usize
    }
    pub fn block_y(&self) -> usize {
        self.d1 as usize
    }
    pub fn block_z(&self) -> usize {
        self.d2 as usize
    }
    pub fn block_orientation(&self) -> FaceOrientation {
        num_traits::FromPrimitive::from_u8(self.d3).unwrap()
    }

    pub fn from_coords(x: usize, y: usize, z: usize, orientation: FaceOrientation) -> Self {
        assert!(x<CHUNK_WIDTH);
        assert!(y<CHUNK_HEIGHT);
        assert!(z<CHUNK_DEPTH);
        assert_eq!(std::mem::size_of::<FaceOrientation>(),std::mem::size_of::<u8>());
        Self::from((x as u8, y as u8, z as u8, orientation as u8))
    }
}
pub struct ChunkFaces{
    faces: Vec<u8_u8_u8_u8>,
}
impl ChunkFaces{
    fn new()->Self{
        Self{faces:Vec::new()}
    }
    fn push(&mut self, x:usize,y:usize,z:usize,ort:FaceOrientation){
        let face = u8_u8_u8_u8::from_coords(x,y,z,ort);
        assert!(!self.faces.contains(&face));
        self.faces.push(face)
    }
    fn remove_block(&mut self, x:usize,y:usize,z:usize){
        let mut i = 0;
        let (x,y,z) = (x as u8,y as u8, z as u8);
        while i<self.faces.len(){
            let face = self.faces[i];
            if face.d0==x&&face.d1==y&&face.d2==z{
                self.remove(i);
            }else{
                i+=1;
            }

        }
    }
    fn remove_face(&mut self, x:usize,y:usize,z:usize, ort:FaceOrientation){
        let face = u8_u8_u8_u8::from_coords(x,y,z,ort).as_u32().clone();
        self.remove(self.faces.iter().position(|x|x.as_u32().clone()==face).unwrap())
    }
    fn remove(&mut self, idx:usize){
        let last = self.faces.len()-1;
        self.faces.swap(idx, last);
        self.faces.pop();
    }
}
pub struct Chunk {
    blocks: [[[Block; CHUNK_WIDTH]; CHUNK_DEPTH]; CHUNK_HEIGHT],
}

impl Chunk {
    fn get_block(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.blocks[y][z % CHUNK_DEPTH][x % CHUNK_WIDTH]
    }
    fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        &mut self.blocks[y][z % CHUNK_DEPTH][x % CHUNK_WIDTH]
    }
}
pub trait WorldChunks{
    fn get_chunk_mut(&mut self, x: usize, z: usize) -> &mut Chunk;
    fn get_chunk(&self, x: usize, z: usize) -> &Chunk;
    fn get_block(&self, x: usize, y: usize, z: usize) -> &Block {
        self.get_chunk(x,z).get_block(x,y,z)
    }
    fn get_block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        self.get_chunk_mut(x,z).get_block_mut(x,y,z)
    }
}
impl <const W: usize, const H: usize> WorldChunks for [[Chunk; W]; H]{
    fn get_chunk_mut(&mut self, x: usize, z: usize) -> &mut Chunk {
        &mut self[z / CHUNK_DEPTH][x / CHUNK_WIDTH]
    }
    fn get_chunk(&self, x: usize, z: usize) -> &Chunk {
        &self[z / CHUNK_DEPTH][x / CHUNK_WIDTH]
    }
}
pub trait WorldFaces{
    fn get_chunk_faces_mut(&mut self, x: usize, z: usize) -> &mut ChunkFaces;
    fn get_chunk_faces(&self, x: usize, z: usize) -> &ChunkFaces;
}
impl <const W: usize, const H: usize> WorldFaces for [[ChunkFaces; W]; H]{
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
impl <const W: usize, const H: usize> WorldChunks for World<W, H> {
    fn get_chunk_mut(&mut self, x: usize, z: usize) -> &mut Chunk {
        self.blocks.get_chunk_mut(x,z)
    }

    fn get_chunk(&self, x: usize, z: usize) -> &Chunk {
        self.blocks.get_chunk(x,z)
    }
}

impl <const W: usize, const H: usize> WorldFaces for World<W, H> {
    fn get_chunk_faces_mut(&mut self, x: usize, z: usize) -> &mut ChunkFaces {
        self.faces.get_chunk_faces_mut(x,z)
    }

    fn get_chunk_faces(&self, x: usize, z: usize) -> &ChunkFaces {
        self.faces.get_chunk_faces(x,z)
    }
}

impl<const W: usize, const H: usize> World<W, H> {

    pub fn set_block(&mut self, x:usize,y:usize,z:usize, block:Block){
        let b = self.get_block_mut(x,y,z);
        let transparency_change = b.is_transparent() != block.is_transparent();
        *b = block;
        if transparency_change{
            Self::for_each_neighbour(x,y,z, |x,y,z,ort|{

            });
        }
    }

    fn for_each_neighbour<F:FnMut(usize,usize,usize,FaceOrientation)>(x:usize,y:usize,z:usize, mut f:F){
        if y<CHUNK_HEIGHT-1 {
            f(x,y+1,z, FaceOrientation::YPlus)
        }
        if y>=1 {
            f(x,y-1,z, FaceOrientation::YMinus)
        }
        if x<W*CHUNK_WIDTH-1 {
            f(x+1,y,z, FaceOrientation::XPlus)
        }
        if x>=1 {
            f(x-1,y,z, FaceOrientation::XMinus)
        }
        if z<H*CHUNK_DEPTH-1 {
            f(x,y,z+1, FaceOrientation::ZPlus)
        }
        if z>=1 {
            f(x,y,z-1, FaceOrientation::ZMinus)
        }
    }
    pub fn borrow_chunks_and_faces_mut(&mut self) -> (&mut [[Chunk; W]; H], &mut [[ChunkFaces; W]; H]) {
        let Self{blocks,faces} = self;
        (blocks,faces)
    }
    pub fn compute_faces(&mut self) {
        for x in 0..W * CHUNK_WIDTH {
            for z in 0..H * CHUNK_DEPTH {
                let (chunks,faces) = self.borrow_chunks_and_faces_mut();
                let chunk = chunks.get_chunk(x,z);
                let faces = faces.get_chunk_faces_mut(x,z);
                faces.faces.clear();
                for y in 0..CHUNK_HEIGHT{
                    let block = chunk.get_block(x,y,z);
                    if !block.is_transparent() {
                        Self::for_each_neighbour(x,y,z,|x,y,z,ort|{
                           if chunks.get_block(x,y,z).is_transparent(){
                               faces.push(x,y,z,ort);
                           }
                        });
                    }
                }
            }
        }
    }
}
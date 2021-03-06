use gl;
use crate::render_gl::gl_error::drain_gl_errors;
use gl::Gl;

/**
There is a strict convention for GLSL attribute location.
Location 0 is meant to hold vec3 with vertex position.
Location 1 is meant to hold vec3 with RGB color.
Location 2 is meant to hold vec2 with texture UV coordinates.
Location 3 is meant to hold vec3 with normal vectors.
Location 4 and 5 is meant to hold vec3 with precomputed tangent and bitangent vectors.
Location 6 is for extra scalar attributes (like size, length, temperature, light intensity etc)
Locations above 10 (inclusive) are reserved for shaders with instancing (glDrawArraysInstanced).
Location 10 is meant to hold vec3 vector with instance position.
Location 11 is meant to hold some u8 integer meta-data of each instance.
Location 12 is meant to hold some u16 integer meta-data of each instance.
Location 13 is meant to hold some u32 integer meta-data of each instance.
Location 14 is meant to hold quat with instance rotation.
*/
pub trait VertexAttribPointers {
    fn vertex_attrib_pointers(gl: &::gl::Gl);
}

pub trait VertexAttrib {
    const NUMBER_OF_COMPONENTS: gl::types::GLint;
    const GL_TYPE: gl::types::GLenum;
    unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        if Self::GL_TYPE == gl::UNSIGNED_INT ||
            Self::GL_TYPE == gl::INT ||
            Self::GL_TYPE == gl::BYTE ||
            Self::GL_TYPE == gl::UNSIGNED_BYTE ||
            Self::GL_TYPE == gl::SHORT ||
            Self::GL_TYPE == gl::UNSIGNED_SHORT {
            gl.VertexAttribIPointer(
                location as gl::types::GLuint,
                Self::NUMBER_OF_COMPONENTS, // the number of components per generic vertex attribute
                Self::GL_TYPE,              // data type
                stride as gl::types::GLint,
                offset as *const gl::types::GLvoid,
            );
        } else {
            gl.VertexAttribPointer(
                location as gl::types::GLuint,
                Self::NUMBER_OF_COMPONENTS, // the number of components per generic vertex attribute
                Self::GL_TYPE,              // data type
                gl::FALSE,                  // normalized (int-to-float conversion)
                stride as gl::types::GLint,
                offset as *const gl::types::GLvoid,
            );
        }
        drain_gl_errors(gl);
    }
}
impl VertexAttrib for glm::Quat{
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 4;
    const GL_TYPE: gl::types::GLenum = gl::FLOAT;
}
impl VertexAttrib for glm::Vec4{
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 4;
    const GL_TYPE: gl::types::GLenum = gl::FLOAT;
}
impl VertexAttrib for glm::Vec3{
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 3;
    const GL_TYPE: gl::types::GLenum = gl::FLOAT;
}
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct f32_f32_f32 {
    pub d0: f32,
    pub d1: f32,
    pub d2: f32,
}

impl f32_f32_f32 {
    pub fn new(d0: f32, d1: f32, d2: f32) -> f32_f32_f32 {
        f32_f32_f32 { d0, d1, d2 }
    }
    pub fn x(&self) -> f32 {
        self.d0
    }
    pub fn y(&self) -> f32 {
        self.d1
    }
    pub fn z(&self) -> f32 {
        self.d2
    }
}

impl VertexAttrib for f32_f32_f32 {
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 3;
    const GL_TYPE: gl::types::GLenum = gl::FLOAT;
}

impl From<(f32, f32, f32)> for f32_f32_f32 {
    fn from(other: (f32, f32, f32)) -> Self {
        f32_f32_f32::new(other.0, other.1, other.2)
    }
}

impl From<&[f32; 3]> for f32_f32_f32 {
    fn from(other: &[f32; 3]) -> Self {
        f32_f32_f32::new(other[0], other[1], other[2])
    }
}

impl From<&glm::Vec3> for f32_f32_f32 {
    fn from(other: &glm::Vec3) -> Self {
        f32_f32_f32::new(other[0], other[1], other[2])
    }
}

impl From<glm::Vec3> for f32_f32_f32 {
    fn from(other: glm::Vec3) -> Self {
        f32_f32_f32::new(other[0], other[1], other[2])
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct f32_f32_f32_f32 {
    pub d0: f32,
    pub d1: f32,
    pub d2: f32,
    pub d3: f32,
}

impl f32_f32_f32_f32 {
    pub fn new(d0: f32, d1: f32, d2: f32,d3: f32) -> Self {
        Self { d0, d1, d2 , d3}
    }
    pub fn x(&self) -> f32 {
        self.d0
    }
    pub fn y(&self) -> f32 {
        self.d1
    }
    pub fn z(&self) -> f32 {
        self.d2
    }
    pub fn w(&self) -> f32 {
        self.d3
    }
}

impl VertexAttrib for f32_f32_f32_f32 {
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 4;
    const GL_TYPE: gl::types::GLenum = gl::FLOAT;
}

impl From<(f32, f32, f32, f32)> for f32_f32_f32_f32 {
    fn from(other: (f32, f32, f32, f32)) -> Self {
        f32_f32_f32_f32::new(other.0, other.1, other.2, other.3)
    }
}

impl From<&[f32; 4]> for f32_f32_f32_f32 {
    fn from(other: &[f32; 4]) -> Self {
        f32_f32_f32_f32::new(other[0], other[1], other[2], other[4])
    }
}

impl From<&glm::Vec4> for f32_f32_f32_f32 {
    fn from(other: &glm::Vec4) -> Self {
        Self::new(other[0], other[1], other[2], other[4])
    }
}

impl From<&glm::Quat> for f32_f32_f32_f32 {
    fn from(other: &glm::Quat) -> Self {
        Self::new(other.i, other.j, other.k, other.w)
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct f32_f32 {
    pub d0: f32,
    pub d1: f32,
}

impl f32_f32 {
    pub fn new(d0: f32, d1: f32) -> f32_f32 {
        f32_f32 { d0, d1 }
    }
}

impl VertexAttrib for f32_f32 {
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 2;
    const GL_TYPE: gl::types::GLenum = gl::FLOAT;
}

impl From<(f32, f32)> for f32_f32 {
    fn from(other: (f32, f32)) -> Self {
        f32_f32::new(other.0, other.1)
    }
}

impl From<&[f32; 2]> for f32_f32 {
    fn from(other: &[f32; 2]) -> Self {
        f32_f32::new(other[0], other[1])
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C, packed)]
pub struct u8_u8 {
    pub d0: u8,
    pub d1: u8,
}

impl u8_u8 {
    pub fn new(d0: u8, d1: u8) -> u8_u8 {
        u8_u8 { d0, d1 }
    }
}

impl VertexAttrib for u8_u8 {
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 2;
    const GL_TYPE: gl::types::GLenum = gl::UNSIGNED_BYTE;
}

impl From<(u8, u8)> for u8_u8 {
    fn from(other: (u8, u8)) -> Self {
        u8_u8::new(other.0, other.1)
    }
}

impl From<&[u8; 2]> for u8_u8 {
    fn from(other: &[u8; 2]) -> Self {
        u8_u8::new(other[0], other[1])
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct u32_u32 {
    pub d0: u32,
    pub d1: u32,
}

impl u32_u32 {
    pub fn new(d0: u32, d1: u32) -> Self {
        Self { d0, d1 }
    }
}

impl VertexAttrib for u32_u32 {
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 2;
    const GL_TYPE: gl::types::GLenum = gl::UNSIGNED_INT;
}

impl From<(u32, u32)> for u32_u32 {
    fn from(other: (u32, u32)) -> Self {
        Self::new(other.0, other.1)
    }
}

impl From<&[u32; 2]> for u32_u32 {
    fn from(other: &[u32; 2]) -> Self {
        Self::new(other[0], other[1])
    }
}


#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C, packed)]
pub struct u8_u8_u8_u8 {
    pub d0: u8,
    pub d1: u8,
    pub d2: u8,
    pub d3: u8,
}

impl u8_u8_u8_u8 {
    pub fn as_u32(&self) -> &u32 {
        unsafe { std::mem::transmute::<&u8_u8_u8_u8, &u32>(self) }
    }
    pub fn new(d0: u8, d1: u8, d2: u8, d3: u8) -> u8_u8_u8_u8 {
        u8_u8_u8_u8 { d0, d1, d2, d3 }
    }
}

impl VertexAttrib for u8_u8_u8_u8 {
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 4;
    const GL_TYPE: gl::types::GLenum = gl::UNSIGNED_BYTE; //GLSL does not actually support bytes. u32 is the smallest data type. Hence we encode 4 bytes as one int.
}

impl From<(u8, u8, u8, u8)> for u8_u8_u8_u8 {
    fn from(other: (u8, u8, u8, u8)) -> Self {
        u8_u8_u8_u8::new(other.0, other.1, other.2, other.3)
    }
}

impl From<&[u8; 4]> for u8_u8_u8_u8 {
    fn from(other: &[u8; 4]) -> Self {
        u8_u8_u8_u8::new(other[0], other[1], other[2], other[3])
    }
}

impl From<u32> for u8_u8_u8_u8 {
    fn from(other: u32) -> Self {
        unsafe { std::mem::transmute::<u32, u8_u8_u8_u8>(other) }
    }
}

impl VertexAttrib for u8 {
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 1;
    const GL_TYPE: gl::types::GLenum = gl::UNSIGNED_BYTE;
}
impl VertexAttrib for u16 {
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 1;
    const GL_TYPE: gl::types::GLenum = gl::UNSIGNED_SHORT;
}
impl VertexAttrib for u32 {
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 1;
    const GL_TYPE: gl::types::GLenum = gl::UNSIGNED_INT;
}

impl VertexAttrib for f32 {
    const NUMBER_OF_COMPONENTS: gl::types::GLint = 1;
    const GL_TYPE: gl::types::GLenum = gl::FLOAT;
}


impl VertexAttribPointers for u8 {
    fn vertex_attrib_pointers(gl: &Gl) {
        unsafe{
            u8::vertex_attrib_pointer(gl,1, 11,0);
            gl.VertexAttribDivisor(11, 1);
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct VertexClr {
    #[location = 0]
    pos: f32_f32_f32,
    //position
    #[location = 1]
    clr: f32_f32_f32, //color
}

impl VertexClr {
    pub fn new(pos: impl Into<f32_f32_f32>, clr: impl Into<f32_f32_f32>) -> Self {
        Self {
            pos: pos.into(),
            clr: clr.into(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct VertexAlphaClr {
    #[location = 0]
    pos: f32_f32_f32,
    //position
    #[location = 1]
    clr: f32_f32_f32_f32, //color
}

impl VertexAlphaClr {
    pub fn new(pos: impl Into<f32_f32_f32>, clr: impl Into<f32_f32_f32_f32>) -> Self {
        Self {
            pos: pos.into(),
            clr: clr.into(),
        }
    }
}



#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct VertexSizeAlphaClr {
    #[location = 0]
    pos: f32_f32_f32,
    #[location = 6]
    size:f32,
    //position
    #[location = 1]
    clr: f32_f32_f32_f32, //color
}

impl VertexSizeAlphaClr {
    pub fn new(pos: impl Into<f32_f32_f32>, size:f32, clr: impl Into<f32_f32_f32_f32>) -> Self {
        Self {
            pos: pos.into(),
            size,
            clr: clr.into(),
        }
    }
}


#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct Instance {
    #[location = 10]
    #[divisor = 1]
    pos: f32_f32_f32, //position
}

impl Instance {
    pub fn new(pos: impl Into<f32_f32_f32>) -> Self {
        Self { pos: pos.into() }
    }
}


#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct VertexTexCol {
    #[location = 0]
    pos: f32_f32_f32,
    #[location = 1]
    clr: f32_f32_f32,
    #[location = 2]
    tex: f32_f32, //texture
}

impl VertexTexCol {
    pub fn new(
        pos: impl Into<f32_f32_f32>,
        clr: impl Into<f32_f32_f32>,
        tex: impl Into<f32_f32>,
    ) -> Self {
        Self {
            pos: pos.into(),
            tex: tex.into(),
            clr: clr.into(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct VertexTex {
    #[location = 0]
    pos: f32_f32_f32,
    //position
    #[location = 2]
    tex: f32_f32, //texture
}

impl VertexTex {
    pub fn new(pos: impl Into<f32_f32_f32>, tex: impl Into<f32_f32>) -> Self {
        Self {
            pos: pos.into(),
            tex: tex.into(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct VertexTexNor {
    #[location = 0]
    pos: f32_f32_f32,
    //position
    #[location = 2]
    tex: f32_f32,
    //texture
    #[location = 3]
    nor: f32_f32_f32, //normal
}

impl VertexTexNor {
    pub fn new(
        pos: impl Into<f32_f32_f32>,
        nor: impl Into<f32_f32_f32>,
        tex: impl Into<f32_f32>,
    ) -> Self {
        Self {
            pos: pos.into(),
            tex: tex.into(),
            nor: nor.into(),
        }
    }

    pub fn pos(&self) -> &f32_f32_f32 {
        &self.pos
    }
    pub fn tex(&self) -> &f32_f32 {
        &self.tex
    }
    pub fn nor(&self) -> &f32_f32_f32 {
        &self.nor
    }
    pub fn set_nor(&mut self, x: f32, y: f32, z: f32) {
        self.nor.d0 = x;
        self.nor.d1 = y;
        self.nor.d2 = z;
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct VertexTexNorTan {
    #[location = 0]
    pos: f32_f32_f32,
    //position
    #[location = 2]
    tex: f32_f32,
    //texture
    #[location = 3]
    nor: f32_f32_f32,
    //normal
    #[location = 4]
    tan: f32_f32_f32,
    //tangent
    #[location = 5]
    bitan: f32_f32_f32, //bitangent
}

impl VertexTexNorTan {
    pub fn new(
        pos: impl Into<f32_f32_f32>,
        nor: impl Into<f32_f32_f32>,
        tex: impl Into<f32_f32>,
        tan: impl Into<f32_f32_f32>,
        bitan: impl Into<f32_f32_f32>,
    ) -> Self {
        Self {
            pos: pos.into(),
            tex: tex.into(),
            nor: nor.into(),
            tan: tan.into(),
            bitan: bitan.into(),
        }
    }

    pub fn convert(ver_tex_nor: VertexTexNor, tan: f32_f32_f32, bitan: f32_f32_f32) -> Self {
        Self::new(
            ver_tex_nor.pos,
            ver_tex_nor.nor,
            ver_tex_nor.tex,
            tan,
            bitan,
        )
    }
}


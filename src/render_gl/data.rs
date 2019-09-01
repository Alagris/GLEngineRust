use gl;


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
        f32_f32_f32 {
            d0,
            d1,
            d2,
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            3, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid,
        );
    }
}

impl From<(f32, f32, f32)> for f32_f32_f32 {
    fn from(other: (f32, f32, f32)) -> Self {
        f32_f32_f32::new(other.0, other.1, other.2)
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
        f32_f32 {
            d0,
            d1,
        }
    }

    pub unsafe fn vertex_attrib_pointer(gl: &gl::Gl, stride: usize, location: usize, offset: usize) {
        gl.EnableVertexAttribArray(location as gl::types::GLuint);
        gl.VertexAttribPointer(
            location as gl::types::GLuint,
            2, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            stride as gl::types::GLint,
            offset as *const gl::types::GLvoid,
        );
    }
}

impl From<(f32, f32)> for f32_f32 {
    fn from(other: (f32, f32)) -> Self {
        f32_f32::new(other.0, other.1)
    }
}


#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
#[derive(VertexAttribPointers)]
pub struct Vertex {
    #[location = 0]
    pos: f32_f32_f32,
    //position
    #[location = 1]
    clr: f32_f32_f32, //color
}

impl Vertex {
    pub fn new(pos: f32_f32_f32, clr: f32_f32_f32) -> Self {
        Self {
            pos,
            clr,
        }
    }

    pub fn new_t(pos: (f32, f32, f32), clr: (f32, f32, f32)) -> Self {
        Self::new(pos.into(), clr.into())
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
    pub fn new(pos: f32_f32_f32, clr: f32_f32_f32, tex: f32_f32) -> Self {
        Self {
            pos,
            clr,
            tex,
        }
    }

    pub fn new_t(pos: (f32, f32, f32), clr: (f32, f32, f32), tex: (f32, f32)) -> Self {
        Self::new(pos.into(), clr.into(), tex.into())
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
    pub fn new(pos: f32_f32_f32, tex: f32_f32) -> Self {
        Self {
            pos,
            tex,
        }
    }

    pub fn new_t(pos: (f32, f32, f32), tex: (f32, f32)) -> Self {
        Self::new(pos.into(), tex.into())
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
    pub fn new(pos: f32_f32_f32, nor: f32_f32_f32, tex: f32_f32) -> Self {
        Self {
            pos,
            tex,
            nor,
        }
    }

    pub fn new_t(pos: (f32, f32, f32), nor: (f32, f32, f32), tex: (f32, f32)) -> Self {
        Self::new(pos.into(), nor.into(), tex.into())
    }
}


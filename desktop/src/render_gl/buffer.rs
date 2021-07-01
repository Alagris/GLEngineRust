use gl;
use failure::err_msg;
use crate::render_gl::gl_error::drain_gl_errors;

pub trait BufferType {
    const BUFFER_TYPE: gl::types::GLuint;
}


pub struct Buffer<B, T> where B: BufferType {
    gl: gl::Gl,
    vbo: gl::types::GLuint,
    _marker: ::std::marker::PhantomData<B>,
    _marker2: ::std::marker::PhantomData<T>,
}

impl<B, T> Buffer<B, T> where B: BufferType {
    pub fn new(gl: &gl::Gl) -> Buffer<B, T> {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo);
        }

        Buffer {
            gl: gl.clone(),
            vbo,
            _marker: ::std::marker::PhantomData,
            _marker2: ::std::marker::PhantomData,
        }
    }

    pub fn target()->gl::types::GLuint{
        B::BUFFER_TYPE
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindBuffer(B::BUFFER_TYPE, self.vbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindBuffer(B::BUFFER_TYPE, 0);
        }
    }

    pub fn mem_size(&self)->usize{
        let mut size:gl::types::GLint = 0;
        unsafe{
            self.bind();
            self.gl.GetBufferParameteriv(B::BUFFER_TYPE, gl::BUFFER_SIZE, &mut size);
            drain_gl_errors(&self.gl);
            self.unbind();
        }
        let size = size as usize;
        assert_eq!(size%std::mem::size_of::<T>(),0);
        size
    }
    pub fn len(&self)->usize{
        self.mem_size()/std::mem::size_of::<T>()
    }

    pub fn static_draw_data(&self, data: &[T]) {
        self.bind();
        unsafe {
            self.gl.BufferData(
                B::BUFFER_TYPE, // target
                (data.len() * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );
        }
        self.unbind();
    }

    pub fn update(&self, data: &[T]) -> Result<(), failure::Error> {
        let len = data.len()*std::mem::size_of::<T>();
        if self.len() == len {
            self.bind();
            unsafe {
                self.gl.BufferSubData(B::BUFFER_TYPE, 0, len as gl::types::GLsizeiptr, data.as_ptr() as *const gl::types::GLvoid);
            }
            self.unbind();
            Ok(())
        } else {
            Err("Incorrect size").map_err(err_msg)
        }
    }
}

impl<B, T> Drop for Buffer<B, T> where B: BufferType {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteBuffers(1, &mut self.vbo);
        }
    }
}


pub struct BufferTypeArray;

impl BufferType for BufferTypeArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ARRAY_BUFFER;
}

pub struct BufferTypeElementArray;

impl BufferType for BufferTypeElementArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ELEMENT_ARRAY_BUFFER;
}

pub type ArrayBuffer<T> = Buffer<BufferTypeArray, T>;
pub type ElementArrayBuffer<T> = Buffer<BufferTypeElementArray, T>;


pub struct VertexArray {
    gl: gl::Gl,
    vao: gl::types::GLuint,
}

impl VertexArray {
    pub fn new(gl: &gl::Gl) -> VertexArray {
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut vao);
            drain_gl_errors(gl);
        }

        VertexArray {
            gl: gl.clone(),
            vao,
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindVertexArray(self.vao);
            drain_gl_errors(&self.gl);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindVertexArray(0);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteVertexArrays(1, &mut self.vao);
        }
    }
}

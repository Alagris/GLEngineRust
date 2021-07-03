use crate::render_gl::gl_error::drain_gl_errors;
use failure::{err_msg, Error};
use gl;
use crate::render_gl::data::VertexAttribPointers;

pub trait BufferUsage {
    const BUFFER_USAGE: gl::types::GLenum;
}

pub struct BufferStaticDraw;

impl BufferUsage for BufferStaticDraw {
    const BUFFER_USAGE: gl::types::GLenum = gl::STATIC_DRAW;
}

pub struct BufferDynamicDraw;

impl BufferUsage for BufferDynamicDraw {
    const BUFFER_USAGE: gl::types::GLenum = gl::DYNAMIC_DRAW;
}


pub struct BufferDynamicFixedLen;

impl BufferUsage for BufferDynamicFixedLen {
    const BUFFER_USAGE: gl::types::GLenum = gl::DYNAMIC_DRAW;
}

pub trait AnyBuffer<T> {
    fn new(data: &[T], gl: &gl::Gl) -> Self;
    fn capacity(&self) -> usize;
}

pub struct Buffer<B, T, U> where B: BufferType, U: BufferUsage {
    gl: gl::Gl,
    vbo: gl::types::GLuint,
    len: usize,
    _marker_target: ::std::marker::PhantomData<B>,
    _marker_data_type: ::std::marker::PhantomData<T>,
    _marker_usage: ::std::marker::PhantomData<U>,
}

impl<B, T, U> Buffer<B, T, U> where B: BufferType, U: BufferUsage {
    pub fn target() -> gl::types::GLuint {
        B::BUFFER_TYPE
    }
    pub fn usage() -> gl::types::GLenum {
        U::BUFFER_USAGE
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
    pub fn mem_size(&self) -> usize {
        let mut size: gl::types::GLint = 0;
        unsafe {
            self.bind();
            self.gl.GetBufferParameteriv(B::BUFFER_TYPE, gl::BUFFER_SIZE, &mut size);
            drain_gl_errors(&self.gl);
            self.unbind();
        }
        let size = size as usize;
        assert_eq!(size % std::mem::size_of::<T>(), 0);
        size
    }
    pub fn len(&self) -> usize {
        self.len
    }
    fn create_buffer(&self, capacity: usize, data: &[T]) {
        self.bind();
        let ptr = if !data.is_empty() {
            data.as_ptr() as *const gl::types::GLvoid
        } else {
            std::ptr::null()
        };
        unsafe {
            self.gl.BufferData(
                B::BUFFER_TYPE,                                                     // target
                (capacity * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                ptr, // pointer to data
                U::BUFFER_USAGE,                           // usage
            );
        }
        self.unbind();
    }
    fn gen_buffer(gl: &gl::Gl) -> gl::types::GLuint {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo);
        }
        vbo
    }
    unsafe fn update_unchecked(&self, data: &[T]) {
        self.bind();
        self.gl.BufferSubData(
            B::BUFFER_TYPE,
            0,
            (data.len() * std::mem::size_of::<T>()) as gl::types::GLsizeiptr,
            data.as_ptr() as *const gl::types::GLvoid,
        );
        self.unbind();
    }
}
impl<B, T> AnyBuffer<T> for Buffer<B, T, BufferStaticDraw> where B: BufferType {
    fn new(data: &[T], gl: &gl::Gl) -> Self {
        let me = Buffer {
            gl: gl.clone(),
            vbo: Self::gen_buffer(gl),
            len: data.len(),
            _marker_target: Default::default(),
            _marker_data_type: Default::default(),
            _marker_usage: Default::default(),
        };
        me.create_buffer(data.len(), data);
        me
    }
    fn capacity(&self) -> usize {
        self.len()
    }
}

impl<B, T> Buffer<B, T, BufferDynamicDraw> where B: BufferType {
    pub fn with_capacity(capacity: usize, gl: &gl::Gl) -> Self {
        let me = Buffer {
            gl: gl.clone(),
            vbo: Self::gen_buffer(gl),
            len: 0,
            _marker_target: Default::default(),
            _marker_data_type: Default::default(),
            _marker_usage: Default::default(),
        };
        me.create_buffer(capacity, &[]);
        me
    }
    /**The capacity will grow exponentially, doubling each time a reallocation is performed.
    */
    fn amortization(necessary_length: usize) -> usize {
        if necessary_length > usize::MAX >> 1 {//without this, the loop could become infinite in some edge cases
            usize::MAX
        } else {
            let mut pow_2 = 1;
            while pow_2 < necessary_length {
                pow_2 <<= 1;
            }
            pow_2
        }
    }
    pub fn update(&mut self, data: &[T]) {
        let len = data.len();
        let capacity = self.capacity();
        if len <= capacity {
            unsafe {
                self.update_unchecked(data);
            }
            self.len = data.len();
        } else {
            let new_capacity = Self::amortization(len);
            self.create_buffer(new_capacity, &[]);
            unsafe {
                self.update_unchecked(data);
            }
        }
    }
}

impl<B, T> AnyBuffer<T> for Buffer<B, T, BufferDynamicDraw> where B: BufferType {
    fn new(data: &[T], gl: &gl::Gl) -> Self {
        let capacity = Self::amortization(data.len());
        let mut me = Self::with_capacity(capacity, gl);
        me.update(data);
        me
    }

    fn capacity(&self) -> usize {
        self.mem_size() / std::mem::size_of::<T>()
    }
}

impl<B, T> AnyBuffer<T> for Buffer<B, T, BufferDynamicFixedLen> where B: BufferType {
    fn new(data: &[T], gl: &gl::Gl) -> Self {
        let me = Buffer {
            gl: gl.clone(),
            vbo: Self::gen_buffer(gl),
            len: data.len(),
            _marker_target: Default::default(),
            _marker_data_type: Default::default(),
            _marker_usage: Default::default(),
        };
        me.create_buffer(data.len(), data);
        me
    }
    fn capacity(&self) -> usize {
        self.len()
    }
}
impl<B, T> Buffer<B, T, BufferDynamicFixedLen> where B: BufferType {
    pub fn update(&mut self, data: &[T]) -> Result<(), failure::Error> {
        let len = data.len();
        let expected_len = self.len();
        if len == expected_len {
            unsafe {
                Ok(self.update_unchecked(data))
            }
        } else {
            Err(format!("Expected length {} but got data of size {}", expected_len, len)).map_err(err_msg)
        }
    }
}

impl<B, T, U> Drop for Buffer<B, T, U> where B: BufferType, U: BufferUsage {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteBuffers(1, &mut self.vbo);
        }
    }
}

pub trait BufferType {
    const BUFFER_TYPE: gl::types::GLuint;
}

pub struct BufferTypeArray;

impl BufferType for BufferTypeArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ARRAY_BUFFER;
}

pub struct BufferTypeElementArray;

impl BufferType for BufferTypeElementArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ELEMENT_ARRAY_BUFFER;
}

pub type ArrayBuffer<T> = Buffer<BufferTypeArray, T, BufferStaticDraw>;
pub type DynamicBuffer<T> = Buffer<BufferTypeArray, T, BufferDynamicDraw>;
pub type ElementArrayBuffer<T> = Buffer<BufferTypeElementArray, T, BufferStaticDraw>;

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

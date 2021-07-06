use crate::render_gl::buffer::{ BufferDynamicDraw};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use crate::render_gl::gl_error::drain_gl_errors;

pub trait BufferUsage {
    const BUFFER_USAGE: gl::types::GLenum;
}
pub struct Constant{}

impl BufferUsage for Constant {
    const BUFFER_USAGE: gl::types::GLenum = gl::STATIC_DRAW;
}

pub struct Variable{}

impl BufferUsage for Variable {
    const BUFFER_USAGE: gl::types::GLenum = gl::DYNAMIC_DRAW;
}


pub trait BufferType {
    const BUFFER_TYPE: gl::types::GLuint;
}

pub struct Std140 {}

impl BufferType for Std140 {
    const BUFFER_TYPE: gl::types::GLuint = gl::UNIFORM_BUFFER;
}

pub struct Std430 {}

impl BufferType for Std430 {
    const BUFFER_TYPE: gl::types::GLuint = gl::SHADER_STORAGE_BUFFER;
}

pub struct UniformBuffer<B: BufferType, T, U: BufferUsage, const BindingPoint: u32> {
    //BindingPoint is a compile-time constant,
    // because shaders could have it hard-coded and it's better to keep track of it explicitly.
    data: Box<T>,
    // This thing could potentially be really large, so it's better to box it
    gl: gl::Gl,
    ubo: gl::types::GLuint,
    _usage: PhantomData<U>,
    _type: PhantomData<B>,
}

impl<B: BufferType, T, U: BufferUsage, const BindingPoint: u32> UniformBuffer<B, T, U, BindingPoint> {
    pub fn bind_base(&self){
        unsafe{
            self.gl.BindBufferBase(B::BUFFER_TYPE, BindingPoint, self.ubo)
        }
    }
    pub fn new(data: T, gl: &gl::Gl) -> Self {
        if B::BUFFER_TYPE==gl::UNIFORM_BUFFER{
            assert_eq!(std::mem::size_of::<T>() % 4, 0,"GLSL requires that all variables are padded to 16 bits for UNIFORM_BUFFER");
        }
        let ubo = Self::gen_buffer(gl);
        let data = Box::new(data);
        let me = Self {
            data,
            gl: gl.clone(),
            ubo,
            _usage: PhantomData,
            _type: PhantomData
        };
        me.create_buffer();
        me
    }

    pub fn gen_buffer(gl: &gl::Gl) -> gl::types::GLuint {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo);
        }
        drain_gl_errors(gl);
        vbo
    }

    fn create_buffer(&self){
        self.bind();
        unsafe {
            self.gl.BufferData(
                B::BUFFER_TYPE,                                                     // target
                ::std::mem::size_of::<T>() as gl::types::GLsizeiptr, // size of data in bytes
                self.data.as_ref() as *const T as *const gl::types::GLvoid, // pointer to data
                U::BUFFER_USAGE,                           // usage
            );
            drain_gl_errors(&self.gl);
        }
        self.unbind();
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindBuffer(B::BUFFER_TYPE, self.ubo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindBuffer(B::BUFFER_TYPE, 0);
        }
    }
}

impl<B: BufferType, T, U: BufferUsage, const BindingPoint: u32> Deref for UniformBuffer<B, T, U, BindingPoint>  {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<B, T, const BindingPoint:u32> UniformBuffer<B, T, Variable,BindingPoint> where B: BufferType {

    pub fn update(&mut self) {
        self.bind();
        unsafe{
            self.gl.BufferSubData(
                B::BUFFER_TYPE,
                0,
                std::mem::size_of::<T>() as gl::types::GLsizeiptr,
                self.data.as_ref() as * const T as *const gl::types::GLvoid,
            );
            drain_gl_errors(&self.gl);
        }
        self.unbind();
    }
}

impl<B:BufferType, T, const BindingPoint:u32> DerefMut for UniformBuffer<B, T, Variable,BindingPoint>  {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

// Data types aligned to 16 bytes because GLSL requires everything to be aligned like this in uniform buffer
#[repr(align(16))]
pub struct f32_a16(f32);
#[repr(align(16))]
pub struct f64_a16(f64);
#[repr(align(16))]
pub struct i64_a16(i64);
#[repr(align(16))]
pub struct i32_a16(i32);
#[repr(align(16))]
pub struct i16_a16(i16);
#[repr(align(16))]
pub struct i8_a16(i8);
#[repr(align(16))]
pub struct u64_a16(u64);
#[repr(align(16))]
pub struct u32_a16(u32);
#[repr(align(16))]
pub struct u16_a16(u16);
#[repr(align(16))]
pub struct u8_a16(u8);
#[repr(align(16))]
pub struct vec3_a16(glm::Vec3);
#[repr(align(16))]
pub struct ivec3_a16(glm::IVec3);
#[repr(align(16))]
pub struct uvec3_a16(glm::UVec3);


pub type ShaderStorageBuffer<T,U:BufferUsage,const BindingPoint:u32> = UniformBuffer<Std430, T, U,BindingPoint>;
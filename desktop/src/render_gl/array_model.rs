use crate::render_gl::buffer::{ArrayBuffer, VertexArray, BufferUsage, BufferTypeArray, Buffer};
use crate::render_gl::data::VertexAttribPointers;

use gl;

use crate::render_gl::gl_error::drain_gl_errors;

pub enum Primitive{
    Points = gl::POINTS as isize,
    Lines = gl::LINES as isize,
    Triangles = gl::TRIANGLES as isize,
    LineStrip = gl::LINE_STRIP as isize,
}

impl Primitive{
    pub fn gl_enum(self)->gl::types::GLenum{
        self as gl::types::GLenum
    }
}

pub struct ArrayModel<T: VertexAttribPointers, U:BufferUsage> {
    vbo: Buffer<BufferTypeArray,T,U>,
    vao: VertexArray,
    gl: gl::Gl,
}

impl<T: VertexAttribPointers, U:BufferUsage> ArrayModel<T,U> {
    pub fn vbo(&self) -> &Buffer<BufferTypeArray,T,U> {
        &self.vbo
    }
    pub fn vbo_mut(&mut self) -> &mut Buffer<BufferTypeArray,T,U> {
        &mut self.vbo
    }
    pub fn gl(&self) -> &gl::Gl {
        &self.gl
    }
    pub fn vao(&self) -> &VertexArray {
        &self.vao
    }
    pub fn new(vbo: Buffer<BufferTypeArray,T,U>, gl: &gl::Gl) -> Self{
        let vao = VertexArray::new(gl);
        vao.bind();
        vbo.bind();
        T::vertex_attrib_pointers(gl);
        vbo.unbind();
        vao.unbind();
        drain_gl_errors(gl);
        Self {
            vbo,
            vao,
            gl: gl.clone(),
        }
    }

    pub fn bind(&self) {
        self.vao.bind();
    }
    pub fn unbind(&self) {
        self.vao.unbind();
    }
    pub fn len_vertices(&self) -> usize {
        self.vbo.len()
    }
    pub fn draw_vertices(&self, primitive: Primitive, vertices:i32) {
        unsafe {
            self.bind();
            self.gl.DrawArrays(primitive.gl_enum(), 0, vertices);
            self.unbind();
            drain_gl_errors(self.gl());
        }
    }
    pub fn draw(&self, primitive: Primitive) {
        let vertices = self.len_vertices() as i32;
        self.draw_vertices(primitive,vertices)
    }
}

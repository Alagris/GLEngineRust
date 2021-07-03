use crate::render_gl::buffer::{ArrayBuffer, BufferUsage, Buffer, BufferTypeArray};
use crate::render_gl::data::VertexAttribPointers;

use core::ptr;
use gl;

use crate::render_gl::gl_error::drain_gl_errors;
use crate::render_gl::model::Model;
use std::ops::{Deref, DerefMut};
use crate::render_gl::array_model::ArrayModel;

pub struct InstancedArrayModel<T: VertexAttribPointers, I: VertexAttribPointers,TU:BufferUsage,IU:BufferUsage> {
    ibo: Buffer<BufferTypeArray,I,IU>, //instance buffer
    model: ArrayModel<T,TU>,
}

impl<T: VertexAttribPointers, I: VertexAttribPointers,TU:BufferUsage,IU:BufferUsage> Deref for InstancedArrayModel<T, I, TU, IU> {
    type Target = ArrayModel<T,TU>;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}
impl<T: VertexAttribPointers, I: VertexAttribPointers,TU:BufferUsage,IU:BufferUsage> DerefMut for InstancedArrayModel<T, I, TU, IU> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model
    }
}
impl<T: VertexAttribPointers, I: VertexAttribPointers,TU:BufferUsage,IU:BufferUsage> InstancedArrayModel<T, I, TU, IU>{
    pub fn len_instances(&self) -> usize {
        self.ibo.len()
    }
    pub fn ibo(&self) -> &Buffer<BufferTypeArray,I,IU> {
        &self.ibo
    }
    pub fn ibo_mut(&mut self) -> &mut Buffer<BufferTypeArray,I,IU> {
        &mut self.ibo
    }
    pub fn new(ibo: Buffer<BufferTypeArray,I,IU>, model: ArrayModel<T,TU>) -> Self{
        model.vao().bind();
        ibo.bind();
        I::vertex_attrib_pointers(model.gl());
        ibo.unbind();
        model.vao().unbind();
        drain_gl_errors(model.gl());
        Self { ibo, model }
    }

    fn draw_instanced(&self, primitive: gl::types::GLenum, instance_count: usize) {
        let vertices = self.len_vertices();
        unsafe {
            self.bind();
            self.model.gl().DrawArraysInstanced(
                primitive,
                0,
                vertices as gl::types::GLsizei,
                instance_count as gl::types::GLsizei,
            );
            drain_gl_errors(self.model.gl());
            self.unbind();
        }
    }
    pub fn draw_instanced_triangles(&self, instance_count: usize) {
        self.draw_instanced(gl::TRIANGLES, instance_count);
    }

    pub fn draw_instanced_lines(&self, instance_count: usize) {
        self.draw_instanced(gl::LINES, instance_count);
    }
    pub fn draw_instanced_line_strip(&self, instance_count: usize) {
        self.draw_instanced(gl::LINE_STRIP, instance_count);
    }
}

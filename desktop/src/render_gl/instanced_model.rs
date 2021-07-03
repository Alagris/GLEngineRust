use crate::render_gl::buffer::{ArrayBuffer, BufferUsage, BufferTypeArray, Buffer};
use crate::render_gl::data::VertexAttribPointers;

use core::ptr;
use gl;

use crate::render_gl::gl_error::drain_gl_errors;
use crate::render_gl::model::Model;
use std::ops::{Deref, DerefMut};

pub struct InstancedModel<T: VertexAttribPointers, I: VertexAttribPointers, TU:BufferUsage, IU:BufferUsage> {
    ibo: Buffer<BufferTypeArray,I,IU>, //instance buffer
    model: Model<T,TU>,
}

impl<T: VertexAttribPointers, I: VertexAttribPointers, TU:BufferUsage, IU:BufferUsage> Deref for InstancedModel<T, I, TU, IU> {
    type Target = Model<T, TU>;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl<T: VertexAttribPointers, I: VertexAttribPointers, TU:BufferUsage, IU:BufferUsage> DerefMut for InstancedModel<T, I, TU, IU> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model
    }
}

impl<T: VertexAttribPointers, I: VertexAttribPointers, TU:BufferUsage, IU:BufferUsage> InstancedModel<T, I, TU, IU> {
    pub fn len_instances(&self) -> usize {
        self.ibo.len()
    }
    pub fn ibo(&self) -> &Buffer<BufferTypeArray,I,IU> {
        &self.ibo
    }
    pub fn ibo_mut(&mut self) -> &mut Buffer<BufferTypeArray,I,IU> {
        &mut self.ibo
    }
    pub fn new(ibo: Buffer<BufferTypeArray,I,IU>, model: Model<T, TU>) -> Self{
        model.vao().bind();
        model.ebo().bind();
        ibo.bind();
        I::vertex_attrib_pointers(model.gl());
        ibo.unbind();
        model.ebo().unbind();
        model.vao().unbind();
        drain_gl_errors(model.gl());
        Self { ibo, model }
    }

    fn draw_instanced(&self, primitive: gl::types::GLenum, instance_count: usize) {
        let indices = self.len_indices() as i32;
        unsafe {
            self.bind();
            self.model.gl().DrawElementsInstanced(
                primitive,
                indices,
                gl::UNSIGNED_INT,
                ptr::null(),
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

use crate::render_gl::buffer::{ArrayBuffer, BufferUsage, BufferTypeArray};
use crate::render_gl::data::VertexAttribPointers;

use core::ptr;
use gl;

use crate::render_gl::gl_error::drain_gl_errors;
use crate::render_gl::model::Model;
use crate::render_gl::buffer::Buffer;
use std::ops::Deref;
use crate::render_gl::logical_model::LogicalModel;

pub struct InstancedLogicalModel<I: VertexAttribPointers, U:BufferUsage> {
    ibo: Buffer<BufferTypeArray,I,U>, //instance buffer
    model: LogicalModel,
}

impl<I: VertexAttribPointers, U:BufferUsage> Deref for InstancedLogicalModel<I,U> {
    type Target = LogicalModel;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl<I: VertexAttribPointers, U:BufferUsage> InstancedLogicalModel<I,U> {
    pub fn ibo(&self) -> &Buffer<BufferTypeArray,I,U> {
        &self.ibo
    }
    pub fn ibo_mut(&mut self) -> &mut Buffer<BufferTypeArray,I,U> {
        &mut self.ibo
    }
    pub fn new(ibo: Buffer<BufferTypeArray,I,U>, gl:&gl::Gl) -> Self{
        let model = LogicalModel::new(gl);
        model.vao().bind();
        ibo.bind();
        I::vertex_attrib_pointers(gl);
        ibo.unbind();
        model.vao().unbind();
        drain_gl_errors(gl);
        Self { ibo, model }
    }

    fn draw_instanced(&self,
                      primitive: gl::types::GLenum,
                      first_vertex: gl::types::GLint,
                      vertex_count: gl::types::GLsizei,
                      instance_count: usize) {
        unsafe {
            self.bind();
            self.model.gl().DrawArraysInstanced(
                primitive,
                first_vertex,
                vertex_count,
                instance_count as gl::types::GLsizei,
            );
            drain_gl_errors(self.model.gl());
            self.unbind();
        }
    }
    pub fn draw_instanced_triangles(&self,
                                    first_vertex: gl::types::GLint,
                                    vertex_count: gl::types::GLsizei,
                                    instance_count: usize) {
        self.draw_instanced(gl::TRIANGLES, first_vertex, vertex_count,instance_count);
    }

    pub fn draw_instanced_lines(&self,
                                first_vertex: gl::types::GLint,
                                vertex_count: gl::types::GLsizei,
                                instance_count: usize) {
        self.draw_instanced(gl::LINES, first_vertex, vertex_count,instance_count);
    }
    pub fn draw_instanced_line_strip(&self,
                                     first_vertex: gl::types::GLint,
                                     vertex_count: gl::types::GLsizei,
                                     instance_count: usize) {
        self.draw_instanced(gl::LINE_STRIP, first_vertex, vertex_count,instance_count);
    }
}

use crate::render_gl::buffer::ArrayBuffer;
use crate::render_gl::data::VertexAttribPointers;

use core::ptr;
use gl;

use crate::render_gl::gl_error::drain_gl_errors;
use crate::render_gl::model::Model;
use std::ops::Deref;
use crate::render_gl::logical_model::LogicalModel;

pub struct InstancedLogicalModel<I: VertexAttribPointers> {
    ibo: ArrayBuffer<I>, //instance buffer
    model: LogicalModel,
}

impl< I: VertexAttribPointers> Deref for InstancedLogicalModel<I> {
    type Target = LogicalModel;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl<I: VertexAttribPointers> InstancedLogicalModel<I> {
    pub fn len_instances(&self) -> usize {
        self.ibo.len()
    }
    pub fn ibo(&self) -> &ArrayBuffer<I> {
        &self.ibo
    }
    pub fn new(instances: &[I], gl:&gl::Gl) -> Result<Self, failure::Error> {
        let model = LogicalModel::new(gl)?;
        let ibo = ArrayBuffer::new(gl);

        ibo.static_draw_data(instances);

        model.vao().bind();
        ibo.bind();
        I::vertex_attrib_pointers(gl);
        ibo.unbind();
        model.vao().unbind();
        drain_gl_errors(gl);
        let me = Self { ibo, model };
        assert_eq!(me.len_instances(), instances.len());
        Ok(me)
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
    pub fn update_instances(&mut self, instances: &[I]) -> Result<(), failure::Error> {
        self.ibo().update(instances)
    }
}

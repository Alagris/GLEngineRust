use std::fs::File;
use std::io::BufReader;
use crate::render_gl::data::{Vertex, VertexTex, VertexTexCol, VertexTexNor, f32_f32_f32, VertexTexNorTan, VertexAttribPointers};
use crate::render_gl::buffer::{ArrayBuffer, ElementArrayBuffer, VertexArray};
use obj::{Obj, IndexTuple, SimplePolygon};
use core::ptr;
use gl;
use std::collections::HashMap;
use failure::{err_msg, Error};
use std::collections::hash_map::Entry;
use std::net::Incoming;
use glm::U3;
use glm::U1;
use std::path::Path;
use crate::resources::Resources;
use std::num::NonZeroUsize;
use std::hash::Hash;
use crate::render_gl::util::type_name;
use std::fmt::Debug;
use crate::render_gl::model::Model;
use std::ops::Deref;
use crate::render_gl::gl_error::drain_gl_errors;

pub struct InstancedModel<T:VertexAttribPointers,I:VertexAttribPointers> {
    ibo: ArrayBuffer<I>,//instance buffer
    model:Model<T>
}

impl <T:VertexAttribPointers,I:VertexAttribPointers> Deref for InstancedModel<T,I>{
    type Target = Model<T>;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl <T:VertexAttribPointers,I:VertexAttribPointers> InstancedModel<T,I> {
    pub fn len_instances(&self)->usize{
        self.ibo.len()
    }
    pub fn ibo(&self)-> &ArrayBuffer<I>{
        &self.ibo
    }
    pub fn new(instances:&[I],model:Model<T>) -> Result<Self, failure::Error> {
        let ibo = ArrayBuffer::new(model.gl());

        ibo.bind();
        ibo.static_draw_data(instances);
        ibo.unbind();

        model.vao().bind();
        model.ebo().bind();
        I::vertex_attrib_pointers(model.gl());
        model.ebo().unbind();
        model.vao().unbind();
        drain_gl_errors(model.gl());
        let me = Self { ibo, model };
        assert_eq!(me.len_instances(),instances.len());
        Ok(me)
    }



    fn draw_instanced(&self, primitive: gl::types::GLenum, instance_count:usize) {
        let indices = self.len_indices() as i32;
        unsafe {
            self.bind();
            self.model.gl().DrawElementsInstanced(primitive, indices, gl::UNSIGNED_INT, ptr::null(),instance_count as gl::types::GLsizei);
            drain_gl_errors(self.model.gl());
            self.unbind();

        }
    }
    pub fn draw_instanced_triangles(&self, instance_count:usize) {
        self.draw_instanced(gl::TRIANGLES, instance_count);
    }

    pub fn draw_instanced_lines(&self, instance_count:usize) {
        self.draw_instanced(gl::LINES, instance_count);
    }
    pub fn draw_instanced_line_strip(&self, instance_count:usize) {
        self.draw_instanced(gl::LINE_STRIP, instance_count);
    }

}


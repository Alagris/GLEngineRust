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
use crate::render_gl::gl_error::drain_gl_errors;

pub struct LogicalModel{
    vao: VertexArray,
    gl: gl::Gl,
}

impl LogicalModel {
    pub fn gl(&self)-> &gl::Gl{
        &self.gl
    }
    pub fn vao(&self)-> &VertexArray{
        &self.vao
    }
    pub fn new(gl: &gl::Gl) -> Result<Self, failure::Error> {
        Ok(Self { vao:VertexArray::new(gl), gl: gl.clone() })
    }

    pub fn bind(&self) {
        self.vao.bind();
    }
    pub fn unbind(&self) {
        self.vao.unbind();
    }
    fn draw(&self, primitive: gl::types::GLenum, first:gl::types::GLint,count:gl::types::GLsizei) {
        unsafe {
            self.bind();
            self.gl.DrawArrays(primitive, first, count);
            self.unbind();
            drain_gl_errors(self.gl());
        }
    }
    pub fn draw_triangles(&self,first:gl::types::GLint,count:gl::types::GLsizei) {
        self.draw(gl::TRIANGLES,first, count);
    }

    pub fn draw_lines(&self,first:gl::types::GLint,count:gl::types::GLsizei) {
        self.draw(gl::LINES,first, count);
    }
    pub fn draw_line_strip(&self,first:gl::types::GLint,count:gl::types::GLsizei) {
        self.draw(gl::LINE_STRIP,first, count);
    }
}

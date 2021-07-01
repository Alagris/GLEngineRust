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

pub struct ArrayModel<T:VertexAttribPointers> {
    vbo: ArrayBuffer<T>,
    vao: VertexArray,
    gl: gl::Gl,
}

impl <T:VertexAttribPointers> ArrayModel<T> {
    pub fn vbo(&self)-> &ArrayBuffer<T>{
        &self.vbo
    }
    pub fn gl(&self)-> &gl::Gl{
        &self.gl
    }
    pub fn vao(&self)-> &VertexArray{
        &self.vao
    }
    pub fn new(vertices:&[T], gl: &gl::Gl) -> Result<Self, failure::Error> {
        let vbo = ArrayBuffer::new(gl);
        let vao = VertexArray::new(gl);

        vbo.static_draw_data(vertices);

        vao.bind();
        vbo.bind();
        T::vertex_attrib_pointers(gl);
        vbo.unbind();
        vao.unbind();
        drain_gl_errors(gl);
        let me = Self { vbo, vao, gl: gl.clone() };
        assert_eq!(me.len_vertices(),vertices.len());
        Ok(me)
    }

    pub fn bind(&self) {
        self.vao.bind();
    }
    pub fn unbind(&self) {
        self.vao.unbind();
    }
    pub fn len_vertices(&self)->usize{
        self.vbo.len()
    }
    fn draw(&self, primitive: gl::types::GLenum) {
        let vertices = self.len_vertices() as i32;
        unsafe {
            self.bind();
            self.gl.DrawArrays(primitive, 0, vertices);
            self.unbind();
            drain_gl_errors(self.gl());
        }
    }
    pub fn draw_triangles(&self) {
        self.draw(gl::TRIANGLES);
    }

    pub fn draw_lines(&self) {
        self.draw(gl::LINES);
    }
    pub fn draw_line_strip(&self) {
        self.draw(gl::LINE_STRIP);
    }
    pub fn update_vbo(&mut self, vbo: &[T]) -> Result<(), failure::Error> {
        self.vbo.update(vbo)
    }
}

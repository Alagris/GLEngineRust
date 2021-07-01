use crate::render_gl::buffer::VertexArray;

use gl;

use crate::render_gl::gl_error::drain_gl_errors;

pub struct LogicalModel {
    vao: VertexArray,
    gl: gl::Gl,
}

impl LogicalModel {
    pub fn gl(&self) -> &gl::Gl {
        &self.gl
    }
    pub fn vao(&self) -> &VertexArray {
        &self.vao
    }
    pub fn new(gl: &gl::Gl) -> Result<Self, failure::Error> {
        Ok(Self {
            vao: VertexArray::new(gl),
            gl: gl.clone(),
        })
    }

    pub fn bind(&self) {
        self.vao.bind();
    }
    pub fn unbind(&self) {
        self.vao.unbind();
    }
    fn draw(
        &self,
        primitive: gl::types::GLenum,
        first: gl::types::GLint,
        count: gl::types::GLsizei,
    ) {
        unsafe {
            self.bind();
            self.gl.DrawArrays(primitive, first, count);
            self.unbind();
            drain_gl_errors(self.gl());
        }
    }
    pub fn draw_triangles(&self, first: gl::types::GLint, count: gl::types::GLsizei) {
        self.draw(gl::TRIANGLES, first, count);
    }

    pub fn draw_lines(&self, first: gl::types::GLint, count: gl::types::GLsizei) {
        self.draw(gl::LINES, first, count);
    }
    pub fn draw_line_strip(&self, first: gl::types::GLint, count: gl::types::GLsizei) {
        self.draw(gl::LINE_STRIP, first, count);
    }
}

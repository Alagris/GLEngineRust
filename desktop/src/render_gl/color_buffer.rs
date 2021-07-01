use gl;
extern crate nalgebra_glm as glm;
pub struct ColorBuffer {
    pub color: glm::TVec4<f32>,
}

impl ColorBuffer {
    pub fn new(color: glm::TVec4<f32>) -> ColorBuffer {
        ColorBuffer { color }
    }

    pub fn from_color(color: glm::TVec3<f32>) -> ColorBuffer {
        Self::new(glm::vec4(color[0], color[1], color[2], 1f32))
    }

    pub fn update_color(&mut self, color: glm::TVec3<f32>) {
        self.color = glm::vec4(color[0], color[1], color[2], 1f32);
    }

    pub fn set_used(&self, gl: &gl::Gl) {
        unsafe {
            gl.ClearColor(self.color.x, self.color.y, self.color.z, 1.0);
        }
    }

    pub fn clear(&self, gl: &gl::Gl) {
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}

impl From<(f32, f32, f32)> for ColorBuffer {
    fn from(other: (f32, f32, f32)) -> Self {
        Self::new(glm::vec4(other.0, other.1, other.2, 1.0))
    }
}

impl From<(f32, f32, f32, f32)> for ColorBuffer {
    fn from(other: (f32, f32, f32, f32)) -> Self {
        Self::new(glm::vec4(other.0, other.1, other.2, other.3))
    }
}

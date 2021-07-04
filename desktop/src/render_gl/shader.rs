use crate::render_gl::texture::{Cubemap, Texture};
use crate::resources::{self, Resources};
use gl;
use std;
use std::ffi::{CStr, CString};
use crate::render_gl::gl_error::drain_gl_errors;
use std::marker::PhantomData;
use crate::render_gl::uniform_buffer::{UniformBuffer, BufferUsage, BufferType, Constant, Variable, Std140, Std430};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to load resource {}", name)]
    ResourceLoad {
        name: String,
        #[cause]
        inner: resources::Error,
    },
    #[fail(display = "Can not determine shader type for resource {}", name)]
    CanNotDetermineShaderTypeForResource { name: String },
    #[fail(display = "Failed to compile shader {}: {}", name, message)]
    CompileError { name: String, message: String },
    #[fail(display = "Failed to link program {}: {}", name, message)]
    LinkError { name: String, message: String },
}

#[derive(Debug, Copy, Clone)]
pub struct UniformMatrix4fv {
    id: gl::types::GLint,
}

#[derive(Debug, Copy, Clone)]
pub struct UniformBufferBindingPoint<B:BufferType, T, const BindingPoint:u32> {
    id: gl::types::GLuint,
    _data_type:PhantomData<T>,
    _buffer_type:PhantomData<B>,
}

#[derive(Debug, Copy, Clone)]
pub struct UniformMatrix3fv {
    id: gl::types::GLint,
}

#[derive(Debug, Copy, Clone)]
pub struct UniformVec3fv {
    id: gl::types::GLint,
}

#[derive(Debug, Copy, Clone)]
pub struct UniformVec4fv {
    id: gl::types::GLint,
}

#[derive(Debug, Copy, Clone)]
pub struct Uniform1f {
    id: gl::types::GLint,
}

#[derive(Debug, Copy, Clone)]
pub struct Uniform1i {
    id: gl::types::GLint,
}

#[derive(Debug, Copy, Clone)]
pub struct UniformTexture {
    id: gl::types::GLint,
}

#[derive(Debug, Copy, Clone)]
pub struct UniformCubeTexture {
    id: gl::types::GLint,
}

pub struct Program {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Program, Error> {
        const POSSIBLE_EXT: [&str; 2] = [".vert", ".frag"];

        let mut resource_names = POSSIBLE_EXT
            .iter()
            .map(|file_extension| format!("{}{}", name, file_extension))
            .collect::<Vec<String>>();

        let geom_file = format!("{}.geom", name);
        if res.has_file(&geom_file) {
            resource_names.push(geom_file);
        } else {
            println!("Shader {} ignored", geom_file);
        }

        let shaders = resource_names
            .iter()
            .map(|resource_name| Shader::from_res(gl, res, resource_name))
            .collect::<Result<Vec<Shader>, Error>>()?;

        Program::from_shaders(gl, &shaders[..]).map_err(|message| Error::LinkError {
            name: name.into(),
            message,
        })
    }

    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };

        for shader in shaders {
            unsafe {
                gl.AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl.LinkProgram(program_id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            return Err(Self::query_error_string(gl, program_id));
        }

        for shader in shaders {
            unsafe {
                gl.DetachShader(program_id, shader.id());
            }
        }

        Ok(Program {
            gl: gl.clone(),
            id: program_id,
        })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }

    fn get_uniform(&self, name: &str) -> Result<gl::types::GLint, String> {
        let id: gl::types::GLint;
        unsafe {
            id = self.gl.GetUniformLocation(self.id, CString::new(name).unwrap().into_raw());
        }
        if id == -1 {
            return Err(format!("uniform '{}' not found!",name));
        }
        Ok(id)
    }

    fn get_uniform_block(&self, name: &str) -> Result<gl::types::GLuint, String> {
        let id: gl::types::GLuint;
        unsafe {
            id = self.gl.GetUniformBlockIndex(self.id, CString::new(name).unwrap().into_raw());
        }
        if id == gl::INVALID_INDEX{
            return Err(format!("uniform block '{}' not found!",name));
        }
        Ok(id)
    }
    pub fn get_uniform_std430<T,const BindingPoint:u32>(&self, name: &str) -> Result<UniformBufferBindingPoint<Std430,T,BindingPoint>, String> {
        self.get_uniform_buffer(name)
    }
    pub fn get_uniform_std140<T,const BindingPoint:u32>(&self, name: &str) -> Result<UniformBufferBindingPoint<Std140,T,BindingPoint>, String> {
        self.get_uniform_buffer(name)
    }
    pub fn get_uniform_buffer<B:BufferType, T,const BindingPoint:u32>(&self, name: &str) -> Result<UniformBufferBindingPoint<B,T,BindingPoint>, String> {
        self.get_uniform_block(name).map(|id| UniformBufferBindingPoint { id, _data_type: PhantomData, _buffer_type: PhantomData })
    }
    pub fn get_uniform_matrix4fv(&self, name: &str) -> Result<UniformMatrix4fv, String> {
        self.get_uniform(name).map(|id| UniformMatrix4fv { id })
    }
    pub fn get_uniform_matrix3fv(&self, name: &str) -> Result<UniformMatrix3fv, String> {
        self.get_uniform(name).map(|id| UniformMatrix3fv { id })
    }

    pub fn get_uniform_vec3fv(&self, name: &str) -> Result<UniformVec3fv, String> {
        self.get_uniform(name).map(|id| UniformVec3fv { id })
    }

    pub fn get_uniform_vec4fv(&self, name: &str) -> Result<UniformVec4fv, String> {
        self.get_uniform(name).map(|id| UniformVec4fv { id })
    }

    pub fn get_uniform_1f(&self, name: &str) -> Result<Uniform1f, String> {
        self.get_uniform(name).map(|id| Uniform1f { id })
    }

    pub fn get_uniform_1i(&self, name: &str) -> Result<Uniform1i, String> {
        self.get_uniform(name).map(|id| Uniform1i { id })
    }

    pub fn get_uniform_texture(&self, name: &str) -> Result<UniformTexture, String> {
        self.get_uniform(name).map(|id| UniformTexture { id })
    }

    pub fn get_uniform_cube_texture(&self, name: &str) -> Result<UniformCubeTexture, String> {
        self.get_uniform(name).map(|id| UniformCubeTexture { id })
    }

    pub fn set_uniform_buffer<B:BufferType, T,U:BufferUsage,const BindingPoint:u32>(
        &self,
        binding_point: UniformBufferBindingPoint<B,T,BindingPoint>,
        buffer: &UniformBuffer<B,T,U,BindingPoint>
    ) {
        unsafe {
            self.gl.UniformBlockBinding(self.id, binding_point.id, BindingPoint);
            drain_gl_errors(&self.gl);
            self.gl.BindBufferBase(gl::UNIFORM_BUFFER, BindingPoint, buffer.ubo());
            drain_gl_errors(&self.gl);
        }
    }

    pub fn set_uniform_texture(
        &self,
        uniform: UniformTexture,
        texture: &Texture,
        texture_binding_unit: u32,
    ) {
        unsafe {
            self.gl.ActiveTexture(gl::TEXTURE0 + texture_binding_unit); // Texture unit 0
            texture.bind();
            self.gl.Uniform1i(uniform.id, texture_binding_unit as i32);
        }
    }

    pub fn set_uniform_cube_texture(
        &self,
        uniform: UniformCubeTexture,
        texture: &Cubemap,
        texture_binding_unit: u32,
    ) {
        unsafe {
            self.gl.ActiveTexture(gl::TEXTURE0 + texture_binding_unit); // Texture unit 0
            texture.bind();
            self.gl.Uniform1i(uniform.id, texture_binding_unit as i32);
        }
    }

    pub fn set_uniform_1i(&self, uniform: Uniform1i, value: i32) {
        unsafe {
            self.gl.Uniform1i(uniform.id, value);
        }
    }
    pub fn set_uniform_1f(&self, uniform: Uniform1f, value: f32) {
        unsafe {
            self.gl.Uniform1f(uniform.id, value);
        }
    }

    pub fn set_uniform_matrix4fv(&self, uniform: UniformMatrix4fv, value: &[f32]) {
        unsafe {
            self.gl.UniformMatrix4fv(uniform.id, 1, gl::FALSE, value.as_ptr());
        }
    }

    pub fn set_uniform_matrix3fv(&self, uniform: UniformMatrix3fv, value: &[f32]) {
        unsafe {
            self.gl.UniformMatrix3fv(uniform.id, 1, gl::FALSE, value.as_ptr());
        }
    }

    pub fn set_uniform_vec3fv(&self, uniform: UniformVec3fv, value: &[f32]) {
        unsafe {
            self.gl.Uniform3fv(uniform.id, 1, value.as_ptr());
        }
    }

    pub fn set_uniform_vec4fv(&self, uniform: UniformVec4fv, value: &[f32]) {
        unsafe {
            self.gl.Uniform4fv(uniform.id, 1, value.as_ptr());
        }
    }

    fn query_error_string(gl: &gl::Gl, program_id: gl::types::GLuint) -> String {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl.GetProgramInfoLog(
                program_id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );
        }
        error.to_string_lossy().into_owned()
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Shader, Error> {
        println!("Loading shader {}", name);
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 3] = [
            (".vert", gl::VERTEX_SHADER),
            (".frag", gl::FRAGMENT_SHADER),
            (".geom", gl::GEOMETRY_SHADER),
        ];

        let shader_kind = POSSIBLE_EXT
            .iter()
            .find(|&&(file_extension, _)| name.ends_with(file_extension))
            .map(|&(_, kind)| kind)
            .ok_or_else(|| Error::CanNotDetermineShaderTypeForResource { name: name.into() })?;

        let source = res.load_cstring(name).map_err(|e| Error::ResourceLoad {
            name: name.into(),
            inner: e,
        })?;

        Shader::from_source(gl, &source, shader_kind).map_err(|message| Error::CompileError {
            name: name.into(),
            message,
        })
    }

    pub fn from_source(
        gl: &gl::Gl,
        source: &CStr,
        kind: gl::types::GLenum,
    ) -> Result<Shader, String> {
        let id = shader_from_source(gl, source, kind)?;
        Ok(Shader { gl: gl.clone(), id })
    }

    pub fn from_vert_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::FRAGMENT_SHADER)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

fn shader_from_source(
    gl: &gl::Gl,
    source: &CStr,
    kind: gl::types::GLenum,
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl.CreateShader(kind) };
    unsafe {
        gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        drain_gl_errors(gl);
        gl.CompileShader(id);
        drain_gl_errors(gl);
    }
    let mut success: gl::types::GLint = 1;
    unsafe {
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}

use crate::resources::Resources;
use failure::err_msg;
use gl;
use image::DynamicImage;
use image::GenericImageView;
use std::path::Path;

pub trait TextureType {
    const TEXTURE_TYPE: gl::types::GLuint;
}

pub struct Texture2D;

impl TextureType for Texture2D {
    const TEXTURE_TYPE: gl::types::GLuint = gl::TEXTURE_2D;
}

pub struct TextureCube;

impl TextureType for TextureCube {
    const TEXTURE_TYPE: gl::types::GLuint = gl::TEXTURE_CUBE_MAP;
}

pub struct Tex<B: TextureType> {
    gl: gl::Gl,
    texture: gl::types::GLuint,
    _marker: ::std::marker::PhantomData<B>,
}

impl<B: TextureType> Tex<B> {
    pub fn bind(&self) {
        unsafe {
            self.gl.BindTexture(B::TEXTURE_TYPE, self.texture);
        }
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.texture
    }
    unsafe fn bind_texture(gl: &gl::Gl, texture: u32) {
        gl.BindTexture(B::TEXTURE_TYPE, texture);
    }
    unsafe fn tex_parameteri(gl: &gl::Gl, what: gl::types::GLuint, value: i32) {
        gl.TexParameteri(B::TEXTURE_TYPE, what, value);
    }
}
#[derive(Copy, Clone,Eq, PartialEq)]
pub enum Filter{
    Linear=gl::LINEAR as isize, Nearest=gl::NEAREST as isize
}
impl Tex<Texture2D> {
    pub fn from_res(
        resource_name: &str,
        res: &Resources,
        gl: &gl::Gl) -> Result<Self, failure::Error> {
        Self::from_res_with_filter(resource_name,res,Filter::Linear,gl)
    }
    pub fn from_res_with_filter(
        resource_name: &str,
        res: &Resources,
        filter:Filter,
        gl: &gl::Gl,
    ) -> Result<Self, failure::Error> {
        println!("Loading texture {}", resource_name);
        Self::new_with_filter(&res.path(resource_name), filter, gl)
    }
    pub fn new(file: &Path, gl: &gl::Gl) -> Result<Self, failure::Error> {
        Self::new_with_filter(file,Filter::Linear,gl)
    }
    pub fn new_with_filter(file: &Path,filter:Filter, gl: &gl::Gl) -> Result<Self, failure::Error> {
        let mut texture = 0;
        let img = image::open(file).map_err(err_msg)?;
        let data = img.as_bytes();
        unsafe {
            gl.GenTextures(1, &mut texture);
            Self::bind_texture(gl, texture); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
                                             // set the texture wrapping parameters
            Self::tex_parameteri(gl, gl::TEXTURE_WRAP_S, gl::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
            Self::tex_parameteri(gl, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            // set texture filtering parameters
            Self::tex_parameteri(gl, gl::TEXTURE_MIN_FILTER, filter as i32);
            Self::tex_parameteri(gl, gl::TEXTURE_MAG_FILTER, filter as i32);
            // load image, create texture and generate mipmaps
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const gl::types::GLvoid,
            );
            gl.GenerateMipmap(gl::TEXTURE_2D);
            Self::bind_texture(gl, 0);
        }

        Ok(Self {
            gl: gl.clone(),
            texture,
            _marker: ::std::marker::PhantomData,
        })
    }
}

impl Tex<TextureCube> {
    pub fn from_res(
        files: [&str; 6],
        res: &Resources,
        gl: &gl::Gl,
    ) -> Result<Self, failure::Error> {
        println!("Loading cubemap from: {:?}", files);
        let files = files.map(|f| res.path(f));
        Self::new(files, gl)
    }
    pub fn new(files: [impl AsRef<Path>; 6], gl: &gl::Gl) -> Result<Self, failure::Error> {
        let mut texture = 0;
        let data: Result<Vec<DynamicImage>, _> = files
            .iter()
            .map(|file| image::open(file).map_err(err_msg))
            .collect();
        let data = data?;
        unsafe {
            gl.GenTextures(1, &mut texture);
            Self::bind_texture(gl, texture); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
        }
        for (i, img) in data.iter().enumerate() {
            unsafe {
                gl.TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,
                    gl::RGB as i32,
                    img.width() as i32,
                    img.height() as i32,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    img.as_bytes().as_ptr() as *const gl::types::GLvoid,
                );
            }
        }
        unsafe {
            // set the texture wrapping parameters
            Self::tex_parameteri(gl, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
            Self::tex_parameteri(gl, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            Self::tex_parameteri(gl, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
            // set texture filtering parameters
            Self::tex_parameteri(gl, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            Self::tex_parameteri(gl, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            // load image, create texture and generate mipmaps
            Self::bind_texture(gl, 0);
            //gl.GenerateMipmap(gl::TEXTURE_CUBE_MAP);
        }

        Ok(Self {
            gl: gl.clone(),
            texture,
            _marker: ::std::marker::PhantomData,
        })
    }
}

impl<B: TextureType> Drop for Tex<B> {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteTextures(1, &self.texture);
        }
    }
}

pub type Texture = Tex<Texture2D>;
pub type Cubemap = Tex<TextureCube>;

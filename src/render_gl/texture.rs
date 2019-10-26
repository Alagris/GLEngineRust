use gl;
use image::GenericImage;
use image::GenericImageView;
use std::path::Path;
use failure::err_msg;

pub struct Texture {
    gl: gl::Gl,
    texture: gl::types::GLuint,
}

impl Texture {
    pub fn new(file: &Path,gl: &gl::Gl) -> Result<Self,failure::Error> {
        let mut texture = 0;
        let img = image::open(file).map_err(err_msg)?;
        let data = img.raw_pixels();

        unsafe {
            gl.GenTextures(1, &mut texture);
            gl.BindTexture(gl::TEXTURE_2D, texture); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
            // set the texture wrapping parameters
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            // set texture filtering parameters
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            // load image, create texture and generate mipmaps
            gl.TexImage2D(gl::TEXTURE_2D,
                          0,
                          gl::RGB as i32,
                          img.width() as i32,
                          img.height() as i32,
                          0,
                          gl::RGB,
                          gl::UNSIGNED_BYTE,
                          data.as_ptr() as *const gl::types::GLvoid);
            gl.GenerateMipmap(gl::TEXTURE_2D);
        }

        Ok(Self{gl:gl.clone(),texture})
    }

    pub fn bind(&self){
        unsafe{
            self.gl.BindTexture(gl::TEXTURE_2D, self.texture);
        }
    }

    pub fn id(&self) -> gl::types::GLuint{
        self.texture
    }
}

impl Drop for Texture{
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteTextures(1, &self.texture);
        }
    }
}
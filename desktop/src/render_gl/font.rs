use std::path::Path;
use crate::resources::Resources;
use crate::render_gl::Program;
use crate::render_gl::texture::{Texture, Filter};
use crate::render_gl::buffer::{DynamicBuffer, BufferTypeArray, BufferDynamicDraw};
use crate::compute_cl::buffer::Buffer;
use crate::render_gl::instanced_logical_model::InstancedLogicalModel;
use crate::render_gl::shader::{UniformVec4fv, UniformTexture};
use failure::err_msg;

pub struct AsciiFont{
    font_program: Program,
    font_texture: Texture,
    model: InstancedLogicalModel<u8,BufferDynamicDraw>,
    text_position_and_size_uniform:UniformVec4fv,
    texture_uniform: UniformTexture
}

impl AsciiFont{
    pub fn new(gl:&gl::Gl, res:&Resources, font_bmp_path:&str)->Result<Self,failure::Error>{
        let font_program = Program::from_res(gl, res, "shaders/font")?;
        let font_texture = Texture::from_res_with_filter(font_bmp_path,res,false,Filter::Nearest,gl)?;
        let buffer = DynamicBuffer::<u8>::with_capacity(64,gl);
        let model = InstancedLogicalModel::new(buffer,gl);
        let text_position_and_size_uniform = font_program.get_uniform_vec4fv("text_position_and_size").map_err(err_msg)?;
        let texture_uniform = font_program.get_uniform_texture("myTextureSampler").map_err(err_msg)?;
        Ok(Self{font_program, model,font_texture,text_position_and_size_uniform,texture_uniform})
    }
    pub fn draw(&mut self, text:&str,x:f32,y:f32,glyph_width:f32,glyph_height:f32){
        let Self{ font_program, font_texture, model, text_position_and_size_uniform, texture_uniform } = self;
        model.ibo_mut().update(text.as_bytes());
        font_program.set_used();
        font_program.set_uniform_texture(*texture_uniform, font_texture,0);
        font_program.set_uniform_vec4fv(*text_position_and_size_uniform, &[x,y,glyph_width,glyph_height]);
        model.draw_instanced_triangles(0,6, text.len());

    }
}
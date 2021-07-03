use crate::render_gl::data::{VertexAttribPointers, VertexTexNor, VertexTexNorTan};

use gl;

use failure::err_msg;

use crate::render_gl::model::{compute_tangents, LoadableFromObj, Model};
use crate::resources::Resources;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use crate::render_gl::buffer::{BufferUsage, Buffer, AnyBuffer, BufferTypeArray, BufferDynamicFixedLen};

pub struct ModelData<T: VertexAttribPointers,TU:BufferUsage> {
    model: Model<T, TU>,
    indices: Vec<i32>,
}

impl<T: LoadableFromObj + VertexAttribPointers,TU:BufferUsage> ModelData<T, TU> where Buffer<BufferTypeArray, T, TU>: AnyBuffer<T> {
    pub fn from_res(
        resource_name: &str,
        res: &Resources,
        gl: &gl::Gl,
    ) -> Result<Self, failure::Error> {
        println!("Loading plain model {:?}", resource_name);
        let input = res.path(resource_name);
        Self::from_obj(input, gl)
    }

    pub fn from_obj(input: impl AsRef<Path>, gl: &gl::Gl) -> Result<Self, failure::Error> {
        let (ver, ind) = T::load(input)?;
        Ok(Self::new(ver.as_slice(), ind, gl))
    }
}
impl<T: VertexAttribPointers, TU:BufferUsage> ModelData<T,TU>  where Buffer<BufferTypeArray, T, TU>: AnyBuffer<T> {
    pub fn new(ver: &[T], indices: Vec<i32>, gl: &gl::Gl) -> Self{
        let model = Model::new(Buffer::new(ver,gl), indices.as_slice(), gl);
        Self { model, indices }
    }
}

impl <TU:BufferUsage> ModelData<VertexTexNorTan,TU> where Buffer<BufferTypeArray, VertexTexNorTan, TU>: AnyBuffer<VertexTexNorTan> {
    pub fn new_from_tex_nor(
        ver: &[VertexTexNor],
        indices: Vec<i32>,
        gl: &gl::Gl,
    ) -> Result<Self, failure::Error> {
        let vertices = compute_tangents(ver, indices.as_slice())?;
        Ok(ModelData::new(vertices.as_slice(), indices, gl))
    }
}
impl ModelData<VertexTexNorTan,BufferDynamicFixedLen>{
    pub fn update_from_tex_nor(&mut self, vbo: &[VertexTexNor]) -> Result<(), failure::Error> {
        let new_len = vbo.len();
        if self.model.len_vertices() == new_len {
            let vertices = compute_tangents(vbo, &self.indices)?;
            self.model.vbo_mut().update(vertices.as_slice())
        } else {
            Err("Incorrect size").map_err(err_msg)
        }
    }
}

impl<T: VertexAttribPointers,TU:BufferUsage> Deref for ModelData<T,TU> {
    type Target = Model<T,TU>;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}
impl<T: VertexAttribPointers,TU:BufferUsage> DerefMut for ModelData<T,TU> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model
    }
}

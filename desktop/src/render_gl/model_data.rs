use crate::render_gl::data::{VertexAttribPointers, VertexTexNor, VertexTexNorTan};

use gl;

use failure::err_msg;

use crate::render_gl::model::{compute_tangents, LoadableFromObj, Model};
use crate::resources::Resources;
use std::ops::Deref;
use std::path::Path;

pub struct ModelData<T: VertexAttribPointers> {
    model: Model<T>,
    indices: Vec<i32>,
}

impl<T: LoadableFromObj + VertexAttribPointers> ModelData<T> {
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
        Self::new(ver.as_slice(), ind, gl)
    }
}
impl<T: VertexAttribPointers> ModelData<T> {
    pub fn new(ver: &[T], indices: Vec<i32>, gl: &gl::Gl) -> Result<Self, failure::Error> {
        let model = Model::<T>::new(ver, indices.as_slice(), gl)?;
        Ok(Self { model, indices })
    }
}

impl ModelData<VertexTexNorTan> {
    pub fn new_from_tex_nor(
        ver: &[VertexTexNor],
        indices: Vec<i32>,
        gl: &gl::Gl,
    ) -> Result<Self, failure::Error> {
        let vertices = compute_tangents(ver, indices.as_slice())?;
        ModelData::new(vertices.as_slice(), indices, gl)
    }
    pub fn update_from_tex_nor(&mut self, vbo: &[VertexTexNor]) -> Result<(), failure::Error> {
        let new_len = vbo.len();
        if self.model.len_vertices() == new_len {
            let vertices = compute_tangents(vbo, &self.indices)?;
            self.model.update_vbo(vertices.as_slice())
        } else {
            Err("Incorrect size").map_err(err_msg)
        }
    }
}

impl<T: VertexAttribPointers> Deref for ModelData<T> {
    type Target = Model<T>;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

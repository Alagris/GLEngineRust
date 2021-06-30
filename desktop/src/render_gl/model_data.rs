use std::fs::File;
use std::io::BufReader;
use crate::render_gl::data::{Vertex, VertexTex, VertexTexCol, VertexTexNor, f32_f32_f32, VertexTexNorTan, VertexAttribPointers};
use crate::render_gl::buffer::{ArrayBuffer, ElementArrayBuffer, VertexArray};
use obj::{Obj, IndexTuple, SimplePolygon};
use core::ptr;
use gl;
use std::collections::HashMap;
use failure::err_msg;
use std::collections::hash_map::Entry;
use std::net::Incoming;
use glm::U3;
use glm::U1;
use std::path::Path;
use crate::resources::Resources;
use crate::render_gl::model::{Model, compute_tangents, LoadableFromObj};
use std::ops::Deref;

pub struct ModelData<T:VertexAttribPointers>{
    model:Model<T>,
    indices:Vec<i32>
}

impl <T:LoadableFromObj+VertexAttribPointers> ModelData<T>{
    pub fn from_res(resource_name: &str, res:&Resources, gl: &gl::Gl) -> Result<Self, failure::Error> {
        println!("Loading plain model {:?}", resource_name);
        let input = res.path(resource_name);
        Self::from_obj(input, gl)
    }

    pub fn from_obj(input: impl AsRef<Path>, gl: &gl::Gl) -> Result<Self, failure::Error> {
        let (ver,ind) = T::load(input)?;
        Self::new(ver.as_slice(),ind, gl)
    }
}
impl <T:VertexAttribPointers> ModelData<T> {

    pub fn new(ver: &[T], indices: Vec<i32>, gl: &gl::Gl) -> Result<Self, failure::Error> {
        let model = Model::<T>::new(ver,indices.as_slice(),gl)?;
        Ok(Self{model,indices})
    }

}

impl ModelData<VertexTexNorTan>{
    pub fn new_from_tex_nor(ver: &[VertexTexNor], indices: Vec<i32>, gl: &gl::Gl) -> Result<Self, failure::Error> {
        let vertices = compute_tangents(ver, indices.as_slice())?;
        ModelData::new(vertices.as_slice(), indices,gl)
    }
    pub fn update_from_tex_nor(&mut self, vbo: &[VertexTexNor]) -> Result<(), failure::Error> {
        let new_len = vbo.len();
        if self.model.len_vertices() == new_len{
            let vertices = compute_tangents(vbo, &self.indices)?;
            self.model.update_vbo(vertices.as_slice())
        } else {
            Err("Incorrect size").map_err(err_msg)
        }
    }
}

impl <T:VertexAttribPointers> Deref for ModelData<T>{
    type Target = Model<T>;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}
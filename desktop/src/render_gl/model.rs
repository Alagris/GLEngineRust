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

pub struct Model<T:VertexAttribPointers> {
    vbo: ArrayBuffer<T>,
    ebo: ElementArrayBuffer<i32>,
    vao: VertexArray,
    gl: gl::Gl,
}

impl <T:VertexAttribPointers> Model<T> {
    pub fn vbo(&self)-> &ArrayBuffer<T>{
        &self.vbo
    }
    pub fn gl(&self)-> &gl::Gl{
        &self.gl
    }
    pub fn ebo(&self)-> &ElementArrayBuffer<i32>{
        &self.ebo
    }
    pub fn vao(&self)-> &VertexArray{
        &self.vao
    }
    pub fn new(vertices:&[T],indices:&[i32], gl: &gl::Gl) -> Result<Self, failure::Error> {
        let vbo = ArrayBuffer::new(gl);
        let ebo = ElementArrayBuffer::new(gl);
        let vao = VertexArray::new(gl);

        vbo.bind();
        vbo.static_draw_data(vertices);
        vbo.unbind();
        ebo.bind();
        ebo.static_draw_data(indices);
        ebo.unbind();

        vao.bind();
        vbo.bind();
        ebo.bind();
        T::vertex_attrib_pointers(gl);
        ebo.unbind();
        vbo.unbind();
        vao.unbind();
        drain_gl_errors(gl);
        let me = Self { vbo, ebo, vao, gl: gl.clone() };
        assert_eq!(me.len_indices(),indices.len());
        assert_eq!(me.len_vertices(),vertices.len());
        Ok(me)
    }

    pub fn bind(&self) {
        self.vao.bind();
        self.ebo.bind();
    }
    pub fn unbind(&self) {
        self.vao.unbind();
        self.ebo.unbind();
    }
    pub fn len_vertices(&self)->usize{
        self.vbo.len()
    }
    pub fn len_indices(&self)->usize{
        self.ebo.len()
    }
    fn draw(&self, primitive: gl::types::GLenum) {
        let indices = self.len_indices() as i32;
        unsafe {
            self.bind();
            self.gl.DrawElements(primitive, indices, gl::UNSIGNED_INT, ptr::null());
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


pub trait LoadableFromObj:Sized{
    fn load(input: impl AsRef<Path>)->Result<(Vec<Self>, Vec<i32>), failure::Error>;
}
impl LoadableFromObj for VertexTex{
    fn load(input: impl AsRef<Path>) -> Result<(Vec<Self>, Vec<i32>), Error> {
        load(input, |vertex|{
            let vertex_index = vertex.0;
            let texture_index = vertex.1.ok_or("texture UV index is mandatory! Fix you obj file").map_err(err_msg)?;
            Ok((vertex_index, texture_index))
        }, |&(ver,tex), obj|{
            let ver_value = obj.data.position.get(ver).ok_or("index pointing to nonexistent vertex").map_err(err_msg)?;
            let tex_value = obj.data.texture.get(tex).ok_or("index pointing to nonexistent texture UV").map_err(err_msg)?;
            Ok(VertexTex::new(ver_value, tex_value))
        })
    }
}
impl LoadableFromObj for VertexTexNor{
    fn load(input: impl AsRef<Path>) -> Result<(Vec<Self>, Vec<i32>), Error> {
        load(input, |vertex|{
            let vertex_index = vertex.0;
            let texture_index = vertex.1.ok_or("texture UV index is mandatory! Fix you obj file").map_err(err_msg)?;
            let normal_index = vertex.2.ok_or("normal index is mandatory! Fix you obj file").map_err(err_msg)?;
            Ok((vertex_index, texture_index, normal_index))
        }, |&(ver,tex,nor), obj|{

            let ver_value = obj.data.position.get(ver).ok_or("index pointing to nonexistent vertex").map_err(err_msg)?;
            let nor_value = obj.data.normal.get(nor).ok_or("index pointing to nonexistent normal").map_err(err_msg)?;
            let tex_value = obj.data.texture.get(tex).ok_or("index pointing to nonexistent texture UV").map_err(err_msg)?;
            Ok(VertexTexNor::new(ver_value, nor_value, tex_value))
        })
    }
}
impl LoadableFromObj for VertexTexNorTan{
    fn load(input: impl AsRef<Path>) -> Result<(Vec<Self>, Vec<i32>), Error> {
        VertexTexNor::load(input).and_then(|(ver_tex_nor,indices)|compute_tangents(&ver_tex_nor,&indices).map(|ver_tex_nor_tan|(ver_tex_nor_tan,indices)))
    }
}

impl <T:LoadableFromObj+VertexAttribPointers> Model<T>{
    pub fn from_res(resource_name: &str, res:&Resources, gl: &gl::Gl) -> Result<Self, failure::Error> {
        println!("Loading {} model {:?}",type_name::<T>(), resource_name);
        let input = res.path(resource_name);
        Self::from_obj(input, gl)
    }

    pub fn from_obj(input: impl AsRef<Path>, gl: &gl::Gl) -> Result<Self, failure::Error> {
        let (ver,ind) = T::load(input)?;
        Self::new(ver.as_slice(),ind.as_slice(), gl)
    }
}
impl Model<VertexTexNorTan>{
    pub fn new_from_tex_nor(ver: &[VertexTexNor], indices: Vec<i32>, gl: &gl::Gl) -> Result<Self, failure::Error> {
        let vertices = compute_tangents(ver, indices.as_slice())?;
        Self::new(vertices.as_slice(), indices.as_slice(),gl)
    }
}
pub fn load<T:Debug, K:Eq+Hash>(input: impl AsRef<Path>, f:fn(&IndexTuple)->Result<K,failure::Error>, k:fn(&K, &Obj)->Result<T,failure::Error>) -> Result<(Vec<T>, Vec<i32>), failure::Error>{
    let obj = Obj::load(input)?;
    let mut vertices: Vec<T> = vec![];
    let mut indices: Vec<i32> = vec![];
    type Cache<K> = HashMap<K, usize>;
    let mut cache: Cache<K> = HashMap::new();

    let triangles = &obj.data.objects.first().ok_or("No objects in obj file").map_err(err_msg)?
        .groups.first().ok_or("No groups in obj file").map_err(err_msg)?
        .polys;
    for triangle in triangles {
        if triangle.0.len() != 3 { return Err("Obj file contains non-triangle polygon").map_err(err_msg); }
        for vertex in &triangle.0 {
            let key = f(vertex)?;
            let idx = match cache.entry(key) {
                Entry::Occupied(e) => e.get().clone(),
                Entry::Vacant(e) => {
                    let new_index = vertices.len();
                    let ver_tex = k(e.key(), &obj)?;
                    vertices.push(ver_tex);
                    e.insert(new_index);
                    new_index
                }
            };
            indices.push(idx as i32);
        }
    }
    Ok((vertices, indices))
}


pub fn compute_tangents(vertices:&[VertexTexNor], indices:&[i32]) -> Result<Vec<VertexTexNorTan>, failure::Error> {
    let mut tangents = vec![glm::vec3(0f32, 0f32, 0f32); vertices.len()];
    let mut bitangents = vec![glm::vec3(0f32, 0f32, 0f32); vertices.len()];
    let mut counts = vec![0; vertices.len()];
    if indices.len() % 3 != 0 {
        return Err("number of indices is not multiple of 3!").map_err(err_msg);
    }
    for index in indices.windows(3) {
        let i0 = index[0];
        let i1 = index[1];
        let i2 = index[2];
        let v0 = vertices[i0 as usize];
        let v1 = vertices[i1 as usize];
        let v2 = vertices[i2 as usize];
        let p0 = v0.pos();
        let p1 = v1.pos();
        let p2 = v2.pos();
        let uv0 = v0.tex();
        let uv1 = v1.tex();
        let uv2 = v2.tex();
        let p0 = glm::vec3(p0.d0, p0.d1, p0.d2);
        let p1 = glm::vec3(p1.d0, p1.d1, p1.d2);
        let p2 = glm::vec3(p2.d0, p2.d1, p2.d2);
        let uv0 = glm::vec2(uv0.d0, uv0.d1);
        let uv1 = glm::vec2(uv1.d0, uv1.d1);
        let uv2 = glm::vec2(uv2.d0, uv2.d1);
        let delta_pos1 = p1 - p0;
        let delta_pos2 = p2 - p0;
        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let r = 1f32 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
        let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

        for &i in index {
            let i = i as usize;
            let pre_size = counts[i];
            let post_size = pre_size + 1;
            tangents[i] = (tangents[i] * pre_size as f32 + tangent) / post_size as f32;
            bitangents[i] = (bitangents[i] * pre_size as f32 + bitangent) / post_size as f32;
            counts[i] = post_size;
        }
    }
    fn orthogonalize(normal: &f32_f32_f32, tangent: &glm::TMat<f32, 3, 1>, bitangent: &glm::TMat<f32, 3, 1>) -> f32_f32_f32 {
        let n = glm::vec3(normal.x(), normal.y(), normal.z());
        let t = tangent;
        let b = bitangent;

        // Gram-Schmidt orthogonalize
        let t = glm::normalize(&(t - n * glm::dot(&n, &t)));

        // Calculate handedness
        let c = glm::cross::<f32>(&n, &t);
        let t = if glm::dot(&c, &b) < 0.0f32 {
            t * -1.0f32
        } else {
            t
        };
        f32_f32_f32::new(t.x, t.y, t.z)
    }

    let converted_vertices = vertices.iter().zip(tangents.iter()).zip(bitangents.iter()).map(|((v, &t), &b)|
        VertexTexNorTan::convert(*v, orthogonalize(v.nor(), &t, &b), f32_f32_f32::new(b.x, b.y, b.z))
    ).collect();
    Ok(converted_vertices)
}
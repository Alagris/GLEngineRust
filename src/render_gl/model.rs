use std::fs::File;
use std::io::BufReader;
use crate::render_gl::data::{Vertex, VertexTex, VertexTexCol, VertexTexNor, f32_f32_f32, VertexTexNorTan};
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

pub struct Model {
    vertices: Vec<VertexTexNorTan>,
    indices: Vec<i32>,
    vbo: ArrayBuffer,
    ebo: ElementArrayBuffer,
    vao: VertexArray,
    gl: gl::Gl,
}

impl Model {
    pub fn from_file(path: &str, gl: &gl::Gl) -> Result<Model, failure::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::new(&mut reader, gl)
    }


    pub fn new<T: std::io::Read>(input: &mut BufReader<T>, gl: &gl::Gl) -> Result<Model, failure::Error> {
        let ver_ind = load_ver_nor_tex(input)?;
        let (vertices, indices) = compute_tangents(ver_ind)?;
        let vbo = ArrayBuffer::new(&gl);
        let ebo = ElementArrayBuffer::new(&gl);
        let vao = VertexArray::new(&gl);

        vbo.bind();
        vbo.static_draw_data(&vertices);
        vbo.unbind();
        ebo.bind();
        ebo.static_draw_data(&indices);
        ebo.unbind();

        vao.bind();
        vbo.bind();
        ebo.bind();
        VertexTexNorTan::vertex_attrib_pointers(&gl);
        ebo.unbind();
        vbo.unbind();
        vao.unbind();

        Ok(Model { vertices, indices, vbo, ebo, vao, gl: gl.clone() })
    }

    pub fn bind(&self) {
        self.vao.bind();
        self.ebo.bind();
    }

    fn draw(&self, primitive: gl::types::GLenum) {
        unsafe {
            self.gl.DrawElements(primitive, self.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());
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
}

fn load_ver_nor_tex<T: std::io::Read>(input: &mut BufReader<T>) -> Result<(Vec<VertexTexNor>, Vec<i32>), failure::Error> {
    let obj = Obj::<SimplePolygon>::load_buf(input)?;

    let mut vertices: Vec<VertexTexNor> = vec![];
    let mut indices: Vec<i32> = vec![];
    type Cache = HashMap<usize, HashMap<usize, HashMap<usize, usize>>>;
    let mut cache: Cache = HashMap::new();

    fn insert_to_cache(cache: &mut Cache, vertices: &mut Vec<VertexTexNor>, obj: &Obj<Vec<IndexTuple>>, vertex: usize, normal: usize, tex: usize) -> Result<usize, failure::Error> {
        let vertex_entry = cache.entry(vertex).or_insert(HashMap::new());
        let normal_entry = vertex_entry.entry(normal).or_insert(HashMap::new());
        let tex_entry = normal_entry.entry(tex);
        match tex_entry {
            Entry::Occupied(e) => Ok(*e.get()),
            Entry::Vacant(e) => {
                let vertex_value = obj.position.get(vertex).ok_or("index pointing to nonexistent vertex").map_err(err_msg)?;
                let normal_value = obj.normal.get(normal).ok_or("index pointing to nonexistent normal").map_err(err_msg)?;
                let tex_value = obj.texture.get(tex).ok_or("index pointing to nonexistent texture UV").map_err(err_msg)?;
                let new_index = vertices.len();
                let vertex_tuple = (vertex_value[0], vertex_value[1], vertex_value[2]);
                let normal_tuple = (normal_value[0], normal_value[1], normal_value[2]);
                let tex_tuple = (tex_value[0], tex_value[1]);
                let vertex_nor_tex = VertexTexNor::new_t(vertex_tuple, normal_tuple, tex_tuple);
                vertices.push(vertex_nor_tex);
                e.insert(new_index);
                Ok(new_index)
            }
        }
    }


    let triangles = &obj.objects.first().ok_or("No objects in obj file").map_err(err_msg)?
        .groups.first().ok_or("No groups in obj file").map_err(err_msg)?
        .polys;
    for triangle in triangles {
        if triangle.len() != 3 { return Err("Obj file contains non-triangle polygon").map_err(err_msg); }
        for vertex in triangle {
            let vertex_index = vertex.0;
            let texture_index = vertex.1.ok_or("texture UV index is mandatory! Fix you obj file").map_err(err_msg)?;
            let normal_index = vertex.2.ok_or("normal index is mandatory! Fix you obj file").map_err(err_msg)?;
            let index = insert_to_cache(&mut cache, &mut vertices, &obj, vertex_index, normal_index, texture_index)?;
            indices.push(index as i32);
        }
    }
    Ok((vertices, indices))
}

fn compute_tangents((vertices, indices): (Vec<VertexTexNor>, Vec<i32>)) -> Result<(Vec<VertexTexNorTan>, Vec<i32>), failure::Error> {
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
    fn orthogonalize(normal:&f32_f32_f32,tangent:&glm::TMat<f32,U3,U1>,bitangent:&glm::TMat<f32,U3,U1>) -> f32_f32_f32{
        let n = glm::vec3(normal.x(), normal.y(), normal.z());
        let t = tangent;
        let b = bitangent;

        // Gram-Schmidt orthogonalize
        let t = glm::normalize(&(t - n * glm::dot(&n, &t)));

        // Calculate handedness
        let c = glm::cross::<f32,U1>(&n, &t);
        let t = if glm::dot(&c, &b) < 0.0f32 {
            t * -1.0f32
        }else{
            t
        };
        f32_f32_f32::new(t.x, t.y, t.z)
    }

    let converted_vertices = vertices.iter().zip(tangents.iter()).zip(bitangents.iter()).map(|((v, &t), &b)|
        VertexTexNorTan::convert(*v, orthogonalize(v.nor(),&t,&b), f32_f32_f32::new(b.x, b.y, b.z))
    ).collect();
    Ok((converted_vertices, indices))
}
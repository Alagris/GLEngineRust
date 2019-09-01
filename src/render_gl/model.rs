use std::fs::File;
use std::io::BufReader;
use crate::render_gl::data::{Vertex, VertexTex, VertexTexCol, VertexTexNor};
use crate::render_gl::buffer::{ArrayBuffer, ElementArrayBuffer, VertexArray};
use obj::{Obj, IndexTuple, SimplePolygon};
use core::ptr;
use gl;
use std::collections::HashMap;
use failure::err_msg;
use std::collections::hash_map::Entry;
use std::net::Incoming;

pub struct Model {
    vertices: Vec<VertexTexNor>,
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
        VertexTexNor::vertex_attrib_pointers(&gl);
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
use crate::render_gl::data::VertexTex;

pub fn quad(width: f32, height: f32) -> [VertexTex; 6] {
    let quad_vertices = [
        // positions     // colors
        VertexTex::new((-width, height, 0.), (0.0, 1.0)),
        VertexTex::new((width, -height, 0.), (1.0, 0.0)),
        VertexTex::new((-width, -height, 0.), (0.0, 0.0)),
        VertexTex::new((-width, height, 0.), (0.0, 1.0)),
        VertexTex::new((width, -height, 0.), (1.0, 0.0)),
        VertexTex::new((width, height, 0.), (1.0, 1.0)),
    ];
    quad_vertices
}

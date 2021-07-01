use std::collections::VecDeque;

use crate::render_gl::data::VertexTexNor;

use rand::Rng;

struct Vertex {
    x: f32,
    y: f32,
    h: f32,
    is_border: bool,
    //is on border
    drainage: f32,
}

impl Vertex {
    fn new(xy: (f32, f32), is_border: bool, drainage: f32) -> Vertex {
        let (x, y) = xy;
        Vertex {
            x,
            y,
            h: 0f32,
            is_border,
            drainage,
        }
    }
    fn dist(&self, other: &Self) -> f32 {
        let v0 = self;
        let v1 = other;
        let x_diff = v0.x - v1.x;
        let y_diff = v0.y - v1.y;
        (x_diff * x_diff + y_diff * y_diff).sqrt()
    }
    fn h_diff(&self, other: &Self) -> f32 {
        self.h - other.h
    }
}

fn grid_index_to_vertex(i: usize, width: usize) -> (usize, usize) {
    (i % width, i / width)
}

fn grid_vertex_to_index(v: (usize, usize), width: usize) -> usize {
    let (x, y) = v;
    x + y * width
}

struct Mat<T> {
    mat: Vec<T>,
    w: usize,
}

impl<T> Mat<T> {
    fn index_to_vertex(&self, i: usize) -> (usize, usize) {
        grid_index_to_vertex(i, self.w)
    }

    fn vertex_to_index(&self, v: (usize, usize)) -> usize {
        grid_vertex_to_index(v, self.w)
    }

    fn get(&self, x: usize, y: usize) -> &T {
        &self.mat[self.vertex_to_index((x, y))]
    }
    fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        let i = self.vertex_to_index((x, y));
        &mut self.mat[i]
    }
}

impl<T: Clone> Mat<T> {
    fn new(w: usize, h: usize, default: T) -> Mat<T> {
        Mat {
            mat: vec![default; w * h],
            w,
        }
    }
}

impl<T: Clone> std::ops::Index<(usize, usize)> for Mat<T> {
    type Output = T;

    fn index(&self, v: (usize, usize)) -> &T {
        // or as appropriate for row- or column-major data
        &self.mat[self.vertex_to_index(v)]
    }
}

impl<T: Clone> std::ops::IndexMut<(usize, usize)> for Mat<T> {
    fn index_mut(&mut self, v: (usize, usize)) -> &mut Self::Output {
        let i = self.vertex_to_index(v);
        &mut self.mat[i]
    }
}

pub struct Graph {
    w: f32,
    h: f32,
    vertices: Vec<Vertex>,
    egdes: Mat<bool>,
}

fn triangular_grid_vertex_to_2d_point(x: usize, y: usize, triangle_size: f32) -> (f32, f32) {
    let half_side = triangle_size / 2f32;
    let triangle_height = (triangle_size * triangle_size - half_side * half_side).sqrt();
    if y % 2 == 0 {
        (x as f32 * triangle_size, triangle_height * y as f32)
    } else {
        (
            x as f32 * triangle_size + half_side,
            triangle_height * y as f32,
        )
    }
}

fn reg_triangle_area(a: f32) -> f32 {
    3f32.sqrt() * a * a / 2f32
}

fn size_of_plane(w: usize, h: usize, triangle_size: f32) -> (f32, f32) {
    let half = triangle_size / 2f32;
    let triangle_height = (triangle_size * triangle_size - half * half).sqrt();
    (w as f32 * triangle_size + half, h as f32 * triangle_height)
}

impl Graph {
    fn pass_height_of_edge(&self, edge: usize) -> f32 {
        let (v0, v1) = self.edge_index_to_vertex_indices(edge);
        self.pass_height_between_vertices(v0, v1)
    }
    fn pass_height_between_vertices(&self, vertex0: usize, vertex1: usize) -> f32 {
        self.vertices[vertex0].h.max(self.vertices[vertex1].h)
    }
    fn vertex_indices_to_edge_index(&self, from: usize, to: usize) -> usize {
        let size = self.vertices.len();
        grid_vertex_to_index((from, to), size)
    }
    fn edge_index_to_vertex_indices(&self, edge_index: usize) -> (usize, usize) {
        let size = self.vertices.len();
        grid_index_to_vertex(edge_index, size)
    }

    pub fn to_ver_nor_tex(&self) -> Vec<VertexTexNor> {
        let v = self.vertices.len();
        let mut vertices: Vec<VertexTexNor> = Vec::with_capacity(v);
        for (_vertex_index, vertex) in self.vertices.iter().enumerate() {
            let x = vertex.x;
            let y = vertex.y;
            let h_ratio = y / self.h;
            let w_ratio = x / self.w;
            vertices.push(VertexTexNor::new(
                (x, vertex.h, y),
                (0., 1., 0.),
                (w_ratio, h_ratio),
            ));
        }
        vertices
    }
    pub fn to_indices(&self) -> Vec<i32> {
        let v = self.vertices.len();
        let mut indices: Vec<i32> = vec![];
        for x in 0..v {
            for y in (x + 1)..v {
                if self.egdes[(x, y)] {
                    let mut zs = (v, v);
                    for z in (y + 1)..v {
                        if self.egdes[(y, z)] && self.egdes[(z, x)] {
                            if zs.0 == v {
                                zs.0 = z;
                            } else if zs.1 == v {
                                zs.1 = z;
                            } else {
                                break;
                            }
                        }
                    }
                    if zs.0 < v {
                        indices.push(x as i32);
                        indices.push(y as i32);
                        indices.push(zs.0 as i32);
                        if zs.1 < v {
                            indices.push(x as i32);
                            indices.push(y as i32);
                            indices.push(zs.1 as i32);
                        }
                    }
                }
            }
        }
        indices
    }

    pub fn regular(w: usize, h: usize, triangle_size: f32) -> Graph {
        let nodes = w * h;
        let (total_w, total_h) = size_of_plane(w, h, triangle_size);
        let mut g = Graph {
            w: total_w,
            h: total_h,
            vertices: vec![],
            egdes: Mat::new(nodes, nodes, false),
        };
        let mut rng = rand::thread_rng();
        for y in 0..h {
            for x in 0..w {
                let (px, py) = triangular_grid_vertex_to_2d_point(x, y, triangle_size);
                g.vertices.push(Vertex::new(
                    (
                        px + rng.gen_range(-triangle_size / 2.0..triangle_size / 2.),
                        py + rng.gen_range(-triangle_size / 2.0..triangle_size / 2.),
                    ),
                    x == 0 || y == 0 || x == w - 1 || y == h - 1,
                    reg_triangle_area(triangle_size),
                ));
            }
        }
        fn add_edge(
            mut g: Graph,
            width: usize,
            height: usize,
            from: (usize, usize),
            to: (usize, usize),
        ) -> Graph {
            assert!(from.0 < width, "{} < {}", from.0, width);
            assert!(from.1 < height, "{} < {}", from.1, height);
            assert!(to.0 < width, "{} < {}", to.0, width);
            assert!(to.1 < height, "{} < {}", to.1, height);
            let from = grid_vertex_to_index(from, width);
            let to = grid_vertex_to_index(to, width);
            g.egdes[(from, to)] = true;
            g.egdes[(to, from)] = true;
            //            stdin().read_line(&mut String::new());
            //            g.draw(width, height, 1f32);
            g
        }
        for y in 0..h {
            for x in 0..(w - 1) {
                g = add_edge(g, w, h, (x, y), (x + 1, y));
            }
        }
        for y in 0..(h - 1) {
            for x in 0..w {
                g = add_edge(g, w, h, (x, y), (x, y + 1));
            }
        }
        for y in (0..(h - 1)).step_by(2) {
            for x in 1..w {
                g = add_edge(g, w, h, (x, y), (x - 1, y + 1));
            }
        }
        for y in (1..(h - 1)).step_by(2) {
            for x in 0..(w - 1) {
                g = add_edge(g, w, h, (x, y), (x + 1, y + 1));
            }
        }

        g
    }

    fn is_edge_for_index(&self, from: usize, to: usize) -> bool {
        self.egdes[(from, to)]
    }
}

struct Tree {
    vertex_to_downstream_vertex: Vec<usize>, //one edge per vertex
}

impl Tree {
    fn from(g: &Graph) -> Tree {
        let vertices_count = g.vertices.len();
        let mut tree = Tree {
            vertex_to_downstream_vertex: Vec::with_capacity(vertices_count),
        };
        for vertex in 0..vertices_count {
            let mut min_h = g.vertices[vertex].h;
            let mut min_neighbour = vertex;
            for neighbour in 0..vertices_count {
                if g.is_edge_for_index(vertex, neighbour) {
                    let h = g.vertices[neighbour].h;
                    if h < min_h {
                        min_h = h;
                        min_neighbour = neighbour;
                    }
                }
            }
            tree.vertex_to_downstream_vertex.push(min_neighbour);
        }
        tree
    }
    fn slope(&self, vertex: usize, g: &Graph) -> f32 {
        let downstream = self.vertex_to_downstream_vertex[vertex];
        if downstream == vertex {
            0f32
        } else {
            let v0 = &g.vertices[vertex];
            let v1 = &g.vertices[downstream];
            let dist = v0.dist(v1);
            assert_ne!(dist, 0.0);
            v0.h_diff(v1) / dist
        }
    }
}

struct Lakes {
    vertex_to_lake_central_vertex: Vec<usize>,
    lake_index_to_lake_central_vertex: Vec<usize>,
}

impl Lakes {
    fn new(tree: &Tree) -> Lakes {
        let vertices_count = tree.vertex_to_downstream_vertex.len();
        let mut vertex_to_lake_central_vertex: Vec<usize> = (0..vertices_count).collect();
        loop {
            let mut has_change = false;
            for (vertex, &downstream) in tree.vertex_to_downstream_vertex.iter().enumerate() {
                if vertex_to_lake_central_vertex[vertex]
                    != vertex_to_lake_central_vertex[downstream]
                {
                    vertex_to_lake_central_vertex[vertex] =
                        vertex_to_lake_central_vertex[downstream];
                    has_change = true;
                }
            }
            if !has_change {
                break;
            }
        }
        let mut lake_index_to_lake_central_vertex: Vec<usize> =
            vertex_to_lake_central_vertex.clone();
        lake_index_to_lake_central_vertex.sort();
        lake_index_to_lake_central_vertex.dedup();
        Lakes {
            vertex_to_lake_central_vertex,
            lake_index_to_lake_central_vertex,
        }
    }
    fn count(&self) -> usize {
        let mut arr = vec![false; self.vertex_to_lake_central_vertex.len()];
        for &lake in &self.vertex_to_lake_central_vertex {
            arr[lake] = true;
        }
        arr.iter().filter(|&&b| b).count()
    }
    fn vertex_to_lake_central_vertex(&self, vertex: usize) -> usize {
        self.vertex_to_lake_central_vertex[vertex]
    }
    fn lake_central_vertex_to_lake_index(&self, vertex: usize) -> usize {
        self.lake_index_to_lake_central_vertex
            .iter()
            .position(|&e| e == vertex)
            .unwrap()
    }
}

struct LakeTree {
    lake_index_to_lake_index: Vec<usize>,
    lake_index_and_lake_index_to_pass_edge: Mat<usize>,
}

impl LakeTree {
    fn lake_passes(lakes: &Lakes, g: &Graph) -> Mat<usize> {
        let lakes_count = lakes.lake_index_to_lake_central_vertex.len();
        let edge_count = g.egdes.mat.len();
        let mut lake_index_and_lake_index_to_pass_edge =
            Mat::new(lakes_count, lakes_count, edge_count);
        for (edge, _) in g.egdes.mat.iter().enumerate().filter(|(_u, &b)| b) {
            let (from, to) = g.edge_index_to_vertex_indices(edge);
            let from_lake = lakes.vertex_to_lake_central_vertex[from];
            let to_lake = lakes.vertex_to_lake_central_vertex[to];
            if from_lake != to_lake {
                let max_h = g.pass_height_between_vertices(from, to);
                let from_lake_index = lakes.lake_central_vertex_to_lake_index(from_lake);
                let to_lake_index = lakes.lake_central_vertex_to_lake_index(to_lake);
                let pass_edge =
                    lake_index_and_lake_index_to_pass_edge[(from_lake_index, to_lake_index)];
                let new_pass_edge = if pass_edge == edge_count {
                    edge
                } else {
                    if max_h < g.pass_height_of_edge(pass_edge) {
                        edge
                    } else {
                        pass_edge
                    }
                };
                lake_index_and_lake_index_to_pass_edge[(from_lake_index, to_lake_index)] =
                    new_pass_edge;
                lake_index_and_lake_index_to_pass_edge[(to_lake_index, from_lake_index)] =
                    new_pass_edge;
            }
        }
        for lake_index in 0..lakes_count {
            let mut exists_at_least_one_pass = false;
            for other_lake_index in 0..lakes_count {
                assert_eq!(
                    lake_index_and_lake_index_to_pass_edge[(lake_index, other_lake_index)],
                    lake_index_and_lake_index_to_pass_edge[(other_lake_index, lake_index)]
                );
                let pass_edge =
                    lake_index_and_lake_index_to_pass_edge[(lake_index, other_lake_index)];
                if pass_edge < edge_count {
                    let (pass_vertex0, pass_vertex1) = g.edge_index_to_vertex_indices(pass_edge);
                    let lake_central_vertex = lakes.lake_index_to_lake_central_vertex[lake_index];
                    let other_lake_central_vertex =
                        lakes.lake_index_to_lake_central_vertex[other_lake_index];
                    if lakes.vertex_to_lake_central_vertex[pass_vertex0] == lake_central_vertex {
                        assert_eq!(
                            lakes.vertex_to_lake_central_vertex[pass_vertex1],
                            other_lake_central_vertex
                        );
                    } else {
                        assert_eq!(
                            lakes.vertex_to_lake_central_vertex[pass_vertex1],
                            lake_central_vertex
                        );
                        assert_eq!(
                            lakes.vertex_to_lake_central_vertex[pass_vertex0],
                            other_lake_central_vertex
                        );
                    }
                }
                if lake_index_and_lake_index_to_pass_edge[(lake_index, other_lake_index)]
                    < edge_count
                {
                    exists_at_least_one_pass = true;
                }
            }
            assert!(exists_at_least_one_pass, "not for {}", lake_index);
        }
        for (vertex_index, _border_vertex) in
            g.vertices.iter().enumerate().filter(|(_i, v)| v.is_border)
        {
            let lake = lakes.vertex_to_lake_central_vertex(vertex_index);
            assert_eq!(lake, vertex_index);
        }
        lake_index_and_lake_index_to_pass_edge
    }
    fn new(
        lakes: &Lakes,
        g: &Graph,
        lake_index_and_lake_index_to_pass_edge: Mat<usize>,
    ) -> LakeTree {
        let lakes_count = lakes.lake_index_to_lake_central_vertex.len();
        let edge_count = g.egdes.mat.len();
        let mut lake_index_to_lake_index = vec![lakes_count; lakes_count];

        let mut lake_queue0 = VecDeque::new();
        let mut lake_queue1 = VecDeque::new();
        for (lake_index, &_lake_central_vertex) in lakes
            .lake_index_to_lake_central_vertex
            .iter()
            .enumerate()
            .filter(|(_lake_index, &lake_central_vertex)| g.vertices[lake_central_vertex].is_border)
        {
            lake_queue0.push_back(lake_index);
            lake_index_to_lake_index[lake_index] = lake_index;
        }
        let mut use_lake_queue0 = true;

        loop {
            let (queue0, queue1) = if use_lake_queue0 {
                (&lake_queue0, &mut lake_queue1)
            } else {
                (&lake_queue1, &mut lake_queue0)
            };
            if queue0.is_empty() {
                break;
            }
            for inflow_lake_index in 0..lakes_count {
                if lake_index_to_lake_index[inflow_lake_index] == lakes_count {
                    let mut min_pass_h = std::f32::MAX;
                    let mut min_lake_index = lakes_count;

                    for &outflow_lake_index in queue0 {
                        let pass_edge = lake_index_and_lake_index_to_pass_edge
                            [(outflow_lake_index, inflow_lake_index)];

                        if pass_edge < edge_count {
                            let pass_h = g.pass_height_of_edge(pass_edge);

                            if pass_h < min_pass_h {
                                min_lake_index = outflow_lake_index;
                                min_pass_h = pass_h;
                            }
                        }
                    }
                    if min_lake_index < lakes_count {
                        lake_index_to_lake_index[inflow_lake_index] = min_lake_index;
                        if !queue1.contains(&inflow_lake_index) {
                            queue1.push_back(inflow_lake_index);
                        }
                    }
                }
            }

            if use_lake_queue0 {
                lake_queue0.clear();
            } else {
                lake_queue1.clear();
            }
            use_lake_queue0 = !use_lake_queue0;
        }

        for (lake_index, &outflow_lake_index) in lake_index_to_lake_index.iter().enumerate() {
            assert_ne!(
                outflow_lake_index, lakes_count,
                "no outflow for {}",
                lake_index
            );
        }
        LakeTree {
            lake_index_to_lake_index,
            lake_index_and_lake_index_to_pass_edge,
        }
    }
}

impl Tree {
    fn add_lake_passes(
        lake_tree: &LakeTree,
        lakes: &Lakes,
        g: &Graph,
        mut stream_tree: Tree,
    ) -> Tree {
        for (from_vertex, &to_vertex) in stream_tree.vertex_to_downstream_vertex.iter().enumerate()
        {
            if from_vertex == to_vertex {
                let from_lake = lakes.vertex_to_lake_central_vertex[from_vertex];
                assert_eq!(from_lake, from_vertex);
            }
        }

        for (from_lake_index, &to_lake_index) in
            lake_tree.lake_index_to_lake_index.iter().enumerate()
        {
            assert_ne!(to_lake_index, lakes.lake_index_to_lake_central_vertex.len());
            assert_eq!(
                to_lake_index == from_lake_index,
                g.vertices[lakes.lake_index_to_lake_central_vertex[from_lake_index]].is_border
            );
            if to_lake_index != from_lake_index {
                let to_lake_central_vertex = lakes.lake_index_to_lake_central_vertex[to_lake_index];
                let from_lake_central_vertex =
                    lakes.lake_index_to_lake_central_vertex[from_lake_index];
                let pass_edge_index = lake_tree.lake_index_and_lake_index_to_pass_edge
                    [(from_lake_index, to_lake_index)];
                assert!(
                    pass_edge_index < g.egdes.mat.len(),
                    "{} < {}",
                    pass_edge_index,
                    g.egdes.mat.len()
                );
                let (pass_vertex0, pass_vertex1) = g.edge_index_to_vertex_indices(pass_edge_index);
                stream_tree.vertex_to_downstream_vertex[from_lake_index] = if lakes
                    .vertex_to_lake_central_vertex[pass_vertex1]
                    == from_lake_central_vertex
                {
                    assert_eq!(
                        lakes.vertex_to_lake_central_vertex[pass_vertex0], to_lake_central_vertex,
                        "from_lake_index={}, to_lake_index={}",
                        from_lake_index, to_lake_index
                    );
                    pass_vertex0
                } else {
                    assert_eq!(
                        lakes.vertex_to_lake_central_vertex[pass_vertex1], to_lake_central_vertex,
                        "from_lake_index={}, to_lake_index={}",
                        from_lake_index, to_lake_index
                    );
                    assert_eq!(
                        lakes.vertex_to_lake_central_vertex[pass_vertex0], from_lake_central_vertex,
                        "from_lake_index={}, to_lake_index={}",
                        from_lake_index, to_lake_index
                    );
                    pass_vertex1
                }
            }
        }

        stream_tree
    }

    fn stream_power_eq<U>(&self, vertex: usize, g: &Graph, u: &U, k: f32) -> f32
    where
        U: Fn(f32, f32) -> f32,
    {
        let v = &g.vertices[vertex];
        stream_power_eq(u(v.x, v.y), k, v.drainage, self.slope(vertex, g))
    }
}

struct Drainage {
    vertex_to_drainage: Vec<f32>,
}

fn stream_power_eq(u: f32, k: f32, A: f32, s: f32) -> f32 {
    u - k * A.sqrt() * s
}

impl Drainage {
    fn from<U>(stream_tree: &Tree, g: &Graph, _u: &U, _k: f32, _dt: f32) -> Drainage
    where
        U: Fn(f32, f32) -> f32,
    {
        let mut vertex_to_drainage = vec![-1f32; g.vertices.len()];
        for (_from_vertex, &to_vertex) in stream_tree.vertex_to_downstream_vertex.iter().enumerate()
        {
            vertex_to_drainage[to_vertex] = 0f32;
        }
        let mut vertex_queue = (VecDeque::new(), VecDeque::new());
        for (i, v) in g.vertices.iter().enumerate() {
            if vertex_to_drainage[i] == -1f32 {
                vertex_queue.0.push_back(i);
            }
            vertex_to_drainage[i] = v.drainage;
        }

        let mut use_queue0 = true;
        loop {
            let (queue0, queue1) = if use_queue0 {
                (&vertex_queue.0, &mut vertex_queue.1)
            } else {
                (&vertex_queue.1, &mut vertex_queue.0)
            };
            if queue0.is_empty() {
                break;
            }
            for &vertex in queue0 {
                let accumulated_drainage = vertex_to_drainage[vertex];
                let downstream = stream_tree.vertex_to_downstream_vertex[vertex];
                if downstream != vertex {
                    //otherwise it's a river mouth
                    queue1.push_back(downstream);
                    vertex_to_drainage[downstream] += accumulated_drainage;
                }
            }
            if use_queue0 {
                vertex_queue.0.clear();
            } else {
                vertex_queue.1.clear();
            }
            use_queue0 = !use_queue0;
        }
        Drainage { vertex_to_drainage }
    }
}

pub fn iterate<U: Fn(f32, f32) -> f32>(
    mut g: Graph,
    _w: usize,
    _h: usize,
    u: &U,
    k: f32,
    dt: f32,
) -> Graph {
    let tree = Tree::from(&g);
    let lakes = Lakes::new(&tree);
    let lake_index_and_lake_index_to_pass_edge = LakeTree::lake_passes(&lakes, &g);
    let lake_tree = LakeTree::new(&lakes, &g, lake_index_and_lake_index_to_pass_edge);

    let tree = Tree::add_lake_passes(&lake_tree, &lakes, &g, tree);
    let drainage = Drainage::from(&tree, &g, u, k, dt);
    let mut visited_vertices = vec![false; g.vertices.len()];
    for (i, v) in g
        .vertices
        .iter_mut()
        .enumerate()
        .filter(|(_i, v)| v.is_border)
    {
        v.h = 0f32;
        visited_vertices[i] = true;
    }
    loop {
        let mut found_any = false;
        for (upstream, &downstream) in tree.vertex_to_downstream_vertex.iter().enumerate() {
            if visited_vertices[downstream] && !visited_vertices[upstream] {
                let i = upstream;
                let j = downstream;
                let hi_t_dt = {
                    let xj = &g.vertices[j];
                    let xi = &g.vertices[i];
                    let hj_t_dt = xj.h;
                    let hi_t = xi.h;
                    assert!(!hj_t_dt.is_nan());
                    assert!(hj_t_dt.is_finite());
                    assert!(!hi_t.is_nan());
                    assert!(hi_t.is_finite());
                    let ai = drainage.vertex_to_drainage[i];
                    let dist = xi.dist(xj);
                    let ui = u(xi.x, xi.y);
                    assert!(!dist.is_nan());
                    assert!(dist.is_finite());
                    assert!(!ai.is_nan());
                    assert!(ai.is_finite());
                    assert!(!k.is_nan());
                    assert!(k.is_finite());
                    let k_ai_m_over_dist = k * ai.sqrt() / dist;
                    assert!(!k_ai_m_over_dist.is_nan());
                    assert!(k_ai_m_over_dist.is_finite());
                    (hi_t + dt * (ui + k_ai_m_over_dist * hj_t_dt)) / (1f32 + k_ai_m_over_dist * dt)
                };
                assert!(!hi_t_dt.is_nan());
                assert!(hi_t_dt.is_finite());
                assert!(hi_t_dt.is_sign_positive(), "{}", hi_t_dt);
                g.vertices[i].h = hi_t_dt;
                found_any = true;
                visited_vertices[upstream] = true;
            }
        }
        if !found_any {
            break;
        }
    }
    g
}

//fn main() {
//    let mut g = Graph::regular(w, h, 1f32);
//    g = iterate(g,10, 10, &|x, y| (x + y).ln(), 1f32, 1f32);
//}

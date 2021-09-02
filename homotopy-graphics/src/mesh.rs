use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use homotopy_common::declare_idx;
use homotopy_common::idx::IdxVec;

declare_idx! {
    pub struct VertexId = usize;
    pub struct ElementId = usize;
    pub struct SquareId = usize;
    pub struct CubeId = usize;
}

/// Represents concrete Cube Mesh to be subdivided and rendered.
#[derive(Clone, PartialEq, Default)]
pub struct CubeMesh {
    pub vertices: IdxVec<VertexId, Vertex>,
    pub cubes: IdxVec<CubeId, [VertexId; 8]>,
    division_memory: HashMap<Vec<VertexId>, VertexId>,
}

impl CubeMesh {
    pub fn new() -> Self {
        Self {
            vertices: IdxVec::new(),
            cubes: IdxVec::new(),
            division_memory: HashMap::new(),
        }
    }

    fn create_new(&mut self, verts: &[VertexId]) -> VertexId {
        let vertices: Vec<&Vertex> = verts
            .iter()
            .map(|v_id| self.vertices.get(*v_id).unwrap())
            .collect();
        let first_bound = vertices[0].boundary;
        let mut bound = vertices.iter().fold(first_bound, |a, v| max(a, v.boundary));
        bound = max(
            bound,
            match verts.len() {
                2 => 1,
                4 => 2,
                _ => panic!(),
            },
        );

        let mut new_vert = Vertex::new(0.0, 0.0, 0.0, 0.0, bound);
        let scale = 1.0
            / match vertices.len() {
                2 => 2.0,
                4 => 4.0,
                _ => panic!("Unexpected number of vertices"),
            };
        for v in vertices {
            new_vert.add_scaled(v, scale);
        }
        let v_id = self.vertices.push(new_vert);
        self.division_memory.insert(verts.to_owned(), v_id);
        v_id
    }

    /// Returns a VertexId that coresponds to the average of the suplied vertices.
    pub fn linearly_divide(&mut self, mut verts: Vec<VertexId>) -> VertexId {
        verts.sort();
        let mut c = verts.clone();
        c.dedup();
        match (verts.len(), c.len()) {
            (2 | 4, 1) => c[0],
            (2, 2) | (4, 4 | 3) => self
                .division_memory
                .get(&verts)
                .copied()
                .unwrap_or_else(|| self.create_new(&verts)),
            (4, 2) => self.linearly_divide(c),
            _ => panic!(),
        }
    }

    pub fn mk_vertex(&mut self, vertex: Vertex) -> VertexId {
        self.vertices.push(vertex)
    }

    pub fn mk_cube(&mut self, vertices: [VertexId; 8]) -> CubeId {
        self.cubes.push(vertices)
    }
}

impl fmt::Debug for CubeMesh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Square Mesh {{ vertices: [ ")?;
        Ok(for (id, v) in self.vertices.iter() {
            writeln!(f, "{:?}: {:?}", id, v)?;
        })?;
        writeln!(f, "], \n squares: [")?;
        Ok(for (id, v) in self.cubes.iter() {
            writeln!(f, "{:?}: {:?}", id, v)?;
        })?;
        writeln!(f, "] }}")
    }
}

/// Represents concrete Square Mesh to be subdivided and rendered.
#[derive(Clone, PartialEq, Default)]
pub struct SquareMesh {
    pub vertices: IdxVec<VertexId, Vertex>,
    pub squares: IdxVec<SquareId, [VertexId; 4]>,
    division_memory: HashMap<(VertexId, VertexId), VertexId>,
}

impl SquareMesh {
    pub fn new() -> Self {
        Self {
            vertices: IdxVec::new(),
            squares: IdxVec::new(),
            division_memory: HashMap::new(),
        }
    }

    fn create_new(&mut self, fst: VertexId, snd: VertexId) -> VertexId {
        let fst_v = self.vertices.get(fst).unwrap();
        let snd_v = self.vertices.get(snd).unwrap();
        let new_b = max(fst_v.boundary, snd_v.boundary);
        let new_v = Vertex::new(
            (fst_v.x + snd_v.x) * 0.5,
            (fst_v.y + snd_v.y) * 0.5,
            (fst_v.z + snd_v.z) * 0.5,
            (fst_v.t + snd_v.t) * 0.5,
            min(2, new_b),
        );
        let new_id = self.vertices.push(new_v);
        self.division_memory.insert((fst, snd), new_id);
        self.division_memory.insert((snd, fst), new_id);
        new_id
    }

    /// Returns a VertexId that coresponds to the average of the suplied vertices.
    pub fn linearly_divide(&mut self, fst: VertexId, snd: VertexId) -> VertexId {
        if fst == snd {
            fst
        } else {
            self.division_memory
                .get(&(fst, snd))
                .copied()
                .unwrap_or_else(|| self.create_new(fst, snd))
        }
    }

    pub fn mk_vertex(&mut self, vertex: Vertex) -> VertexId {
        self.vertices.push(vertex)
    }

    pub fn mk_square(&mut self, vertices: [VertexId; 4]) -> SquareId {
        self.squares.push(vertices)
    }
}

impl fmt::Debug for SquareMesh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Square Mesh {{ vertices: [ ")?;
        Ok(for (id, v) in self.vertices.iter() {
            writeln!(f, "{:?}: {:?}", id, v)?;
        })?;
        writeln!(f, "], \n squares: [")?;
        Ok(for (id, v) in self.squares.iter() {
            writeln!(f, "{:?}: {:?}", id, v)?;
        })?;
        writeln!(f, "] }}")
    }
}

/// Represents all cubical surface elements.
#[derive(Clone, PartialEq, Default)]
pub struct Mesh {
    pub vertices: IdxVec<VertexId, Vertex>,
    pub elements: IdxVec<ElementId, Element>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: IdxVec::new(),
            elements: IdxVec::new(),
        }
    }

    pub fn mk_vertex(&mut self, vertex: Vertex) -> VertexId {
        self.vertices.push(vertex)
    }

    pub fn mk_element(&mut self, n: Dimension, verts: &[VertexId]) -> ElementId {
        match n {
            0 => self.elements.push(Element::Cube0(verts[0])),
            n => {
                let subcube_0 = self.mk_element(n - 1, &verts[..verts.len() / 2]);
                let subcube_1 = self.mk_element(n - 1, &verts[verts.len() / 2..]);
                let cube_n = Element::CubeN(CubeN::new(n, subcube_0, subcube_1));
                self.elements.push(cube_n)
            }
        }
    }

    pub fn order_of(&self, element: ElementId) -> Dimension {
        match self.elements[element] {
            Element::Cube0(_) => 0,
            Element::CubeN(CubeN { n, .. }) => n,
        }
    }

    /// Flattens an cubical element into a list of its vertices.
    pub fn flatten(&self, element: ElementId) -> Vec<VertexId> {
        match self.elements[element] {
            Element::Cube0(v_id) => vec![v_id],
            Element::CubeN(CubeN {
                subcube_0,
                subcube_1,
                ..
            }) => {
                let mut s0 = self.flatten(subcube_0);
                let mut s1 = self.flatten(subcube_1);
                s0.append(&mut s1);
                s0
            }
        }
    }
}

impl fmt::Debug for Mesh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Mesh {{ vertices: [ ")?;
        Ok(for (id, v) in self.vertices.iter() {
            writeln!(f, "{:?}: {:?} ", id, v)?;
        })?;
        writeln!(f, "], \n elements: [ ")?;
        Ok(for (id, v) in self.elements.iter() {
            writeln!(f, "{:?}: {:?} ", id, v)?;
        })?;
        write!(f, "] }}")
    }
}

/// Represents cubical surface elements (points, lines, squares, cubes, ...)
/// that have the cubical property (composed of exactly opposite subfaces).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Element {
    Cube0(VertexId),
    CubeN(CubeN),
}

/// Represents an n-cube by recording the two (n-1)-cubes that make it and its order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CubeN {
    n: Dimension,
    subcube_0: ElementId,
    subcube_1: ElementId,
}

impl CubeN {
    fn new(n: Dimension, subcube_0: ElementId, subcube_1: ElementId) -> Self {
        Self {
            n,
            subcube_0,
            subcube_1,
        }
    }
}

impl Element {
    pub fn order(&self) -> Dimension {
        match self {
            Element::Cube0(_) => 0,
            Element::CubeN(c) => c.n,
        }
    }
}

type Dimension = u8;

type Coordinate = f64;

/// Represents a Vertex in a 4-space
#[derive(Clone, PartialEq)]
pub struct Vertex {
    pub x: Coordinate,
    pub y: Coordinate,
    pub z: Coordinate,
    pub t: Coordinate,
    pub boundary: Dimension,
    // generator: Generator, should this be represented as an id?
}

impl Vertex {
    pub fn new(x: f64, y: f64, z: f64, t: f64, boundary: u8) -> Self {
        Self {
            x,
            y,
            z,
            t,
            boundary,
        }
    }

    pub fn add_scaled(&mut self, other: &Self, scale: f64) {
        self.x += other.x * scale;
        self.y += other.y * scale;
        self.z += other.z * scale;
        self.t += other.t * scale;
    }

    pub fn scale(&mut self, scale: f64) {
        self.x *= scale;
        self.y *= scale;
        self.z *= scale;
        self.t *= scale;
    }

    pub fn copy_from(&mut self, other: &Self) {
        self.x = other.x;
        self.y = other.y;
        self.z = other.z;
        self.t = other.t;
        self.boundary = other.boundary;
    }
}

impl fmt::Debug for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(coordinates: ({},{},{},{}), boundary: {})",
            self.x, self.y, self.z, self.t, self.boundary
        )
    }
}

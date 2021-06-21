use std::hash::Hash;

use crate::declare_idx;
use crate::idx::IdxVec;

declare_idx! { pub struct VertexId = usize; }
declare_idx! { pub struct ElementId = usize; }

type Dimension = u8;

#[derive(Copy, Clone, PartialEq)]
struct Coordinate {
    val: f64,
}

/// Represents a Vertex in a 4-space
#[derive(Clone, PartialEq)]
pub struct Vertex {
    x: Coordinate,
    y: Coordinate,
    z: Coordinate,
    t: Coordinate,
    boundary: Dimension,
    // generator: Generator, should this be represented as an id?
}

/// Represents a cubical surface mesh using an indexed list of vertices and an
/// indexed list of cubical surface elements that organise the vertices.
#[derive(Clone, PartialEq)]
pub struct Mesh {
    vertices: IdxVec<VertexId, Vertex>,
    elements: IdxVec<ElementId, Element>,
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
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
}

/// Represents cubical surface elements (points, lines, squares, cubes, ...)
/// that have the cubical property (composed of exactly opposite subfaces).
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Element {
    Cube0(VertexId),
    CubeN(CubeN),
}

impl Coordinate {
    pub fn new(val: f64) -> Coordinate {
        Coordinate { val }
    }
}

impl Vertex {
    pub fn new(x: f64, y: f64, z: f64, t: f64, boundary: u8) -> Vertex {
        Vertex {
            x: Coordinate::new(x),
            y: Coordinate::new(y),
            z: Coordinate::new(z),
            t: Coordinate::new(t),
            boundary,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CubeN {
    n: Dimension,
    subcube_0: ElementId,
    subcube_1: ElementId,
}

impl CubeN {
    fn new(n: Dimension, subcube_0: ElementId, subcube_1: ElementId) -> CubeN {
        CubeN {
            n,
            subcube_0,
            subcube_1,
        }
    }
}

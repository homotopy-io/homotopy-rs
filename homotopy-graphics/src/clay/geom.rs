use std::collections::HashMap;
use std::iter::FusedIterator;
use std::ops::{Deref, DerefMut};

use ultraviolet::Vec4;

use homotopy_common::declare_idx;
use homotopy_common::idx::IdxVec;

declare_idx! {
    pub struct Vertex = usize;
    pub struct Element = usize;
    pub struct Square = usize;
    pub struct Cube = usize;
}

pub type Dimension = u8;

/// Represents a vertex in a 4-space
#[derive(Debug, Clone, PartialEq)]
pub struct VertexData {
    pub vertex: Vec4,
    pub boundary: Dimension,
    // generator: Generator
}

/// Represents cubical surface elements (points, lines, squares, cubes, ...)
/// that have the cubical property (composed of exactly opposite subfaces).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ElementData {
    Cube0(Vertex),
    CubeN(CubeN),
}

/// Represents an n-cube by recording the two (n - 1)-cubes that make it and
/// its order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CubeN {
    n: Dimension,
    subcube_0: Element,
    subcube_1: Element,
}

/// Represents concrete square mesh to be subdivided and rendered.
#[derive(Clone, PartialEq, Default)]
pub struct SquareMesh {
    pub vertices: IdxVec<Vertex, VertexData>,
    pub squares: IdxVec<Square, [Vertex; 4]>,
    division_memory: HashMap<(Vertex, Vertex), Vertex>,
}

/// Represents concrete cube mesh to be subdivided and rendered.
#[derive(Clone, PartialEq, Default)]
pub struct CubeMesh {
    pub vertices: IdxVec<Vertex, VertexData>,
    pub cubes: IdxVec<Cube, [Vertex; 8]>,
    division_memory: HashMap<Vec<Vertex>, Vertex>,
}

/// Represents all cubical surface elements
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh {
    pub vertices: IdxVec<Vertex, VertexData>,
    pub elements: IdxVec<Element, ElementData>,
}

impl Deref for VertexData {
    type Target = Vec4;

    fn deref(&self) -> &Self::Target {
        &self.vertex
    }
}

impl DerefMut for VertexData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vertex
    }
}

impl ElementData {
    fn order(&self) -> Dimension {
        match self {
            ElementData::Cube0(_) => 0,
            ElementData::CubeN(cube) => cube.n,
        }
    }
}

impl Mesh {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn mk_vertex(&mut self, vertex: VertexData) -> Vertex {
        self.vertices.push(vertex)
    }

    pub fn mk_element_0(&mut self, vertex: Vertex) -> Element {
        self.elements.push(ElementData::Cube0(vertex))
    }

    pub fn mk_element_n(&mut self, subcube_0: Element, subcube_1: Element) -> Element {
        assert_eq!(self.order_of(subcube_0), self.order_of(subcube_1));

        self.elements.push(ElementData::CubeN(CubeN {
            n: self.order_of(subcube_0) + 1,
            subcube_0,
            subcube_1,
        }))
    }

    pub fn mk_element_from(&mut self, verts: &[Vertex]) -> Element {
        assert!(!verts.is_empty());

        match verts {
            [] => panic!(),
            [v] => self.mk_element_0(*v),
            verts => {
                let subcube_0 = self.mk_element_from(&verts[..verts.len() / 2]);
                let subcube_1 = self.mk_element_from(&verts[verts.len() / 2..]);
                self.mk_element_n(subcube_0, subcube_1)
            }
        }
    }

    pub fn flatten(&self, element: Element) -> impl Iterator<Item = Vertex> + '_ {
        Flattener {
            mesh: self,
            to_visit: vec![element],
        }
    }

    pub fn order_of(&self, element: Element) -> Dimension {
        self.elements[element].order()
    }
}

pub struct Flattener<'a> {
    mesh: &'a Mesh,
    to_visit: Vec<Element>,
}

impl<'a> Iterator for Flattener<'a> {
    type Item = Vertex;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let element = self.to_visit.pop()?;
            match &self.mesh.elements[element] {
                ElementData::Cube0(vertex) => return Some(*vertex),
                ElementData::CubeN(cube) => {
                    self.to_visit.push(cube.subcube_1);
                    self.to_visit.push(cube.subcube_0);
                }
            }
        }
    }
}

impl<'a> FusedIterator for Flattener<'a> {}

pub trait VertexExt {
    fn with_boundary(self, n: Dimension) -> VertexData;
}

impl VertexExt for Vec4 {
    fn with_boundary(self, n: Dimension) -> VertexData {
        VertexData {
            vertex: self,
            boundary: n,
        }
    }
}

/*
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

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
*/

use std::{
    cmp::Ordering,
    collections::HashMap,
    convert::TryInto,
    iter::FusedIterator,
    ops::{Deref, DerefMut},
};

use homotopy_common::{
    declare_idx,
    idx::{Idx, IdxVec},
};
use homotopy_core::{
    common::{DimensionError, Height},
    cubicalisation::{Bias, Coord, EdgeId, NodeId},
    Diagram, DiagramN, Generator,
};
use ultraviolet::{Vec3, Vec4};

use crate::gl;

declare_idx! {
    pub struct Vertex = u16;
    pub struct Element = usize;
    pub struct Square = usize;
    pub struct Cube = usize;
}

pub type Dimension = u8;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Boundary {
    /// Corner - no freedom to move
    Zero = 0,
    /// Edge - free to move along line
    One = 1,
    /// Surface - free to move in space
    Two = 2,
    /// Volume - free to move in time and space
    Three = 3,
}

impl Boundary {
    /// Increase the boundary by 1.
    fn inc(self) -> Self {
        match self {
            Self::Zero => Self::One,
            Self::One => Self::Two,
            _ => Self::Three,
        }
    }

    /// Calculate the boundary of a given location in a diagram.
    fn at_location(diagram: &Diagram, location: &[Height]) -> Self {
        match location.split_first() {
            None => Self::Zero,
            Some((&height, location)) => {
                let diagram: &DiagramN = diagram.try_into().unwrap();
                let max = diagram.size();
                let slice = diagram.slice(height).unwrap();
                match height {
                    Height::Regular(j) if j == 0 || j == max => Self::at_location(&slice, location),
                    _ => Self::at_location(&slice, location).inc(),
                }
            }
        }
    }
}

/// Represents a vertex in a 4-space
#[derive(Debug, Clone, PartialEq)]
pub struct VertexData {
    pub vertex: Vec4,
    pub boundary: Boundary,
    pub generator: Generator,
}

pub trait MeshData {
    type Idx: Idx;

    fn remap<T>(&mut self, remapper: &mut VertexRemapper<T>)
    where
        T: MeshData;
}

pub trait FromMesh<T>: MeshData + Sized
where
    T: MeshData,
{
    fn try_from(mesh: &Mesh<T>, element: T::Idx) -> Option<Self>;
}

impl<T> FromMesh<Self> for T
where
    T: MeshData + Clone,
{
    fn try_from(mesh: &Mesh<Self>, element: Self::Idx) -> Option<Self> {
        mesh.elements.get(element).cloned()
    }
}

/// Represents cubical surface elements (points, lines, squares, cubes, ...)
/// that have the cubical property (composed of exactly opposite subfaces).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ElementData {
    Cube0(Vertex),
    CubeN(CubeN),
}

impl MeshData for ElementData {
    type Idx = Element;

    fn remap<T>(&mut self, remapper: &mut VertexRemapper<T>)
    where
        T: MeshData,
    {
        match self {
            ElementData::Cube0(vertex) => *vertex = remapper.get(*vertex),
            ElementData::CubeN(_) => {}
        }
    }
}

/// Represents an n-cube by recording the two (n - 1)-cubes that make it and
/// its order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CubeN {
    n: Dimension,
    subcube_0: Element,
    subcube_1: Element,
}

pub type SquareData = [Vertex; 4];
pub type CubeData = [Vertex; 8];

impl MeshData for SquareData {
    type Idx = Square;

    fn remap<T>(&mut self, remapper: &mut VertexRemapper<T>)
    where
        T: MeshData,
    {
        for v in self.iter_mut() {
            *v = remapper.get(*v);
        }
    }
}

impl MeshData for CubeData {
    type Idx = Cube;

    fn remap<T>(&mut self, remapper: &mut VertexRemapper<T>)
    where
        T: MeshData,
    {
        for v in self.iter_mut() {
            *v = remapper.get(*v);
        }
    }
}

impl FromMesh<ElementData> for SquareData {
    fn try_from(mesh: &Mesh<ElementData>, element: Element) -> Option<Self> {
        if mesh.order_of(element) == 2 {
            mesh.flatten(element).collect::<Vec<_>>().try_into().ok()
        } else {
            None
        }
    }
}

impl FromMesh<ElementData> for CubeData {
    fn try_from(mesh: &Mesh<ElementData>, element: Element) -> Option<Self> {
        if mesh.order_of(element) == 3 {
            mesh.flatten(element).collect::<Vec<_>>().try_into().ok()
        } else {
            None
        }
    }
}

/// Represents all cubical surface elements
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh<T = ElementData>
where
    T: MeshData,
{
    pub vertices: IdxVec<Vertex, VertexData>,
    pub elements: IdxVec<T::Idx, T>,
}

/// Represents concrete square mesh to be subdivided and rendered.
pub type SquareMesh = Mesh<SquareData>;
/// Represents concrete cube mesh to be subdivided and rendered.
pub type CubeMesh = Mesh<CubeData>;

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

    pub fn build(diagram: &DiagramN) -> Result<Self, DimensionError> {
        let diagram = Diagram::from(diagram.clone());

        // Cubicalise the diagram.
        let graph = diagram
            .clone()
            .cubicalise(&[Bias::Left].repeat(diagram.dimension() - 1))?;

        let mut mesh = Self::new();

        // Compute the coordinates of every node.
        let mut boundaries: HashMap<Coord, Boundary> = HashMap::new();
        let mut generators: HashMap<Coord, Generator> = HashMap::new();
        let mut coordinates: HashMap<Coord, Vec<Vec4>> = HashMap::new();
        for node in graph.nodes.values() {
            let key = &node.key;
            boundaries
                .entry(key.clone())
                .or_insert_with(|| Boundary::at_location(&diagram, key));
            generators
                .entry(key.clone())
                .or_insert_with(|| node.diagram.max_generator());
            coordinates.entry(key.clone()).or_default().push(
                [0, 1, 2, 3]
                    .map(|i| {
                        let n = node.coord.len();
                        if i >= n {
                            0.0
                        } else {
                            node.coord[n - i - 1].to_int() as f32
                        }
                    })
                    .into(),
            );
        }
        let coordinates: HashMap<Coord, Vec4> = coordinates
            .into_iter()
            .map(|(k, vs)| {
                let n = vs.len();
                (k, vs.into_iter().sum::<Vec4>() / n as f32)
            })
            .collect();

        // VERTICES
        let mut vertices: HashMap<Coord, Vertex> = HashMap::new();
        for (key, coord) in coordinates {
            let boundary = boundaries[&key];
            let generator = generators[&key];
            vertices.insert(
                key,
                mesh.mk_vertex(coord.with_boundary_and_generator(boundary, generator)),
            );
        }

        // ELEMENTS: 0-CUBES
        let mut elements_0d: IdxVec<NodeId, Element> = IdxVec::new();
        for node in graph.nodes.values() {
            let vertex = vertices[&node.key];
            elements_0d.push(mesh.mk_element_0(vertex));
        }

        // ELEMENTS: 1-CUBES
        let mut elements_1d: IdxVec<EdgeId, Element> = IdxVec::new();
        for edge in graph.edges.values() {
            let s = edge.source;
            let t = edge.target;
            // TODO(calintat): Orient edge.
            let subcube_0 = elements_0d[s];
            let subcube_1 = elements_0d[t];
            elements_1d.push(mesh.mk_element_n(subcube_0, subcube_1));
        }

        // ELEMENTS: 2-CUBES
        let mut elements_2d: HashMap<[EdgeId; 2], Element> = HashMap::new();
        for square in graph.squares() {
            let b = square.bottom;
            let t = square.top;
            // TODO(calintat): Orient square.
            let subcube_0 = elements_1d[b];
            let subcube_1 = elements_1d[t];

            elements_2d.insert([b, t], mesh.mk_element_n(subcube_0, subcube_1));
        }

        // ELEMENTS: 3-CUBES
        // let mut elements_3d: HashMap<[EdgeId; 4], Element> = HashMap::new();
        // for cube in graph.cubes() {
        //     let bf = cube.bottom_front;
        //     let bb = cube.bottom_back;
        //     let tf = cube.top_front;
        //     let tb = cube.top_back;
        //     // TODO(calintat): Orient cube.
        //     let subcube_0 = elements_2d[&[bf, bb]];
        //     let subcube_1 = elements_2d[&[tf, tb]];
        //     elements_3d.insert([bf, bb, tf, tb], mesh.mk_element_n(subcube_0, subcube_1));
        // }

        Ok(mesh)
    }
}

impl<T> Mesh<T>
where
    T: MeshData,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn mk_vertex(&mut self, vertex: VertexData) -> Vertex {
        self.vertices.push(vertex)
    }

    pub fn into<U>(self) -> Mesh<U>
    where
        U: FromMesh<T>,
    {
        let mut remapper = VertexRemapper::new(&self);
        let elements = self
            .elements
            .keys()
            .filter_map(|element| {
                let mut element = U::try_from(&self, element)?;
                remapper.remap(&mut element);
                Some(element)
            })
            .collect();

        Mesh {
            vertices: remapper.into_verts(),
            elements,
        }
    }
}

pub struct SquareMeshBuffers {
    pub element_buffer: gl::buffer::ElementBuffer,
    pub wireframe_element_buffer: gl::buffer::ElementBuffer,
    pub vertex_buffer: gl::buffer::Buffer<Vec3>,
    pub normal_buffer: gl::buffer::Buffer<Vec3>,
}

impl SquareMesh {
    pub fn mk_square(&mut self, square: SquareData) -> Square {
        self.elements.push(square)
    }

    pub fn buffer(&self, ctx: &gl::GlCtx) -> gl::Result<SquareMeshBuffers> {
        let vertices = self
            .vertices
            .values()
            .map(|v| v.xyz())
            .collect::<IdxVec<_, _>>();
        let mut elements = Vec::with_capacity(self.elements.len() * 6);
        let mut wireframe_elements = Vec::with_capacity(self.elements.len() * 12);
        let mut normals = IdxVec::splat(Vec3::zero(), vertices.len());

        {
            let mut push_element = |i: Vertex, j: Vertex, k: Vertex| {
                let i = i.index() as u16;
                let j = j.index() as u16;
                let k = k.index() as u16;

                wireframe_elements.push(i);
                wireframe_elements.push(j);
                wireframe_elements.push(j);
                wireframe_elements.push(k);
                wireframe_elements.push(k);
                wireframe_elements.push(i);

                elements.push(i);
                elements.push(j);
                elements.push(k);
            };

            let mut push_tri = |i: Vertex, j: Vertex, k: Vertex| {
                if i != j && j != k && k != i {
                    push_element(i, j, k);

                    let a = vertices[i];
                    let b = vertices[j];
                    let c = vertices[k];
                    let n = (b - a).cross(c - a);

                    normals[i] += n;
                    normals[j] += n;
                    normals[k] += n;
                }
            };

            // Triangulate mesh
            for square in self.elements.values() {
                // Bottom right triangle
                push_tri(square[0], square[1], square[3]);
                // Top left triangle
                push_tri(square[0], square[3], square[2]);
            }
        }

        // Average normals
        for normal in normals.values_mut() {
            normal.normalize();
        }

        // Buffer data
        let element_buffer =
            ctx.mk_element_buffer(&elements, gl::buffer::ElementKind::Triangles)?;
        let wireframe_element_buffer =
            ctx.mk_element_buffer(&wireframe_elements, gl::buffer::ElementKind::Lines)?;
        let vertex_buffer = ctx.mk_buffer(&vertices.into_raw())?;
        let normal_buffer = ctx.mk_buffer(&normals.into_raw())?;

        Ok(SquareMeshBuffers {
            element_buffer,
            wireframe_element_buffer,
            vertex_buffer,
            normal_buffer,
        })
    }
}

pub struct CubeMeshBuffers {
    pub element_buffer: gl::buffer::ElementBuffer,
    pub wireframe_element_buffer: gl::buffer::ElementBuffer,
    pub vertex_start_buffer: gl::buffer::Buffer<Vec4>,
    pub vertex_end_buffer: gl::buffer::Buffer<Vec4>,
    pub wireframe_vertex_buffer: gl::buffer::Buffer<Vec3>,
    // TODO(@doctorn) normals
}

impl CubeMesh {
    pub fn mk_cube(&mut self, cube: CubeData) -> Cube {
        self.elements.push(cube)
    }

    pub fn buffer(&self, ctx: &gl::GlCtx) -> gl::Result<CubeMeshBuffers> {
        declare_idx! {
            struct Segment = u16;
        }

        #[inline]
        fn parity_sort<F>(vertices: &mut [Vertex; 4], f: F) -> bool
        where
            F: Fn(Vertex, Vertex) -> Ordering,
        {
            let mut parity = true;

            macro_rules! bubble_pass  {
                ($($i:literal),*$(,)*) => {
                    $(if f(vertices[$i + 1], vertices[$i]) == Ordering::Less {
                        vertices.swap($i, $i + 1);
                        parity = !parity;
                    })*
                };
            }

            bubble_pass!(
                0, 1, 2, // First pass finds top element
                0, 1, // Second pass finds next element
                0, // Final pass orders remaining two elements
            );

            parity
        }

        let mut segment_cache = HashMap::new();
        let mut elements = Vec::with_capacity(self.elements.len() * 30);
        let mut wireframe_elements = Vec::with_capacity(self.elements.len() * 8);
        let mut segment_starts = IdxVec::with_capacity(self.elements.len() * 30);
        let mut segment_ends = IdxVec::with_capacity(self.elements.len() * 30);
        let wireframe_vertices = self.vertices.values().map(|v| v.xyz()).collect::<Vec<_>>();

        {
            let mut push_segment = |i: Vertex, j: Vertex| {
                if let Some(segment) = segment_cache.get(&(i, j)) {
                    *segment
                } else {
                    wireframe_elements.push(i.index() as u16);
                    wireframe_elements.push(j.index() as u16);

                    let start = segment_starts.push(*self.vertices[i]);
                    let end = segment_ends.push(*self.vertices[j]);
                    debug_assert!(start == end);
                    segment_cache.insert((i, j), start);
                    start
                }
            };

            let mut push_tri = |i: Segment, j: Segment, k: Segment| {
                if i != j && j != k && k != i {
                    elements.push(i.index() as u16);
                    elements.push(j.index() as u16);
                    elements.push(k.index() as u16);
                }
            };

            let mut push_tetra = |i: Vertex, j: Vertex, k: Vertex, l: Vertex| {
                let ([i, j, k, l], parity) = {
                    let mut vertices = [i, j, k, l];
                    let parity = parity_sort(&mut vertices, |i, j| {
                        self.vertices[i].w.partial_cmp(&self.vertices[j].w).unwrap()
                    });
                    (vertices, parity)
                };

                let ij = push_segment(i, j);
                let ik = push_segment(i, k);
                let il = push_segment(i, l);
                let jk = push_segment(j, k);
                let jl = push_segment(j, l);
                let kl = push_segment(k, l);

                if parity {
                    push_tri(ij, il, ik);
                    push_tri(jl, ik, jk);
                    push_tri(jl, il, ik);
                    push_tri(jl, il, kl);
                } else {
                    push_tri(il, ij, ik);
                    push_tri(ik, jl, jk);
                    push_tri(il, jl, ik);
                    push_tri(il, jl, kl);
                }
            };

            // Triangulate mesh
            for cube in self.elements.values() {
                push_tetra(cube[1], cube[4], cube[5], cube[7]);
                push_tetra(cube[0], cube[4], cube[1], cube[2]);
                push_tetra(cube[1], cube[7], cube[3], cube[2]);
                push_tetra(cube[4], cube[6], cube[7], cube[2]);
                push_tetra(cube[1], cube[7], cube[2], cube[4]);
            }
        }

        // Buffer data
        let element_buffer =
            ctx.mk_element_buffer(&elements, gl::buffer::ElementKind::Triangles)?;
        let wireframe_element_buffer =
            ctx.mk_element_buffer(&wireframe_elements, gl::buffer::ElementKind::Lines)?;
        let vertex_start_buffer = ctx.mk_buffer(&segment_starts.into_raw())?;
        let vertex_end_buffer = ctx.mk_buffer(&segment_ends.into_raw())?;
        let wireframe_vertex_buffer = ctx.mk_buffer(&wireframe_vertices)?;

        Ok(CubeMeshBuffers {
            element_buffer,
            wireframe_element_buffer,
            vertex_start_buffer,
            vertex_end_buffer,
            wireframe_vertex_buffer,
        })
    }
}

impl<T> Default for Mesh<T>
where
    T: MeshData,
{
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            elements: Default::default(),
        }
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

pub struct VertexRemapper<'a, T>
where
    T: MeshData,
{
    mesh: &'a Mesh<T>,
    remapping: HashMap<Vertex, Vertex>,
    data: IdxVec<Vertex, VertexData>,
}

impl<'a, T> VertexRemapper<'a, T>
where
    T: MeshData,
{
    fn new(mesh: &'a Mesh<T>) -> Self {
        Self {
            mesh,
            remapping: Default::default(),
            data: Default::default(),
        }
    }

    fn get(&mut self, unmapped: Vertex) -> Vertex {
        if let Some(vertex) = self.remapping.get(&unmapped) {
            return *vertex;
        }

        let vertex = self.data.push(self.mesh.vertices[unmapped].clone());

        self.remapping.insert(unmapped, vertex);
        vertex
    }

    fn remap<U>(&mut self, data: &mut U)
    where
        U: MeshData,
    {
        U::remap(data, self);
    }

    fn into_verts(self) -> IdxVec<Vertex, VertexData> {
        self.data
    }
}

pub trait VertexExt {
    fn with_boundary_and_generator(self, boundary: Boundary, generator: Generator) -> VertexData;
}

impl VertexExt for Vec4 {
    fn with_boundary_and_generator(self, boundary: Boundary, generator: Generator) -> VertexData {
        VertexData {
            vertex: self,
            boundary,
            generator,
        }
    }
}

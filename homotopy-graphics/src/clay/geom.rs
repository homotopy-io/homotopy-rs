use std::{
    cmp::Ordering,
    collections::HashMap,
    convert::{TryFrom, TryInto},
    iter::FusedIterator,
    ops::{Deref, DerefMut},
};

use homotopy_common::{
    declare_idx,
    graph::{Edge, Node},
    idx::{Idx, IdxVec},
};
use homotopy_core::{
    common::{DimensionError, Direction, SliceIndex},
    cubicalisation::Bias,
    graph::Coord,
    Diagram, DiagramN, Generator,
};
use ultraviolet::{Mat3, Vec3, Vec4};

use crate::gl;

declare_idx! {
    pub struct Vertex = u16;
    pub struct Element = usize;
    pub struct Point = usize;
    pub struct Line = usize;
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
    pub fn inc(self) -> Self {
        match self {
            Self::Zero => Self::One,
            Self::One => Self::Two,
            _ => Self::Three,
        }
    }

    /// Calculate the boundary of a given location in a diagram.
    fn at_coord(diagram: &Diagram, coord: &[SliceIndex]) -> Self {
        match coord {
            [] | [_] => Self::Zero,
            [index, coord @ ..] => {
                let d: &DiagramN = diagram.try_into().unwrap();
                let slice = d.slice(*index).unwrap();
                let boundary = Self::at_coord(&slice, coord);
                match index {
                    SliceIndex::Boundary(_) => boundary,
                    SliceIndex::Interior(_) => boundary.inc(),
                }
            }
        }
    }

    fn debug_color(self) -> Vec3 {
        match self {
            Boundary::Zero => Vec3::new(1., 1., 0.),
            Boundary::One => Vec3::new(1., 0., 1.),
            Boundary::Two => Vec3::new(0., 1., 1.),
            Boundary::Three => Vec3::zero(),
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

pub trait MeshData: Clone {
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

pub type PointData = Vertex;
pub type LineData = [Vertex; 2];
pub type SquareData = [Vertex; 4];
pub type CubeData = [Vertex; 8];

impl MeshData for PointData {
    type Idx = Point;

    fn remap<T>(&mut self, remapper: &mut VertexRemapper<T>)
    where
        T: MeshData,
    {
        *self = remapper.get(*self);
    }
}

impl MeshData for LineData {
    type Idx = Line;

    fn remap<T>(&mut self, remapper: &mut VertexRemapper<T>)
    where
        T: MeshData,
    {
        for v in self.iter_mut() {
            *v = remapper.get(*v);
        }
    }
}

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

fn extract_n_cube<T>(mesh: &Mesh<ElementData>, element: Element, order: Dimension) -> Option<T>
where
    T: TryFrom<Vec<Vertex>>,
{
    if mesh.order_of(element) == order {
        let verts = mesh.flatten(element).collect::<Vec<_>>();

        {
            let mut unique = verts.clone();
            unique.dedup();

            if unique.len() <= order as usize {
                return None;
            }
        }

        let codimension = mesh.diagram.dimension()
            - verts
                .iter()
                .map(|v| mesh.vertices[*v].generator.dimension)
                .min()
                .unwrap();

        if codimension > order as usize {
            return None;
        }

        verts.try_into().ok()
    } else {
        None
    }
}

impl FromMesh<ElementData> for PointData {
    fn try_from(mesh: &Mesh<ElementData>, element: Element) -> Option<Self> {
        let vertex: [Self; 1] = extract_n_cube(mesh, element, 0)?;
        Some(vertex[0])
    }
}

impl FromMesh<ElementData> for LineData {
    fn try_from(mesh: &Mesh<ElementData>, element: Element) -> Option<Self> {
        extract_n_cube(mesh, element, 1)
    }
}

impl FromMesh<ElementData> for SquareData {
    fn try_from(mesh: &Mesh<ElementData>, element: Element) -> Option<Self> {
        extract_n_cube(mesh, element, 2)
    }
}

impl FromMesh<ElementData> for CubeData {
    fn try_from(mesh: &Mesh<ElementData>, element: Element) -> Option<Self> {
        extract_n_cube(mesh, element, 3)
    }
}

/// Represents all cubical surface elements
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh<T = ElementData>
where
    T: MeshData,
{
    pub diagram: Diagram,
    pub vertices: IdxVec<Vertex, VertexData>,
    pub elements: IdxVec<T::Idx, T>,
}

/// Represents a set of points.
pub type PointMesh = Mesh<PointData>;
/// Represents a set of piecewise linear curves to be subdivided and rendered.
pub type LineMesh = Mesh<LineData>;
/// Represents a concrete square mesh to be subdivided and rendered.
pub type SquareMesh = Mesh<SquareData>;
/// Represents a concrete cube mesh to be subdivided and rendered.
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

        let mut mesh = Self::new(diagram);

        // Compute the coordinates of every node.
        let mut boundaries: HashMap<Coord, Boundary> = HashMap::new();
        let mut generators: HashMap<Coord, Generator> = HashMap::new();
        let mut coordinates: HashMap<Coord, Vec<Vec4>> = HashMap::new();
        for (n, nd) in graph.inner().nodes() {
            let label = graph.label(n).to_vec();
            boundaries
                .entry(label.clone())
                .or_insert_with(|| Boundary::at_coord(&mesh.diagram, &label));
            generators
                .entry(label.clone())
                .or_insert_with(|| nd.1.max_generator());
            coordinates.entry(label).or_default().push({
                let v = (0..4)
                    .map(|i| {
                        let n = nd.0.len();
                        if i >= n {
                            0.0
                        } else {
                            let j = n - i - 1;
                            nd.0[j].to_int(graph.size(j)) as f32
                        }
                    })
                    .collect::<Vec<_>>();
                (v[0], v[1], v[2], v[3]).into()
            });
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
        let mut elements_0d: IdxVec<Node, Element> = IdxVec::new();
        for n in graph.inner().nodes_keys() {
            let vertex = vertices[graph.label(n)];
            elements_0d.push(mesh.mk_element_0(vertex));
        }

        // ELEMENTS: 1-CUBES
        let mut elements_1d: IdxVec<Edge, Element> = IdxVec::new();
        for (e, edge) in graph.inner().edges() {
            let s = edge.source();
            let t = edge.target();

            let mut subcube_0 = elements_0d[s];
            let mut subcube_1 = elements_0d[t];
            if graph.get_direction(e) == Direction::Backward {
                std::mem::swap(&mut subcube_0, &mut subcube_1);
            }

            elements_1d.push(mesh.mk_element_n(subcube_0, subcube_1));
        }

        // ELEMENTS: 2-CUBES
        let mut elements_2d: HashMap<[Edge; 2], Element> = HashMap::new();
        for square in graph.squares() {
            let b = square.bottom;
            let t = square.top;

            let mut subcube_0 = elements_1d[b];
            let mut subcube_1 = elements_1d[t];
            if graph.get_direction(square.left) == Direction::Backward {
                std::mem::swap(&mut subcube_0, &mut subcube_1);
            }

            elements_2d.insert([b, t], mesh.mk_element_n(subcube_0, subcube_1));
        }

        // ELEMENTS: 3-CUBES
        let mut elements_3d: HashMap<[Edge; 4], Element> = HashMap::new();
        for cube in graph.cubes() {
            let bl = cube.bottom_left;
            let br = cube.bottom_right;
            let tl = cube.top_left;
            let tr = cube.top_right;

            let mut subcube_0 = elements_2d[&[bl, br]];
            let mut subcube_1 = elements_2d[&[tl, tr]];
            if graph.get_direction(cube.left_front) == Direction::Backward {
                std::mem::swap(&mut subcube_0, &mut subcube_1);
            }

            elements_3d.insert([bl, br, tl, tr], mesh.mk_element_n(subcube_0, subcube_1));
        }

        Ok(mesh)
    }
}

impl<T> Mesh<T>
where
    T: MeshData,
{
    pub fn new(diagram: Diagram) -> Self {
        Self {
            vertices: Default::default(),
            elements: Default::default(),
            diagram,
        }
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
            diagram: self.diagram,
        }
    }
}

pub struct LineBuffers {
    pub element_buffer: gl::buffer::ElementBuffer,
    pub vertex_buffer: gl::buffer::Buffer<Vec3>,
}

impl LineMesh {
    pub fn mk_line(&mut self, line: LineData) -> Line {
        self.elements.push(line)
    }

    pub fn buffer(&self, ctx: &gl::GlCtx) -> gl::Result<LineBuffers> {
        let vertices = self.vertices.values().map(|v| v.xyz()).collect::<Vec<_>>();
        let mut elements = Vec::with_capacity(self.elements.len());

        for line in self.elements.values() {
            elements.push(line[0].index() as u16);
            elements.push(line[1].index() as u16);
        }

        let element_buffer = ctx.mk_element_buffer(&elements, gl::buffer::ElementKind::Lines)?;
        let vertex_buffer = ctx.mk_buffer(&vertices)?;

        Ok(LineBuffers {
            element_buffer,
            vertex_buffer,
        })
    }
}

pub struct SquareMeshBuffers {
    pub element_buffer: gl::buffer::ElementBuffer,
    pub wireframe_element_buffer: gl::buffer::ElementBuffer,
    pub wireframe_color_buffer: gl::buffer::Buffer<Vec3>,
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
        let boundary_colors = self
            .vertices
            .values()
            .map(|v| v.boundary.debug_color())
            .collect::<Vec<_>>();
        let mut elements = Vec::with_capacity(self.elements.len() * 6);
        let mut wireframe_elements = Vec::with_capacity(self.elements.len() * 12);
        let mut normals = IdxVec::splat(Vec3::zero(), vertices.len());

        {
            let mut push_wireframe_element = |i: Vertex, j: Vertex| {
                wireframe_elements.push(i.index() as u16);
                wireframe_elements.push(j.index() as u16);
            };

            let mut push_element = |i: Vertex, j: Vertex, k: Vertex| {
                push_wireframe_element(i, j);
                push_wireframe_element(j, k);
                push_wireframe_element(k, i);

                elements.push(i.index() as u16);
                elements.push(j.index() as u16);
                elements.push(k.index() as u16);
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
        let wireframe_color_buffer = ctx.mk_buffer(&boundary_colors)?;
        let vertex_buffer = ctx.mk_buffer(&vertices.into_raw())?;
        let normal_buffer = ctx.mk_buffer(&normals.into_raw())?;

        Ok(SquareMeshBuffers {
            element_buffer,
            wireframe_element_buffer,
            wireframe_color_buffer,
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
    pub normal_start_buffer: gl::buffer::Buffer<Vec4>,
    pub normal_end_buffer: gl::buffer::Buffer<Vec4>,
}

declare_idx! { struct Segment = u16; }

pub struct CubeMeshBufferer<'a> {
    mesh: &'a CubeMesh,

    segment_cache: HashMap<(Vertex, Vertex), Segment>,
    elements: Vec<u16>,
    wireframe_elements: Vec<u16>,

    segment_starts: IdxVec<Segment, Vec4>,
    segment_ends: IdxVec<Segment, Vec4>,
    normals: IdxVec<Vertex, Vec4>,

    wireframe_vertices: Vec<Vec3>,
}

impl<'a> CubeMeshBufferer<'a> {
    fn new(mesh: &'a CubeMesh) -> Self {
        let len = mesh.elements.len();
        let segments = 30 * len;

        Self {
            mesh,

            segment_cache: HashMap::new(),
            elements: Vec::with_capacity(segments),
            wireframe_elements: Vec::with_capacity(len * 8),

            segment_starts: IdxVec::with_capacity(segments),
            segment_ends: IdxVec::with_capacity(segments),
            normals: IdxVec::splat(Vec4::zero(), mesh.vertices.len()),

            wireframe_vertices: mesh.vertices.values().map(|v| v.xyz()).collect(),
        }
    }

    fn mk_segment_uncached(&mut self, i: Vertex, j: Vertex) -> Segment {
        self.wireframe_elements.push(i.index() as u16);
        self.wireframe_elements.push(j.index() as u16);

        let segment = self.segment_starts.push(*self.mesh.vertices[i]);
        self.segment_ends.push(*self.mesh.vertices[j]);

        self.segment_cache.insert((i, j), segment);
        segment
    }

    fn mk_segment(&mut self, i: Vertex, j: Vertex) -> Segment {
        self.segment_cache
            .get(&(i, j))
            .copied()
            .unwrap_or_else(|| self.mk_segment_uncached(i, j))
    }

    fn push_tri(&mut self, i: Segment, j: Segment, k: Segment) {
        if i != j && j != k && k != i {
            self.elements.push(i.index() as u16);
            self.elements.push(j.index() as u16);
            self.elements.push(k.index() as u16);
        }
    }

    fn push_tetra(&mut self, i: Vertex, j: Vertex, k: Vertex, l: Vertex) {
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

        let ([i, j, k, l], parity) = {
            let mut vertices = [i, j, k, l];
            let parity = parity_sort(&mut vertices, |i, j| {
                self.mesh.vertices[i]
                    .w
                    .partial_cmp(&self.mesh.vertices[j].w)
                    .unwrap()
            });
            (vertices, parity)
        };

        let ij = self.mk_segment(i, j);
        let ik = self.mk_segment(i, k);
        let il = self.mk_segment(i, l);
        let jk = self.mk_segment(j, k);
        let jl = self.mk_segment(j, l);
        let kl = self.mk_segment(k, l);

        {
            let origin = *self.mesh.vertices[i];
            let v_0 = *self.mesh.vertices[j] - origin;
            let v_1 = *self.mesh.vertices[k] - origin;
            let v_2 = *self.mesh.vertices[l] - origin;

            let xs = Vec3::new(v_0.x, v_1.x, v_2.x);
            let ys = Vec3::new(v_0.y, v_1.y, v_2.y);
            let zs = Vec3::new(v_0.z, v_1.z, v_2.z);
            let ws = Vec3::new(v_0.w, v_1.w, v_2.w);

            let m_0 = Mat3::new(ys, zs, ws);
            let m_1 = Mat3::new(xs, zs, ws);
            let m_2 = Mat3::new(xs, ys, ws);
            let m_3 = Mat3::new(xs, ys, zs);

            let mut normal = Vec4::new(
                -m_0.determinant(),
                m_1.determinant(),
                -m_2.determinant(),
                m_3.determinant(),
            );

            if !parity {
                normal = -normal;
            }

            self.normals[i] += normal;
            self.normals[j] += normal;
            self.normals[k] += normal;
            self.normals[l] += normal;
        }

        if parity {
            self.push_tri(ij, il, ik);
            self.push_tri(jl, ik, jk);
            self.push_tri(jl, il, ik);
            self.push_tri(jl, il, kl);
        } else {
            self.push_tri(il, ij, ik);
            self.push_tri(ik, jl, jk);
            self.push_tri(il, jl, ik);
            self.push_tri(il, jl, kl);
        }
    }

    fn triangulate(&mut self) {
        // Triangulate mesh
        for cube in self.mesh.elements.values() {
            self.push_tetra(cube[1], cube[4], cube[5], cube[7]);
            self.push_tetra(cube[0], cube[4], cube[1], cube[2]);
            self.push_tetra(cube[1], cube[7], cube[3], cube[2]);
            self.push_tetra(cube[4], cube[6], cube[7], cube[2]);
            self.push_tetra(cube[1], cube[7], cube[2], cube[4]);
        }
    }

    fn extract_buffers(mut self, ctx: &gl::GlCtx) -> gl::Result<CubeMeshBuffers> {
        // Average normals
        for normal in self.normals.values_mut() {
            normal.normalize();
        }

        let mut normal_starts = IdxVec::splat(Vec4::zero(), self.segment_cache.len());
        let mut normal_ends = IdxVec::splat(Vec4::zero(), self.segment_cache.len());

        for ((i, j), segment) in &self.segment_cache {
            normal_starts[*segment] = self.normals[*i];
            normal_ends[*segment] = self.normals[*j];
        }

        // Buffer data
        let element_buffer =
            ctx.mk_element_buffer(&self.elements, gl::buffer::ElementKind::Triangles)?;
        let wireframe_element_buffer =
            ctx.mk_element_buffer(&self.wireframe_elements, gl::buffer::ElementKind::Lines)?;
        let vertex_start_buffer = ctx.mk_buffer(&self.segment_starts.into_raw())?;
        let vertex_end_buffer = ctx.mk_buffer(&self.segment_ends.into_raw())?;
        let wireframe_vertex_buffer = ctx.mk_buffer(&self.wireframe_vertices)?;
        let normal_start_buffer = ctx.mk_buffer(&normal_starts.into_raw())?;
        let normal_end_buffer = ctx.mk_buffer(&normal_ends.into_raw())?;

        Ok(CubeMeshBuffers {
            element_buffer,
            wireframe_element_buffer,
            vertex_start_buffer,
            vertex_end_buffer,
            wireframe_vertex_buffer,
            normal_start_buffer,
            normal_end_buffer,
        })
    }
}

impl CubeMesh {
    pub fn mk_cube(&mut self, cube: CubeData) -> Cube {
        self.elements.push(cube)
    }

    pub fn buffer(&self, ctx: &gl::GlCtx) -> gl::Result<CubeMeshBuffers> {
        let mut bufferer = CubeMeshBufferer::new(self);
        bufferer.triangulate();
        bufferer.extract_buffers(ctx)
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

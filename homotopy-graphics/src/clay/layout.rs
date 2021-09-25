use std::{cmp, collections::HashMap, convert::TryInto};

use homotopy_common::{declare_idx, graph::Edge};
use homotopy_core::{
    common::DimensionError,
    cubicalisation::{Bias, CubicalGraph},
    Diagram, DiagramN, Direction, Generator, SliceIndex,
};
use ultraviolet::Vec4;

use super::geom::{Boundary, Mesh, Vert, VertExt};

impl Boundary {
    /// Calculate the boundary of a given location in a diagram.
    fn at_coord(diagram: &Diagram, mut coord: &[SliceIndex]) -> Self {
        let mut diagram = diagram.clone();
        let mut boundary = Self::Zero;

        loop {
            match coord {
                [] | [_] => return boundary,
                [index, tail @ ..] => {
                    let d: DiagramN = diagram.try_into().unwrap();
                    diagram = d.slice(*index).unwrap();
                    coord = tail;
                    match index {
                        SliceIndex::Boundary(_) => {}
                        SliceIndex::Interior(_) => boundary.inc(),
                    }
                }
            }
        }
    }
}

declare_idx! {
    struct CoordIdx = usize;
}

pub struct MeshExtractor {
    graph: CubicalGraph,
    coords: HashMap<Vec<SliceIndex>, Vert>,
    mesh: Mesh,
}

impl MeshExtractor {
    pub fn new(diagram: &DiagramN, cubicalisation_depth: u8) -> Result<Self, DimensionError> {
        let diagram = Diagram::from(diagram.clone());
        let graph = diagram.clone().cubicalise(&[Bias::Left].repeat(cmp::min(
            cubicalisation_depth as usize,
            diagram.dimension().saturating_sub(1),
        )))?;

        let mut mesh = Mesh::new(diagram.clone());
        let mut coords = HashMap::new();
        let mut valence = HashMap::new();

        for (node, data) in graph.inner().nodes() {
            let label = graph.label(node);
            let coord = {
                let dimension = label.len();
                let mk_coord = |i: usize| {
                    if i >= dimension {
                        0.
                    } else {
                        let j = dimension - i - 1;
                        data.0[j].to_int(graph.size(j)) as f32
                    }
                };

                Vec4::new(mk_coord(0), mk_coord(1), mk_coord(2), mk_coord(3))
            };
            let vert = coords.get(label).copied().unwrap_or_else(|| {
                let boundary = Boundary::at_coord(&diagram, graph.label(node));
                let generator = data.1.max_generator();
                let vert = mesh.mk(Vec4::zero().with_boundary_and_generator(boundary, generator));

                coords.insert(label.to_owned(), vert);
                vert
            });

            *mesh.verts[vert] += coord;
            *valence.entry(vert).or_insert(0) += 1;
        }

        for (vert, data) in mesh.verts.iter_mut() {
            **data /= valence[&vert] as f32;
        }

        Ok(Self {
            graph,
            coords,
            mesh,
        })
    }

    #[inline]
    fn source_of(&self, edge: Edge) -> Vert {
        let node = self.graph.inner().source(edge);
        self.coords[self.graph.label(node)]
    }

    #[inline]
    fn target_of(&self, edge: Edge) -> Vert {
        let node = self.graph.inner().target(edge);
        self.coords[self.graph.label(node)]
    }

    #[inline]
    fn minimum_generator(generators: impl Iterator<Item = Generator>) -> Generator {
        generators.min_by_key(|g| g.dimension).unwrap()
    }

    #[inline]
    fn codimension_visible(&self, generator: Generator, threshold: usize) -> bool {
        let codimension = self
            .mesh
            .diagram
            .dimension()
            .saturating_sub(generator.dimension);
        codimension == threshold
    }

    #[inline]
    fn has_n_volume(verts: &[Vert], n: usize) -> bool {
        let mut verts = verts.to_vec();
        verts.dedup();
        verts.len() > n
    }

    #[inline]
    pub fn extract_cubes(mut self) -> Self {
        for cube in self.graph.cubes() {
            let bl = cube.bottom_left;
            let br = cube.bottom_right;
            let tl = cube.top_left;
            let tr = cube.top_right;

            let mut verts = [
                self.source_of(bl),
                self.source_of(br),
                self.target_of(bl),
                self.target_of(br),
                self.source_of(tl),
                self.source_of(tr),
                self.target_of(tl),
                self.target_of(tr),
            ];
            let generator =
                Self::minimum_generator(verts.iter().map(|v| self.mesh.verts[*v].generator));

            if !self.codimension_visible(generator, 3) || !Self::has_n_volume(&verts, 3) {
                continue;
            }

            if self.graph.get_direction(bl) == Direction::Backward {
                verts.swap(0, 1);
                verts.swap(2, 3);
                verts.swap(4, 5);
                verts.swap(6, 7);
            }

            if self.graph.get_direction(tl) == Direction::Backward {
                verts.swap(0, 2);
                verts.swap(1, 3);
                verts.swap(4, 6);
                verts.swap(5, 7);
            }

            if self.graph.get_direction(cube.left_front) == Direction::Backward {
                verts.swap(0, 4);
                verts.swap(1, 5);
                verts.swap(2, 6);
                verts.swap(3, 7);
            }

            self.mesh.mk(verts);
        }

        self
    }

    #[inline]
    pub fn extract_squares(mut self) -> Self {
        for square in self.graph.squares() {
            let b = square.bottom;
            let t = square.top;

            let mut verts = [
                self.source_of(b),
                self.target_of(b),
                self.source_of(t),
                self.target_of(t),
            ];
            let generator =
                Self::minimum_generator(verts.iter().map(|v| self.mesh.verts[*v].generator));

            if !self.codimension_visible(generator, 2) || !Self::has_n_volume(&verts, 2) {
                continue;
            }

            if self.graph.get_direction(b) == Direction::Backward {
                verts.swap(0, 1);
                verts.swap(2, 3);
            }

            if self.graph.get_direction(square.left) == Direction::Backward {
                verts.swap(0, 2);
                verts.swap(1, 3);
            }

            self.mesh.mk(verts);
        }

        self
    }

    #[inline]
    pub fn extract_lines(mut self) -> Self {
        for edge in self.graph.inner().edge_keys() {
            let mut verts = [self.source_of(edge), self.target_of(edge)];
            let generator =
                Self::minimum_generator(verts.iter().map(|v| self.mesh.verts[*v].generator));

            if verts[0] == verts[1] || !self.codimension_visible(generator, 1) {
                continue;
            }

            if self.graph.get_direction(edge) == Direction::Backward {
                verts.swap(0, 1);
            }

            self.mesh.mk(verts);
        }

        self
    }

    #[inline]
    pub fn extract_points(mut self) -> Self {
        for node in self.graph.inner().node_keys() {
            let vert = self.coords[self.graph.label(node)];

            if !self.codimension_visible(self.mesh.verts[vert].generator, 0) {
                continue;
            }

            self.mesh.mk(vert);
        }

        self
    }

    #[inline]
    pub fn build(mut self) -> Mesh {
        let (min, max) = self.mesh.bounds();
        let translation = 0.5 * (max + min);
        let duration = 0.5 * (max.w - min.w);

        for vert in self.mesh.verts.values_mut() {
            **vert -= translation;
            vert.w /= duration;
        }

        self.mesh
    }
}

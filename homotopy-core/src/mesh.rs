use std::{cmp::Ordering, ops::Index};

use homotopy_common::{declare_idx, hash::FastHashMap, idx::IdxVec};
use itertools::{interleave, Itertools};
use petgraph::graph::NodeIndex;

use crate::{
    common::DimensionError,
    graph::{Explodable, ExternalRewrite, SliceGraph},
    Boundary, Diagram, DiagramN, Direction, Height, SliceIndex,
};

declare_idx! {
    pub struct Element = usize;
}

type Orientation = u8;

#[derive(Copy, Clone, Debug)]
enum ElementData {
    Element0(NodeIndex),
    ElementN(CubeInternal),
}

use ElementData::{Element0, ElementN};

#[derive(Copy, Clone, Debug)]
struct CubeInternal {
    faces: [Element; 2],
    partial: bool,
    direction: Direction,
    orientation: Orientation,
}

impl Index<usize> for CubeInternal {
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        &self.faces[index]
    }
}

pub type Mesh2D = Mesh<2>;

#[derive(Clone, Debug)]
pub struct Mesh<const N: usize> {
    graph: SliceGraph<[SliceIndex; N], bool>,
    elements: IdxVec<Element, ElementData>,
}

impl<const N: usize> Mesh<N> {
    /// Constructs the mesh of depth `N` for the given diagram.
    pub fn new(diagram: &DiagramN) -> Result<Self, DimensionError> {
        if diagram.dimension() < N {
            return Err(DimensionError);
        }

        let mut mesh = Self {
            graph: SliceGraph::singleton([Boundary::Source.into(); N], diagram.clone()),
            elements: IdxVec::splat(Element0(NodeIndex::new(0)), 1),
        };

        for i in 0..N {
            mesh = mesh.explode(i)?;
        }

        Ok(mesh)
    }

    /// Iterator of all nodes in the mesh.
    pub fn nodes(&self) -> impl Iterator<Item = ([SliceIndex; N], &Diagram)> {
        self.graph
            .node_weights()
            .map(|(coord, diagram)| (*coord, diagram))
    }

    /// Iterator of all non-partial visible elements in the mesh.
    pub fn elements(&self, directed: bool) -> impl Iterator<Item = Vec<[SliceIndex; N]>> + '_ {
        self.elements.keys().filter_map(move |elem| {
            if self.is_partial(elem).unwrap_or_default() {
                return None;
            }

            let orientation = self.orientation_of(elem);
            let n = orientation.len();

            Some(self.flatten(elem, directed, &orientation)).filter(|points| {
                // Check that the element is visible by looking at the coordinates.
                points.iter().all(|coord| {
                    coord[n..]
                        .iter()
                        .all(|si| matches!(si, SliceIndex::Interior(Height::Singular(_))))
                })
            })
        })
    }

    fn explode(&self, index: usize) -> Result<Self, DimensionError> {
        let explosion = self.graph.explode(
            |_, coord, si| {
                let mut coord = *coord;
                coord[index] = si;
                Some(coord)
            },
            |_, _, _| Some(false),
            |_, partial, r| Some(*partial || r == ExternalRewrite::Flange),
        )?;

        let mut mesh = Self {
            graph: explosion.output,
            elements: IdxVec::default(),
        };

        // Records if there is an element with given faces and whether it is partial or not.
        let mut memory: FastHashMap<[Element; 2], bool> = FastHashMap::default();

        // Maps every element in the original mesh to its corresponding elements in the exploded mesh.
        let mut element_to_elements: IdxVec<Element, Vec<Element>> = IdxVec::default();

        for &element in self.elements.values() {
            let mut elements = vec![];
            match element {
                Element0(n) => {
                    let nodes = &explosion.node_to_nodes[n];
                    let size = nodes.len();
                    for m in nodes {
                        elements.push(mesh.elements.push(Element0(*m)));
                    }
                    for i in 0..size - 1 {
                        let mut faces = [elements[i], elements[i + 1]];
                        let direction = if i == 0 || i < size - 2 && i % 2 == 1 {
                            Direction::Forward
                        } else {
                            faces.swap(0, 1);
                            Direction::Backward
                        };
                        elements.push(mesh.elements.push(ElementN(CubeInternal {
                            faces,
                            direction,
                            partial: false,
                            orientation: index as u8,
                        })));
                    }
                }
                ElementN(cube) => {
                    for &e_0 in &element_to_elements[cube[0]] {
                        for &e_1 in &element_to_elements[cube[1]] {
                            if let Some(partial) = mesh.mk_element(e_0, e_1, &memory) {
                                memory.insert([e_0, e_1], partial);
                                elements.push(mesh.elements.push(ElementN(CubeInternal {
                                    partial,
                                    faces: [e_0, e_1],
                                    direction: cube.direction,
                                    orientation: cube.orientation,
                                })));
                            }
                        }
                    }
                }
            }

            element_to_elements.push(elements);
        }

        Ok(mesh)
    }

    fn mk_element(
        &self,
        e_0: Element,
        e_1: Element,
        memory: &FastHashMap<[Element; 2], bool>,
    ) -> Option<bool> {
        assert_ne!(e_0, e_1);

        let elem_0 = self.elements[e_0];
        let elem_1 = self.elements[e_1];

        let [e_00, e_01, e_10, e_11] = match (elem_0, elem_1) {
            (Element0(a), Element0(b)) => {
                let e = self.graph.find_edge(a, b)?;
                return Some(self.graph[e].0);
            }
            (Element0(_), ElementN(cube_1)) => [e_0, e_0, cube_1[0], cube_1[1]],
            (ElementN(cube_0), Element0(_)) => [cube_0[0], cube_0[1], e_1, e_1],
            (ElementN(cube_0), ElementN(cube_1)) => {
                match cube_0.orientation.cmp(&cube_1.orientation) {
                    Ordering::Less => [cube_0[0], cube_0[1], e_1, e_1],
                    Ordering::Equal => [cube_0[0], cube_0[1], cube_1[0], cube_1[1]],
                    Ordering::Greater => [e_0, e_0, cube_1[0], cube_1[1]],
                }
            }
        };

        Some(
            memory.get(&[e_00, e_10]).copied()?
                || memory.get(&[e_01, e_11]).copied()?
                || self.is_partial(e_0).unwrap_or(true) && self.is_partial(e_1).unwrap_or(true),
        )
    }

    fn is_partial(&self, elem: Element) -> Option<bool> {
        match self.elements[elem] {
            Element0(_) => None,
            ElementN(cube) => Some(cube.partial),
        }
    }

    fn orientation_of(&self, elem: Element) -> Vec<Orientation> {
        match self.elements[elem] {
            Element0(_) => vec![],
            ElementN(cube) => std::iter::once(cube.orientation)
                .chain(self.orientation_of(cube[0]))
                .chain(self.orientation_of(cube[1]))
                .sorted()
                .dedup()
                .collect_vec(),
        }
    }

    fn flatten(
        &self,
        elem: Element,
        directed: bool,
        orientation: &[Orientation],
    ) -> Vec<[SliceIndex; N]> {
        let dim = orientation.len();
        match self.elements[elem] {
            Element0(n) => {
                vec![self.graph[n].0; 2_usize.pow(dim as u32)]
            }
            ElementN(cube) => {
                let mut orientation = orientation.to_owned();
                let index = orientation
                    .iter()
                    .position(|i| *i == cube.orientation)
                    .unwrap();
                orientation.remove(index);

                let mut cube_0 = self.flatten(cube[0], directed, &orientation);
                let mut cube_1 = self.flatten(cube[1], directed, &orientation);

                if !directed && cube.direction == Direction::Backward {
                    std::mem::swap(&mut cube_0, &mut cube_1);
                }

                let chunk_size = 2_usize.pow((dim - index - 1) as u32);

                interleave(cube_0.chunks(chunk_size), cube_1.chunks(chunk_size))
                    .flatten()
                    .copied()
                    .collect_vec()
            }
        }
    }
}

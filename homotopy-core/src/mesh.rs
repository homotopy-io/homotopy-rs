use std::{cmp::Ordering, ops::Index};

use homotopy_common::{declare_idx, idx::IdxVec};
use itertools::{interleave, Itertools};
use petgraph::graph::NodeIndex;

use crate::{
    common::DimensionError,
    graph::{Explodable, ExternalRewrite, SliceGraph},
    DiagramN, Direction, SliceIndex,
};

declare_idx! {
    pub struct Element = usize;
}

type Orientation = usize;

#[derive(Copy, Clone, Debug)]
pub enum ElementData {
    Element0(NodeIndex),
    ElementN(CubeInternal),
}

#[derive(Copy, Clone, Debug)]
pub struct CubeInternal {
    pub faces: [Element; 2],
    pub partial: bool,
    pub direction: Direction,
    pub orientation: Orientation,
}

impl Index<usize> for CubeInternal {
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        &self.faces[index]
    }
}

pub struct Mesh {
    pub graph: SliceGraph<Vec<SliceIndex>, bool>,
    pub elements: IdxVec<Element, ElementData>,
}

impl Mesh {
    pub fn new(diagram: &DiagramN, depth: usize) -> Result<Self, DimensionError> {
        if depth > diagram.dimension() {
            return Err(DimensionError);
        }

        let mut mesh = Self {
            graph: SliceGraph::singleton(vec![], diagram.clone()),
            elements: IdxVec::from_iter([ElementData::Element0(NodeIndex::new(0))]),
        };

        for orientation in 0..depth {
            mesh = mesh.explode(orientation)?;
        }

        Ok(mesh)
    }

    fn explode(&self, orientation: usize) -> Result<Self, DimensionError> {
        use ElementData::{Element0, ElementN};

        let explosion = self.graph.explode(
            |_, coord, si| {
                let mut coord = coord.clone();
                coord.push(si);
                Some(coord)
            },
            |_, _, _| Some(false),
            |_, partial, r| Some(*partial || r == ExternalRewrite::Flange),
        )?;

        let mut mesh = Self {
            graph: explosion.output,
            elements: IdxVec::default(),
        };

        // Maps every element in the original mesh to its corresponding elements in the exploded mesh.
        let mut element_to_elements: IdxVec<Element, Vec<Element>> = IdxVec::default();

        for &element in self.elements.values() {
            let mut children = vec![];
            match element {
                Element0(n) => {
                    let nodes = &explosion.node_to_nodes[n];
                    let size = nodes.len();
                    for m in nodes {
                        children.push(mesh.elements.push(Element0(*m)));
                    }
                    for i in 0..size - 1 {
                        let mut faces = [children[i], children[i + 1]];
                        let direction = if i == 0 || i < size - 2 && i % 2 == 1 {
                            Direction::Forward
                        } else {
                            faces.swap(0, 1);
                            Direction::Backward
                        };
                        children.push(mesh.elements.push(ElementN(CubeInternal {
                            faces,
                            direction,
                            orientation,
                            partial: false,
                        })));
                    }
                }
                ElementN(cube) => {
                    for &e_0 in &element_to_elements[cube[0]] {
                        for &e_1 in &element_to_elements[cube[1]] {
                            if let Some(partial) = mesh.mk_element(e_0, e_1) {
                                children.push(mesh.elements.push(ElementN(CubeInternal {
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

            element_to_elements.push(children);
        }

        Ok(mesh)
    }

    fn mk_element(&self, e_0: Element, e_1: Element) -> Option<bool> {
        use ElementData::{Element0, ElementN};

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

        let partial_0 = self.mk_element(e_00, e_10)?;
        let partial_1 = self.mk_element(e_01, e_11)?;
        Some(
            partial_0
                || partial_1
                || self.is_partial(e_0) && self.is_partial(e_1)
                || self.is_partial(e_0) && matches!(elem_1, Element0(_))
                || self.is_partial(e_1) && matches!(elem_0, Element0(_)),
        )
    }

    fn is_partial(&self, e: Element) -> bool {
        match self.elements[e] {
            ElementData::Element0(_) => false,
            ElementData::ElementN(cube) => cube.partial,
        }
    }

    fn orientation_of(&self, elem: Element) -> Vec<Orientation> {
        match self.elements[elem] {
            ElementData::Element0(_) => vec![],
            ElementData::ElementN(cube) => std::iter::once(cube.orientation)
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
        orientation: &[Orientation],
        directed: bool,
    ) -> Vec<NodeIndex> {
        match self.elements[elem] {
            ElementData::Element0(n) => vec![n; 2_usize.pow(orientation.len() as u32)],
            ElementData::ElementN(cube) => {
                let mut orientation = orientation.to_owned();
                let index = orientation
                    .iter()
                    .position(|&j| j == cube.orientation)
                    .unwrap();
                orientation.remove(index);

                let mut cube_0 = self.flatten(cube[0], &orientation, directed);
                let mut cube_1 = self.flatten(cube[1], &orientation, directed);

                if !directed && cube.direction == Direction::Backward {
                    std::mem::swap(&mut cube_0, &mut cube_1);
                }

                let chunk_size = 2_usize.pow((orientation.len() - index) as u32);

                interleave(cube_0.chunks(chunk_size), cube_1.chunks(chunk_size))
                    .flatten()
                    .copied()
                    .collect_vec()
            }
        }
    }

    /// Returns all non-partial visible elements of the mesh.
    pub fn elements(&self, directed: bool) -> impl Iterator<Item = Vec<NodeIndex>> + '_ {
        use crate::{Height::Singular, SliceIndex::Interior};

        self.elements.keys().filter_map(move |e| {
            if self.is_partial(e) {
                None
            } else {
                let orientation = self.orientation_of(e);
                let dim = orientation.len();

                Some(self.flatten(e, &orientation, directed)).filter(|nodes| {
                    // Check that the element is visible.
                    nodes.iter().all(|n| {
                        let coord = &self.graph[*n].0;
                        coord[dim..]
                            .iter()
                            .all(|si| matches!(si, Interior(Singular(_))))
                    })
                })
            }
        })
    }
}

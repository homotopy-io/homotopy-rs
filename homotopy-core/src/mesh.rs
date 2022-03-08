use std::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

use homotopy_common::{declare_idx, idx::IdxVec};
use itertools::{interleave, Itertools};
use petgraph::graph::NodeIndex;

use crate::{
    common::DimensionError,
    graph::{Explodable, ExternalRewrite, SliceGraph},
    DiagramN, SliceIndex,
};

declare_idx! {
    pub struct Element = usize;
}

pub type Orientation = usize;

#[derive(Copy, Clone, Debug)]
pub enum ElementData {
    Element0(NodeIndex),
    ElementN(CubeInternal),
}

#[derive(Copy, Clone, Debug)]
pub struct CubeInternal {
    faces: [Element; 2],
    is_partial: bool,
    orientation: Orientation,
}

impl Deref for CubeInternal {
    type Target = [Element; 2];

    fn deref(&self) -> &Self::Target {
        &self.faces
    }
}

impl DerefMut for CubeInternal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.faces
    }
}

impl ElementData {
    fn is_partial(&self) -> bool {
        match self {
            Self::Element0(_) => false,
            Self::ElementN(cube) => cube.is_partial,
        }
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

        let mut graph = SliceGraph::singleton(vec![], diagram.clone());
        let mut elements = IdxVec::from_iter([ElementData::Element0(NodeIndex::new(0))]);

        for orientation in 0..depth {
            let explosion = graph.explode(
                |_, key, si| {
                    let mut key = key.clone();
                    key.push(si);
                    Some(key)
                },
                |_, _, _| Some(false),
                |_, &is_partial, ro| Some(is_partial || ro == ExternalRewrite::Flange),
            )?;
            graph = explosion.output;
            elements = explode_elements(&elements, &graph, &explosion.node_to_nodes, orientation);
        }

        Ok(Self { graph, elements })
    }
}

fn explode_elements(
    elements: &IdxVec<Element, ElementData>,
    graph: &SliceGraph<Vec<SliceIndex>, bool>,
    node_to_nodes: &IdxVec<NodeIndex, Vec<NodeIndex>>,
    orientation: Orientation,
) -> IdxVec<Element, ElementData> {
    use ElementData::{Element0, ElementN};
    let mut exploded_elements: IdxVec<Element, ElementData> = IdxVec::new();

    // Maps every element in the original mesh to its corresponding elements in the exploded mesh.
    let mut element_to_children: IdxVec<Element, Vec<Element>> = IdxVec::new();

    for &element in elements.values() {
        let mut children = vec![];
        match element {
            Element0(n) => {
                let size = node_to_nodes[n].len();
                for i in 0..size {
                    children.push(exploded_elements.push(Element0(node_to_nodes[n][i])));
                }
                for i in 0..size - 1 {
                    children.push(exploded_elements.push(ElementN(CubeInternal {
                        orientation,
                        faces: [children[i], children[i + 1]],
                        is_partial: false,
                    })));
                }
            }
            ElementN(cube) => {
                for &elem_0 in &element_to_children[cube[0]] {
                    for &elem_1 in &element_to_children[cube[1]] {
                        if let Some(ed) = make_element(
                            cube.orientation,
                            elem_0,
                            elem_1,
                            graph,
                            &exploded_elements,
                        ) {
                            children.push(exploded_elements.push(ed));
                        }
                    }
                }
            }
        }

        element_to_children.push(children);
    }

    exploded_elements
}

// TODO(@calintat): We can cache the results instead of recursing.
fn make_element(
    orientation: Orientation,
    elem_0: Element,
    elem_1: Element,
    graph: &SliceGraph<Vec<SliceIndex>, bool>,
    elements: &IdxVec<Element, ElementData>,
) -> Option<ElementData> {
    use ElementData::{Element0, ElementN};

    let [elem_00, elem_01, elem_10, elem_11] = match (elements[elem_0], elements[elem_1]) {
        (Element0(s), Element0(t)) => {
            if s == t {
                return Some(Element0(s));
            }
            return Some(ElementN(CubeInternal {
                orientation,
                faces: [elem_0, elem_1],
                is_partial: graph[graph.find_edge_undirected(s, t)?.0].0,
            }));
        }
        (Element0(_), ElementN(cube_1)) => [elem_0, elem_0, cube_1[0], cube_1[1]],
        (ElementN(cube_0), Element0(_)) => [cube_0[0], cube_0[1], elem_1, elem_1],
        (ElementN(cube_0), ElementN(cube_1)) => match cube_0.orientation.cmp(&cube_1.orientation) {
            Ordering::Less => [cube_0[0], cube_0[1], elem_1, elem_1],
            Ordering::Equal => [cube_0[0], cube_0[1], cube_1[0], cube_1[1]],
            Ordering::Greater => [elem_0, elem_0, cube_1[0], cube_1[1]],
        },
    };

    if let Some(cube_0) = make_element(orientation, elem_00, elem_10, graph, elements) {
        if let Some(cube_1) = make_element(orientation, elem_01, elem_11, graph, elements) {
            return Some(ElementN(CubeInternal {
                orientation,
                faces: [elem_0, elem_1],
                is_partial: cube_0.is_partial()
                    || cube_1.is_partial()
                    || (elements[elem_0].is_partial() && elements[elem_1].is_partial())
                    || (elements[elem_0].is_partial()
                        && matches!(elements[elem_1], ElementData::Element0(_)))
                    || (elements[elem_1].is_partial()
                        && matches!(elements[elem_0], ElementData::Element0(_))),
            }));
        }
    }

    None
}

impl Mesh {
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

    fn flatten(&self, elem: Element, orientation: &[Orientation]) -> Vec<NodeIndex> {
        match self.elements[elem] {
            ElementData::Element0(n) => vec![n; 2_usize.pow(orientation.len() as u32)],
            ElementData::ElementN(cube) => {
                let mut orientation = orientation.to_owned();
                let index = orientation
                    .iter()
                    .position(|&j| j == cube.orientation)
                    .unwrap();
                orientation.remove(index);

                let cube_0 = self.flatten(cube[0], &orientation);
                let cube_1 = self.flatten(cube[1], &orientation);

                let chunk_size = 2_usize.pow((orientation.len() - index) as u32);

                interleave(cube_0.chunks(chunk_size), cube_1.chunks(chunk_size))
                    .flatten()
                    .copied()
                    .collect_vec()
            }
        }
    }

    /// Returns all non-partial visible elements of the mesh.
    pub fn elements(&self) -> impl Iterator<Item = Vec<NodeIndex>> + '_ {
        use crate::{Height::Singular, SliceIndex::Interior};

        self.elements.iter().filter_map(|(e, elem)| {
            if elem.is_partial() {
                None
            } else {
                let orientation = self.orientation_of(e);
                let dim = orientation.len();

                Some(self.flatten(e, &orientation)).filter(|nodes| {
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

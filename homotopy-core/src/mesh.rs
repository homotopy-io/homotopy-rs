use homotopy_common::{declare_idx, idx::IdxVec};
use itertools::{interleave, Itertools};
use petgraph::graph::NodeIndex;

use crate::{
    common::DimensionError,
    graph::{Explodable, RewriteOrigin, SliceGraph},
    DiagramN, SliceIndex,
};

declare_idx! {
    pub struct Element = usize;
}

pub type Orientation = usize;

#[derive(Copy, Clone, Debug)]
pub enum ElementData {
    Element0(NodeIndex),
    ElementN(Orientation, Element, Element),
}

pub struct Mesh {
    pub depth: usize,
    pub graph: SliceGraph<Vec<SliceIndex>>,
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
                |_, _, _| Some(()),
                |_, _, ro| (ro != RewriteOrigin::UnitSlice).then(|| ()),
            )?;
            graph = explosion.output;
            elements = explode_elements(&elements, &graph, &explosion.node_to_nodes, orientation);
        }

        Ok(Self {
            depth,
            graph,
            elements,
        })
    }
}

fn explode_elements(
    elements: &IdxVec<Element, ElementData>,
    graph: &SliceGraph<Vec<SliceIndex>>,
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
                    children.push(exploded_elements.push(ElementN(
                        orientation,
                        children[i],
                        children[i + 1],
                    )));
                }
            }
            ElementN(i, source_elem, target_elem) => {
                for &elem_0 in &element_to_children[source_elem] {
                    for &elem_1 in &element_to_children[target_elem] {
                        if let Some(ed) = make_element(i, elem_0, elem_1, graph, &exploded_elements)
                        {
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
    graph: &SliceGraph<Vec<SliceIndex>>,
    elements: &IdxVec<Element, ElementData>,
) -> Option<ElementData> {
    use ElementData::{Element0, ElementN};

    let [elem_00, elem_01, elem_10, elem_11] = match (elements[elem_0], elements[elem_1]) {
        (Element0(s), Element0(t)) => {
            if s == t {
                return Some(Element0(s));
            }
            if graph.find_edge(s, t).is_some() || graph.find_edge(t, s).is_some() {
                return Some(ElementN(orientation, elem_0, elem_1));
            }
            return None;
        }
        (Element0(_), ElementN(_, elem_10, elem_11)) => [elem_0, elem_0, elem_10, elem_11],
        (ElementN(_, elem_00, elem_01), Element0(_)) => [elem_00, elem_01, elem_1, elem_1],
        (ElementN(i, elem_00, elem_01), ElementN(j, _, _)) if i < j => {
            [elem_00, elem_01, elem_1, elem_1]
        }
        (ElementN(i, _, _), ElementN(j, elem_10, elem_11)) if i > j => {
            [elem_0, elem_0, elem_10, elem_11]
        }
        (ElementN(_, elem_00, elem_01), ElementN(_, elem_10, elem_11)) => {
            [elem_00, elem_01, elem_10, elem_11]
        }
    };

    if make_element(orientation, elem_00, elem_10, graph, elements).is_some()
        && make_element(orientation, elem_01, elem_11, graph, elements).is_some()
    {
        return Some(ElementN(orientation, elem_0, elem_1));
    }

    None
}

impl Mesh {
    fn orientation_of(&self, elem: Element) -> Vec<Orientation> {
        match self.elements[elem] {
            ElementData::Element0(_) => vec![],
            ElementData::ElementN(i, elem_0, elem_1) => std::iter::once(i)
                .chain(self.orientation_of(elem_0))
                .chain(self.orientation_of(elem_1))
                .sorted()
                .dedup()
                .collect_vec(),
        }
    }

    fn flatten(&self, elem: Element, orientation: &[Orientation]) -> Vec<NodeIndex> {
        match self.elements[elem] {
            ElementData::Element0(n) => vec![n; 2_usize.pow(orientation.len() as u32)],
            ElementData::ElementN(i, elem_0, elem_1) => {
                let mut orientation = orientation.to_owned();
                let index = orientation.iter().position(|&j| j == i).unwrap();
                orientation.remove(index);

                let cube_0 = self.flatten(elem_0, &orientation);
                let cube_1 = self.flatten(elem_1, &orientation);

                let chunk_size = 2_usize.pow((orientation.len() - index) as u32);

                interleave(cube_0.chunks(chunk_size), cube_1.chunks(chunk_size))
                    .flatten()
                    .copied()
                    .collect_vec()
            }
        }
    }

    pub fn flatten_elements(&self) -> impl Iterator<Item = Vec<NodeIndex>> + '_ {
        self.elements
            .keys()
            .map(|elem| self.flatten(elem, &self.orientation_of(elem)))
    }
}

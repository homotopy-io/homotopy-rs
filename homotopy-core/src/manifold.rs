use std::collections::BTreeSet;

use homotopy_common::idx::IdxVec;
use im::HashSet;
use itertools::Itertools;
use petgraph::{graph::NodeIndex, visit::EdgeRef};

use crate::{common::Height, diagram::Diagram, scaffold::Scaffold};

type SimplexVec = Vec<NodeIndex>; // An n-simplex is a list of n + 1 vertices.
type Simplex = BTreeSet<NodeIndex>;

#[derive(Debug)]
struct Complex {
    facets: HashSet<Simplex>,
}

impl Complex {
    fn from_faces(faces: HashSet<Simplex>) -> Self {
        let mut facets: HashSet<Simplex> = HashSet::new();
        for face in faces {
            let mut maximal = true;
            for facet in facets.clone() {
                if facet.is_superset(&face) {
                    maximal = false;
                    break;
                }
                if face.is_superset(&facet) {
                    facets.remove(&facet);
                }
            }
            if maximal {
                facets.insert(face);
            }
        }
        Self { facets }
    }

    fn vertices(&self) -> Simplex {
        self.facets.iter().flatten().unique().copied().collect()
    }

    fn count_cofaces(&self, face: &Simplex) -> usize {
        let mut count = 0;
        for facet in &self.facets {
            if has_coface(face, facet) {
                count += 1;
            }
        }
        count
    }

    fn edges(&self) -> HashSet<Simplex> {
        let mut result = HashSet::new();
        for facet in &self.facets {
            for u in facet {
                for v in facet {
                    if u != v {
                        result.insert(BTreeSet::from([*u, *v]));
                    }
                }
            }
        }
        result
    }

    fn twofacets(&self) -> HashSet<Simplex> {
        let mut result = HashSet::new();
        for facet in &self.facets {
            result.extend(simplex_boundary(facet));
        }
        result
    }
}

fn stratum(coord: &[Height]) -> usize {
    coord.iter().fold(0, |stratum, h| {
        stratum
            + match h {
                Height::Regular(_) => 0,
                Height::Singular(_) => 1,
            }
    })
}

const fn is_visible(coord: &[Height]) -> bool {
    let n = coord.len();
    match coord[n - 1] {
        Height::Regular(_) => false,
        Height::Singular(_) => true,
    }
}

fn neighbourhoods(scaffold: &Scaffold<Vec<Height>>) -> IdxVec<NodeIndex, Vec<SimplexVec>> {
    let mut neighbourhoods: IdxVec<NodeIndex, Vec<SimplexVec>> =
        IdxVec::splat(vec![], scaffold.node_count());
    for n in scaffold
        .node_indices()
        .sorted_by_cached_key(|n| stratum(&scaffold[*n].key))
    {
        let mut neighbourhood = vec![vec![n]];
        for e in scaffold.edges_directed(n, petgraph::Direction::Incoming) {
            for simplex in &neighbourhoods[e.source()] {
                neighbourhood.push([simplex.as_slice(), &[n]].concat());
            }
        }
        neighbourhoods[n].extend(neighbourhood);
    }
    neighbourhoods
}

fn filter_nodes(simplices: &[Simplex], nodes: &[NodeIndex]) -> Vec<Simplex> {
    simplices
        .iter()
        .map(|simplex| {
            let mut res = simplex.clone();
            res.retain(|v| nodes.contains(v));
            res
        })
        .collect()
}

fn simplex_boundary(simplex: &Simplex) -> HashSet<Simplex> {
    let mut result = HashSet::new();
    for v in simplex {
        let mut copy = simplex.clone();
        copy.remove(v);
        result.insert(copy);
    }
    result
}

fn is_unit_sphere(complex: &Complex, dim: usize) -> bool {
    let vertices = complex.vertices();
    if complex.facets.len() != dim + 2 || vertices.len() != dim + 2 {
        return false;
    }
    simplex_boundary(&vertices) == complex.facets
}

fn has_coface(face1: &Simplex, face2: &Simplex) -> bool {
    face1.is_subset(face2) && face1 != face2
}

fn collapse_edge(edge: &Simplex, complex: &Complex) -> Complex {
    let mut result = HashSet::new();
    if let [u, v] = edge.iter().copied().collect::<Vec<NodeIndex>>()[..] {
        for face in &complex.facets {
            let mut newface = face.clone();
            if newface.remove(&v) {
                newface.insert(u);
            }
            result.insert(newface);
        }
    }
    Complex::from_faces(result)
}

#[allow(clippy::cognitive_complexity)]
fn is_sphere(complex: &Complex, dim: usize) -> bool {
    if is_unit_sphere(complex, dim) {
        return true;
    }

    for facet in &complex.facets {
        if facet.len() != dim + 1 {
            return false;
        }
    }

    for edge in complex.twofacets() {
        if complex.count_cofaces(&edge) != 2 {
            return false;
        }
    }

    for edge in complex.edges() {
        if is_sphere(&collapse_edge(&edge, complex), dim) {
            return true;
        }
    }
    false
}

#[must_use]
pub fn is_manifold(diagram: Diagram) -> bool {
    let dimension = diagram.dimension();
    assert!(dimension >= 2);

    let scaffold: Scaffold<Vec<Height>> = diagram.fully_explode();
    let max_stratum_node = scaffold
        .node_indices()
        .max_by_key(|n| stratum(&scaffold[*n].key));
    let max_stratum = stratum(&scaffold[max_stratum_node.unwrap()].key);
    let visible_nodes: Vec<NodeIndex> = scaffold
        .node_indices()
        .filter(|n| is_visible(&scaffold[*n].key))
        .collect();

    let neighbourhoods = neighbourhoods(&scaffold);

    for (n, neighbourhood) in neighbourhoods {
        if stratum(&scaffold[n].key) == max_stratum {
            let neighbourhood: Vec<Simplex> = neighbourhood
                .iter()
                .map(|t| t.iter().copied().collect())
                .collect();

            let mut visible_nodes = visible_nodes.clone();
            visible_nodes.retain(|&x| x != n);

            let visible_neighbourhood = filter_nodes(&neighbourhood, &visible_nodes);

            let boundary_complex = Complex::from_faces(HashSet::from(&visible_neighbourhood));

            if !is_sphere(&boundary_complex, dimension - 2) {
                return false;
            }
        }
    }
    true
}

use std::hash::Hash;

use crate::{common::SliceIndex, diagram::DiagramN, mesh::Mesh};

pub type Coordinate = [SliceIndex; 2];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Simplex {
    Surface([Coordinate; 3]),
    Wire([Coordinate; 2]),
    Point([Coordinate; 1]),
}

impl<'a> IntoIterator for &'a Simplex {
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, Coordinate>>;
    type Item = Coordinate;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Simplex::Surface(p) => p.iter().copied(),
            Simplex::Wire(p) => p.iter().copied(),
            Simplex::Point(p) => p.iter().copied(),
        }
    }
}

/// Generate a 2-dimensional simplicial complex for a diagram.
pub fn make_complex(diagram: &DiagramN) -> Vec<Simplex> {
    const TRI_ASSEMBLY_ORDER: [[usize; 3]; 2] = [[0, 1, 3], [0, 3, 2]];

    // Extract cubical mesh.
    let mesh = Mesh::new(diagram, true, 2).unwrap();

    let mut complex = vec![];
    for elem in mesh.elements() {
        let points: Vec<_> = elem
            .iter()
            .map(|n| {
                let coord = &mesh.graph[*n].0;
                [coord[0], coord[1]]
            })
            .collect();

        match points.len() {
            1 => {
                complex.push(Simplex::Point([points[0]]));
            }
            2 => {
                complex.push(Simplex::Wire([points[0], points[1]]));
            }
            4 => {
                complex.extend(TRI_ASSEMBLY_ORDER.into_iter().filter_map(|[i, j, k]| {
                    let tri @ [a, b, c] = [points[i], points[j], points[k]];
                    (a != b && a != c && b != c).then(|| Simplex::Surface(tri))
                }));
            }
            _ => (),
        }
    }
    complex
}

use std::hash::Hash;

use crate::{common::SliceIndex, mesh::Mesh, Diagram};

pub type Coordinate<const N: usize> = [SliceIndex; N];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Simplex<const N: usize> {
    Surface([Coordinate<N>; 3]),
    Wire([Coordinate<N>; 2]),
    Point([Coordinate<N>; 1]),
}

impl<'a, const N: usize> IntoIterator for &'a Simplex<N> {
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, Coordinate<N>>>;
    type Item = Coordinate<N>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Simplex::Surface(p) => p.iter().copied(),
            Simplex::Wire(p) => p.iter().copied(),
            Simplex::Point(p) => p.iter().copied(),
        }
    }
}

/// Generate a 2-dimensional simplicial complex for a diagram.
pub fn make_complex<const N: usize>(diagram: &Diagram) -> Vec<(Simplex<N>, bool)> {
    const TRI_ASSEMBLY_ORDER: [[usize; 3]; 2] = [[0, 1, 3], [0, 3, 2]];

    // Extract cubical mesh.
    let mesh = Mesh::new(diagram).unwrap();

    let mut complex = vec![];
    for cube in mesh.cubes() {
        match cube.dimension() {
            0 => {
                complex.push((Simplex::Point([cube[0]]), cube.visible));
            }
            1 => {
                complex.push((Simplex::Wire([cube[0], cube[1]]), cube.visible));
            }
            2 => {
                complex.extend(TRI_ASSEMBLY_ORDER.into_iter().filter_map(|[i, j, k]| {
                    let tri @ [a, b, c] = [cube[i], cube[j], cube[k]];
                    (a != b && a != c && b != c).then_some((Simplex::Surface(tri), cube.visible))
                }));
            }
            _ => (),
        }
    }
    complex
}

use crate::common::*;
use crate::diagram::DiagramN;
use crate::rewrite::RewriteN;
use std::convert::TryInto;
use std::hash::Hash;

pub type Coordinate = (SliceIndex, SliceIndex);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Simplex {
    Surface([Coordinate; 3]),
    Wire([Coordinate; 2]),
    Point([Coordinate; 1]),
}

/// TODO: Simplices on the boundary
/// TODO: Complexes in higher dimensions

/// Generate a 2-dimensional simplicial complex for a diagram.
pub fn make_complex(diagram: &DiagramN) -> Vec<Simplex> {
    use Height::*;
    let mut complex = Vec::new();

    let slices: Vec<DiagramN> = diagram
        .slices()
        .map(|slice| slice.try_into().unwrap())
        .collect();

    let cospans = diagram.cospans();

    for y in 0..diagram.size() {
        let slice = &slices[Singular(y).to_int()];
        let forward = cospans[y].forward.to_n().unwrap();
        let backward = cospans[y].backward.to_n().unwrap();

        let targets = {
            let mut targets = forward.targets();
            targets.extend(backward.targets());
            targets
        };

        for x in 0..slice.size() {
            generate_cell(x, y, y, forward, &mut complex);
            generate_cell(x, y, y + 1, backward, &mut complex);

            if targets.iter().any(|t| *t == x) {
                complex.push(Simplex::Point([(Singular(x).into(), Singular(y).into())]));
            }
        }
    }

    complex
}

fn generate_cell(sx: usize, sy: usize, ry: usize, rewrite: &RewriteN, complex: &mut Vec<Simplex>) {
    use Height::*;

    let rxs = rewrite.singular_preimage(sx);

    for rx in rxs.clone() {
        // Surface to the left of a wire
        complex.push(Simplex::Surface([
            (Regular(rx).into(), Regular(ry).into()),
            (Singular(rx).into(), Regular(ry).into()),
            (Singular(sx).into(), Singular(sy).into()),
        ]));

        // Surface to the right of a wire
        complex.push(Simplex::Surface([
            (Regular(rx + 1).into(), Regular(ry).into()),
            (Singular(rx).into(), Regular(ry).into()),
            (Singular(sx).into(), Singular(sy).into()),
        ]));

        // Wire
        complex.push(Simplex::Wire([
            (Singular(rx).into(), Regular(ry).into()),
            (Singular(sx).into(), Singular(sy).into()),
        ]));
    }

    // Surface to the left
    complex.push(Simplex::Surface([
        (Regular(rxs.start).into(), Regular(ry).into()),
        (Regular(sx).into(), Singular(sy).into()),
        (Singular(sx).into(), Singular(sy).into()),
    ]));

    // Surface to the right
    complex.push(Simplex::Surface([
        (Regular(rxs.end).into(), Regular(ry).into()),
        (Regular(sx + 1).into(), Singular(sy).into()),
        (Singular(sx).into(), Singular(sy).into()),
    ]));
}

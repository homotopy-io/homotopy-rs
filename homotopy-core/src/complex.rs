use std::{convert::TryInto, hash::Hash};

use crate::{
    common::{Boundary, Height, SliceIndex},
    diagram::DiagramN,
    rewrite::RewriteN,
};

pub type Coordinate = (SliceIndex, SliceIndex);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Simplex {
    Surface([Coordinate; 3]),
    Wire([Coordinate; 2]),
    Point([Coordinate; 1]),
}

impl Simplex {
    pub fn first(&self) -> Coordinate {
        match self {
            Self::Surface(p) => p[0],
            Self::Wire(p) => p[0],
            Self::Point(p) => p[0],
        }
    }
}

impl IntoIterator for Simplex {
    type IntoIter = std::vec::IntoIter<Coordinate>;
    type Item = Coordinate;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Surface(p) => p.to_vec().into_iter(),
            Self::Wire(p) => p.to_vec().into_iter(),
            Self::Point(p) => p.to_vec().into_iter(),
        }
    }
}

/// TODO: Clean this up. The boundary code is a bit verbose.
/// TODO: Complexes in higher dimensions

/// Generate a 2-dimensional simplicial complex for a diagram.
pub fn make_complex(diagram: &DiagramN) -> Vec<Simplex> {
    use Height::{Regular, Singular};
    let mut complex = Vec::new();

    let slices: Vec<DiagramN> = diagram
        .slices()
        .map(|slice| slice.try_into().unwrap())
        .collect();

    let cospans = diagram.cospans();

    // Interior
    for y in 0..diagram.size() {
        let forward: RewriteN = cospans[y].forward.clone().try_into().unwrap();
        let backward: RewriteN = cospans[y].backward.clone().try_into().unwrap();

        generate_rewrite(
            &slices[Regular(y).to_int()],
            &slices[Singular(y).to_int()],
            Regular(y).into(),
            Singular(y).into(),
            &forward,
            &mut complex,
        );

        generate_rewrite(
            &slices[Regular(y + 1).to_int()],
            &slices[Singular(y).to_int()],
            Regular(y + 1).into(),
            Singular(y).into(),
            &backward,
            &mut complex,
        );

        let targets = {
            let mut targets = forward.targets();
            targets.extend(backward.targets());
            targets.sort_unstable();
            targets.dedup();
            targets
        };

        for x in targets {
            complex.push(Simplex::Point([(Singular(x).into(), Singular(y).into())]));
        }
    }

    // Source boundary
    generate_rewrite(
        slices.first().unwrap(),
        slices.first().unwrap(),
        Regular(0).into(),
        Boundary::Source.into(),
        &RewriteN::identity(diagram.dimension() - 1),
        &mut complex,
    );

    // Target boundary
    generate_rewrite(
        slices.last().unwrap(),
        slices.last().unwrap(),
        Regular(diagram.size()).into(),
        Boundary::Target.into(),
        &RewriteN::identity(diagram.dimension() - 1),
        &mut complex,
    );

    complex
}

fn generate_cell(
    sx: usize,
    sy: SliceIndex,
    ry: SliceIndex,
    rewrite: &RewriteN,
    complex: &mut Vec<Simplex>,
) {
    use Height::{Regular, Singular};

    let rxs = rewrite.singular_preimage(sx);

    for rx in rxs.clone() {
        // Surface to the left of a wire
        complex.push(Simplex::Surface([
            (Regular(rx).into(), ry),
            (Singular(rx).into(), ry),
            (Singular(sx).into(), sy),
        ]));

        // Surface to the right of a wire
        complex.push(Simplex::Surface([
            (Regular(rx + 1).into(), ry),
            (Singular(rx).into(), ry),
            (Singular(sx).into(), sy),
        ]));

        // Wire
        complex.push(Simplex::Wire([
            (Singular(rx).into(), ry),
            (Singular(sx).into(), sy),
        ]));
    }

    // Surface to the left
    complex.push(Simplex::Surface([
        (Regular(rxs.start).into(), ry),
        (Regular(sx).into(), sy),
        (Singular(sx).into(), sy),
    ]));

    // Surface to the right
    complex.push(Simplex::Surface([
        (Regular(rxs.end).into(), ry),
        (Regular(sx + 1).into(), sy),
        (Singular(sx).into(), sy),
    ]));
}

fn generate_rewrite(
    rs: &DiagramN,
    ss: &DiagramN,
    ry: SliceIndex,
    sy: SliceIndex,
    rewrite: &RewriteN,
    complex: &mut Vec<Simplex>,
) {
    use Height::Regular;

    // Interior
    for x in 0..ss.size() {
        generate_cell(x, sy, ry, rewrite, complex);
    }

    // Left boundary
    generate_square(
        (Regular(0).into(), ry),
        (Boundary::Source.into(), ry),
        (Regular(0).into(), sy),
        (Boundary::Source.into(), sy),
        complex,
    );

    // Right boundary
    let end_regular = Regular(rs.size()).into();
    let end_singular = Regular(ss.size()).into();

    generate_square(
        (end_regular, ry),
        (Boundary::Target.into(), ry),
        (end_singular, sy),
        (Boundary::Target.into(), sy),
        complex,
    );
}

fn generate_square(
    a: Coordinate,
    b: Coordinate,
    c: Coordinate,
    d: Coordinate,
    complex: &mut Vec<Simplex>,
) {
    complex.push(Simplex::Surface([a, b, d]));
    complex.push(Simplex::Surface([a, c, d]));
}

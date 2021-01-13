//! Diagrams of higher dimensions can be projected into 2 dimensions to be presented in a user
//! interface. This module contains diagram analyses which that calculate various aspects about the
//! 2-dimensional projection of a diagram.
//!
//! In order to avoid potentially costly recomputations and accidental quadratic complexity when a
//! diagram is traversed again for every point, the analyses are performed for the entire diagram
//! at once and the results are cached for efficient random-access retrieval.
use crate::common::*;
use crate::diagram::DiagramN;
use crate::rewrite::RewriteN;
use serde::Serialize;
use std::collections::BTreeSet;
use std::convert::*;

/// Diagram analysis that determines the generator displayed at any point in the 2-dimensional
/// projection of a diagram. Currently this is the first maximum-dimensional generator, but will
/// change to incorporate information about homotopies.
#[derive(Debug, Clone, Serialize)]
pub struct Generators(Vec<Vec<Generator>>);

impl Generators {
    pub fn new(diagram: &DiagramN) -> Self {
        if diagram.dimension() < 2 {
            // TODO: Make this into an error
            panic!();
        }

        // TODO: Projection
        Generators(
            diagram
                .slices()
                .map(|slice| {
                    DiagramN::try_from(slice)
                        .unwrap()
                        .slices()
                        .map(|p| p.max_generator())
                        .collect()
                })
                .collect(),
        )
    }

    pub fn get(&self, x: SliceIndex, y: SliceIndex) -> Option<Generator> {
        let slice = match y {
            SliceIndex::Boundary(Boundary::Source) => self.0.first()?,
            SliceIndex::Boundary(Boundary::Target) => self.0.last()?,
            SliceIndex::Interior(height) => self.0.get(height.to_int())?,
        };

        match x {
            SliceIndex::Boundary(Boundary::Source) => slice.first().cloned(),
            SliceIndex::Boundary(Boundary::Target) => slice.last().cloned(),
            SliceIndex::Interior(height) => slice.get(height.to_int()).cloned(),
        }
    }
}

/// Diagram analysis that finds the depth of cells in the 2-dimensional projection of a diagram.
#[derive(Debug, Clone)]
pub struct Depths(Vec<Vec<Option<usize>>>);

impl Depths {
    pub fn new(diagram: &DiagramN) -> Self {
        if diagram.dimension() < 2 {
            // TODO: Make this into an error.
            panic!();
        }

        Depths(
            diagram
                .slices()
                .map(|slice| {
                    let slice: DiagramN = slice.try_into().unwrap();
                    (0..slice.size())
                        .map(|height| depth(&slice, height))
                        .collect()
                })
                .collect(),
        )
    }

    pub fn get(&self, x: SingularHeight, y: SliceIndex) -> Option<usize> {
        let slice = match y {
            SliceIndex::Boundary(Boundary::Source) => self.0.first()?,
            SliceIndex::Boundary(Boundary::Target) => self.0.last()?,
            SliceIndex::Interior(height) => self.0.get(height.to_int())?,
        };

        slice.get(x).cloned().flatten()
    }
}

fn depth(diagram: &DiagramN, height: usize) -> Option<usize> {
    if diagram.dimension() < 2 {
        return None;
    }

    let cospan = diagram.cospans().get(height)?;
    let forward: &RewriteN = (&cospan.forward).try_into().unwrap();
    let backward: &RewriteN = (&cospan.backward).try_into().unwrap();

    let forward = forward.targets().first().cloned();
    let backward = backward.targets().first().cloned();

    match (forward, backward) {
        (Some(forward), Some(backward)) => Some(std::cmp::min(forward, backward)),
        (Some(forward), None) => Some(forward),
        (None, Some(backward)) => Some(backward),
        (None, None) => None,
    }
}

/// Diagram analysis that finds the depth of the target for each wire going into a non-trivial
/// position in the 2-dimensional projection of a diagram. This can be used to render homotopies
/// appropriately by distinguishing under- and over-passes. This analysis builds on `Depths`.
#[derive(Debug, Clone)]
pub struct WireDepths {
    forward: Vec<Vec<Option<usize>>>,
    backward: Vec<Vec<Option<usize>>>,
}

impl WireDepths {
    pub fn new(diagram: &DiagramN, depths: &Depths) -> Self {
        assert!(diagram.dimension() >= 2);

        let slices: Vec<DiagramN> = diagram.slices().map(|s| s.try_into().unwrap()).collect();
        let cospans = diagram.cospans();

        let mut forward = Vec::new();
        let mut backward = Vec::new();

        for i in 0..diagram.size() {
            forward.push(wire_depths(
                &slices[Height::Regular(i).to_int()],
                (&cospans[i].forward).try_into().unwrap(),
                i,
                depths,
            ));

            backward.push(wire_depths(
                &slices[Height::Regular(i + 1).to_int()],
                (&cospans[i].backward).try_into().unwrap(),
                i,
                depths,
            ));
        }

        WireDepths { forward, backward }
    }

    pub fn get_forward(&self, x: SingularHeight, y: RegularHeight) -> Option<usize> {
        self.forward.get(y)?.get(x)?.as_ref().cloned()
    }

    pub fn get_backward(&self, x: SingularHeight, y: RegularHeight) -> Option<usize> {
        self.backward.get(y)?.get(x)?.as_ref().cloned()
    }
}

fn wire_depths(
    slice: &DiagramN,
    rewrite: &RewriteN,
    height: RegularHeight,
    depths: &Depths,
) -> Vec<Option<usize>> {
    if slice.dimension() < 2 {
        return [None].repeat(slice.size());
    }

    let mut wire_depths = Vec::new();

    for i in 0..slice.size() {
        let rewrite_slice: RewriteN = rewrite.slice(i).try_into().unwrap();
        let source_depth = depths.get(i, Height::Regular(height).into());

        wire_depths
            .push(source_depth.map(|source_depth| rewrite_slice.singular_image(source_depth)));
    }

    wire_depths
}

/// Diagram analysis that determines for each singular point in the 2-dimensional projection of a
/// diagram whether it is an identity or contains some non-trivial cell or homotopy.
#[derive(Debug, Clone)]
pub struct Identities(Vec<Vec<bool>>);

impl Identities {
    pub fn new(diagram: &DiagramN) -> Self {
        assert!(diagram.dimension() >= 2);

        let slices: Vec<_> = diagram.slices().collect();
        let cospans = diagram.cospans();
        let mut identities = Vec::new();

        for i in 0..diagram.size() {
            let slice: &DiagramN = (&slices[Height::Singular(i).to_int()]).try_into().unwrap();
            let forward: &RewriteN = (&cospans[i].forward).try_into().unwrap();
            let backward: &RewriteN = (&cospans[i].backward).try_into().unwrap();

            let targets = {
                let mut targets = BTreeSet::new();
                targets.extend(forward.targets());
                targets.extend(backward.targets());
                targets
            };

            identities.push((0..slice.size()).map(|j| !targets.contains(&j)).collect());
        }

        Identities(identities)
    }

    pub fn is_identity(&self, x: SingularHeight, y: SingularHeight) -> bool {
        let row = match self.0.get(y) {
            Some(row) => row,
            None => return true,
        };

        match row.get(x) {
            Some(id) => *id,
            None => true,
        }
    }
}

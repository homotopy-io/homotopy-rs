use crate::common::*;
use crate::diagram::DiagramN;
use crate::rewrite::RewriteN;
use serde::Serialize;
use std::convert::*;

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

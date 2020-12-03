use crate::common::*;
use crate::diagram::{DiagramN};
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

use crate::common::*;
use crate::diagram::*;
use std::convert::*;

#[derive(Debug, Clone, Copy)]
pub struct BoundaryPath(pub Boundary, pub usize);

impl BoundaryPath {
    pub fn split(path: &[SliceIndex]) -> (Option<BoundaryPath>, Vec<Height>) {
        use SliceIndex::*;

        let mut boundary_path: Option<BoundaryPath> = None;
        let mut interior = Vec::new();

        for height in path.iter().rev() {
            match (boundary_path, height) {
                (Some(mut bp), _) => bp.1 += 1,
                (None, Boundary(b)) => boundary_path = Some(BoundaryPath(*b, 0)),
                (None, Interior(h)) => interior.insert(0, *h),
            }
        }

        (boundary_path, interior)
    }

    pub fn follow(&self, diagram: &DiagramN) -> Option<Diagram> {
        let mut diagram = diagram.clone();

        for _ in 0..self.1 {
            diagram = diagram.source().try_into().ok()?;
        }

        diagram.slice(self.0)
    }
}

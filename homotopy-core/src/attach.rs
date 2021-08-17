use crate::common::{Boundary, Height, SliceIndex};
use crate::diagram::{Diagram, DiagramN};
use crate::rewrite::Cospan;
use serde::{Deserialize, Serialize};
use std::convert::{From, Into, TryInto};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BoundaryPath(pub Boundary, pub usize);

impl BoundaryPath {
    pub fn split(path: &[SliceIndex]) -> (Option<Self>, Vec<Height>) {
        use SliceIndex::{Boundary, Interior};

        let mut boundary_path: Option<Self> = None;
        let mut interior = Vec::new();

        for height in path.iter().rev() {
            match (&mut boundary_path, height) {
                (Some(bp), _) => bp.1 += 1,
                (None, Boundary(b)) => boundary_path = Some(Self(*b, 0)),
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

    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn boundary(&self) -> Boundary {
        self.0
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn depth(&self) -> usize {
        self.1
    }
}

impl From<Boundary> for BoundaryPath {
    fn from(boundary: Boundary) -> Self {
        Self(boundary, 0)
    }
}

pub fn attach<F, E>(diagram: &DiagramN, path: &BoundaryPath, build: F) -> Result<DiagramN, E>
where
    F: FnOnce(Diagram) -> Result<Vec<Cospan>, E>,
{
    let (diagram, _) = attach_worker(diagram, path, build)?;
    Ok(diagram)
}

fn attach_worker<F, E>(
    diagram: &DiagramN,
    path: &BoundaryPath,
    build: F,
) -> Result<(DiagramN, usize), E>
where
    F: FnOnce(Diagram) -> Result<Vec<Cospan>, E>,
{
    match path {
        BoundaryPath(Boundary::Source, 0) => {
            let mut cospans = build(diagram.source())?;
            let mut source = diagram.source();

            for cospan in cospans.iter().rev() {
                source = source.rewrite_forward(&cospan.backward).unwrap();
                source = source.rewrite_backward(&cospan.forward).unwrap();
            }

            let offset = cospans.len();
            cospans.extend(diagram.cospans().iter().cloned());
            Ok((DiagramN::new_unsafe(source, cospans), offset))
        }

        BoundaryPath(Boundary::Target, 0) => {
            let added_cospans = build(diagram.target())?;
            let offset = added_cospans.len();
            let mut cospans = diagram.cospans().to_vec();
            cospans.extend(added_cospans);
            Ok((DiagramN::new_unsafe(diagram.source(), cospans), offset))
        }

        BoundaryPath(boundary, depth) => {
            let source: DiagramN = diagram.source().try_into().unwrap();
            let (source, offset) =
                attach_worker(&source, &BoundaryPath(*boundary, depth - 1), build)?;

            let cospans = match boundary {
                Boundary::Source => {
                    let mut pad = vec![0; depth - 1];
                    pad.push(offset);
                    diagram.cospans().iter().map(|c| c.pad(&pad)).collect()
                }
                Boundary::Target => diagram.cospans().to_vec(),
            };

            Ok((DiagramN::new_unsafe(source.into(), cospans), offset))
        }
    }
}

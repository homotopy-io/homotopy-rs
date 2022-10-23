use std::convert::{From, Into, TryInto};

use crate::{
    common::{Boundary, BoundaryPath, DimensionError},
    diagram::{Diagram, DiagramN},
    rewrite::Cospan,
};

pub fn attach<F, E>(diagram: &DiagramN, path: BoundaryPath, build: F) -> Result<DiagramN, E>
where
    F: FnOnce(Diagram) -> Result<Vec<Cospan>, E>,
    E: From<DimensionError>,
{
    let (diagram, _) = attach_worker(diagram, path, build)?;
    Ok(diagram)
}

fn attach_worker<F, E>(
    diagram: &DiagramN,
    path: BoundaryPath,
    build: F,
) -> Result<(DiagramN, usize), E>
where
    F: FnOnce(Diagram) -> Result<Vec<Cospan>, E>,
    E: From<DimensionError>,
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
            Ok((DiagramN::new(source, cospans), offset))
        }

        BoundaryPath(Boundary::Target, 0) => {
            let added_cospans = build(diagram.target())?;
            let offset = added_cospans.len();
            let mut cospans = diagram.cospans().to_vec();
            cospans.extend(added_cospans);
            Ok((DiagramN::new(diagram.source(), cospans), offset))
        }

        BoundaryPath(boundary, depth) => {
            let source: DiagramN = diagram.source().try_into()?;
            let (source, offset) =
                attach_worker(&source, BoundaryPath(boundary, depth - 1), build)?;

            let cospans = match boundary {
                Boundary::Source => {
                    let mut pad = vec![0; depth - 1];
                    pad.push(offset);
                    diagram.cospans().iter().map(|c| c.pad(&pad)).collect()
                }
                Boundary::Target => diagram.cospans().to_vec(),
            };

            Ok((DiagramN::new(source.into(), cospans), offset))
        }
    }
}

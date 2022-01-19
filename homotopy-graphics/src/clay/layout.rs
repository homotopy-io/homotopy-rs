use homotopy_common::{declare_idx, idx::IdxVec};
use homotopy_core::{
    common::DimensionError, layout::Layout, mesh, DiagramN, Generator, Height, SliceIndex,
};
use petgraph::graph::NodeIndex;
use ultraviolet::Vec4;

use crate::clay::geom::{
    cubical::CubicalMesh, Boundary, Carries, Mesh, Vert, WithBoundaryAndGenerator, WithGenerator,
};

impl Boundary {
    /// Calculate the boundary of a given location in a diagram.
    fn at_coord(mut coord: &[SliceIndex]) -> Self {
        let mut boundary = Self::Zero;

        loop {
            match coord {
                [] | [_] => return boundary,
                [index, tail @ ..] => {
                    coord = tail;
                    if let SliceIndex::Interior(_) = index {
                        boundary.inc();
                    }
                }
            }
        }
    }
}

declare_idx! {
    struct CoordIdx = usize;
}

pub fn extract_mesh(diagram: &DiagramN, depth: usize) -> Result<CubicalMesh, DimensionError> {
    let mesh = mesh::Mesh::new(diagram, depth)?;
    let layout = Layout::new(diagram, depth)?;

    let mut geometry = CubicalMesh::new(diagram.clone().into());
    let mut node_to_vert: IdxVec<NodeIndex, Vert> = IdxVec::with_capacity(mesh.graph.node_count());

    for elem in mesh.flatten_elements() {
        if elem.len() == 1 {
            let n = elem[0];
            let path = &mesh.graph[n].0;

            let coord = {
                let coord = layout.get(path);
                if coord.len() == 3 {
                    Vec4::new(coord[0], coord[1], coord[2], 0.0)
                } else {
                    Vec4::new(coord[0], coord[1], coord[2], coord[3])
                }
            };

            let stratum = path
                .iter()
                .map(|&index| match index {
                    SliceIndex::Boundary(_) => -1,
                    SliceIndex::Interior(Height::Regular(_)) => 0,
                    SliceIndex::Interior(Height::Singular(_)) => 1,
                })
                .sum();
            let boundary = Boundary::at_coord(path);
            let generator = mesh.graph[n].1.max_generator();

            let vert = geometry.mk(coord.with_boundary_and_generator(stratum, boundary, generator));

            node_to_vert.push(vert);

            if codimension_visible(diagram.dimension(), generator, 0) {
                geometry.mk(vert);
            }
        } else {
            let n = elem.len().log2() as usize;

            if n >= depth {
                continue;
            }

            let verts = elem.iter().map(|n| node_to_vert[*n]).collect::<Vec<_>>();
            let generator = minimum_generator(verts.iter().map(|v| geometry.verts[*v].generator));

            if !codimension_visible(diagram.dimension(), generator, n) {
                continue;
            }

            match n {
                1 => {
                    let verts: [Vert; 2] = verts.try_into().unwrap();
                    geometry.mk(verts);

                    // Extract curves
                    let curve = geometry.curves.values_mut().find(|curve| {
                        let &curve_target = curve.last().unwrap();
                        codimension_visible(
                            diagram.dimension(),
                            geometry.verts[curve_target].generator,
                            1,
                        ) && curve_target == verts[0]
                    });
                    if let Some(curve) = curve {
                        curve.push(verts[1]);
                    } else {
                        geometry
                            .curves
                            .push(verts.to_vec().with_generator(generator));
                    }
                }
                2 => {
                    let verts: [Vert; 4] = verts.try_into().unwrap();
                    geometry.mk(verts);
                }
                3 => {
                    let verts: [Vert; 8] = verts.try_into().unwrap();
                    geometry.mk(verts);
                }
                _ => panic!(),
            }
        }
    }

    let (min, max) = geometry.bounds();
    let translation = 0.5 * (max + min);
    let duration = 0.5 * (max.w - min.w);

    for vert in geometry.verts.values_mut() {
        **vert -= translation;
        vert.w /= duration;
    }

    Ok(geometry)
}

#[inline]
fn minimum_generator(generators: impl Iterator<Item = Generator>) -> Generator {
    generators.min_by_key(|g| g.dimension).unwrap()
}

#[inline]
fn codimension_visible(dimension: usize, generator: Generator, threshold: usize) -> bool {
    let codimension = dimension.saturating_sub(generator.dimension);
    codimension == threshold
}

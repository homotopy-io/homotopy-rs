use crate::common::{Generator, Height, SingularHeight};
use crate::diagram::{Diagram, DiagramN};
use crate::normalization::normalize;
use crate::rewrite::{Cone, Cospan, Rewrite, RewriteN};
use std::collections::HashMap;
use std::convert::Into;
use std::convert::TryInto;
use std::rc::Rc;
use thiserror::Error;

type Point = Vec<SingularHeight>;

pub type Signature = HashMap<Generator, Diagram>;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("diagram contains an unknown generator: {0:?}")]
    UnknownGenerator(Generator),

    #[error("diagram is ill-typed")]
    IllTyped,
}

pub fn typecheck<'a, S>(diagram: &Diagram, signature: S) -> Result<(), TypeError>
where
    S: Fn(Generator) -> Option<&'a Diagram> + Copy,
{
    let diagram = match diagram {
        Diagram::Diagram0(g) => {
            if g.dimension == 0 {
                return Ok(());
            } else {
                return Err(TypeError::IllTyped);
            }
        }
        Diagram::DiagramN(d) => d,
    };

    typecheck(&diagram.source(), signature)?;

    let slices: Vec<_> = diagram.slices().collect();

    for (i, cospan) in diagram.cospans().iter().enumerate() {
        let target_embeddings = target_points(&[cospan.forward.clone(), cospan.backward.clone()])
            .into_iter()
            .map(|(t, g)| (Embedding::from_point(&t), g));

        for (target_embedding, generator) in target_embeddings {
            let source = restrict_diagram(
                &slices[Height::Regular(i).to_int()],
                &target_embedding.preimage(&cospan.forward),
            );

            let forward = restrict_rewrite(&cospan.forward, &target_embedding);
            let backward = restrict_rewrite(&cospan.backward, &target_embedding);
            let restricted = DiagramN::new_unsafe(source, vec![Cospan { forward, backward }]);
            let signature_diagram =
                signature(generator).ok_or(TypeError::UnknownGenerator(generator))?;

            let restricted = normalize(&restricted.into());
            let mut signature_diagram = normalize(&signature_diagram);

            while signature_diagram.dimension() < restricted.dimension() {
                signature_diagram = signature_diagram.identity().into();
            }

            if restricted != signature_diagram {
                return Err(TypeError::IllTyped);
            }
        }
    }

    Ok(())
}

fn target_points(rewrites: &[Rewrite]) -> Vec<(Point, Generator)> {
    if rewrites.is_empty() {
        return vec![];
    }

    if rewrites[0].dimension() == 0 {
        let target = rewrites.iter().find_map(|r| match r {
            Rewrite::Rewrite0(r) => r.target(),
            Rewrite::RewriteN(_) => panic!(),
        });

        match target {
            Some(target) => {
                return vec![(vec![], target)];
            }
            None => return vec![],
        }
    }

    assert!(rewrites
        .iter()
        .all(|r| r.dimension() == rewrites[0].dimension()));

    let mut target_rewrites: HashMap<usize, Vec<Rewrite>> = HashMap::new();

    for rewrite in rewrites.iter() {
        let rewrite: RewriteN = rewrite.clone().try_into().unwrap();
        for target_height in rewrite.targets() {
            for source_height in rewrite.singular_preimage(target_height) {
                target_rewrites
                    .entry(target_height)
                    .or_insert_with(Vec::new)
                    .push(rewrite.slice(source_height));
            }
        }
    }

    let mut targets = Vec::new();

    for (target_height, rewrite_slices) in &target_rewrites {
        targets.extend(target_points(rewrite_slices).into_iter().map(|mut p| {
            p.0.insert(0, *target_height);
            p
        }));
    }

    targets
}

#[derive(Debug, Clone)]
enum Embedding {
    Regular(usize, Rc<Embedding>),
    Singular(usize, Vec<Rc<Embedding>>),
    Zero,
}

impl Embedding {
    /// Construct an embedding which contains precisely one singular point.
    fn from_point(point: &[SingularHeight]) -> Self {
        let mut embedding = Self::Zero;

        for &height in point.iter().rev() {
            embedding = Self::Singular(height, vec![Rc::new(embedding)]);
        }

        embedding
    }

    fn preimage(&self, rewrite: &Rewrite) -> Self {
        match self {
            Self::Zero => Self::Zero,
            Self::Regular(height, slice) => {
                let rewrite: &RewriteN = rewrite.try_into().unwrap();
                let preimage_height = rewrite.regular_image(*height);
                Self::Regular(preimage_height, slice.clone())
            }
            Self::Singular(height, slices) => {
                let rewrite: &RewriteN = rewrite.try_into().unwrap();
                let preimage_height = rewrite.regular_image(*height);
                let preimage_slices: Vec<_> = slices
                    .iter()
                    .enumerate()
                    .flat_map(|(target_height, slice)| {
                        rewrite
                            .singular_preimage(target_height + height)
                            .map(|source_height| {
                                Rc::new(slice.preimage(&rewrite.slice(source_height)))
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();

                if preimage_slices.is_empty() {
                    let cospan = &rewrite.cone_over_target(*height).unwrap().target;
                    Self::Regular(
                        preimage_height,
                        Rc::new(slices[0].preimage(&cospan.forward)),
                    )
                } else {
                    Self::Singular(preimage_height, preimage_slices)
                }
            }
        }
    }
}

fn restrict_diagram(diagram: &Diagram, embedding: &Embedding) -> Diagram {
    match embedding {
        Embedding::Zero => {
            assert_eq!(diagram.dimension(), 0);
            diagram.clone()
        }
        Embedding::Regular(height, slice) => {
            let diagram: DiagramN = diagram.clone().try_into().unwrap();
            restrict_diagram(&diagram.slice(Height::Regular(*height)).unwrap(), slice)
                .identity()
                .into()
        }
        Embedding::Singular(height, slices) => {
            let diagram: &DiagramN = diagram.try_into().unwrap();
            assert!(diagram.size() >= height + slices.len());
            let source = restrict_diagram(
                &diagram.source(),
                &slices[0].preimage(&diagram.cospans()[*height].forward),
            );
            let cospans = diagram.cospans()[*height..*height + slices.len()]
                .iter()
                .enumerate()
                .map(|(i, cospan)| Cospan {
                    forward: restrict_rewrite(&cospan.forward, &slices[i]),
                    backward: restrict_rewrite(&cospan.backward, &slices[i]),
                })
                .collect();
            DiagramN::new_unsafe(source, cospans).into()
        }
    }
}

/// Restrict a rewrite to the preimage over the a subdiagram of the target.
fn restrict_rewrite(rewrite: &Rewrite, embedding: &Embedding) -> Rewrite {
    match embedding {
        Embedding::Zero => {
            assert_eq!(rewrite.dimension(), 0);
            rewrite.clone()
        }
        Embedding::Regular(_, _) => Rewrite::identity(rewrite.dimension()),
        Embedding::Singular(height, slices) => {
            let rewrite: &RewriteN = rewrite.try_into().unwrap();
            let mut restricted_cones: Vec<Cone> = Vec::new();

            for target_height in rewrite.targets() {
                if target_height < *height {
                    continue;
                } else if target_height >= height + slices.len() {
                    break;
                }

                let embedding_slice = &slices[target_height - *height];

                // TODO: This is quite ugly
                let cone = rewrite.cone_over_target(target_height).unwrap();

                let restricted_slices: Vec<_> = cone
                    .slices
                    .iter()
                    .map(|cone_slice| restrict_rewrite(cone_slice, embedding_slice))
                    .collect();

                let restricted_source: Vec<_> = cone
                    .source
                    .iter()
                    .enumerate()
                    .map(|(i, cospan)| {
                        let embedding = embedding_slice.preimage(&cone.slices[i]);
                        let forward = restrict_rewrite(&cospan.forward, &embedding);
                        let backward = restrict_rewrite(&cospan.backward, &embedding);
                        Cospan { forward, backward }
                    })
                    .collect();

                let restricted_target = {
                    let slice = embedding_slice;
                    let forward = restrict_rewrite(&cone.target.forward, &slice);
                    let backward = restrict_rewrite(&cone.target.backward, &slice);
                    Cospan { forward, backward }
                };

                restricted_cones.push(Cone {
                    index: cone.index - rewrite.regular_image(*height),
                    slices: restricted_slices,
                    source: restricted_source,
                    target: restricted_target,
                });
            }

            RewriteN::new(rewrite.dimension(), restricted_cones).into()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Boundary;

    use super::*;

    #[test]
    fn associativity() {
        let x = Generator::new(0, 0);
        let f = Generator::new(1, 1);
        let m = Generator::new(2, 2);
        let a = Generator::new(3, 3);

        let f_d = DiagramN::new(f, x, x).unwrap();
        let ff_d = f_d.attach(&f_d, Boundary::Target, &[]).unwrap();
        let m_d = DiagramN::new(m, ff_d, f_d.clone()).unwrap();
        let left_d = m_d.attach(&m_d, Boundary::Source, &[0]).unwrap();
        let right_d = m_d.attach(&m_d, Boundary::Source, &[1]).unwrap();
        let a_d = DiagramN::new(a, left_d, right_d).unwrap();

        let mut signature = HashMap::<Generator, Diagram>::new();
        signature.insert(x, x.into());
        signature.insert(f, f_d.into());
        signature.insert(m, m_d.into());
        signature.insert(a, a_d.clone().into());

        typecheck(&a_d.into(), |generator| signature.get(&generator)).unwrap();
    }
}

use itertools::Itertools;
use thiserror::Error;

use crate::{rewrite::Cone, Cospan, Diagram, DiagramN, Orientation, Rewrite, Rewrite0, RewriteN};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Error)]
pub enum BubbleError {
    #[error("Expected an atomic diagram")]
    NotAtomic,
}

impl DiagramN {
    pub fn bubble(&self) -> Result<Self, BubbleError> {
        use Orientation::Zero;

        // Check that the diagram is atomic.
        let cospan = self
            .cospans()
            .iter()
            .exactly_one()
            .cloned()
            .map_err(|_err| BubbleError::NotAtomic)?;

        let f0 = cospan.forward.orientation_transform(Zero);
        let b0 = cospan.backward.orientation_transform(Zero);

        let inverse = cospan.inverse();

        let singular0 = self.source().rewrite_forward(&cospan.forward).unwrap();
        let singular1 = self.source().rewrite_forward(&inverse.backward).unwrap();

        let contract = RewriteN::new(
            self.dimension(),
            vec![Cone::new(
                0,
                vec![cospan, inverse],
                Cospan {
                    forward: f0.clone(),
                    backward: f0.clone(),
                },
                vec![f0.clone(), b0, f0.clone()],
                vec![
                    Rewrite::directed_identity(singular0),
                    Rewrite::directed_identity(singular1),
                ],
            )],
        );

        let expand = RewriteN::new(
            self.dimension(),
            vec![Cone::new_unit(
                0,
                Cospan {
                    forward: f0.clone(),
                    backward: f0.clone(),
                },
                f0,
            )],
        );

        Ok(Self::new(
            self.source().identity().into(),
            vec![Cospan {
                forward: expand.into(),
                backward: contract.into(),
            }],
        ))
    }
}

impl Rewrite {
    fn directed_identity(source: Diagram) -> Self {
        use Orientation::Zero;
        match source {
            Diagram::Diagram0(s) => Rewrite0::new(s, s.orientation_transform(Zero), None).into(),
            Diagram::DiagramN(source) => RewriteN::new(
                source.dimension(),
                source
                    .cospans()
                    .iter()
                    .zip(source.singular_slices())
                    .enumerate()
                    .map(|(i, (cs, singular))| {
                        let f0 = cs.forward.orientation_transform(Zero);
                        let b0 = cs.backward.orientation_transform(Zero);
                        Cone::new(
                            i,
                            vec![cs.clone()],
                            Cospan {
                                forward: f0.clone(),
                                backward: b0.clone(),
                            },
                            vec![f0, b0],
                            vec![Rewrite::directed_identity(singular)],
                        )
                    })
                    .collect(),
            )
            .into(),
        }
    }
}

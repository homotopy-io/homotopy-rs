use crate::{
    rewrite::{Cone, Label},
    Cospan, Diagram, Orientation, Rewrite, Rewrite0, RewriteN,
};

pub fn bubble(source: &Diagram, cospan: Cospan) -> (Diagram, Cospan) {
    use Orientation::Zero;

    let f0 = cospan.forward.orientation_transform(Zero);
    let b0 = cospan.backward.orientation_transform(Zero);

    let inverse = cospan.inverse();

    let singular0 = source.clone().rewrite_forward(&cospan.forward).unwrap();
    let singular1 = source.clone().rewrite_forward(&inverse.backward).unwrap();

    let contract = RewriteN::new(
        source.dimension() + 1,
        vec![Cone::new_n(
            0,
            vec![cospan, inverse],
            Cospan {
                forward: f0.clone(),
                backward: f0.clone(),
            },
            vec![b0],
            vec![
                Rewrite::directed_identity(&singular0),
                Rewrite::directed_identity(&singular1),
            ],
        )],
    );

    let expand = RewriteN::new(
        source.dimension() + 1,
        vec![Cone::new_0(
            0,
            Cospan {
                forward: f0.clone(),
                backward: f0.clone(),
            },
            f0,
        )],
    );

    (
        source.identity().into(),
        Cospan {
            forward: expand.into(),
            backward: contract.into(),
        },
    )
}

impl Rewrite {
    fn directed_identity(source: &Diagram) -> Self {
        use Orientation::Zero;
        match source {
            Diagram::Diagram0(s) => {
                let t = s.orientation_transform(Zero);
                Rewrite0::new(*s, t, Label::new(None)).into()
            }
            Diagram::DiagramN(source) => {
                let singular = source.singular_slices();
                let cones = source
                    .cospans()
                    .iter()
                    .enumerate()
                    .map(|(i, cs)| {
                        Cone::new_n(
                            i,
                            vec![cs.clone()],
                            Cospan {
                                forward: cs.forward.orientation_transform(Zero),
                                backward: cs.backward.orientation_transform(Zero),
                            },
                            vec![],
                            vec![Rewrite::directed_identity(&singular[i])],
                        )
                    })
                    .collect();
                RewriteN::new(source.dimension(), cones).into()
            }
        }
    }
}

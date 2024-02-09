use homotopy_core::{
    common::{BoundaryPath, Label},
    factorization::factorize,
    rewrite::Cone,
    signature::SignatureBuilder,
    Boundary, Cospan, DiagramN, Generator, Height, Rewrite, Rewrite0, RewriteN, SliceIndex,
};

#[test]
#[allow(clippy::many_single_char_names)]
fn bead_rewrite_base() -> Result<(), String> {
    let mut builder = SignatureBuilder::default();
    let x = builder.add_zero();
    let e = builder
        .add(x, x)
        .map_err(|_err| "couldn't make new diagram")?;
    let bead = builder
        .add(e.clone(), e.clone())
        .map_err(|_err| "couldn't make new diagram")?;
    let bead_rewrite = builder
        .add(bead.clone(), bead.clone())
        .map_err(|_err| "couldn't make new diagram")?;

    // x -> e <- x -> e <- x
    let source = e
        .attach(&e, Boundary::Target, &[])
        .map_err(|_err| "attachment invalid")?;

    // x -> bead_rewrite <- x -> e <- x
    let target = DiagramN::new(
        x.into(),
        vec![
            DiagramN::try_from(
                DiagramN::try_from(
                    bead_rewrite
                        .slice(SliceIndex::Interior(Height::Singular(0)))
                        .ok_or("couldn't slice diagram")?,
                )
                .map_err(|_err| "malformed diagram")?
                .slice(SliceIndex::Interior(Height::Singular(0)))
                .ok_or("couldn't slice diagram")?,
            )
            .map_err(|_err| "malformed diagram")?
            .cospans()[0]
                .clone(),
            source.cospans()[1].clone(),
        ],
    );

    // x -> bead_rewrite <- x -> bead <- x
    let terminal = DiagramN::new(
        x.into(),
        vec![
            target.cospans()[0].clone(),
            DiagramN::try_from(
                bead.slice(SliceIndex::Interior(Height::Singular(0)))
                    .ok_or("couldn't slice diagram")?,
            )
            .map_err(|_err| "malformed diagram")?
            .cospans()[0]
                .clone(),
        ],
    );

    // x -> e <------------ x ---> e <-- x
    // |     \              |      |     |
    // v      v             v      v     v
    // x -> bead_rewrite <- x -> bead <- x
    let f = RewriteN::from_slices(
        1,
        source.cospans(),
        terminal.cospans(),
        vec![
            vec![
                terminal.cospans()[0].forward.clone(),
                terminal.cospans()[0].backward.clone(),
            ],
            vec![
                terminal.cospans()[1].forward.clone(),
                terminal.cospans()[1].backward.clone(),
            ],
        ],
        vec![
            vec![RewriteN::try_from(
                DiagramN::try_from(
                    bead_rewrite
                        .slice(SliceIndex::Interior(Height::Singular(0)))
                        .ok_or("couldn't slice diagram")?,
                )
                .map_err(|_err| "malformed diagram")?
                .cospans()[0]
                    .backward
                    .clone(),
            )
            .map_err(|_err| "malformed rewrite")?
            .slice(0)],
            vec![RewriteN::try_from(bead.cospans()[0].forward.clone())
                .map_err(|_err| "malformed diagram")?
                .slice(0)],
        ],
    )
    .into();

    // x -> bead_rewrite <- x ---> e <-- x
    // |        |           |      |     |
    // v        v           v      v     v
    // x -> bead_rewrite <- x -> bead <- x
    let g = RewriteN::from_slices(
        1,
        target.cospans(),
        terminal.cospans(),
        vec![
            vec![
                terminal.cospans()[0].forward.clone(),
                terminal.cospans()[0].backward.clone(),
            ],
            vec![
                terminal.cospans()[1].forward.clone(),
                terminal.cospans()[1].backward.clone(),
            ],
        ],
        vec![
            vec![Rewrite::identity(0)],
            vec![RewriteN::try_from(bead.cospans()[0].forward.clone())
                .map_err(|_err| "malformed diagram")?
                .slice(0)],
        ],
    )
    .into();

    // x -> e <------------ x -> e <- x
    // |     \              |    |    |
    // v      v             v    v    v
    // x -> bead_rewrite <- x -> e <- x
    let h = RewriteN::from_slices(
        1,
        source.cospans(),
        target.cospans(),
        vec![
            vec![
                target.cospans()[0].forward.clone(),
                target.cospans()[0].backward.clone(),
            ],
            vec![
                target.cospans()[1].forward.clone(),
                target.cospans()[1].backward.clone(),
            ],
        ],
        vec![
            vec![RewriteN::try_from(
                DiagramN::try_from(
                    bead_rewrite
                        .slice(SliceIndex::Interior(Height::Singular(0)))
                        .ok_or("couldn't slice diagram")?,
                )
                .map_err(|_err| "malformed diagram")?
                .cospans()[0]
                    .backward
                    .clone(),
            )
            .map_err(|_err| "malformed rewrite")?
            .slice(0)],
            vec![Rewrite::identity(0)],
        ],
    )
    .into();

    let mut with_identity = factorize(&f, &Rewrite::identity(1));
    assert_eq!(with_identity.next(), Some(f.clone()));

    let mut with_f = factorize(&f, &f);
    assert_eq!(with_f.next(), Some(Rewrite::identity(1)));

    let mut with_g = factorize(&f, &g);
    assert_eq!(with_g.next(), Some(h));

    Ok(())
}

#[test]
fn scalar_braid() {
    use Boundary::{Source, Target};
    use Height::Regular;

    let x = Generator::new(0, 0);
    let f = Generator::new(1, 2);
    let s = Generator::new(2, 3);

    let f_cospan = Cospan {
        forward: Rewrite0::new(
            x,
            f,
            Some(Label::new(
                BoundaryPath(Source, 1),
                std::iter::once(vec![]).collect(),
            )),
        )
        .into(),
        backward: Rewrite0::new(
            x,
            f,
            Some(Label::new(
                BoundaryPath(Target, 1),
                std::iter::once(vec![]).collect(),
            )),
        )
        .into(),
    };
    let s_cospan = Cospan {
        forward: Rewrite0::new(
            x,
            s,
            Some(Label::new(
                BoundaryPath(Source, 2),
                std::iter::once(vec![]).collect(),
            )),
        )
        .into(),
        backward: Rewrite0::new(
            x,
            s,
            Some(Label::new(
                BoundaryPath(Target, 2),
                std::iter::once(vec![]).collect(),
            )),
        )
        .into(),
    };

    let rewrite_f = RewriteN::new(
        1,
        vec![
            Cone::new_unit(
                0,
                s_cospan.clone(),
                Rewrite0::new(
                    x,
                    s,
                    Some(Label::new(
                        BoundaryPath(Source, 0),
                        std::iter::once(vec![Regular(0), Regular(0)]).collect(),
                    )),
                )
                .into(),
            ),
            Cone::new_unit(
                0,
                f_cospan.clone(),
                Rewrite0::new(
                    x,
                    f,
                    Some(Label::new(
                        BoundaryPath(Source, 0),
                        std::iter::once(vec![Regular(0)]).collect(),
                    )),
                )
                .into(),
            ),
        ],
    )
    .into();

    let rewrite_g = RewriteN::new(
        1,
        vec![Cone::new_unit(
            0,
            s_cospan.clone(),
            Rewrite0::new(
                x,
                s,
                Some(Label::new(
                    BoundaryPath(Source, 1),
                    std::iter::once(vec![Regular(0)]).collect(),
                )),
            )
            .into(),
        )],
    )
    .into();

    let mut fact = factorize(&rewrite_f, &rewrite_g);

    assert!(fact.next().is_some());
}

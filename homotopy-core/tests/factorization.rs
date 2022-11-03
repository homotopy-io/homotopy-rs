use homotopy_core::{
    common::BoundaryPath, factorization::factorize, rewrite::Cone, signature::SignatureBuilder,
    Boundary, Cospan, Diagram, DiagramN, Generator, Height, Rewrite, Rewrite0, RewriteN,
    SliceIndex,
};

#[test]
fn bead_rewrite_base() -> Result<(), String> {
    let mut builder = SignatureBuilder::default();
    let x = builder.add_zero();
    let e = builder
        .add(x.clone(), x.clone())
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
        .map_err(|_| "attachment invalid")?;

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
    );

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
    );

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
    );

    let mut with_identity = factorize(
        f.clone().into(),
        Rewrite::identity(1),
        terminal.clone().into(),
    );
    assert_eq!(with_identity.next(), Some(f.clone().into()));

    let mut with_f = factorize(f.clone().into(), f.clone().into(), terminal.clone().into());
    assert_eq!(with_f.next(), Some(Rewrite::identity(1)));

    let mut with_g = factorize(f.into(), g.into(), terminal.into());
    assert_eq!(with_g.next(), Some(h.into()));

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
        forward: Rewrite0::new(x, f, Some((f, BoundaryPath(Source, 1), vec![]))).into(),
        backward: Rewrite0::new(x, f, Some((f, BoundaryPath(Target, 1), vec![]))).into(),
    };
    let s_cospan = Cospan {
        forward: Rewrite0::new(x, s, Some((s, BoundaryPath(Source, 2), vec![]))).into(),
        backward: Rewrite0::new(x, s, Some((s, BoundaryPath(Target, 2), vec![]))).into(),
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
                    Some((s, BoundaryPath(Source, 0), vec![Regular(0), Regular(0)])),
                )
                .into(),
            ),
            Cone::new_unit(
                0,
                f_cospan.clone(),
                Rewrite0::new(x, f, Some((f, BoundaryPath(Source, 0), vec![Regular(0)]))).into(),
            ),
        ],
    )
    .into();

    let rewrite_g = RewriteN::new(
        1,
        vec![Cone::new_unit(
            0,
            s_cospan.clone(),
            Rewrite0::new(x, s, Some((s, BoundaryPath(Source, 1), vec![Regular(0)]))).into(),
        )],
    )
    .into();

    let target = DiagramN::new(Diagram::Diagram0(x.into()), vec![s_cospan, f_cospan]).into();

    let mut fact = factorize(rewrite_f, rewrite_g, target);

    assert!(fact.next().is_some())
}

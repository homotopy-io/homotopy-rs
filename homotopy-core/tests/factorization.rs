use homotopy_core::{
    factorization::factorize, signature::SignatureBuilder, Boundary, DiagramN, Height, Rewrite,
    RewriteN, SliceIndex,
};

#[test]
fn bead_rewrite_base() -> Result<(), String> {
    let mut builder = SignatureBuilder::new();
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
        x.clone(),
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
        x,
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
        source.clone().into(),
        target.clone().into(),
    );
    assert_eq!(with_identity.next(), Some(f.clone().into()));

    let mut with_f = factorize(
        f.clone().into(),
        f.clone().into(),
        source.clone().into(),
        target.clone().into(),
    );
    assert_eq!(with_f.next(), Some(Rewrite::identity(1)));

    let mut with_g = factorize(f.into(), g.into(), source.into(), target.into());
    assert_eq!(with_g.next(), Some(h.into()));

    Ok(())
}

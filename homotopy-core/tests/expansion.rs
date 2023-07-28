use homotopy_core::{
    examples,
    signature::{GeneratorInfo, Signature},
    Bias, Boundary, DiagramN, Direction, Generator, Height,
};

#[test]
fn matchsticks() {
    use Height::Singular;

    let (sig, diagram) = examples::matchsticks();

    let contracted = diagram
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            1,
            Some(Bias::Lower),
            &sig,
        )
        .unwrap()
        .target();

    let _expanded = contracted
        .identity()
        .expand(
            Boundary::Target.into(),
            &mut [],
            [Singular(0), Singular(1)],
            Direction::Forward,
            &sig,
        )
        .unwrap();
}

// expansions like:
// |   |        |   |
// |   |        e   |
// |   |        |   |
// e   >    ⤳   |   >
// |  / \       |  / \
// | |   |      | |   |
#[test]
fn bead_with_half_braid() -> anyhow::Result<()> {
    let (mut sig, half_braid) = examples::half_braid();
    let s = DiagramN::try_from(
        sig.generator_info(Generator::new(1, 2))
            .unwrap()
            .diagram()
            .clone(),
    )?;
    let e = sig.add(s.clone(), s.clone())?;
    let e_above_left_half_braid =
        half_braid
            .attach(&s, Boundary::Source, &[])?
            .attach(&e, Boundary::Target, &[])?;
    let e_below_left_half_braid =
        half_braid
            .attach(&s, Boundary::Source, &[])?
            .attach(&e, Boundary::Source, &[])?;
    let e_above_right_half_braid =
        half_braid
            .attach(&s, Boundary::Target, &[])?
            .attach(&e, Boundary::Target, &[1])?;
    let e_below_right_half_braid =
        half_braid
            .attach(&s, Boundary::Target, &[])?
            .attach(&e, Boundary::Source, &[2])?;

    let e_tensor_half_braid = e_above_left_half_braid
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            1,
            None,
            &sig,
        )?
        .target();
    assert_eq!(
        e_tensor_half_braid
            .clone()
            .identity()
            .expand(
                Boundary::Target.into(),
                &mut [],
                [Height::Singular(0), Height::Singular(0)],
                Direction::Forward,
                &sig
            )?
            .target(),
        e_above_left_half_braid.into()
    );
    assert_eq!(
        e_tensor_half_braid
            .identity()
            .expand(
                Boundary::Target.into(),
                &mut [],
                [Height::Singular(0), Height::Singular(0)],
                Direction::Backward,
                &sig
            )?
            .target(),
        e_below_left_half_braid.into()
    );

    let half_braid_tensor_e = e_below_right_half_braid
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            1,
            None,
            &sig,
        )?
        .target();
    assert_eq!(
        half_braid_tensor_e
            .clone()
            .identity()
            .expand(
                Boundary::Target.into(),
                &mut [],
                [Height::Singular(0), Height::Singular(1)],
                Direction::Forward,
                &sig
            )?
            .target(),
        e_above_right_half_braid.into()
    );
    assert_eq!(
        half_braid_tensor_e
            .identity()
            .expand(
                Boundary::Target.into(),
                &mut [],
                [Height::Singular(0), Height::Singular(1)],
                Direction::Backward,
                &sig
            )?
            .target(),
        e_below_right_half_braid.into()
    );
    Ok(())
}

// expanding bead:
// |   |
//  \ /    | |
//   >   ⤳ | |
//  / \    | |
// |   |
// should not create weak identity
#[test]
fn braid_smooth() -> anyhow::Result<()> {
    let (sig, touching) = examples::touching();
    // make touching have only one singular height
    let touching = touching
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            1,
            None,
            &sig,
        )?
        .target();
    let expanded: DiagramN = touching
        .identity()
        .expand(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            [Height::Singular(0), Height::Singular(0)],
            Direction::Backward,
            &sig,
        )?
        .target()
        .try_into()?;
    assert_eq!(expanded.cospans().len(), 0);
    let f = DiagramN::try_from(
        sig.generator_info(Generator::new(1, 1))
            .unwrap()
            .diagram()
            .clone(),
    )?;
    assert_eq!(expanded, f.attach(&f, Boundary::Target, &[])?.identity());

    Ok(())
}

// ensure expansion never stretches
// |
// e
// |
#[test]
fn bead_no_stretch() -> anyhow::Result<()> {
    let (sig, e) = examples::bead_series(1);
    let f = DiagramN::try_from(
        sig.generator_info(Generator::new(1, 1))
            .unwrap()
            .diagram()
            .clone(),
    )?;
    let e_f_f_inverse =
        e.attach(&f, Boundary::Target, &[])?
            .attach(&f.inverse(), Boundary::Target, &[])?;
    assert!(e_f_f_inverse
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            1,
            Direction::Forward,
            1,
            None,
            &sig
        )
        .is_err());

    let (mut sig, touching) = examples::touching();
    // make touching have only one singular height
    let touching: DiagramN = touching
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            1,
            None,
            &sig,
        )?
        .target()
        .try_into()?;
    let f = DiagramN::try_from(
        sig.generator_info(Generator::new(1, 1))
            .unwrap()
            .diagram()
            .clone(),
    )?;
    let e = sig.add(f.clone(), f.clone())?;
    let e_touching = touching
        .attach(&f, Boundary::Source, &[])?
        .attach(&e, Boundary::Source, &[])?
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            1,
            None,
            &sig,
        )?
        .target();
    assert!(e_touching
        .identity()
        .expand(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            [Height::Singular(1), Height::Singular(0)],
            Direction::Forward,
            &sig
        )
        .is_err());
    Ok(())
}

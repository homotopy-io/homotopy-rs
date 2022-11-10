use homotopy_core::{
    examples, signature::SignatureBuilder, Bias, Boundary, Diagram, DiagramN, Height,
};
use insta::assert_debug_snapshot;
use pretty_assertions::assert_eq;

#[test]
fn scalar() {
    let (sig, scalar) = examples::scalar();
    let scalar_inverse = scalar.inverse();
    let scalar_then_scalar = DiagramN::new(
        scalar.source(),
        [scalar.cospans(), scalar.cospans()].concat(),
    );
    let scalar_then_inverse = DiagramN::new(
        scalar.source(),
        [scalar.cospans(), scalar_inverse.cospans()].concat(),
    );
    let inverse_then_scalar = DiagramN::new(
        scalar_inverse.source(),
        [scalar_inverse.cospans(), scalar.cospans()].concat(),
    );

    assert!(scalar_then_scalar
        .identity()
        .contract(Boundary::Target.into(), &[], 0, Some(Bias::Same), &sig)
        .is_err());

    assert_debug_snapshot!(
        "scalar_biased_left",
        scalar_then_scalar
            .identity()
            .contract(Boundary::Target.into(), &[], 0, Some(Bias::Lower), &sig)
            .unwrap()
            .target()
    );

    assert_debug_snapshot!(
        "scalar_biased_right",
        scalar_then_scalar
            .identity()
            .contract(Boundary::Target.into(), &[], 0, Some(Bias::Higher), &sig)
            .unwrap()
            .target()
    );

    assert_debug_snapshot!(
        "scalar_inverse_unbiased",
        scalar_then_inverse
            .identity()
            .contract(Boundary::Target.into(), &[], 0, Some(Bias::Same), &sig)
            .unwrap()
            .target()
    );

    assert_debug_snapshot!(
        "inverse_scalar_unbiased",
        inverse_then_scalar
            .identity()
            .contract(Boundary::Target.into(), &[], 0, Some(Bias::Same), &sig)
            .unwrap()
            .target()
    );
}

#[test]
fn three_scalars() {
    let (sig, scalar) = examples::scalar();
    let three = scalar
        .attach(&scalar, Boundary::Target, &[])
        .unwrap()
        .attach(&scalar, Boundary::Target, &[])
        .unwrap();
    let l = three
        .identity()
        .contract(Boundary::Target.into(), &[], 0, Some(Bias::Higher), &sig)
        .unwrap()
        .target();
    // l is .
    //      . .
    let into_middle =
        l.identity()
            .contract(Boundary::Target.into(), &[], 0, Some(Bias::Lower), &sig);
    assert!(into_middle.is_ok());
    let into_left =
        l.identity()
            .contract(Boundary::Target.into(), &[], 0, Some(Bias::Higher), &sig);
    assert!(into_left.is_ok());
}

#[test]
fn braids() {
    // make braids by contraction
    let (sig, crossing) = examples::crossing();
    assert_debug_snapshot!(
        "crossing",
        crossing
            .identity()
            .contract(Boundary::Target.into(), &[], 0, None, &sig)
            .unwrap()
            .target()
    );

    let (sig, touching) = examples::touching();
    assert_debug_snapshot!(
        "touching",
        touching
            .identity()
            .contract(Boundary::Target.into(), &[], 0, None, &sig)
            .unwrap()
            .target()
    );
}

#[test]
fn beads() {
    let (sig, diagram) = examples::two_beads();

    let contracted = diagram
        .identity()
        .contract(Boundary::Target.into(), &[], 0, None, &sig)
        .unwrap();

    assert_debug_snapshot!("beads", contracted);
}

#[test]
fn stacks() {
    let (sig, diagram) = examples::stacks();

    let contracted = diagram
        .identity()
        .contract(Boundary::Target.into(), &[], 0, None, &sig)
        .unwrap();

    assert_debug_snapshot!("stacks", contracted);
}

#[test]
fn inverses_1d() {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let f = sig.add(x, x).unwrap();
    let f_inverse = f.inverse();
    let f_then_inverse = DiagramN::new(f.source(), vec![f.cospans(), f_inverse.cospans()].concat());
    assert_debug_snapshot!(
        "1-endomorphism_then_inverse",
        f_then_inverse
            .identity()
            .contract(Boundary::Target.into(), &[], 0, None, &sig)
            .unwrap()
            .target()
    );

    let inverse_then_f = DiagramN::new(
        f_inverse.source(),
        vec![f.cospans(), f_inverse.cospans()].concat(),
    );
    assert_debug_snapshot!(
        "1-inverse_then_endomorphism",
        inverse_then_f
            .identity()
            .contract(Boundary::Target.into(), &[], 0, None, &sig)
            .unwrap()
            .target()
    );
}

#[test]
fn inverses_2d() {
    let (sig, e) = examples::two_endomorphism();
    let e_inverse = e.inverse();

    let e_then_e = DiagramN::new(e.source(), [e.cospans(), e.cospans()].concat());
    assert!(e_then_e
        .identity()
        .contract(Boundary::Target.into(), &[], 0, None, &sig)
        .is_err());

    let e_then_inverse = DiagramN::new(e.source(), [e.cospans(), e_inverse.cospans()].concat());
    assert_debug_snapshot!(
        "2-endomorphism_then_inverse",
        e_then_inverse
            .identity()
            .contract(Boundary::Target.into(), &[], 0, None, &sig)
            .unwrap()
            .target()
    );

    let inverse_then_e = DiagramN::new(
        e_inverse.source(),
        [e_inverse.cospans(), e.cospans()].concat(),
    );
    assert_debug_snapshot!(
        "2-inverse_then_endomorphism",
        inverse_then_e
            .identity()
            .contract(Boundary::Target.into(), &[], 0, None, &sig)
            .unwrap()
            .target()
    );
}

#[test]
fn snake() {
    let (sig, snake) = examples::real_snake();

    assert_eq!(
        snake
            .identity()
            .contract(Boundary::Target.into(), &[], 0, None, &sig)
            .expect("failed to contract snake")
            .target(),
        Diagram::from(snake.source().identity())
    );
}

#[test]
fn bubble() {
    let (sig, bubble) = examples::bubble();
    assert_eq!(
        bubble
            .identity()
            .contract(Boundary::Target.into(), &[], 0, None, &sig)
            .expect("failed to contract bubble")
            .target(),
        Diagram::from(bubble.source().identity())
    );
}

// cone-wise smoothing should smooth out the cap next to a wire
#[test]
fn bead_then_cap() -> anyhow::Result<()> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let f = sig.add(x, x)?;
    let g = sig.add(x, x)?;
    let endo_f = sig.add(f.clone(), f.clone())?;
    let endo_g = sig.add(g.clone(), g.clone())?;
    let endo_f_then_inverse = endo_f.attach(&endo_f.inverse(), Boundary::Target, &[0])?;
    let g_then_endo_f_then_inverse = endo_f_then_inverse.attach(&g, Boundary::Source, &[])?;
    let endo_g_then_endo_f_then_inverse =
        g_then_endo_f_then_inverse.attach(&endo_g, Boundary::Source, &[0])?;
    let endo_g_tensor_endo_f_then_inverse = endo_g_then_endo_f_then_inverse
        .identity()
        .contract(Boundary::Target.into(), &[], 0, None, &sig)?
        .target();
    assert_eq!(
        endo_g_tensor_endo_f_then_inverse
            .identity()
            .contract(Boundary::Target.into(), &[], 0, None, &sig)?
            .target(),
        endo_g.attach(&f, Boundary::Target, &[])?.into()
    );
    Ok(())
}

// cone-wise smoothing should only happen at the top level, never in a recursive level
#[test]
fn cap_braid() -> anyhow::Result<()> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let s = sig.add(x.identity(), x.identity())?;
    let cap: DiagramN = s
        .attach(&s.inverse(), Boundary::Target, &[])?
        .identity()
        .contract(Boundary::Target.into(), &[], 0, Some(Bias::Same), &sig)?;
    let wire_then_cap = cap.attach(&s, Boundary::Source, &[0])?;
    let wire_over_cap: DiagramN = wire_then_cap
        .identity()
        .contract(
            Boundary::Target.into(),
            &[Height::Singular(0)],
            0,
            Some(Bias::Lower),
            &sig,
        )?
        .target()
        .try_into()?;
    assert_eq!(wire_over_cap.cospans().len(), 1);
    let wire_under_cap: DiagramN = wire_then_cap
        .identity()
        .contract(
            Boundary::Target.into(),
            &[Height::Singular(0)],
            0,
            Some(Bias::Higher),
            &sig,
        )?
        .target()
        .try_into()?;
    assert_eq!(wire_under_cap.cospans().len(), 1);

    Ok(())
}

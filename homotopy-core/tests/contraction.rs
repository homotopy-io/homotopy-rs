use homotopy_core::{examples, signature::SignatureBuilder, Bias, Boundary, DiagramN};
use insta::assert_debug_snapshot;

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
    let (sig, diagram) = examples::three_beads();

    let contracted = diagram
        .identity()
        .contract(Boundary::Target.into(), &[], 1, None, &sig)
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
    let mut sig = SignatureBuilder::new();
    let x = sig.add_zero();
    let f = sig.add(x.clone(), x).unwrap();
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

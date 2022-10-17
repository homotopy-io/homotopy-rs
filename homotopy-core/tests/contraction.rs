use homotopy_core::{
    examples,
    typecheck::{typecheck, Mode},
    Bias, Boundary,
};
use insta::assert_debug_snapshot;

#[test]
fn scalar() {
    let (sig, d) = examples::two_scalars();

    assert!(d
        .identity()
        .contract(Boundary::Target.into(), &[], 0, None, &sig)
        .is_err());

    assert_debug_snapshot!(
        "scalar_biased_left",
        d.identity()
            .contract(Boundary::Target.into(), &[], 0, Some(Bias::Lower), &sig)
            .unwrap()
            .target()
    );

    assert_debug_snapshot!(
        "scalar_biased_right",
        d.identity()
            .contract(Boundary::Target.into(), &[], 0, Some(Bias::Higher), &sig)
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

    typecheck(&contracted.into(), &sig, Mode::Deep).unwrap();
}

#[test]
fn stacks() {
    let (sig, diagram) = examples::stacks();

    let contracted = diagram
        .identity()
        .contract(Boundary::Target.into(), &[], 0, None, &sig)
        .unwrap();

    typecheck(&contracted.into(), &sig, Mode::Deep).unwrap();
}

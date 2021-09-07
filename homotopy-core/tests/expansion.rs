use homotopy_core::{
    typecheck::{typecheck, Mode},
    *,
};

#[test]
#[allow(clippy::many_single_char_names)]
fn matchsticks() {
    use Height::*;

    let (sig, diagram) = examples::matchsticks();

    let contracted = diagram
        .identity()
        .contract(&Boundary::Target.into(), &[], 0, Some(Bias::Lower), &sig)
        .unwrap()
        .target();

    let expanded = contracted
        .identity()
        .expand(
            &Boundary::Target.into(),
            &[Singular(0), Singular(1)],
            Direction::Forward,
            &sig,
        )
        .unwrap();

    typecheck(&expanded.into(), &sig, Mode::Deep).unwrap();
}

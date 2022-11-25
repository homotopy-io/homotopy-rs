use homotopy_core::{examples, Bias, Boundary, Direction, Height};

#[test]
fn matchsticks() {
    use Height::Singular;

    let (sig, diagram) = examples::matchsticks();

    let contracted = diagram
        .identity()
        .contract(Boundary::Target.into(), &[], 0, Some(Bias::Lower), &sig)
        .unwrap()
        .target();

    let _expanded = contracted
        .identity()
        .expand(
            Boundary::Target.into(),
            &[Singular(0), Singular(1)],
            Direction::Forward,
            &sig,
        )
        .unwrap();
}

use homotopy_core::signature::SignatureBuilder;
use homotopy_core::typecheck::{typecheck, Mode};
use homotopy_core::*;

#[test]
#[allow(clippy::many_single_char_names)]
fn matchsticks() {
    use Height::*;

    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let f = sig.add(x.clone(), x.clone()).unwrap();
    let up = sig.add(f.clone(), x.identity()).unwrap();
    let down = sig.add(x.identity(), f).unwrap();
    let diagram = up.attach(&down, Boundary::Target, &[]).unwrap();

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

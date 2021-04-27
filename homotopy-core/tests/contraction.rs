use homotopy_core::signature::SignatureBuilder;
use homotopy_core::typecheck::{typecheck, Mode};
use homotopy_core::*;
use insta::*;

#[test]
fn scalar() {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let s = sig.add(x.identity(), x.identity()).unwrap();
    let t = sig.add(x.identity(), x.identity()).unwrap();
    let d = s.attach(&t, Boundary::Target, &[]).unwrap();

    assert!(d
        .identity()
        .contract(&Boundary::Target.into(), &[], 0, None, &sig)
        .is_err());

    assert_debug_snapshot!(
        "scalar_biased_left",
        d.identity()
            .contract(&Boundary::Target.into(), &[], 0, Some(Bias::Lower), &sig)
            .unwrap()
            .target()
    );

    assert_debug_snapshot!(
        "scalar_biased_right",
        d.identity()
            .contract(&Boundary::Target.into(), &[], 0, Some(Bias::Higher), &sig)
            .unwrap()
            .target()
    );
}

#[test]
#[allow(clippy::many_single_char_names)]
fn beads() {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let f = sig.add(x.clone(), x).unwrap();
    let a = sig.add(f.clone(), f.clone()).unwrap();
    let b = sig.add(f.clone(), f.clone()).unwrap();
    let c = sig.add(f.clone(), f.clone()).unwrap();

    let diagram = a
        .attach(&f, Boundary::Target, &[])
        .unwrap()
        .attach(&b, Boundary::Target, &[1])
        .unwrap()
        .attach(&c, Boundary::Target, &[0])
        .unwrap();

    let contracted = diagram
        .identity()
        .contract(&Boundary::Target.into(), &[], 1, None, &sig)
        .unwrap();

    typecheck(&contracted.into(), &sig, Mode::Deep).unwrap();
}

#[test]
#[allow(clippy::many_single_char_names)]
fn stacks() {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let f = sig.add(x.identity(), x.identity()).unwrap();
    let m = sig.add(f.clone(), x.identity().identity()).unwrap();

    let diagram = m
        .attach(&f, Boundary::Target, &[])
        .unwrap()
        .attach(&m, Boundary::Target, &[])
        .unwrap();

    let contracted = diagram
        .identity()
        .contract(&Boundary::Target.into(), &[], 0, None, &sig)
        .unwrap();

    typecheck(&contracted.into(), &sig, Mode::Deep).unwrap();
}

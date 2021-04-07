use std::collections::HashMap;

use homotopy_core::typecheck::typecheck;
use homotopy_core::*;
use insta::*;

#[test]
fn scalar() {
    let x = Diagram::from(Generator::new(0, 0));
    let s = DiagramN::new(Generator::new(1, 2), x.identity(), x.identity()).unwrap();
    let t = DiagramN::new(Generator::new(2, 2), x.identity(), x.identity()).unwrap();
    let diagram = s.attach(&t, Boundary::Target, &[]).unwrap();

    assert!(diagram
        .identity()
        .contract(&Boundary::Target.into(), &[], 0, None)
        .is_none());

    assert_debug_snapshot!(
        "scalar_biased_left",
        diagram
            .identity()
            .contract(&Boundary::Target.into(), &[], 0, Some(Bias::Lower))
            .unwrap()
            .target()
    );

    assert_debug_snapshot!(
        "scalar_biased_right",
        diagram
            .identity()
            .contract(&Boundary::Target.into(), &[], 0, Some(Bias::Higher))
            .unwrap()
            .target()
    );
}

#[test]
fn beads() {
    let x = Diagram::from(Generator::new(0, 0));
    let f = DiagramN::new(Generator::new(1, 1), x.clone(), x.clone()).unwrap();
    let a = DiagramN::new(Generator::new(2, 2), f.clone(), f.clone()).unwrap();
    let b = DiagramN::new(Generator::new(3, 2), f.clone(), f.clone()).unwrap();

    let diagram = a
        .attach(&f, Boundary::Target, &[])
        .unwrap()
        .attach(&b, Boundary::Target, &[1])
        .unwrap();

    let contracted = diagram
        .identity()
        .contract(&Boundary::Target.into(), &[], 0, None)
        .unwrap();

    let mut signature = HashMap::<Generator, Diagram>::new();
    signature.insert(x.max_generator(), x);
    signature.insert(f.max_generator(), f.into());
    signature.insert(a.max_generator(), a.into());
    signature.insert(b.max_generator(), b.into());
    typecheck(&contracted.into(), |generator| signature.get(&generator)).unwrap();
}

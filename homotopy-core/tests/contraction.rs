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
#[allow(clippy::many_single_char_names)]
fn beads() {
    let x = Diagram::from(Generator::new(0, 0));
    let f = DiagramN::new(Generator::new(1, 1), x.clone(), x.clone()).unwrap();
    let a = DiagramN::new(Generator::new(2, 2), f.clone(), f.clone()).unwrap();
    let b = DiagramN::new(Generator::new(3, 2), f.clone(), f.clone()).unwrap();
    let c = DiagramN::new(Generator::new(4, 2), f.clone(), f.clone()).unwrap();

    let diagram = a
        .attach(&f, Boundary::Target, &[])
        .unwrap()
        .attach(&b, Boundary::Target, &[1])
        .unwrap()
        .attach(&c, Boundary::Target, &[0])
        .unwrap();

    let contracted = diagram
        .identity()
        .contract(&Boundary::Target.into(), &[], 1, None)
        .unwrap();

    let mut signature = HashMap::<Generator, Diagram>::new();
    signature.insert(x.max_generator(), x);
    signature.insert(f.max_generator(), f.into());
    signature.insert(a.max_generator(), a.into());
    signature.insert(b.max_generator(), b.into());
    signature.insert(c.max_generator(), c.into());
    typecheck(&contracted.into(), |generator| signature.get(&generator)).unwrap();
}

// Crash in contraction

#[test]
#[allow(clippy::many_single_char_names)]
fn stacks() {
    let x = Diagram::from(Generator::new(0, 0));
    let f = DiagramN::new(Generator::new(1, 2), x.identity(), x.identity()).unwrap();
    let m = DiagramN::new(Generator::new(2, 3), f.clone(), x.identity().identity()).unwrap();

    let diagram = m
        .attach(&f, Boundary::Target, &[])
        .unwrap()
        .attach(&m, Boundary::Target, &[])
        .unwrap();

    let contracted = diagram
        .identity()
        .contract(&Boundary::Target.into(), &[], 0, None)
        .unwrap();

    let mut signature = HashMap::<Generator, Diagram>::new();
    signature.insert(x.max_generator(), x);
    signature.insert(f.max_generator(), f.into());
    signature.insert(m.max_generator(), m.into());
    typecheck(&contracted.into(), |generator| signature.get(&generator)).unwrap();
}

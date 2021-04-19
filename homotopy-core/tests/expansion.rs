use std::collections::HashMap;

use homotopy_core::typecheck::typecheck;
use homotopy_core::*;

#[test]
#[allow(clippy::many_single_char_names)]
fn matchsticks() {
    use Height::*;

    let x = Diagram::from(Generator::new(0, 0));
    let f = DiagramN::new(Generator::new(1, 1), x.clone(), x.clone()).unwrap();
    let up = DiagramN::new(Generator::new(2, 2), f.clone(), x.identity()).unwrap();
    let down = DiagramN::new(Generator::new(3, 2), x.identity(), f.clone()).unwrap();

    let mut signature = HashMap::<Generator, Diagram>::new();
    signature.insert(x.max_generator(), x);
    signature.insert(f.max_generator(), f.into());
    signature.insert(up.max_generator(), up.clone().into());
    signature.insert(down.max_generator(), down.clone().into());

    let diagram = up.attach(&down, Boundary::Target, &[]).unwrap();

    let contracted = diagram
        .identity()
        .contract(&Boundary::Target.into(), &[], 0, Some(Bias::Lower))
        .unwrap()
        .target();

    let expanded = contracted
        .identity()
        .expand(
            &Boundary::Target.into(),
            &[Singular(0), Singular(1)],
            Direction::Forward,
        )
        .unwrap();

    typecheck(&expanded.into(), |generator| signature.get(&generator)).unwrap();
}

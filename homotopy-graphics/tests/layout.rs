use homotopy_core::*;
use homotopy_graphics::svg::layout::*;
use insta::*;

#[test]
fn assoc() {
    let x = Diagram::from(Generator::new(0, 0));
    let f = DiagramN::from_generator(Generator::new(1, 1), x.clone(), x).unwrap();
    let ff = f.attach(&f, Boundary::Target, &[]).unwrap();
    let m = DiagramN::from_generator(Generator::new(2, 2), ff, f).unwrap();
    let left = m.attach(&m, Boundary::Source, &[0]).unwrap();
    let right = m.attach(&m, Boundary::Source, &[1]).unwrap();
    let assoc =
        DiagramN::from_generator(Generator::new(3, 3), left.clone(), right.clone()).unwrap();

    assert_json_snapshot!("assoc_left", Layout::new(&left, 100).unwrap());
    assert_json_snapshot!("assoc_right", Layout::new(&right, 100).unwrap());
    assert_json_snapshot!("assoc_projected", Layout::new(&assoc, 100).unwrap());
}

#[test]
fn scalar() {
    let x = Diagram::from(Generator::new(0, 0));
    let s = DiagramN::from_generator(Generator::new(2, 2), x.identity(), x.identity()).unwrap();

    assert_json_snapshot!(Layout::new(&s, 100).unwrap());
}

use homotopy_core::*;

pub fn example_assoc() -> DiagramN {
    let x = Diagram::from(Generator::new(0, 0));
    let f = DiagramN::new(Generator::new(1, 1), x.clone(), x).unwrap();
    let ff = f.attach(&f, Boundary::Target, &[]).unwrap();
    let m = DiagramN::new(Generator::new(2, 2), ff, f).unwrap();
    let left = m.attach(&m, Boundary::Source, &[0]).unwrap();
    let right = m.attach(&m, Boundary::Source, &[1]).unwrap();
    DiagramN::new(Generator::new(3, 3), left, right).unwrap()
}

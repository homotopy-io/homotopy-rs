use homotopy_core::*;

pub fn example_assoc() -> DiagramN {
    let x = Diagram::from(Generator::new(0, 0));
    let f = DiagramN::new(Generator::new(1, 1), x.clone(), x.clone()).unwrap();
    let ff = f.attach(f.clone(), Boundary::Target, &[]).unwrap();
    let m = DiagramN::new(Generator::new(2, 2), ff.clone(), f.clone()).unwrap();
    let left = m.attach(m.clone(), Boundary::Source, &[0]).unwrap();
    let right = m.attach(m.clone(), Boundary::Source, &[1]).unwrap();
    let assoc = DiagramN::new(Generator::new(3, 3), left, right).unwrap();
    assoc
}

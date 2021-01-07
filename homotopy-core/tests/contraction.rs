use homotopy_core::*;
use insta::*;

#[test]
fn scalar() {
    let x = Diagram::from(Generator::new(0, 0));
    let s = DiagramN::new(Generator::new(1, 2), x.identity(), x.identity()).unwrap();
    let t = DiagramN::new(Generator::new(2, 2), x.identity(), x.identity()).unwrap();
    let diagram = s.attach(t.clone(), Boundary::Target, &[]).unwrap();

    assert!(diagram.contract(&[], 0, None).is_none());
    assert_debug_snapshot!(
        "scalar_biased_left",
        diagram.contract(&[], 0, Some(Bias::Lower)).unwrap()
    );
    assert_debug_snapshot!(
        "scalar_biased_right",
        diagram.contract(&[], 0, Some(Bias::Higher)).unwrap()
    );
}

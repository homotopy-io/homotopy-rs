use homotopy_core::{
    examples,
    signature::{GeneratorInfo, Signature},
    Boundary, Generator,
};
use insta::assert_debug_snapshot;

#[test]
fn assoc_left() {
    assert_debug_snapshot!(examples::associator().1.source());
}

#[test]
fn assoc_right() {
    assert_debug_snapshot!(examples::associator().1.target());
}

// ensure diagram with (source, target)
//  |
//  m     |
// / \  ⤳ |
// u  |   |
//    |
// can be created (source and target are each 3D)
#[test]
fn monoid_with_unit_to_id() -> anyhow::Result<()> {
    let (mut sig, s) = examples::scalar();
    let x = sig
        .generator_info(Generator::new(0, 0))
        .unwrap()
        .diagram()
        .clone();

    let monoid = sig.add(s.attach(&s, Boundary::Target, &[])?, s.clone())?;
    let unit = sig.add(x.identity().identity(), s.clone())?;
    let monoid_with_unit = monoid.attach(&unit, Boundary::Source, &[])?;
    assert!(sig.add(monoid_with_unit, s.identity()).is_ok());

    Ok(())
}

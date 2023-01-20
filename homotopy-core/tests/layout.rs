use homotopy_core::{examples, layout::Layout};
use insta::assert_debug_snapshot;

#[test]
fn monoid_unit() {
    let (_, diagram) = examples::monoid_unit();
    assert_debug_snapshot!(Layout::<2>::new(&diagram.into()).unwrap());
}

#[test]
fn scalar_and_beads() {
    let (_, diagram) = examples::scalar_and_beads();
    assert_debug_snapshot!(Layout::<2>::new(&diagram.into()).unwrap());
}

#[test]
fn associator() {
    let (_, diagram) = examples::associator();
    assert_debug_snapshot!(Layout::<3>::new(&diagram.into()).unwrap());
}

#[test]
fn snake() {
    let (_, diagram) = examples::algebraic_snake();
    assert_debug_snapshot!(Layout::<3>::new(&diagram.into()).unwrap());
}

#[test]
fn lips() {
    let (_, diagram) = examples::lips();
    assert_debug_snapshot!(Layout::<4>::new(&diagram.into()).unwrap());
}

#[test]
fn pants_unit() {
    let (_, diagram) = examples::pants_unit();
    assert_debug_snapshot!(Layout::<4>::new(&diagram.into()).unwrap());
}

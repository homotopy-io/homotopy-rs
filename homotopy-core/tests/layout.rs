use homotopy_core::{examples, layout::Layout};
use insta::assert_debug_snapshot;

#[test]
fn monoid_unit() {
    let (_, diagram) = examples::monoid_unit();

    let layout = Layout::<2>::new(&diagram.into()).unwrap();

    assert_debug_snapshot!(layout.positions);
}

#[test]
fn scalar_and_beads() {
    let (_, diagram) = examples::scalar_and_beads();

    let layout = Layout::<2>::new(&diagram.into()).unwrap();

    assert_debug_snapshot!(layout.positions);
}

#[test]
fn associator() {
    let (_, diagram) = examples::associator();

    let layout = Layout::<3>::new(&diagram.into()).unwrap();

    assert_debug_snapshot!(layout.positions);
}

#[test]
fn snake() {
    let (_, diagram) = examples::algebraic_snake();

    let layout = Layout::<3>::new(&diagram.into()).unwrap();

    assert_debug_snapshot!(layout.positions);
}

#[test]
fn lips() {
    let (_, diagram) = examples::lips();

    let layout = Layout::<4>::new(&diagram.into()).unwrap();

    assert_debug_snapshot!(layout.positions);
}

#[test]
fn pants_unit() {
    let (_, diagram) = examples::pants_unit();

    let layout = Layout::<4>::new(&diagram.into()).unwrap();

    assert_debug_snapshot!(layout.positions);
}

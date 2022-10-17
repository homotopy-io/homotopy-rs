use homotopy_core::examples;
use insta::assert_debug_snapshot;

#[test]
fn assoc_left() {
    assert_debug_snapshot!(examples::associator().1.source());
}

#[test]
fn assoc_right() {
    assert_debug_snapshot!(examples::associator().1.target());
}

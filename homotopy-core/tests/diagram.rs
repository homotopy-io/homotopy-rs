use insta::*;

mod common;

#[test]
fn assoc_left() {
    assert_debug_snapshot!(common::example_assoc().source());
}

#[test]
fn assoc_right() {
    assert_debug_snapshot!(common::example_assoc().target());
}

use homotopy_core::{examples, mesh::Mesh};
use insta::assert_debug_snapshot;

#[test]
fn associator() {
    let (_, diagram) = examples::associator();

    let mesh = Mesh::<3>::new(&diagram.into()).unwrap();
    let elements = mesh.elements(true).collect::<Vec<_>>();

    assert_debug_snapshot!(elements);
}

#[test]
fn snake() {
    let (_, diagram) = examples::snake();

    let mesh = Mesh::<3>::new(&diagram.into()).unwrap();
    let elements = mesh.elements(true).collect::<Vec<_>>();

    assert_debug_snapshot!(elements);
}

#[test]
fn lips() {
    let (_, diagram) = examples::lips();

    let mesh = Mesh::<4>::new(&diagram.into()).unwrap();
    let elements = mesh.elements(true).collect::<Vec<_>>();

    assert_debug_snapshot!(elements);
}

#[test]
fn pants_unit() {
    let (_, diagram) = examples::pants_unit();

    let mesh = Mesh::<4>::new(&diagram.into()).unwrap();
    let elements = mesh.elements(true).collect::<Vec<_>>();

    assert_debug_snapshot!(elements);
}

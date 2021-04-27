use homotopy_core::serialize::*;
use homotopy_core::*;
use insta::*;

mod common;

#[test]
fn serialization() {
    let empty_ser = Serialization::default();
    let empty_sig = Signature::default();
    assert_eq!(empty_ser, empty_sig.into());

    // test for stability
    let g = Generator::new(3, 3);
    let d = Diagram::from(common::example_assoc());
    assert_debug_snapshot!(Keyed::<Key<Diagram>>::key(&d));
    let sig: Signature = vec![(g, d)].into_iter().collect();
    let ser: Serialization = sig.clone().into();
    let bs = Vec::<u8>::from(ser);
    assert_eq!(sig, Signature::from(Serialization::from(bs)));
}

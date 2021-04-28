use homotopy_core::serialize::*;
use homotopy_core::*;
use insta::*;

#[test]
fn serialization() {
    let empty_ser = Serialization::default();
    let empty_sig = Signature::default();
    assert_eq!(empty_ser, empty_sig.into());

    // test for stability
    let (sb, dn) = examples::associator();
    let sig = sb.0;
    let d = Diagram::from(dn);
    assert_debug_snapshot!(Keyed::<Key<Diagram>>::key(&d));
    let ser: Serialization = sig.clone().into();
    let bs = Vec::<u8>::from(ser);
    assert_eq!(sig, Signature::from(Serialization::from(bs)));
}

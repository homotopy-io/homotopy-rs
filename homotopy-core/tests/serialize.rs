use homotopy_core::{serialize::*, *};
use insta::*;

#[test]
fn serialize_associator() {
    let (_, diagram) = examples::associator();

    let (serialized, key) = {
        let mut store = Store::new();
        let key = store.pack_diagram(&diagram.clone().into());
        let serialized = rmp_serde::encode::to_vec_named(&store).unwrap();
        (serialized, key)
    };

    assert_debug_snapshot!(base64::encode(&serialized));

    let deserialized = {
        let store: Store = rmp_serde::decode::from_slice(&serialized).unwrap();
        store.unpack_diagram(key).unwrap()
    };

    assert_eq!(Diagram::from(diagram), deserialized);
}

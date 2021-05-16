use homotopy_core::serialize::*;
use homotopy_core::*;
use insta::*;

#[test]
fn serialize_associator() {
    let (_, diagram) = examples::associator();

    let (serialized, key) = {
        let mut store = Store::new();
        let key = store.pack_diagram(&diagram.clone().into());
        let serialized = serde_json::to_string(&store).unwrap();
        (serialized, key)
    };

    assert_debug_snapshot!(serialized);

    let deserialized = {
        let store: Store = serde_json::from_str(&serialized).unwrap();
        store.unpack_diagram(key).unwrap()
    };

    assert_eq!(Diagram::from(diagram), deserialized);
}

use homotopy_core::typecheck::{typecheck, Mode};
pub use homotopy_model::model::{history::Proof, proof::Action, serialize};
use insta::*;

// Example of how to write tests using action dumps.
#[test]
fn construct_associator() {
    // Mind the beginning r#" and "#!
    // It is needed to avoid going crazy over quoting.
    let action_dump = r#"["CreateGeneratorZero",{"Select":0},{"SetBoundary":"source"},{"Select":0},{"SetBoundary":"target"},{"SelectGenerator":{"dimension":1,"id":1}},{"SelectPoints":[["target"]]},{"SetBoundary":"source"},{"SelectGenerator":{"dimension":1,"id":1}},{"SetBoundary":"target"},{"SelectGenerator":{"dimension":2,"id":2}},{"SelectPoints":[["source",1]]},{"SetBoundary":"source"},{"SelectGenerator":{"dimension":2,"id":2}},{"SelectPoints":[["source",3]]},{"SetBoundary":"target"},{"SelectGenerator":{"dimension":3,"id":3}}]"#;

    let actions: Vec<Action> = serde_json::from_str(&action_dump).unwrap();
    let mut proof: Proof = Default::default();

    for a in actions.iter() {
        proof
            .update(a)
            .expect("Actions should replay without errors.");
        // Typecheck as we go along
        if let Some(workspace) = &proof.workspace {
            typecheck(&workspace.diagram, &proof.signature, Mode::Deep)
                .expect(&format!("Typechecking failure at action: {:?}.", a));
        }
    }

    // Snapshot the end result
    assert_debug_snapshot!("construct_associator_signature", proof.signature);
    assert_debug_snapshot!("construct_associator_workspace", proof.workspace);
}

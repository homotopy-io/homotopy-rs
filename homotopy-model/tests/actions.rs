use homotopy_core::typecheck::{typecheck, Mode};
pub use homotopy_model::{history::Proof, proof::Action, serialize};
use insta::assert_debug_snapshot;

fn actions_test_helper(json: &str) -> Proof {
    let (_safe, actions): (bool, Vec<Action>) = serde_json::from_str(json).unwrap();
    let mut proof: Proof = Default::default();

    for a in &actions {
        proof
            .update(a)
            .expect("Actions should replay without errors.");
        // Typecheck as we go along
        if let Some(workspace) = &proof.workspace {
            typecheck(&workspace.diagram, &proof.signature, Mode::Deep)
                .unwrap_or_else(|_| panic!("Typechecking failure at action: {:?}.", a));
        }
    }

    proof
}

#[test]
fn construct_associator() {
    let action_dump = include_str!("examples/associator.json");

    let proof = actions_test_helper(action_dump);

    // Snapshot the end result
    assert_debug_snapshot!("construct_associator_signature", proof.signature);
    assert_debug_snapshot!("construct_associator_workspace", proof.workspace);
}

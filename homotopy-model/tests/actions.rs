pub use homotopy_model::{history::Proof, proof::Action, serialize};
use insta::assert_debug_snapshot;

fn actions_test_helper(json: &str) -> Proof {
    let (_safe, actions): (bool, Vec<Action>) = serde_json::from_str(json).unwrap();
    let mut proof: Proof = Default::default();

    for a in &actions {
        proof
            .update(a)
            .expect("Actions should replay without errors.");
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

#[ignore]
#[test]
fn construct_braid_and_contract() {
    let action_dump = include_str!("examples/braid_contraction.json");

    let proof = actions_test_helper(action_dump);

    // Snapshot the end result
    assert_debug_snapshot!("construct_braid_contraction_signature", proof.signature);
    assert_debug_snapshot!("construct_braid_contraction_workspace", proof.workspace);
}

/*
// Requires labelled
#[test]
fn construct_interchanges() {
    let action_dump = include_str!("examples/interchanges.json");

    let proof = actions_test_helper(&action_dump);

    // Snapshot the end result
    assert_debug_snapshot!("construct_interchanges_signature", proof.signature);
    assert_debug_snapshot!("construct_interchanges_workspace", proof.workspace);
}
*/

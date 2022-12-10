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

#[test]
fn construct_braiding_half_bubbles() {
    let action_dump = include_str!("examples/braiding_half_bubbles.json");

    let proof = actions_test_helper(action_dump);

    // Snapshot the end result
    assert_debug_snapshot!("construct_braiding_half_bubbles_signature", proof.signature);
    assert_debug_snapshot!("construct_braiding_half_bubbles_workspace", proof.workspace);
}

#[test]
fn construct_adjoint_equivalence() {
    let action_dump = include_str!("examples/equivalence_to_adjoint_equivalence.json");

    let _proof = actions_test_helper(action_dump);
    // Snapshotting with debug takes too long
}

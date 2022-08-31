use homotopy_core::typecheck::{typecheck, Mode};
pub use homotopy_model::model::{history::Proof, proof::Action, serialize};
use insta::*;

fn actions_test_helper(json: &str) -> Proof {
    let actions: Vec<Action> = serde_json::from_str(json).unwrap();
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

    proof
}

#[test]
fn construct_associator() {
    let action_dump = include_str!("examples/associator.json");

    let proof = actions_test_helper(&action_dump);

    // Snapshot the end result
    assert_debug_snapshot!("construct_associator_signature", proof.signature);
    assert_debug_snapshot!("construct_associator_workspace", proof.workspace);
}

#[test]
fn construct_braid_and_contract() {
    let action_dump = include_str!("examples/braid_contraction.json");

    let proof = actions_test_helper(&action_dump);

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

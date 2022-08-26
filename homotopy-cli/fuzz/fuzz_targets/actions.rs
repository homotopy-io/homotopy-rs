
#![no_main]

pub use history::Proof;
pub use homotopy_model::{history, migration, proof, proof::Action, serialize};

libfuzzer_sys::fuzz_target!(|actions: Vec<Action>| {
    let mut proof: Proof = Default::default();
    for a in actions.iter() {
        if proof.is_valid(a) {
        match proof.update(a) {
            Ok(_) => continue,
            Err(_) => break
        };
        } else {
            break;
        }
    }
});

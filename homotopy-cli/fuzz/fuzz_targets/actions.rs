#![no_main]

pub use history::Proof;
pub use homotopy_model::{history, migration, proof, proof::Action, serialize};

libfuzzer_sys::fuzz_target!(|actions: Vec<Action>| {
    let mut proof: Proof = Default::default();
    for a in actions.iter() {
        // Don't go crazy-dimensional
        if proof
            .workspace
            .as_ref()
            .map(|w| w.visible_diagram().dimension() > 10)
            .unwrap_or_default()
        {
            break;
        }
        if a.is_valid(&proof) {
            match proof.update(a) {
                Ok(_) => continue,
                Err(_) => break,
            };
        } else {
            break;
        }
    }
});

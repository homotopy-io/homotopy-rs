use homotopy_model::proof::{Action::ImportProof, SerializedData};
use homotopy_web::model::{Action::Proof, State};

#[test]
#[ignore]
#[allow(unreachable_code)]
fn action() {
    let data: SerializedData = std::env::var("HOMOTOPY_IMPORT")
        .map_or(Err(futures::io::ErrorKind::NotFound.into()), |fp| {
            std::fs::read(fp)
        })
        .unwrap_or_default()
        .into();
    let mut state: State = Default::default();
    let _ = state.update(Proof(ImportProof(data)));
    let _ = state.update(unimplemented!("trigger action goes here"));
}

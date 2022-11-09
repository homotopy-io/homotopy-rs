use homotopy_model::proof::{Action::ImportProof, SerializedData};
use homotopy_web::model::{Action::Proof, State};

#[test]
#[ignore]
#[allow(clippy::diverging_sub_expression)]
fn action() {
    let data: SerializedData = std::env::var("HOMOTOPY_IMPORT")
        .map_or(Err(futures::io::ErrorKind::NotFound.into()), |fp| {
            std::fs::read(fp)
        })
        .unwrap_or_default()
        .into();
    let mut state: State = Default::default();
    state
        .update(Proof(ImportProof(data)))
        .expect("failed to import");
    #[allow(unreachable_code, clippy::unimplemented)]
    state
        .update(unimplemented!("trigger action goes here"))
        .expect("failed to trigger action");
}

use std::{
    fs::{read, write},
    path::PathBuf,
};

use anyhow::{anyhow, Context};
pub use history::Proof;
use homotopy_core::common::Mode;
pub use homotopy_model::model::{history, migration, proof, proof::Action, serialize};
use structopt::StructOpt;

// Struct for CLI options
#[derive(Debug, StructOpt)]
#[structopt(
    name = "homotopy-cli",
    about = "Handy tool to debug proofs! Made by yours truly."
)]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    input_hom: Option<PathBuf>,

    #[structopt(short = "a", long, parse(from_os_str))]
    input_actions: Option<PathBuf>,

    #[structopt(short, long, parse(from_os_str))]
    output_hom: Option<PathBuf>,
}

fn import_actions(path: &PathBuf) -> anyhow::Result<Vec<Action>> {
    let data = read(path)?;
    Ok(serde_json::from_slice(&data)?)
}

fn import_hom(path: &PathBuf) -> anyhow::Result<Proof> {
    let data = read(path)?;
    let ((signature, workspace), metadata) = match serialize::deserialize(&data) {
        Some(res) => res,
        None => migration::deserialize(&data)
            .context("Failed to deserialize or migrate from legacy format.")?,
    };

    for g in signature.iter() {
        g.diagram
            .check(Mode::Deep)
            .map_err(|e| anyhow!("Signature diagram deep check failed: {:?}", e))?;
    }
    if let Some(w) = workspace.as_ref() {
        w.diagram
            .check(Mode::Deep)
            .map_err(|e| anyhow!("Workspace diagram deep check failed: {:?}", e))?;
    }

    let mut proof: Proof = Default::default();
    proof.signature = signature;
    proof.workspace = workspace;
    proof.metadata = metadata;
    Ok(proof)
}

fn export_hom(path: &PathBuf, proof: &Proof) -> anyhow::Result<()> {
    let data = serialize::serialize(
        proof.signature.clone(),
        proof.workspace.clone(),
        proof.metadata.clone(),
    );
    write(path, &data).context("Could not export .hom file.")
}

fn main() -> anyhow::Result<()> {
    // Give me options.
    let opt = Opt::from_args();
    let mut proof = match opt.input_hom {
        Some(path) => import_hom(&path).context("Could not import .hom file.")?,
        None => Default::default(),
    };

    let actions = match opt.input_actions {
        Some(path) => import_actions(&path).context("Could not import action file.")?,
        None => Default::default(),
    };

    for a in actions.iter() {
        println!("Performing action: {:?}", a);
        proof.update(a)?;
    }

    if let Some(path) = opt.output_hom {
        export_hom(&path, &proof)?;
    }

    Ok(())
}

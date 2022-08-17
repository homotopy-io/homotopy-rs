use homotopy_core::migration::OldProof;
use homotopy_graphics::style::{Color, VertexShape};
use serde::Deserialize;

use super::{Signature, Workspace};
use crate::model::proof::{generators::GeneratorInfo, SignatureItem};

#[derive(Deserialize)]
struct Export {
    #[serde(rename = "metadata")]
    _metadata: Metadata,
    proof: String,
}

#[derive(Deserialize)]
struct Metadata {
    #[serde(rename = "title")]
    _title: String,
    #[serde(rename = "author")]
    _author: String,
    #[serde(rename = "abstract")]
    _user_abstract: String,
}

pub fn deserialize(data: &[u8]) -> Option<(Signature, Option<Workspace>)> {
    // Deserialize
    let export: Export = match serde_json::from_slice(data) {
        Err(error) => {
            log::error!("Migration tool: cannot load file. Error: {}", error);
            None
        }
        Ok(export) => Some(export),
    }?;

    let proof = match OldProof::new(&export.proof) {
        Err(error) => {
            log::error!("Migration tool: corrupted proof string. Error: {}", error);
            None
        }
        Ok(proof) => Some(proof),
    }?;

    load(proof)
}

fn load(proof: OldProof) -> Option<(Signature, Option<Workspace>)> {
    let mut signature: Signature = Default::default();

    // generators gives a sorted list of GeneratorInfos
    for v in proof.generator_info {
        let color: Color = v.color.parse().ok()?;
        let info = GeneratorInfo {
            generator: v.generator,
            name: v.name,
            oriented: false,
            invertible: false,
            single_preview: true,
            color,
            shape: VertexShape::default(),
            diagram: v.diagram,
        };
        signature.insert_item(SignatureItem::Item(info));
    }

    let workspace = None;
    Some((signature, workspace))
}

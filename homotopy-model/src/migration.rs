use homotopy_core::migration::OldProof;
use homotopy_graphics::style::{Color, VertexShape};
use serde::Deserialize;

use crate::proof::{generators::GeneratorInfo, Metadata, Signature, SignatureItem, Workspace};

#[derive(Deserialize)]
struct Export {
    metadata: OldMetadata,
    proof: String,
}

#[derive(Deserialize)]
struct OldMetadata {
    #[serde(default)]
    title: String,
    #[serde(default)]
    author: String,
    #[serde(rename = "abstract")]
    #[serde(default)]
    user_abstract: String,
}

pub fn deserialize(data: &[u8]) -> Option<((Signature, Option<Workspace>), Metadata)> {
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

    let metadata = Metadata {
        title: (!export.metadata.title.is_empty()).then_some(export.metadata.title),
        author: (!export.metadata.author.is_empty()).then_some(export.metadata.author),
        abstr: (!export.metadata.user_abstract.is_empty()).then_some(export.metadata.user_abstract),
    };

    let sw = load(proof)?;
    Some((sw, metadata))
}

fn load(proof: OldProof) -> Option<(Signature, Option<Workspace>)> {
    let mut signature: Signature = Default::default();

    for v in proof.generator_info {
        let color: Color = v.color.parse().ok()?;
        let info = GeneratorInfo {
            generator: v.generator,
            name: v.name,
            oriented: true,
            invertible: false,
            single_preview: true,
            color,
            shape: VertexShape::default(),
            diagram: v.diagram.clone(),
        };
        signature.insert_item(SignatureItem::Item(info));
    }

    let workspace = proof.workspace.map(|w| Workspace::new(w.diagram));

    Some((signature, workspace))
}

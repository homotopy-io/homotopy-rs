use homotopy_core::migration::OldProof;
use homotopy_graphics::style::{Color, VertexShape};
use serde::Deserialize;

use super::{Signature, Workspace};
use crate::model::proof::{generators::GeneratorInfo, Metadata, SignatureItem, View};

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

    let sw = load(proof)?;
    Some((sw, export.metadata.into()))
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

    let workspace = match proof.workspace {
        Some(w) => Some(Workspace {
            diagram: w.diagram.clone(),
            path: Default::default(),
            view: View::new(w.diagram.dimension().min(2) as u8),
            attach: Default::default(),
            attachment_highlight: Default::default(),
            slice_highlight: Default::default(),
        }),
        None => None,
    };

    Some((signature, workspace))
}

impl Into<Metadata> for OldMetadata {
    fn into(self) -> Metadata {
        Metadata {
            title: (!self.title.is_empty()).then_some(self.title),
            author: (!self.author.is_empty()).then_some(self.author),
            abstr: (!self.user_abstract.is_empty()).then_some(self.user_abstract),
        }
    }
}

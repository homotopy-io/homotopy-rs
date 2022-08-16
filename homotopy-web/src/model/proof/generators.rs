use homotopy_core::{common::Generator, Diagram};
use homotopy_graphics::style::{Color, GeneratorStyle, SignatureStyleData, VertexShape};
use serde::Serialize;

use super::signature::Signature;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratorInfo {
    pub generator: Generator,
    pub name: String,
    pub framed: bool,
    pub invertible: bool,
    pub single_preview: bool,
    pub color: Color,
    pub shape: VertexShape,
    pub diagram: Diagram,
}

impl SignatureStyleData for Signature {
    type Style = GeneratorInfo;

    fn as_pairs(&self) -> Vec<(Generator, &Self::Style)> {
        self.iter().map(|info| (info.generator, info)).collect()
    }

    fn generator_style(&self, g: Generator) -> Option<&Self::Style> {
        self.generator_info(g)
    }
}

impl GeneratorStyle for GeneratorInfo {
    fn label(&self) -> Option<String> {
        // TODO(thud): Decide whether to show a label
        // Some(self.name.clone())
        None
    }

    fn shape(&self) -> Option<VertexShape> {
        Some(self.shape.clone())
    }

    fn color(&self) -> Color {
        self.color.clone()
    }
}

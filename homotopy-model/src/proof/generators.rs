use homotopy_core::{common::Generator, Diagram};
use homotopy_graphics::style::{Color, GeneratorStyle, VertexShape};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GeneratorInfo {
    pub generator: Generator,
    pub name: String,
    pub oriented: bool,
    pub invertible: bool,
    pub single_preview: bool,
    pub color: Color,
    pub shape: VertexShape,
    pub diagram: Diagram,
}

impl GeneratorStyle for GeneratorInfo {
    fn label(&self) -> Option<String> {
        // TODO(thud): Decide whether to show a label
        // Some(self.name.clone())
        None
    }

    fn shape(&self) -> VertexShape {
        self.shape.clone()
    }

    fn color(&self) -> Color {
        self.color.clone()
    }
}

impl homotopy_core::signature::GeneratorInfo for GeneratorInfo {
    fn diagram(&self) -> &Diagram {
        &self.diagram
    }

    fn is_invertible(&self) -> bool {
        self.invertible
    }
}

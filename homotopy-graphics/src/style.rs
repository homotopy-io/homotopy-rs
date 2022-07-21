use homotopy_core::Generator;

pub trait GeneratorStyle {
    fn label(&self) -> Option<String>;
    fn shape(&self) -> Option<VertexShape>;
    // TODO(thud): migrate color-related code to here for consistency
}

pub trait SignatureStyleData {
    type T: GeneratorStyle;

    fn generator_style(&self, g: Generator) -> Option<&Self::T>;
}

pub enum VertexShape {
    Circle,
    Square,
}

impl Default for VertexShape {
    fn default() -> Self {
        Self::Circle
    }
}

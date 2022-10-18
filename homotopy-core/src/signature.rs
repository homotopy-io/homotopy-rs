use crate::{diagram::NewDiagramError, Diagram, DiagramN, Generator};

pub trait Signature {
    fn diagram(&self, g: Generator) -> Option<&Diagram>;
    fn is_invertible(&self, g: Generator) -> Option<bool>;
}

/// Helper struct for building signatures in tests and benchmarks.
pub struct SignatureBuilder(pub Vec<Diagram>);

impl SignatureBuilder {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn add_zero(&mut self) -> Diagram {
        let generator = Generator::new(self.0.len(), 0);
        self.0.push(generator.into());
        generator.into()
    }

    pub fn add(
        &mut self,
        source: impl Into<Diagram>,
        target: impl Into<Diagram>,
    ) -> Result<DiagramN, NewDiagramError> {
        let source: Diagram = source.into();
        let target: Diagram = target.into();
        let generator = Generator::new(self.0.len(), source.dimension() + 1);
        let diagram = DiagramN::from_generator(generator, source, target)?;
        self.0.push(diagram.clone().into());
        Ok(diagram)
    }
}

impl Default for SignatureBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Signature for SignatureBuilder {
    fn diagram(&self, g: Generator) -> Option<&Diagram> {
        self.0.get(g.id)
    }

    fn is_invertible(&self, g: Generator) -> Option<bool> {
        Some(g.dimension > 0)
    }
}

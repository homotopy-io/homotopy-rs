use crate::{diagram::NewDiagramError, Diagram, DiagramN, Generator};
use std::collections::HashMap;

pub trait Signature {
    fn generator(&self, id: Generator) -> Option<Diagram>;
}

#[derive(Clone, Copy)]
pub struct SignatureClosure<F>(pub F)
where
    F: Fn(Generator) -> Option<Diagram>;

impl<F> Signature for SignatureClosure<F>
where
    F: Fn(Generator) -> Option<Diagram>,
{
    fn generator(&self, id: Generator) -> Option<Diagram> {
        self.0(id)
    }
}

/// Helper struct for building signatures in tests and benchmarks.
pub struct SignatureBuilder(pub HashMap<Generator, Diagram>);

impl SignatureBuilder {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_zero(&mut self) -> Diagram {
        let generator = Generator::new(self.0.len(), 0);
        self.0.insert(generator, generator.into());
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
        let diagram = DiagramN::new(generator, source, target)?;
        self.0.insert(generator, diagram.clone().into());
        Ok(diagram)
    }
}

impl Default for SignatureBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Signature for SignatureBuilder {
    fn generator(&self, id: Generator) -> Option<Diagram> {
        self.0.get(&id).cloned()
    }
}

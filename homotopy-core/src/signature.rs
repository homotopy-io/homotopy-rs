use crate::{
    diagram::{globularity, NewDiagramError},
    Diagram, Diagram0, DiagramN, Generator,
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Depth {
    Finite(usize),
    Infinite,
}

pub trait GeneratorInfo {
    fn diagram(&self) -> &Diagram;
    fn invertibility(&self) -> Option<Depth>;

    fn is_invertible(&self) -> bool {
        self.invertibility().is_some()
    }
}

pub trait Signature {
    type Info: GeneratorInfo;
    fn generators(&self) -> impl Iterator<Item = Generator>;
    fn generator_info(&self, g: Generator) -> Option<&Self::Info>;

    fn globular_pairs(&self, generator: Generator) -> Vec<Generator> {
        let diagram = self.generator_info(generator).unwrap().diagram();
        self.generators()
            .filter(|&g| {
                g != generator && globularity(self.generator_info(g).unwrap().diagram(), diagram)
            })
            .collect()
    }
}

/// Helper struct for building signatures in tests and benchmarks.
#[derive(Clone, Debug, Default)]
pub struct SignatureBuilder(Vec<GeneratorData>);

#[derive(Clone, Debug)]
pub struct GeneratorData(Generator, Diagram);

impl GeneratorInfo for GeneratorData {
    fn diagram(&self) -> &Diagram {
        &self.1
    }

    fn invertibility(&self) -> Option<Depth> {
        (self.0.dimension > 0).then_some(Depth::Infinite)
    }
}

impl Signature for SignatureBuilder {
    type Info = GeneratorData;

    fn generators(&self) -> impl Iterator<Item = Generator> {
        self.0.iter().map(|gd| gd.0)
    }

    fn generator_info(&self, g: Generator) -> Option<&GeneratorData> {
        self.0.get(g.id)
    }
}

impl SignatureBuilder {
    pub fn add_zero(&mut self) -> Diagram0 {
        let generator = Generator::new(self.0.len(), 0);
        let diagram = Diagram0::from(generator);
        self.0.push(GeneratorData(generator, diagram.into()));
        diagram
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
        self.0
            .push(GeneratorData(generator, diagram.clone().into()));
        Ok(diagram)
    }
}

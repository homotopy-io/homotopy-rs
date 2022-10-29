use crate::{
    diagram::NewDiagramError,
    label::{Label, Neighbourhood},
    Diagram, DiagramN, Generator,
};

pub trait GeneratorInfo {
    fn diagram(&self) -> &Diagram;
    fn is_invertible(&self) -> bool;
    fn neighbourhood(&self) -> &Neighbourhood;
}

pub trait Signature {
    type Info: GeneratorInfo;
    fn generators(&self) -> Vec<Generator>;
    fn generator_info(&self, g: Generator) -> Option<&Self::Info>;

    fn label_equiv(&self, x: Label, y: Label) -> bool {
        match (x, y) {
            (None, None) => true,
            (None, Some(_)) | (Some(_), None) => false,
            (Some((g_0, b_0, coord_0)), Some((g_1, b_1, coord_1))) => {
                if g_0 != g_1 || b_0 != b_1 {
                    return false;
                }
                self.generator_info(Generator::new(g_0, b_0.depth() + coord_0.len() + 1))
                    .unwrap()
                    .neighbourhood()
                    .equiv(b_0, &coord_0, &coord_1)
            }
        }
    }

    fn label_find(&self, x: Label) -> Label {
        match x {
            None => None,
            Some((g, b, coord)) => {
                let coord = self
                    .generator_info(Generator::new(g, b.depth() + coord.len() + 1))
                    .unwrap()
                    .neighbourhood()
                    .find(b, &coord);
                Some((g, b, coord))
            }
        }
    }
}

/// Helper struct for building signatures in tests and benchmarks.
#[derive(Clone, Debug, Default)]
pub struct SignatureBuilder(Vec<GeneratorData>);

#[derive(Clone, Debug)]
pub struct GeneratorData(Generator, Diagram, Neighbourhood);

impl SignatureBuilder {
    pub fn add_zero(&mut self) -> Diagram {
        let generator = Generator::new(self.0.len(), 0);
        self.0.push(GeneratorData(
            generator,
            generator.into(),
            Neighbourhood::default(),
        ));
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
        let (diagram, neighbourhood) = DiagramN::from_generator(generator, source, target, self)?;
        self.0.push(GeneratorData(
            generator,
            diagram.clone().into(),
            neighbourhood,
        ));
        Ok(diagram)
    }
}

impl GeneratorInfo for GeneratorData {
    fn diagram(&self) -> &Diagram {
        &self.1
    }

    fn is_invertible(&self) -> bool {
        self.0.dimension > 0
    }

    fn neighbourhood(&self) -> &Neighbourhood {
        &self.2
    }
}

impl Signature for SignatureBuilder {
    type Info = GeneratorData;

    fn generators(&self) -> Vec<Generator> {
        self.0.iter().map(|gd| gd.0).collect()
    }

    fn generator_info(&self, g: Generator) -> Option<&GeneratorData> {
        self.0.get(g.id)
    }
}

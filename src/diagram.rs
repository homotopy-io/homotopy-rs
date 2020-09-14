use std::rc::Rc;
use crate::common::*;
use crate::rewrite::*;

#[derive(PartialEq, Eq, Clone)]
pub enum Diagram {
    Diagram0(Generator),
    DiagramN(DiagramN),
}

impl Diagram {
    pub fn to_generator(&self) -> Option<Generator> {
        use Diagram::*;
        match self {
            Diagram0(g) => Some(*g),
            DiagramN(_) => None,
        }
    }

    pub fn to_n(&self) -> Option<&DiagramN> {
        use Diagram::*;
        match self {
            Diagram0(_) => None,
            DiagramN(d) => Some(d),
        }
    }

    pub fn dimension(&self) -> usize {
        use Diagram::*;
        match self {
            Diagram0(_) => 0,
            DiagramN(d) => d.dimension(),
        }
    }

    pub fn identity(&self) -> Diagram {
        Diagram::DiagramN(DiagramN::new_unsafe(self.clone(), vec![]))
    }

    pub fn embeds(&self, diagram: &Diagram, embedding: &[usize]) -> bool {
        use Diagram::*;
        match (self, diagram) {
            (Diagram0(g0), Diagram0(g1)) => g0 == g1,
            (Diagram0(_), DiagramN(_)) => false,
            (DiagramN(d), _) => d.embeds(diagram, embedding),
        }
    }

    fn rewrite_forward(self, rewrite: &Rewrite) -> Diagram {
        use Diagram::*;
        use Rewrite::*;
        match self {
            Diagram0(_) => match &rewrite {
                RewriteI => self,
                Rewrite0(_, target) => Diagram0(*target),
                RewriteN(_) => panic!(),
            },
            DiagramN(d) => match &rewrite {
                RewriteI => panic!(),
                Rewrite0(_, _) => panic!(),
                RewriteN(r) => DiagramN(d.rewrite_forward(r)),
            },
        }
    }

    fn rewrite_backward(self, rewrite: &Rewrite) -> Diagram {
        use Diagram::*;
        use Rewrite::*;
        match self {
            Diagram0(_) => match &rewrite {
                RewriteI => self,
                Rewrite0(source, _) => Diagram0(*source),
                RewriteN(_) => panic!(),
            },
            DiagramN(d) => match &rewrite {
                RewriteI => panic!(),
                Rewrite0(_, _) => panic!(),
                RewriteN(r) => DiagramN(d.rewrite_backward(r)),
            },
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct DiagramN(Rc<DiagramInternal>);

impl DiagramN {
    pub fn new_unsafe(source: Diagram, cospans: Vec<Cospan>) -> Self {
        DiagramN(Rc::new(DiagramInternal { source, cospans }))
    }

    pub fn dimension(&self) -> usize {
        self.0.source.dimension() + 1
    }

    pub fn source(&self) -> Diagram {
        self.0.source.clone()
    }

    pub fn target(&self) -> Diagram {
        let mut slice = self.0.source.clone();

        for cospan in &self.0.cospans {
            slice = slice.rewrite_forward(&cospan.forward);
            slice = slice.rewrite_backward(&cospan.backward);
        }

        slice
    }

    pub fn slices(&self) -> Vec<Diagram> {
        let mut slices = vec![self.0.source.clone()];

        for cospan in &self.0.cospans {
            slices.push(
                slices
                    .last()
                    .unwrap()
                    .clone()
                    .rewrite_forward(&cospan.forward),
            );
            slices.push(
                slices
                    .last()
                    .unwrap()
                    .clone()
                    .rewrite_backward(&cospan.backward),
            );
        }

        slices
    }

    pub fn slice(&self, index: SliceIndex) -> Option<Diagram> {
        match index {
            SliceIndex::Boundary(Boundary::Source) => Some(self.source()),
            SliceIndex::Boundary(Boundary::Target) => Some(self.target()),
            SliceIndex::Interior(height) => self.slices().drain(..).nth(height.to_int()),
        }
    }

    fn rewrite_forward(mut self, rewrite: &RewriteN) -> DiagramN {
        let diagram: &mut DiagramInternal = Rc::make_mut(&mut self.0);
        let mut offset: isize = 0;

        for cone in &rewrite.cones {
            let start = (cone.index as isize + offset) as usize;
            let stop = (cone.index as isize + cone.len() as isize + offset) as usize;
            diagram
                .cospans
                .splice(start..stop, vec![cone.target.clone()]);
            offset += (cone.len() - 1) as isize;
        }

        DiagramN(self.0)
    }

    fn rewrite_backward(mut self, rewrite: &RewriteN) -> DiagramN {
        let diagram: &mut DiagramInternal = Rc::make_mut(&mut self.0);
        let mut offset: isize = 0;

        for cone in &rewrite.cones {
            let start = (cone.index as isize + offset) as usize;
            let stop = (cone.index as isize + 1 + offset) as usize;
            diagram.cospans.splice(start..stop, cone.source.clone());
            offset -= (cone.len() - 1) as isize;
        }

        DiagramN(self.0)
    }

    pub fn cospans(&self) -> &[Cospan] {
        &self.0.cospans
    }

    pub fn embeds(&self, diagram: &Diagram, embedding: &[usize]) -> bool {
        use Diagram::*;

        let (regular, rest) = match embedding.split_first() {
            Some((regular, rest)) => (*regular, rest),
            None => (0, embedding),
        };

        let height = SliceIndex::Interior(Height::Regular(regular));

        let slice = match self.slice(height) {
            Some(slice) => slice,
            None => return false,
        };

        match diagram {
            Diagram0(_) => slice.embeds(diagram, rest),
            DiagramN(d) => {
                use std::cmp::Ordering::*;
                match d.dimension().cmp(&self.dimension()) {
                    Greater => false,
                    Less => slice.embeds(diagram, rest),
                    Equal => {
                        slice.embeds(&d.source(), rest)
                            && self.0.cospans.get(regular..d.size())
                                == Some(
                                    &d.0.cospans.iter().map(|c| c.pad(rest)).collect::<Vec<_>>(),
                                )
                    }
                }
            }
        }
    }

    pub fn size(&self) -> usize {
        self.0.cospans.len()
    }

    pub fn attach(
        &self,
        diagram: DiagramN,
        boundary: Boundary,
        embedding: &[usize],
    ) -> Option<DiagramN> {
        use Boundary::*;

        let depth = self.dimension().checked_sub(diagram.dimension())?;

        if depth == 0 {
            let cospans = diagram
                .cospans()
                .iter()
                .map(|c| c.pad(&embedding))
                .collect();

            match boundary {
                Source => {
                    let mut source = self.0.source.clone();

                    for cospan in self.0.cospans.iter().rev() {
                        source = source.rewrite_forward(&cospan.backward);
                        source = source.rewrite_backward(&cospan.forward);
                    }

                    Some(DiagramN::new_unsafe(source, cospans))
                }
                Target => Some(DiagramN::new_unsafe(self.0.source.clone(), cospans)),
            }
        } else {
            let source = match &self.0.source {
                Diagram::Diagram0(_) => panic!(),
                Diagram::DiagramN(s) => s,
            };

            match boundary {
                Source => {
                    let source = Diagram::DiagramN(source.attach(diagram, boundary, embedding)?);
                    let mut padding = vec![0; depth - 1];
                    padding.push(1);
                    let cospans = self.0.cospans.iter().map(|c| c.pad(&padding)).collect();
                    Some(DiagramN::new_unsafe(source, cospans))
                }
                Target => {
                    let source = Diagram::DiagramN(source.attach(diagram, boundary, embedding)?);
                    let cospans = self.0.cospans.clone(); // todo: pad depth
                    Some(DiagramN::new_unsafe(source, cospans))
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
struct DiagramInternal {
    source: Diagram,
    cospans: Vec<Cospan>,
}


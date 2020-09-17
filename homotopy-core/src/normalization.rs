use crate::common::*;
use crate::diagram::*;
use crate::rewrite::*;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, Clone)]
struct Degeneracy {
    trivial: Vec<usize>,
    slices: Vec<Degeneracy>,
}

impl Degeneracy {
    fn new(trivial: Vec<SingularHeight>, slices: Vec<Degeneracy>) -> Self {
        Degeneracy { trivial, slices }
    }

    fn zero() -> Self {
        Degeneracy {
            trivial: Vec::new(),
            slices: Vec::new(),
        }
    }

    fn singular_preimage(&self, i: SingularHeight) -> Range<SingularHeight> {
        for (count, trivial) in self.trivial.iter().enumerate() {
            if *trivial == i {
                return i-count .. i - count;
            } else if *trivial > i {
                return i - count .. i - count + 1;
            }
        }

        i - self.trivial.len() .. i - self.trivial.len() + 1
    }

    fn singular_image(&self, i: SingularHeight) -> SingularHeight {
        for (count, trivial) in self.trivial.iter().enumerate() {
            if i + count < *trivial {
                return i + count;
            }
        }
        i + self.trivial.len()
    }

    fn slice(&self, i: SingularHeight) -> Option<Degeneracy> {
        self.slices.get(i).cloned()
    }
}

#[derive(Debug, Clone)]
struct SinkArrow {
    source: Diagram,
    degeneracy: Degeneracy,
    rewrite: Rewrite,
}

impl SinkArrow {
    fn slice(&self, i: SingularHeight) -> Option<SinkArrow> {
        let degeneracy = self.degeneracy.slice(i)?;
        let source = self.source.to_n()?.slice(Height::Singular(i))?;
        let rewrite = self.rewrite.to_n()?.slice(self.degeneracy.singular_image(i));
        Some(SinkArrow { source, degeneracy, rewrite })
    }

    fn singular_preimage(&self, i: SingularHeight) -> Range<SingularHeight> {
        let range = self.rewrite.to_n().unwrap().singular_preimage(i);
        let start = self.degeneracy.singular_preimage(range.start);
        let end = self.degeneracy.singular_preimage(range.end);
        Range { start: start.start, end: end.end }
    }
}

struct Output {
    diagram: Diagram,
    factors: Vec<Rewrite>,
    degeneracy: Degeneracy,
}

// TODO: Homotopy construction will need singular normalisation

pub fn normalize(diagram: &Diagram) -> Diagram {
    let output = normalize_relative(diagram, &[]);
    output.diagram
}

fn normalize_relative(diagram: &Diagram, sink: &[SinkArrow]) -> Output {
    match diagram {
        Diagram::Diagram0(_) => Output {
            degeneracy: Degeneracy::zero(),
            factors: sink.iter().map(|input| input.rewrite.clone()).collect(),
            diagram: diagram.clone(),
        },
        Diagram::DiagramN(d) => normalize_recursive(d, sink),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Role {
    Forward,
    Backward,
    Slice(usize, SingularHeight),
}

fn normalize_recursive(diagram: &DiagramN, sink: &[SinkArrow]) -> Output {
    use Height::*;

    let slices: Vec<_> = diagram.slices().collect();
    let cospans = diagram.cospans();

    // Normalise the regular levels
    let mut regular_normalised: Vec<Output> = Vec::new();

    for i in 0..slices.len() + 1 {
        let slice = &slices[Regular(i).to_int()];
        regular_normalised.push(normalize_relative(slice, &[]));
    }

    // Collect the subproblems
    let mut subproblems: Vec<Vec<SinkArrow>> = (0..diagram.size()).map(|_| Vec::new()).collect();

    let mut roles: Vec<Vec<Role>> = (0..diagram.size()).map(|_| Vec::new()).collect();

    for i in 0..diagram.size() {
        let cospan = &cospans[i];
        let regular0 = &regular_normalised[i];
        let regular1 = &regular_normalised[i + 1];

        subproblems[i].push(SinkArrow {
            source: regular0.diagram.clone(),
            degeneracy: regular0.degeneracy.clone(),
            rewrite: cospan.forward.clone(),
        });

        roles[i].push(Role::Forward);

        subproblems[i].push(SinkArrow {
            source: regular1.diagram.clone(),
            degeneracy: regular1.degeneracy.clone(),
            rewrite: cospan.backward.clone(),
        });

        roles[i].push(Role::Backward);
    }

    for (i, input) in sink.iter().enumerate() {
        for target in 0..diagram.size() {
            for j in input.singular_preimage(target) {
                let slice = input.slice(j).unwrap();
                subproblems[target].push(slice);
                roles[target].push(Role::Slice(i, j));
            }
        }
    }

    // For each subproblem recursively call relative normalisation
    let mut degeneracies: Vec<Degeneracy> = Vec::new();
    let mut factor_slices: Vec<HashMap<Role, Rewrite>> = Vec::new();

    for (target, subproblem) in subproblems.iter().enumerate() {
        let slice = &slices[Singular(target).to_int()];
        let output = normalize_relative(slice, &subproblem);

        degeneracies.push(output.degeneracy);
        factor_slices.push(
            output
                .factors
                .into_iter()
                .zip(&roles[target])
                .map(|(factor, role)| (*role, factor))
                .collect(),
        );
    }

    // Build the normalized diagram
    let mut normalized_cospans: Vec<Cospan> = Vec::new();

    for i in 0..diagram.size() {
        let forward = factor_slices[i].remove(&Role::Forward).unwrap();
        let backward = factor_slices[i].remove(&Role::Backward).unwrap();
        normalized_cospans.push(Cospan { forward, backward });
    }

    // Assemble the factorisation rewrite slices
    let mut factors: Vec<Vec<Vec<Rewrite>>> = Vec::new();

    for (i, input) in sink.iter().enumerate() {
        let slices: Vec<Vec<Rewrite>> = (0..diagram.size())
            .map(|target| {
                input
                    .singular_preimage(target)
                    .map(|j| factor_slices[target].remove(&Role::Slice(i, j)).unwrap())
                    .collect()
            })
            .collect();

        factors.push(slices);
    }

    // Find all trivial heights
    let trivial: Vec<SingularHeight> = (0..diagram.size())
        .filter(|i| {
            normalized_cospans[*i].is_identity()
                && factors.iter().all(|factor| factor[*i].len() == 0)
        })
        .collect();

    // Remove the cospans at the trivial heights
    for i in trivial.iter().rev() {
        normalized_cospans.remove(*i);

        for factor in &mut factors {
            factor.remove(*i);
        }
    }

    // Assemble the result
    let normalized = DiagramN::new_unsafe(diagram.source(), normalized_cospans);
    let factors = factors
        .into_iter()
        .enumerate()
        .map(|(i, factor)| {
            RewriteN::from_slices(
                diagram.dimension(),
                sink[i].source.to_n().unwrap().cospans(),
                diagram.cospans(),
                factor,
            )
            .into()
        })
        .collect();
    let degeneracy = Degeneracy::new(trivial, degeneracies);
    Output {
        diagram: normalized.into(),
        factors: factors,
        degeneracy: degeneracy,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn degeneracy_singular_image() {
        let degeneracy = Degeneracy::new(vec![1, 4, 5], vec![Degeneracy::zero(); 15]);

        assert_eq!(degeneracy.singular_image(0), 0);
        assert_eq!(degeneracy.singular_image(1), 2);
        assert_eq!(degeneracy.singular_image(2), 3);
        assert_eq!(degeneracy.singular_image(3), 6);
        assert_eq!(degeneracy.singular_image(4), 7);
    }

    #[test]
    fn degeneracy_singular_preimage() {
        let degeneracy = Degeneracy::new(vec![1, 4, 5], vec![Degeneracy::zero(); 15]);

        assert_eq!(degeneracy.singular_preimage(0), 0..1);
        assert_eq!(degeneracy.singular_preimage(1), 1..1);
        assert_eq!(degeneracy.singular_preimage(2), 1..2);
        assert_eq!(degeneracy.singular_preimage(3), 2..3);
        assert_eq!(degeneracy.singular_preimage(6), 3..4);
        assert_eq!(degeneracy.singular_preimage(7), 4..5);
    }
}

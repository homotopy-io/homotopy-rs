use crate::common::*;
use crate::diagram::*;
use crate::rewrite::*;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct SinkArrow {
    source: Diagram,
    rewrite: Rewrite,
}

impl SinkArrow {
    fn slice(&self, i: SingularHeight) -> Option<SinkArrow> {
        let source = self.source.to_n()?.slice(Height::Singular(i))?;
        let rewrite = self.rewrite.to_n()?.slice(i);
        Some(SinkArrow { source, rewrite })
    }

    fn singular_preimage(&self, i: SingularHeight) -> Range<SingularHeight> {
        self.rewrite.to_n().unwrap().singular_preimage(i)
    }
}

pub fn normalize(diagram: &Diagram) -> Diagram {
    let diagram = normalize_regular(diagram);
    let (rewrite, _) = normalize_singular(&diagram, &[]);
    diagram.rewrite_backward(&rewrite)
}

pub fn normalize_regular(diagram: &Diagram) -> Diagram {
    // Zero-dimensional diagrams are already normalized.
    let diagram = match diagram {
        Diagram::Diagram0(_) => {
            return diagram.clone();
        }
        Diagram::DiagramN(d) => d,
    };

    //
    let slices: Vec<_> = diagram.slices().collect();
    let mut rewrites: Vec<Rewrite> = Vec::new();

    for i in 0..diagram.size() + 1 {
        let slice = &slices[Height::Regular(i).to_int()];
        let (rewrite, _) = normalize_singular(slice, &[]);
        rewrites.push(rewrite);
    }

    // Build cospans
    let mut cospans_normalized = Vec::new();
    let cospans = diagram.cospans();
    for i in 0..diagram.size() {
        let forward = Rewrite::compose(rewrites[i].clone(), cospans[i].forward.clone()).unwrap();
        let backward =
            Rewrite::compose(rewrites[i + 1].clone(), cospans[i].backward.clone()).unwrap();
        cospans_normalized.push(Cospan { forward, backward })
    }

    let source_normalized = diagram.source().rewrite_backward(&rewrites[0]);

    DiagramN::new_unsafe(source_normalized, cospans_normalized).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Role {
    Forward,
    Backward,
    Slice(usize, SingularHeight),
}

pub fn normalize_singular(diagram: &Diagram, sink: &[SinkArrow]) -> (Rewrite, Vec<Rewrite>) {
    let diagram = match diagram {
        Diagram::Diagram0(_) => {
            let factors = sink.iter().map(|_| Rewrite0::identity().into()).collect();
            let degeneracy = Rewrite0::identity().into();
            return (degeneracy, factors);
        }
        Diagram::DiagramN(d) => d,
    };

    use Height::*;

    // The diagram can not be normalised any further if one map in the sink is an identity rewrite.
    if sink.iter().any(|input| input.rewrite.is_identity()) {
        let degeneracy = Rewrite::identity(diagram.dimension());
        let factors = sink.iter().map(|input| input.rewrite.clone()).collect();
        return (degeneracy, factors);
    }

    let slices: Vec<_> = diagram.slices().collect();
    let cospans = diagram.cospans();

    // Collect the subproblems
    let mut subproblems: Vec<Vec<SinkArrow>> = (0..diagram.size()).map(|_| Vec::new()).collect();

    let mut roles: Vec<Vec<Role>> = (0..diagram.size()).map(|_| Vec::new()).collect();

    for i in 0..diagram.size() {
        let cospan = &cospans[i];

        subproblems[i].push(SinkArrow {
            source: slices[Regular(i).to_int()].clone(),
            rewrite: cospan.forward.clone(),
        });

        roles[i].push(Role::Forward);

        subproblems[i].push(SinkArrow {
            source: slices[Regular(i + 1).to_int()].clone(),
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
    let mut degeneracies: Vec<Rewrite> = Vec::new();
    let mut factor_slices: Vec<HashMap<Role, Rewrite>> = Vec::new();

    for (target, subproblem) in subproblems.iter().enumerate() {
        let slice = &slices[Singular(target).to_int()];
        let (degeneracy, factors) = normalize_singular(slice, &subproblem);

        degeneracies.push(degeneracy);
        factor_slices.push(
            factors
                .into_iter()
                .zip(&roles[target])
                .map(|(factor, role)| (*role, factor))
                .collect(),
        );
    }

    // Build the normalized diagram
    let mut normalized_cospans: Vec<Cospan> = Vec::new();

    for factor_slice in factor_slices.iter_mut() {
        let forward = factor_slice.remove(&Role::Forward).unwrap();
        let backward = factor_slice.remove(&Role::Backward).unwrap();
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
                && factors.iter().all(|factor| factor[*i].is_empty())
        })
        .collect();

    // Remove the cospans at the trivial heights
    for i in trivial.iter().rev() {
        normalized_cospans.remove(*i);

        for factor in &mut factors {
            factor.remove(*i);
        }
    }

    // Build the parallel degeneracy
    let degeneracy_parallel = RewriteN::from_slices(
        diagram.dimension(),
        &normalized_cospans,
        diagram.cospans(),
        degeneracies.iter().map(|d| vec![d.clone()]).collect(),
    );

    let degeneracy_simple =
        RewriteN::make_degeneracy(diagram.dimension(), &normalized_cospans, &trivial);

    let degeneracy = RewriteN::compose(degeneracy_simple, degeneracy_parallel).unwrap();

    // Assemble the result
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

    (degeneracy.into(), factors)
}

use crate::common::*;
use crate::diagram::*;
use crate::rewrite::*;
use std::collections::HashMap;
use std::convert::*;
use std::rc::Rc;

#[derive(Debug, Clone)]
enum Degeneracy {
    Identity(usize),
    Degeneracy(Vec<SingularHeight>, Vec<Rc<Degeneracy>>),
}

impl Degeneracy {
    fn new(dimension: usize, trivial: Vec<SingularHeight>, slices: Vec<Rc<Degeneracy>>) -> Self {
        if trivial.len() == 0 && slices.iter().all(|slice| slice.is_identity()) {
            Degeneracy::Identity(dimension)
        } else {
            Degeneracy::Degeneracy(trivial, slices)
        }
    }

    fn singular_preimage(&self, i: SingularHeight) -> Height {
        match self {
            Degeneracy::Identity(_) => Height::Singular(i),
            Degeneracy::Degeneracy(trivial, _) => {
                for (count, trivial) in trivial.iter().enumerate() {
                    if *trivial == i {
                        return Height::Regular(i - count);
                    } else if *trivial > i {
                        return Height::Singular(i - count);
                    }
                }

                Height::Singular(i - trivial.len())
            }
        }
    }

    fn is_identity(&self) -> bool {
        match self {
            Degeneracy::Identity(_) => true,
            Degeneracy::Degeneracy(_, _) => false,
        }
    }

    fn to_rewrite(&self, source: &Diagram, target: &Diagram) -> Rewrite {
        assert_eq!(source.dimension(), target.dimension());

        let (trivial, slices) = match self {
            Degeneracy::Identity(dimension) => return Rewrite::identity(*dimension),
            Degeneracy::Degeneracy(trivial, slices) => (trivial, slices),
        };

        let source: &DiagramN = source.try_into().unwrap();
        let target: &DiagramN = target.try_into().unwrap();

        let rewrite_simple =
            RewriteN::make_degeneracy(source.dimension(), source.cospans(), &trivial);
        let middle = source.clone().rewrite_forward(&rewrite_simple);
        let middle_slices: Vec<_> = middle.slices().collect();
        let target_slices: Vec<_> = target.slices().collect();

        let rewrite_slices: Vec<_> = slices
            .iter()
            .enumerate()
            .map(|(i, slice)| {
                let middle_slice = &middle_slices[Height::Singular(i).to_int()];
                let target_slice = &target_slices[Height::Singular(i).to_int()];
                vec![slice.to_rewrite(middle_slice, target_slice)]
            })
            .collect();
        let rewrite_parallel = RewriteN::from_slices(
            middle.dimension(),
            middle.cospans(),
            target.cospans(),
            rewrite_slices,
        );

        RewriteN::compose(rewrite_simple, rewrite_parallel)
            .unwrap()
            .into()
    }

    fn slice_into(&self, target_height: SingularHeight) -> Rc<Self> {
        match self {
            Degeneracy::Identity(dimension) => {
                assert!(*dimension > 0);
                Rc::new(Degeneracy::Identity(dimension - 1))
            }
            Degeneracy::Degeneracy(_, slices) => slices[target_height].clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct SinkArrow {
    source: Diagram,
    degeneracy: Rc<Degeneracy>,
    middle: Diagram,
    rewrite: Rewrite,
}

impl SinkArrow {
    fn is_identity(&self) -> bool {
        self.degeneracy.is_identity() && self.rewrite.is_identity()
    }

    fn singular_preimage(&self, target_height: SingularHeight) -> Vec<SingularHeight> {
        let mut preimage = Vec::new();
        let rewrite = self.rewrite.to_n().unwrap();

        for middle_height in rewrite.singular_preimage(target_height) {
            match self.degeneracy.singular_preimage(middle_height) {
                Height::Singular(source_height) => preimage.push(source_height),
                Height::Regular(_) => {}
            }
        }

        preimage
    }

    fn slices_into(&self, target_height: SingularHeight) -> Vec<(SingularHeight, SinkArrow)> {
        let rewrite = self.rewrite.to_n().unwrap();
        let source = <&DiagramN>::try_from(&self.source).ok().unwrap();
        let middle = <&DiagramN>::try_from(&self.middle).ok().unwrap();
        let mut slices = Vec::new();

        for middle_height in rewrite.singular_preimage(target_height) {
            let source_height = match self.degeneracy.singular_preimage(middle_height) {
                Height::Singular(source_height) => source_height,
                Height::Regular(_) => continue,
            };

            let slice_rewrite = rewrite.slice(middle_height);
            let slice_middle = middle.slice(Height::Singular(middle_height)).unwrap();
            let slice_degeneracy = self.degeneracy.slice_into(middle_height);
            let slice_source = source.slice(Height::Singular(source_height)).unwrap();
            slices.push((
                source_height,
                SinkArrow {
                    source: slice_source,
                    rewrite: slice_rewrite,
                    middle: slice_middle,
                    degeneracy: slice_degeneracy,
                },
            ));
        }

        slices
    }
}

pub fn normalize(diagram: &Diagram) -> Diagram {
    // TODO: Cache by hash

    fn normalize_regular(regular: &Diagram) -> (Diagram, Rc<Degeneracy>) {
        let output = normalize_relative(regular, &[], normalize_regular);
        (output.diagram, output.degeneracy)
    }

    normalize_relative(diagram, &[], normalize_regular).diagram
}

pub fn normalize_singular(diagram: &Diagram) -> Rewrite {
    fn normalize_regular(regular: &Diagram) -> (Diagram, Rc<Degeneracy>) {
        let degeneracy = Degeneracy::Identity(regular.dimension());
        (regular.clone(), Rc::new(degeneracy))
    }

    let output = normalize_relative(diagram, &[], normalize_regular);
    output.degeneracy.to_rewrite(&output.diagram, diagram)
}

struct Output {
    factors: Vec<Rewrite>,
    degeneracy: Rc<Degeneracy>,
    diagram: Diagram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Role {
    Forward,
    Backward,
    Slice(usize, SingularHeight),
}

fn normalize_relative<R>(diagram: &Diagram, sink: &[SinkArrow], mut normalize_regular: R) -> Output
where
    R: FnMut(&Diagram) -> (Diagram, Rc<Degeneracy>) + Copy,
{
    let diagram = match diagram {
        Diagram::Diagram0(_) => {
            let factors = sink.iter().map(|input| input.rewrite.clone()).collect();
            return Output {
                factors,
                degeneracy: Rc::new(Degeneracy::Identity(0)),
                diagram: diagram.clone(),
            };
        }
        Diagram::DiagramN(d) => d,
    };

    use Height::*;

    // Normalize the regular levels
    let slices: Vec<_> = diagram.slices().collect();
    let regular_normalized: Vec<(Diagram, Rc<Degeneracy>)> = (0..(diagram.size() + 1))
        .map(|i| normalize_regular(&slices[Height::Regular(i).to_int()]))
        .collect();

    // The diagram can not be normalised any further if one map in the sink is an identity rewrite.
    if sink.iter().any(|input| input.is_identity()) {
        return Output {
            degeneracy: Rc::new(Degeneracy::Identity(diagram.dimension())),
            factors: sink.iter().map(|input| input.rewrite.clone()).collect(),
            diagram: diagram.clone().into(),
        };
    }

    let slices: Vec<_> = diagram.slices().collect();
    let cospans = diagram.cospans();

    // Collect the subproblems
    let mut subproblems: Vec<Vec<SinkArrow>> = (0..diagram.size()).map(|_| Vec::new()).collect();
    let mut roles: Vec<Vec<Role>> = (0..diagram.size()).map(|_| Vec::new()).collect();

    for i in 0..diagram.size() {
        let cospan = &cospans[i];

        subproblems[i].push(SinkArrow {
            source: regular_normalized[i].0.clone(),
            degeneracy: regular_normalized[i].1.clone(),
            middle: slices[Regular(i).to_int()].clone(),
            rewrite: cospan.forward.clone(),
        });

        roles[i].push(Role::Forward);

        subproblems[i].push(SinkArrow {
            source: regular_normalized[i + 1].0.clone(),
            degeneracy: regular_normalized[i + 1].1.clone(),
            middle: slices[Regular(i + 1).to_int()].clone(),
            rewrite: cospan.backward.clone(),
        });

        roles[i].push(Role::Backward);
    }

    for (i, input) in sink.iter().enumerate() {
        for target in 0..diagram.size() {
            for (j, slice) in input.slices_into(target) {
                subproblems[target].push(slice);
                roles[target].push(Role::Slice(i, j));
            }
        }
    }

    // For each subproblem recursively call relative normalisation
    let mut singular_degeneracies: Vec<Rc<Degeneracy>> = Vec::new();
    let mut factor_slices: Vec<HashMap<Role, Rewrite>> = Vec::new();

    for (target, subproblem) in subproblems.iter().enumerate() {
        let slice = &slices[Singular(target).to_int()];
        let output = normalize_relative(slice, &subproblem, normalize_regular);

        singular_degeneracies.push(output.degeneracy);
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
                    .into_iter()
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

    // Build the degeneracy
    let degeneracy = Degeneracy::new(diagram.dimension(), trivial, singular_degeneracies);

    // Assemble the result
    let diagram_normalized =
        DiagramN::new_unsafe(regular_normalized[0].0.clone(), normalized_cospans);

    let factors = factors
        .into_iter()
        .enumerate()
        .map(|(i, factor)| {
            RewriteN::from_slices(
                diagram.dimension(),
                <&DiagramN>::try_from(&sink[i].source).unwrap().cospans(),
                diagram.cospans(),
                factor,
            )
            .into()
        })
        .collect();

    Output {
        factors,
        degeneracy: Rc::new(degeneracy),
        diagram: diagram_normalized.into(),
    }
}

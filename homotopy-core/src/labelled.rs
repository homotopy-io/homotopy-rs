use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    convert::TryInto,
    iter::FromIterator,
};

use thiserror::Error;

use crate::{
    graph::GraphBuilder,
    rewrite::{Cone, Rewrite0, RewriteN},
    Diagram, Generator, Height, Rewrite, SliceIndex,
};

type Coordinate<T> = Vec<T>;
pub type Simplex<T> = Vec<Coordinate<T>>;
pub type Label = (Generator, Coordinate<Height>);
pub type Complex<T> = Vec<HashSet<Simplex<T>>>;

fn strip_boundaries(coord: &[SliceIndex]) -> Option<Coordinate<Height>> {
    coord.iter().fold(Some(vec![]), |acc, component| {
        if let SliceIndex::Interior(h) = component {
            acc.map(|mut c| {
                c.push(*h);
                c
            })
        } else {
            None
        }
    })
}

pub fn simplices<T>(diagram: Diagram) -> Vec<T>
where
    T: Clone + FromIterator<Simplex<Height>> + IntoIterator<Item = Simplex<Height>>,
{
    let dimension = diagram.dimension();
    let mut graph = GraphBuilder::<_, Diagram, _>::new(vec![], diagram);
    for _d in 0..dimension {
        graph = graph
            .explode(|i, mut coords| {
                coords.push(i);
                coords
            })
            .unwrap();
    }
    // fully exploded graph
    let mut simplices: Vec<T> = vec![graph
        .nodes
        .iter()
        .filter_map(|n| {
            let coord = &n.0;
            strip_boundaries(coord).map(|c| vec![c])
        })
        .collect()];
    let nodes = graph.nodes;
    let edges = {
        // compute the transitive closure of the edge relation to get composite simplices
        let mut reachability: Vec<(usize, usize)> =
            graph.edges.iter().map(|(s, t, _)| (*s, *t)).collect();
        for intermediate in 0..nodes.len() {
            for s in 0..nodes.len() {
                for t in 0..nodes.len() {
                    if !reachability.contains(&(s, t))
                        && reachability.contains(&(s, intermediate))
                        && reachability.contains(&(intermediate, t))
                    {
                        reachability.push((s, t));
                    }
                }
            }
        }
        reachability
    };
    for d in 1..=dimension {
        simplices.push(
            edges
                .iter()
                .filter_map(|e| {
                    if let (Some(s), Some(t)) = (
                        strip_boundaries(&nodes[e.0].0),
                        strip_boundaries(&nodes[e.1].0),
                    ) {
                        Some(
                            simplices[d - 1]
                                .clone()
                                .into_iter()
                                .filter_map(move |simplex| {
                                    if simplex.first().unwrap() == &t {
                                        Some({
                                            let mut composite = simplex;
                                            composite.insert(0, s.clone());
                                            composite
                                        })
                                    } else if simplex.last().unwrap() == &s {
                                        Some({
                                            let mut composite = simplex;
                                            composite.push(t.clone());
                                            composite
                                        })
                                    } else {
                                        None
                                    }
                                }),
                        )
                    } else {
                        None
                    }
                })
                .flatten()
                .collect(),
        );
    }

    simplices
}

pub(crate) fn cone_simplices<T>(cone: Cone) -> (Vec<T>, BTreeMap<Simplex<usize>, Generator>)
where
    T: Clone + FromIterator<Simplex<usize>> + IntoIterator<Item = Simplex<usize>>,
{
    let dimension = cone.internal.target.forward.dimension() + 1;
    let rewrite: Rewrite = RewriteN::new(dimension, vec![cone]).into();
    let mut graph = GraphBuilder::<_, Option<Generator>, _>::new(vec![0], vec![1], rewrite);
    for _d in 0..dimension {
        graph = graph
            .explode(|i, mut coords| {
                coords.push(i);
                coords
            })
            .unwrap();
    }
    let nodes: Vec<Generator> = graph.nodes.iter().filter_map(|(_, g)| *g).collect();
    assert_eq!(graph.nodes.len(), nodes.len());
    let edges = {
        // compute the transitive closure of the edge relation to get composite simplices
        let mut reachability: Vec<(usize, usize)> =
            graph.edges.iter().map(|(s, t, _)| (*s, *t)).collect();
        for intermediate in 0..graph.nodes.len() {
            for s in 0..graph.nodes.len() {
                for t in 0..graph.nodes.len() {
                    if !reachability.contains(&(s, t))
                        && reachability.contains(&(s, intermediate))
                        && reachability.contains(&(intermediate, t))
                    {
                        reachability.push((s, t));
                    }
                }
            }
        }
        reachability
    };
    let mut simplices: Vec<T> = vec![graph.nodes.iter().map(|n| vec![n.0.clone()]).collect()];
    let generators: BTreeMap<Simplex<usize>, Generator> = graph
        .nodes
        .iter()
        .filter_map(|n| n.1.map(|g| (vec![n.0.clone()], g)))
        .collect();
    for d in 1..=dimension + 1 {
        simplices.push(
            edges
                .iter()
                .flat_map(|e| {
                    let s = &graph.nodes[e.0].0;
                    let t = &graph.nodes[e.1].0;
                    simplices[d - 1]
                        .clone()
                        .into_iter()
                        .filter_map(move |simplex| {
                            if simplex.first().unwrap() == t {
                                Some({
                                    let mut composite = simplex;
                                    composite.insert(0, s.clone());
                                    composite
                                })
                            } else if simplex.last().unwrap() == s {
                                Some({
                                    let mut composite = simplex;
                                    composite.push(t.clone());
                                    composite
                                })
                            } else {
                                None
                            }
                        })
                })
                .collect(),
        );
    }

    (simplices, generators)
}

// seek to label target
// pub(crate) fn propagate_forward(cone: &Cone) -> Result<Cone, LabelError> {
//     // this cone is not yet labelled
//     assert!(cone.label.is_none());
//     // source cospans of cone are labelled
//     assert!(cone
//         .internal
//         .source
//         .iter()
//         .all(|s| if s.forward.dimension() == 0 {
//             assert!(s.backward.dimension() == 0);
//             let f: &Rewrite0 = (&s.forward).try_into().unwrap();
//             let b: &Rewrite0 = (&s.backward).try_into().unwrap();
//
//             f.1.is_some() && b.1.is_some()
//         } else {
//             let f: &RewriteN = (&s.forward).try_into().unwrap();
//             let b: &RewriteN = (&s.backward).try_into().unwrap();
//             let has_label = |c: &Cone| -> bool { c.label.is_some() };
//             f.cones().iter().all(has_label) && b.cones().iter().all(has_label)
//         }));
//
//     let (simplices, generators): (Complex<usize>, BTreeMap<Simplex<usize>, Generator>) =
//         cone_simplices(cone.clone());
//     // optimisation: explode Cone -> Vec<Cone> by unique singular contents
//
//     let mut label: BTreeMap<Simplex<usize>, Label> = generators
//         .into_iter()
//         .map(|(s, g)| (s, g.label()))
//         .collect();
//     // 0-simplices already labelled
//
//     // need labels from source of cone
//     // the following is not precise enough: we need the full labelled simplicial complex of the
//     // source diagram, including all identities
//     for (i, cospan) in cone.internal.source.iter().enumerate() {
//         match (cospan.forward.dimension(), cospan.backward.dimension()) {
//             (0, 0) => {
//                 let f: &Rewrite0 = (&cospan.forward).try_into().unwrap();
//                 let b: &Rewrite0 = (&cospan.backward).try_into().unwrap();
//                 label.insert(
//                     vec![vec![0, 2 * i], vec![0, 2 * i + 1]],
//                     f.1.clone().unwrap(),
//                 );
//                 label.insert(
//                     vec![vec![0, 2 * i + 2], vec![0, 2 * i + 1]],
//                     b.1.clone().unwrap(),
//                 );
//             }
//             (n, m) if n == m => {
//                 let f: &RewriteN = (&cospan.forward).try_into().unwrap();
//                 let b: &RewriteN = (&cospan.backward).try_into().unwrap();
//                 for c in f.cones() {
//                     for (simplex, l) in c.label.as_ref().unwrap().iter() {
//                         let mut newsimplex = simplex.clone();
//                         for coord in &mut newsimplex {
//                             coord.insert(0, 2 * i);
//                             coord.insert(0, 0);
//                         }
//                         label.insert(newsimplex, l.clone());
//                     }
//                 }
//                 for c in b.cones() {
//                     for (simplex, l) in c.label.as_ref().unwrap().iter() {
//                         let mut newsimplex = simplex.clone();
//                         for coord in &mut newsimplex {
//                             coord.insert(0, 2 * i + 1);
//                             coord.insert(0, 0);
//                         }
//                         label.insert(newsimplex, l.clone());
//                     }
//                 }
//             }
//             _ => panic!("Malformed cone"),
//         }
//     }
//
//     // find degenerate 1-simplices
//     let mut degenerate_edges: HashMap<Simplex<usize>, Label> = Default::default();
//     let mut unlabelled_edges: HashSet<Simplex<usize>> = Default::default();
//     for edge in &simplices[1] {
//         let s = vec![edge[0].clone()];
//         let t = vec![edge[1].clone()];
//         if label[&s] == label[&t] {
//             // this edge is degenerate
//             label.insert(edge.clone(), label[&s].clone());
//             degenerate_edges.insert(edge.clone(), label[&s].clone());
//         } else if label.get(edge).is_none() {
//             unlabelled_edges.insert(edge.clone());
//         }
//     }
//
//     println!("initial labelling data: {:?}", &label);
//
//     // // remove all simplices not touching the cone tip
//     // // (no longer needed now the degenerate 1-simplices have been found)
//     // for nsimplices in &mut simplices {
//     //     nsimplices.retain(|simplex| simplex.last().unwrap().iter().all(|x| *x == 1));
//     // }
//     // label.retain(|simplex, _| simplex.last().unwrap().iter().all(|x| *x == 1));
//
//     // optimisation: A* search instead of UCS
//     let mut frontier: VecDeque<Simplex<usize>> = simplices[2..]
//         .iter()
//         .map(|nsimplices| nsimplices.iter().cloned().collect::<Vec<_>>())
//         .collect::<Vec<_>>()
//         .concat()
//         .into_iter()
//         .collect();
//     // next: 1-simplices
//     // for each 1-simplex, if its source and target 0 simplices are equal, label the 1-simplex with
//     // the same label
//     // for an n-simplex, if all its n-1-faces have the same label, label this n-simplex with that
//     // label also
//     // TODO
//
//     // next: find the degenerate simplices
//     // only look at atomic 1-simplices - if a n-simplex has k-degrees of degeneracy, this
//     // manifests as k of its generating 1-simplices being degenerate 1-simplices
//     let mut stalled = 0;
//     while stalled < frontier.len() {
//         let simplex = frontier.pop_front().unwrap();
//
//         // hunch: only degenerate along the initial or final 1-simplex
//         let start_edge = &simplex[0..=1];
//         let end_face = &simplex[1..];
//         let first_composite = [&start_edge[..1], &end_face[1..]].concat();
//
//         let end_edge = &simplex[simplex.len() - 2..];
//         let start_face = &simplex[0..=simplex.len() - 2];
//         let last_composite = [&start_face[0..start_face.len() - 1], &end_edge[1..]].concat();
//         if degenerate_edges.get(start_edge).is_some()
//             && (label.get(end_face).is_some() || label.get(&first_composite).is_some())
//         {
//             let l = label
//                 .get(end_face)
//                 .or_else(|| label.get(&first_composite))
//                 .unwrap()
//                 .clone();
//
//             // label the volume
//             if let Some(existing) = label.get(&simplex) {
//                 if existing != &l {
//                     return Err(LabelError::Inconsistent);
//                 }
//             } else {
//                 label.insert(simplex.clone(), l.clone());
//             }
//
//             // label the composite face
//             if label.get(end_face).is_some() {
//                 if let Some(existing) = label.get(&first_composite) {
//                     if existing != &l {
//                         return Err(LabelError::Inconsistent);
//                     }
//                 } else {
//                     label.insert(first_composite.clone(), l.clone());
//                 }
//                 unlabelled_edges.remove(&first_composite);
//             }
//
//             // label the end face
//             if label.get(&first_composite).is_some() {
//                 if let Some(existing) = label.get(end_face) {
//                     if existing != &l {
//                         return Err(LabelError::Inconsistent);
//                     }
//                 } else {
//                     label.insert(end_face.to_vec(), l);
//                 }
//                 unlabelled_edges.remove(end_face);
//             }
//
//             stalled = 0;
//         }
//         if degenerate_edges.get(end_edge).is_some()
//             && (label.get(start_face).is_some() || label.get(&last_composite).is_some())
//         {
//             let l = label
//                 .get(start_face)
//                 .or_else(|| label.get(&last_composite))
//                 .unwrap()
//                 .clone();
//
//             // label the volume
//             if let Some(existing) = label.get(&simplex) {
//                 if existing != &l {
//                     return Err(LabelError::Inconsistent);
//                 }
//             } else {
//                 label.insert(simplex.clone(), l.clone());
//             }
//
//             // label the composite face
//             if label.get(start_face).is_some() {
//                 if let Some(existing) = label.get(&last_composite) {
//                     if existing != &l {
//                         return Err(LabelError::Inconsistent);
//                     }
//                 } else {
//                     label.insert(last_composite.clone(), l.clone());
//                 }
//                 unlabelled_edges.remove(&last_composite);
//             }
//
//             // label the start face
//             if label.get(&last_composite).is_some() {
//                 if let Some(existing) = label.get(start_face) {
//                     if existing != &l {
//                         return Err(LabelError::Inconsistent);
//                     }
//                 } else {
//                     label.insert(start_face.to_vec(), l);
//                 }
//                 unlabelled_edges.remove(start_face);
//             }
//
//             stalled = 0;
//         }
//         if (degenerate_edges.get(start_edge).is_none()
//             || (label.get(end_face).is_none() && label.get(&first_composite).is_none()))
//             && (degenerate_edges.get(end_edge).is_none()
//                 || (label.get(start_face).is_none() && label.get(&last_composite).is_none()))
//         {
//             frontier.push_back(simplex);
//             stalled += 1;
//         }
//     }
//
//     if unlabelled_edges.is_empty() && frontier.is_empty() {
//         // keep only the labels mentioning the cone tip
//         label.retain(|simplex, _| simplex.last().unwrap().iter().all(|x| *x == 1));
//         cone.label(label)
//     } else {
//         Err(LabelError::Incomplete)
//     }
// }

#[derive(Debug, Error)]
pub enum LabelError {
    #[error("inconsistent labels were found")]
    Inconsistent,

    #[error("unable to label all simplices")]
    Incomplete,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        common::Boundary, examples, rewrite, rewrite::Cospan, signature::SignatureBuilder, DiagramN,
    };

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_cone_simplices_scalar() {
        let scalar: DiagramN = examples::scalar().1;
        let forward: RewriteN = scalar.cospans()[0].clone().forward.try_into().unwrap();
        let cone = forward.cones()[0].clone();
        let simplices: Vec<HashSet<Simplex<usize>>> = cone_simplices(cone).0;
        let a: Vec<usize> = vec![0, 0];

        let b: Vec<usize> = vec![1, 0];
        let c: Vec<usize> = vec![1, 1];
        let d: Vec<usize> = vec![1, 2];
        assert_eq!(
            vec![
                vec![a.clone()],
                vec![b.clone()],
                vec![c.clone()],
                vec![d.clone()],
            ]
            .into_iter()
            .collect::<HashSet<_>>(),
            simplices[0]
        );
        assert_eq!(
            vec![
                vec![a.clone(), b.clone()],
                vec![a.clone(), d.clone()],
                vec![b.clone(), c.clone()],
                vec![d.clone(), c.clone()],
                // composite 1-simplices
                vec![a.clone(), c.clone()],
            ]
            .into_iter()
            .collect::<HashSet<_>>(),
            simplices[1]
        );
        assert_eq!(
            vec![vec![a.clone(), b, c.clone()], vec![a, d, c],]
                .into_iter()
                .collect::<HashSet<_>>(),
            simplices[2]
        );
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_cone_simplices_monoid() {
        let monoid: DiagramN = examples::two_monoid().1;
        let forward: RewriteN = monoid.cospans()[0].clone().forward.try_into().unwrap();
        let cone = forward.cones()[0].clone();
        let simplices: Vec<HashSet<Simplex<usize>>> = cone_simplices(cone).0;
        let a: Vec<usize> = vec![0, 0];
        let b: Vec<usize> = vec![0, 1];
        let c: Vec<usize> = vec![0, 2];
        let d: Vec<usize> = vec![0, 3];
        let e: Vec<usize> = vec![0, 4];

        let f: Vec<usize> = vec![1, 0];
        let g: Vec<usize> = vec![1, 1];
        let h: Vec<usize> = vec![1, 2];
        assert_eq!(
            vec![
                vec![a.clone()],
                vec![b.clone()],
                vec![c.clone()],
                vec![d.clone()],
                vec![e.clone()],
                vec![f.clone()],
                vec![g.clone()],
                vec![h.clone()],
            ]
            .into_iter()
            .collect::<HashSet<_>>(),
            simplices[0]
        );
        assert_eq!(
            vec![
                vec![a.clone(), b.clone()],
                vec![c.clone(), b.clone()],
                vec![c.clone(), d.clone()],
                vec![e.clone(), d.clone()],
                vec![f.clone(), g.clone()],
                vec![h.clone(), g.clone()],
                vec![a.clone(), f.clone()],
                vec![b.clone(), g.clone()],
                vec![d.clone(), g.clone()],
                vec![e.clone(), h.clone()],
                // composite 1-simplices
                vec![a.clone(), g.clone()],
                vec![c.clone(), g.clone()],
                vec![e.clone(), g.clone()],
            ]
            .into_iter()
            .collect::<HashSet<_>>(),
            simplices[1]
        );
        assert_eq!(
            vec![
                vec![a.clone(), b.clone(), g.clone()],
                vec![a, f, g.clone()],
                vec![c.clone(), b, g.clone()],
                vec![c, d.clone(), g.clone()],
                vec![e.clone(), d, g.clone()],
                vec![e, h, g],
            ]
            .into_iter()
            .collect::<HashSet<_>>(),
            simplices[2]
        );
    }

    // #[test]
    // fn test_propagate_forward_f_id() {
    //     let x_gen = Generator::new(0, 0);
    //     let mut sig = SignatureBuilder::new();
    //
    //     let x = sig.add_zero();
    //     let f = sig.add(x.clone(), x.clone()).unwrap();
    //     let f_id = DiagramN::new_unsafe(
    //         x,
    //         vec![
    //             f.cospans()[0].clone(),
    //             Cospan {
    //                 forward: Rewrite::Rewrite0(rewrite::Rewrite0(None, Some(x_gen.label()))),
    //                 backward: Rewrite::Rewrite0(rewrite::Rewrite0(None, Some(x_gen.label()))),
    //             },
    //         ],
    //     );
    //     assert_ne!(f, f_id);
    //     let contraction = f_id
    //         .identity()
    //         .contract(&Boundary::Target.into(), &[], 0, None, &sig)
    //         .unwrap();
    //     let contraction_rewrite: &RewriteN =
    //         (&contraction.cospans()[0].forward).try_into().unwrap();
    //     let cone: &Cone = &contraction_rewrite.cones()[0];
    //     // dbg!(propagate_forward(cone).unwrap());
    // }
}

// use std::ops::Range;

// use itertools::Itertools;

// use crate::{
//     factorization::factorize, monotone::Monotone, rewrite::Cone, Cospan, Diagram, DiagramN, Height,
//     Rewrite, Rewrite0, RewriteN,
// };

// mod monotone {
//     use itertools::Itertools;

//     use crate::monotone::Monotone;

//     // Given a cospan a -> 1 <- b in ∆, return an antipushout span a <-h- s -k-> b.
//     pub fn antipushout_base(a: usize, b: usize) -> Vec<(Monotone, Monotone)> {
//         match (a, b) {
//             (0, 1) | (1, 0) => vec![(vec![].into(), vec![].into())],
//             (0, _) | (_, 0) => vec![],
//             (1, b) => vec![(vec![0; b].into(), (0..b).collect())],
//             (a, 1) => vec![((0..a).collect(), vec![0; a].into())],
//             (a, b) => {
//                 let mut antipushouts = vec![];
//                 for (mut h, mut k) in antipushout_base(a - 1, b) {
//                     h.push(a - 1);
//                     k.push(b - 1);
//                     antipushouts.push((h, k));
//                 }
//                 for (mut h, mut k) in antipushout_base(a, b - 1) {
//                     h.push(a - 1);
//                     k.push(b - 1);
//                     antipushouts.push((h, k));
//                 }
//                 antipushouts
//             }
//         }
//     }

//     // Given a cospan a ―f-> t <-g— b in ∆, return an antipushout span a <-h— s —k-> b.
//     pub fn antipushout(
//         f: &Monotone,
//         g: &Monotone,
//         target_size: usize,
//     ) -> Vec<(Monotone, Monotone)> {
//         let f_preimages = (0..target_size).map(|j| f.preimage(j)).collect_vec();
//         let g_preimages = (0..target_size).map(|j| g.preimage(j)).collect_vec();

//         (0..target_size)
//             .map(|j| {
//                 let n = f_preimages[j].len();
//                 let m = g_preimages[j].len();
//                 antipushout_base(n, m)
//             })
//             .multi_cartesian_product()
//             .map(|components| {
//                 let mut h = vec![];
//                 let mut k = vec![];
//                 for (j, (h_j, k_j)) in components.into_iter().enumerate() {
//                     let f_preimage = &f_preimages[j];
//                     let g_preimage = &g_preimages[j];
//                     h.extend(h_j.slices().map(|i| i + f_preimage.start));
//                     k.extend(k_j.slices().map(|i| i + g_preimage.start));
//                 }
//                 (h.into(), k.into())
//             })
//             .collect()
//     }
// }

// /// Given `Rewrite`s A -f> T <g- B, find some `Rewrite`s A <h- S -k> B such that the square is a pushout.
// #[allow(clippy::many_single_char_names)]
// pub fn antipushout(
//     a: &Diagram,
//     b: &Diagram,
//     t: &Diagram,
//     f: &Rewrite,
//     g: &Rewrite,
// ) -> Vec<(Diagram, Rewrite, Rewrite)> {
//     match (a, b, t, f, g) {
//         (
//             Diagram::Diagram0(a),
//             Diagram::Diagram0(b),
//             Diagram::Diagram0(_),
//             Rewrite::Rewrite0(_),
//             Rewrite::Rewrite0(_),
//         ) => {
//             if a != b && a.dimension == b.dimension {
//                 vec![]
//             } else {
//                 let s = std::cmp::min_by_key(*a, *b, |g| g.dimension);
//                 vec![(
//                     s.into(),
//                     Rewrite0::new(s, *a, todo!()).into(),
//                     Rewrite0::new(s, *b, todo!()).into(),
//                 )]
//             }
//         }
//         (
//             Diagram::DiagramN(a),
//             Diagram::DiagramN(b),
//             Diagram::DiagramN(t),
//             Rewrite::RewriteN(f),
//             Rewrite::RewriteN(g),
//         ) => {
//             assert_eq!(f.dimension(), a.dimension());
//             assert_eq!(f.dimension(), t.dimension());
//             assert_eq!(g.dimension(), b.dimension());
//             assert_eq!(g.dimension(), t.dimension());

//             let f_mono = f.singular_monotone(a.size());
//             let g_mono = g.singular_monotone(b.size());

//             monotone::antipushout(&f_mono, &g_mono, t.size())
//                 .iter()
//                 .flat_map(|(h_mono, k_mono)| {
//                     assert_eq!(h_mono.len(), k_mono.len());
//                     if h_mono.is_empty() {
//                         // If the source is empty, construct the trivial rewrites.
//                         let s = t.source().identity();

//                         let h = RewriteN::from_slices_unsafe(
//                             s.dimension(),
//                             &[],
//                             a.cospans(),
//                             todo!(),
//                             vec![vec![]; a.size()],
//                         );

//                         let k = RewriteN::from_slices_unsafe(
//                             s.dimension(),
//                             &[],
//                             b.cospans(),
//                             todo!(),
//                             vec![vec![]; b.size()],
//                         );

//                         vec![(s.into(), h.into(), k.into())]
//                     } else {
//                         std::iter::zip(h_mono.slices(), k_mono.slices())
//                             .map(|(ai, bi)| {
//                                 assert_eq!(f_mono[ai], g_mono[bi]);
//                                 antipushout(
//                                     &a.slice(Height::Singular(ai)).unwrap(),
//                                     &b.slice(Height::Singular(bi)).unwrap(),
//                                     &t.slice(Height::Singular(f_mono[ai])).unwrap(),
//                                     &f.slice(ai),
//                                     &g.slice(bi),
//                                 )
//                             })
//                             .multi_cartesian_product()
//                             .map(|spans| {
//                                 let s_slices =
//                                     spans.iter().map(|span| span.0.clone()).collect_vec();
//                                 let h_slices =
//                                     spans.iter().map(|span| span.1.clone()).collect_vec();
//                                 let k_slices =
//                                     spans.iter().map(|span| span.2.clone()).collect_vec();

//                                 let s = construct_source(a, h_mono, &h_slices, &s_slices);

//                                 let h: RewriteN = todo!();
//                                 let k: RewriteN = todo!();

//                                 (s.into(), h.into(), k.into())
//                             })
//                             .collect_vec()
//                     }
//                 })
//                 .collect_vec()
//         }
//         _ => panic!("Mismatched dimensions"),
//     }
// }

// fn construct_source(
//     target: &DiagramN,
//     mono: &Monotone,
//     slices: &[Rewrite],
//     source_slices: &[Diagram],
// ) -> DiagramN {
//     let mut cospans = vec![];
//     let target_slices = target.singular_slices();
//     for (ti, cospan) in target.cospans().iter().enumerate() {
//         let preimage = mono.preimage(ti);
//         if !preimage.is_empty() {
//             let start = preimage.start;
//             let end = preimage.end;

//             let mut rewrites = vec![factorize(
//                 cospan.forward.clone(),
//                 slices[start].clone(),
//                 target_slices[ti].clone(),
//             )
//             .next()
//             .unwrap()];

//             for si in start..end - 1 {
//                 let span = &antipushout(
//                     &source_slices[si],
//                     &source_slices[si + 1],
//                     &target_slices[ti],
//                     &slices[si],
//                     &slices[si + 1],
//                 )[0];
//                 rewrites.push(span.1.clone());
//                 rewrites.push(span.2.clone());
//             }

//             rewrites.push(
//                 factorize(
//                     cospan.backward.clone(),
//                     slices[end - 1].clone(),
//                     target_slices[ti].clone(),
//                 )
//                 .next()
//                 .unwrap(),
//             );

//             for chunk in rewrites.chunks(2) {
//                 cospans.push(Cospan {
//                     forward: chunk[0].clone(),
//                     backward: chunk[1].clone(),
//                 });
//             }
//         }
//     }

//     DiagramN::new_unsafe(target.source(), cospans)
// }

// pub fn factorize_inc(source: &Diagram, target: &Diagram, rewrite: &Rewrite) -> (Rewrite, Rewrite) {
//     let (x, y) = factorize_inc_helper(target, &[(source.clone(), rewrite.clone())]);
//     (x[0].clone(), y)
// }

// fn factorize_inc_helper(
//     target: &Diagram,
//     incoming: &[(Diagram, Rewrite)],
// ) -> (Vec<Rewrite>, Rewrite) {
//     match target {
//         Diagram::Diagram0(t) => {
//             let sources = incoming
//                 .iter()
//                 .map(|(s, _)| s.max_generator())
//                 .collect_vec();
//             let &m = sources.iter().max_by_key(|g| g.dimension).unwrap();

//             (
//                 sources
//                     .into_iter()
//                     .map(|s| Rewrite::from(Rewrite0::new(s, m, todo!())))
//                     .collect_vec(),
//                 Rewrite::from(Rewrite0::new(m, *t, todo!())),
//             )
//         }
//         Diagram::DiagramN(target) => {
//             let mut p_cones = vec![vec![]; incoming.len()];
//             let mut q_cones = vec![];

//             for ti in 0..target.size() {
//                 let target_slice = target.slice(Height::Singular(ti)).unwrap();

//                 // Recursively factorise the slices into ti.
//                 let (mut ps, q) = {
//                     let mut incoming_slices = Vec::default();
//                     for (source, rewrite) in incoming {
//                         let source: &DiagramN = source.try_into().unwrap();
//                         let rewrite: &RewriteN = rewrite.try_into().unwrap();
//                         for si in rewrite.singular_preimage(ti) {
//                             incoming_slices.push((
//                                 source.slice(Height::Singular(si)).unwrap(),
//                                 rewrite.slice(si),
//                             ));
//                         }
//                     }
//                     factorize_inc_helper(&target_slice, &incoming_slices)
//                 };

//                 let target_cospan = target.cospans()[ti].clone();

//                 let middle_cospan = {
//                     let forward = factorize(
//                         target_cospan.forward.clone(),
//                         q.clone(),
//                         target_slice.clone(),
//                     )
//                     .next()
//                     .unwrap();
//                     let backward = factorize(
//                         target_cospan.backward.clone(),
//                         q.clone(),
//                         target_slice.clone(),
//                     )
//                     .next()
//                     .unwrap();
//                     Cospan { forward, backward }
//                 };

//                 for (index, (source, rewrite)) in incoming.iter().enumerate() {
//                     let source: &DiagramN = source.try_into().unwrap();
//                     let rewrite: &RewriteN = rewrite.try_into().unwrap();

//                     let mut source_cospans = Vec::default();
//                     let Range { start, end } = rewrite.singular_preimage(ti);
//                     for si in start..end {
//                         source_cospans.push(source.cospans()[si].clone());
//                     }

//                     p_cones[index].push({
//                         let mut slices = Vec::default();
//                         for _ in start..end {
//                             slices.push(ps.remove(0));
//                         }
//                         Cone::new(
//                             start,
//                             source_cospans,
//                             middle_cospan.clone(),
//                             todo!(),
//                             slices,
//                         )
//                     });
//                 }

//                 q_cones.push(Cone::new(
//                     ti,
//                     vec![middle_cospan],
//                     target_cospan,
//                     todo!(),
//                     vec![q],
//                 ));
//             }

//             (
//                 p_cones
//                     .into_iter()
//                     .map(|cones| RewriteN::new_unsafe(target.dimension(), cones).into())
//                     .collect_vec(),
//                 RewriteN::new_unsafe(target.dimension(), q_cones).into(),
//             )
//         }
//     }
// }

use homotopy_core::{
    scaffold::{
        Explodable, ExplosionOutput, ExternalRewrite, InternalRewrite, Scaffold, ScaffoldEdge,
        ScaffoldNode,
    },
    Cospan, DiagramN, Rewrite, RewriteN, SliceIndex,
};
use petgraph::graph::DefaultIx;
use proptest::prelude::*;

mod rewrite;

proptest! {
    #[test]
    fn explode_then_reassemble((rewrite, source, target) in rewrite::arb_rewrite_1d_with_source_and_target()) {
        let mut graph = Scaffold::default();
        let src = graph.add_node(ScaffoldNode::new((), source.clone()));
        let tgt = graph.add_node(ScaffoldNode::new((), target.clone()));
        let rwr = graph.add_edge(src, tgt, ScaffoldEdge::new((), rewrite.clone()));

        let ExplosionOutput {
            node_to_nodes,
            node_to_edges,
            edge_to_edges,
            output,
        }: ExplosionOutput<(), Option<ExternalRewrite>, DefaultIx, DefaultIx> = graph
            .explode(
                |_, (), si| match si {
                    SliceIndex::Boundary(_) => None,
                    SliceIndex::Interior(_) => Some(()),
                },
                |_, (), ir| match ir {
                    InternalRewrite::Boundary(_) => None,
                    InternalRewrite::Interior(_, _) => Some(None),
                },
                |_, (), er| match er {
                    ExternalRewrite::Boundary(_) | ExternalRewrite::Sparse(_) => None,
                    ExternalRewrite::RegularSlice { .. } | ExternalRewrite::SingularSlice { .. } => Some(er.into()),
                },
            )
            .unwrap();

        let (reconstructed_source, source_cospans) = {
            let source = output[*node_to_nodes[src].first().unwrap()].diagram.clone();
            let cospans: Vec<_> = node_to_edges[src]
                .chunks_exact(2)
                .map(|chunk| match chunk {
                    [f, b] => Cospan {
                        forward: output.edge_weight(*f).unwrap().rewrite.clone(),
                        backward: output.edge_weight(*b).unwrap().rewrite.clone(),
                    },
                    _ => unreachable!(),
                })
                .collect();
            (DiagramN::new(source, cospans.clone()), cospans)
        };
        prop_assert_eq!(reconstructed_source, source);

        let (reconstructed_target, target_cospans) = {
            let source = output[*node_to_nodes[tgt].first().unwrap()].diagram.clone();
            let cospans: Vec<_> = node_to_edges[tgt]
                .chunks_exact(2)
                .map(|chunk| match chunk {
                    [f, b] => Cospan {
                        forward: output.edge_weight(*f).unwrap().rewrite.clone(),
                        backward: output.edge_weight(*b).unwrap().rewrite.clone(),
                    },
                    _ => unreachable!(),
                })
                .collect();
            (DiagramN::new(source, cospans.clone()), cospans)
        };
        prop_assert_eq!(reconstructed_target, target);

        let reconstructed_rewrite = {
            let mut regular_slices: Vec<Vec<Rewrite>> = vec![vec![]; target_cospans.len()];
            let mut singular_slices: Vec<Vec<Rewrite>> = vec![vec![]; target_cospans.len()];
            edge_to_edges[rwr]
                .iter()
                .map(|&e| {
                    output
                        .edge_weight(e)
                        .and_then(|edge| Some((edge.key?, edge.rewrite.clone())))
                        .unwrap()
                })
                .for_each(|(er, r)| match er {
                    ExternalRewrite::RegularSlice { target_height, .. } => regular_slices[target_height].push(r),
                    ExternalRewrite::SingularSlice { target_height, .. } => singular_slices[target_height].push(r),
                    _ => unreachable!(),
                });
            RewriteN::from_slices(
                1,
                &source_cospans,
                &target_cospans,
                regular_slices,
                singular_slices,
            )
        };
        prop_assert_eq!(reconstructed_rewrite, rewrite);
    }
}

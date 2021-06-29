use std::collections::HashSet;

use homotopy_core::{
    examples,
    labelled::{simplices, Simplex},
    Diagram, DiagramN, Generator, Height,
    Height::*,
};

// A simplicial complex is encoded as a Vec<Iterator<Item = Simplex>>, where the vector groups the
// simplices by dimension. The inner type for each collection of d-simplices may be large, and this
// would warrant collecting as a HashSet instead of as a Vec.

#[test]
pub fn zero_simplex_extraction() {
    let zero: Vec<Vec<Simplex<Height>>> = vec![vec![vec![vec![]]]];
    let simplices: Vec<Vec<Simplex<Height>>> = simplices(examples::one_zero_cell().1);
    assert_eq!(zero, simplices);
}

#[test]
pub fn one_dimensional_endomorphism_extraction() {
    let space = examples::one_zero_cell().1;
    let simplices: Vec<HashSet<Simplex<Height>>> = simplices(
        DiagramN::new(Generator::new(1, 1), space.clone(), space)
            .unwrap()
            .into(),
    );
    // simplicial complex
    assert_eq!(
        vec![
            vec![
                vec![vec![Regular(0)]],
                vec![vec![Singular(0)]],
                vec![vec![Regular(1)]],
            ]
            .into_iter()
            .collect::<HashSet<_>>(),
            vec![
                vec![vec![Regular(0)], vec![Singular(0)]],
                vec![vec![Regular(1)], vec![Singular(0)]],
            ]
            .into_iter()
            .collect::<HashSet<_>>()
        ],
        simplices
    );
}

#[allow(clippy::many_single_char_names)]
#[test]
pub fn two_dimensional_endomorphism_extraction() {
    let simplices: Vec<HashSet<Simplex<Height>>> = simplices(examples::two_endomorphism().1.into());
    let a = vec![Regular(0), Regular(0)];
    let b = vec![Regular(0), Singular(0)];
    let c = vec![Regular(0), Regular(1)];

    let d = vec![Singular(0), Regular(0)];
    let e = vec![Singular(0), Singular(0)];
    let f = vec![Singular(0), Regular(1)];

    let g = vec![Regular(1), Regular(0)];
    let h = vec![Regular(1), Singular(0)];
    let i = vec![Regular(1), Regular(1)];
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
            vec![i.clone()],
        ]
        .into_iter()
        .collect::<HashSet<_>>(),
        simplices[0]
    );

    assert_eq!(
        vec![
            vec![a.clone(), b.clone()],
            vec![c.clone(), b.clone()],
            vec![d.clone(), e.clone()],
            vec![f.clone(), e.clone()],
            vec![g.clone(), h.clone()],
            vec![i.clone(), h.clone()],
            vec![a.clone(), d.clone()],
            vec![g.clone(), d.clone()],
            vec![b.clone(), e.clone()],
            vec![h.clone(), e.clone()],
            vec![c.clone(), f.clone()],
            vec![i.clone(), f.clone()],
            // composite 1-simplices
            vec![a.clone(), e.clone()],
            vec![c.clone(), e.clone()],
            vec![g.clone(), e.clone()],
            vec![i.clone(), e.clone()],
        ]
        .into_iter()
        .collect::<HashSet<_>>(),
        simplices[1]
    );

    assert_eq!(
        vec![
            vec![a.clone(), b.clone(), e.clone()],
            vec![c.clone(), b, e.clone()],
            vec![g.clone(), h.clone(), e.clone()],
            vec![i.clone(), h, e.clone()],
            vec![a, d.clone(), e.clone()],
            vec![g, d, e.clone()],
            vec![c, f.clone(), e.clone()],
            vec![i, f, e],
        ]
        .into_iter()
        .collect::<HashSet<_>>(),
        simplices[2]
    )
}

#[allow(clippy::many_single_char_names)]
#[test]
pub fn two_dimensional_monoid_extraction() {
    let simplices: Vec<HashSet<Simplex<Height>>> = simplices(examples::two_monoid().1.into());
    let a = vec![Regular(0), Regular(0)];
    let b = vec![Regular(0), Singular(0)];
    let c = vec![Regular(0), Regular(1)];
    let d = vec![Regular(0), Singular(1)];
    let e = vec![Regular(0), Regular(2)];

    let f = vec![Singular(0), Regular(0)];
    let g = vec![Singular(0), Singular(0)];
    let h = vec![Singular(0), Regular(1)];

    let i = vec![Regular(1), Regular(0)];
    let j = vec![Regular(1), Singular(0)];
    let k = vec![Regular(1), Regular(1)];
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
            vec![i.clone()],
            vec![j.clone()],
            vec![k.clone()],
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
            vec![i.clone(), j.clone()],
            vec![k.clone(), j.clone()],
            vec![a.clone(), f.clone()],
            vec![i.clone(), f.clone()],
            vec![e.clone(), h.clone()],
            vec![k.clone(), h.clone()],
            vec![b.clone(), g.clone()],
            vec![d.clone(), g.clone()],
            vec![j.clone(), g.clone()],
            // composite 1-simplices
            vec![a.clone(), g.clone()],
            vec![c.clone(), g.clone()],
            vec![e.clone(), g.clone()],
            vec![i.clone(), g.clone()],
            vec![k.clone(), g.clone()],
        ]
        .into_iter()
        .collect::<HashSet<_>>(),
        simplices[1]
    );

    assert_eq!(
        vec![
            vec![a.clone(), b.clone(), g.clone()],
            vec![a, f.clone(), g.clone()],
            vec![c.clone(), b, g.clone()],
            vec![c, d.clone(), g.clone()],
            vec![e.clone(), d, g.clone()],
            vec![e, h.clone(), g.clone()],
            vec![i.clone(), j.clone(), g.clone()],
            vec![i, f, g.clone()],
            vec![k.clone(), j, g.clone()],
            vec![k, h, g],
        ]
        .into_iter()
        .collect::<HashSet<_>>(),
        simplices[2]
    );
}

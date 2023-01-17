use homotopy_core::{
    examples,
    signature::{GeneratorInfo, Signature, SignatureBuilder},
    Bias, Boundary, Diagram, DiagramN, Direction, Generator, Height,
};
use insta::assert_debug_snapshot;
use pretty_assertions::assert_eq;
use test_case::test_case;

#[test]
fn scalar() {
    let (sig, scalar) = examples::scalar();
    let scalar_inverse = scalar.inverse();
    let scalar_then_scalar = DiagramN::new(
        scalar.source(),
        [scalar.cospans(), scalar.cospans()].concat(),
    );
    let scalar_then_inverse = DiagramN::new(
        scalar.source(),
        [scalar.cospans(), scalar_inverse.cospans()].concat(),
    );
    let inverse_then_scalar = DiagramN::new(
        scalar_inverse.source(),
        [scalar_inverse.cospans(), scalar.cospans()].concat(),
    );

    assert!(scalar_then_scalar
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            Some(Bias::Same),
            &sig
        )
        .is_err());

    assert_debug_snapshot!(
        "scalar_biased_left",
        scalar_then_scalar
            .clone()
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                Some(Bias::Lower),
                &sig
            )
            .unwrap()
            .target()
    );

    assert_debug_snapshot!(
        "scalar_biased_right",
        scalar_then_scalar
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                Some(Bias::Higher),
                &sig
            )
            .unwrap()
            .target()
    );

    assert_debug_snapshot!(
        "scalar_inverse_unbiased",
        scalar_then_inverse
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                Some(Bias::Same),
                &sig
            )
            .unwrap()
            .target()
    );

    assert_debug_snapshot!(
        "inverse_scalar_unbiased",
        inverse_then_scalar
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                Some(Bias::Same),
                &sig
            )
            .unwrap()
            .target()
    );
}

#[test]
fn three_scalars() {
    let (sig, scalar) = examples::scalar();
    let three = scalar
        .attach(&scalar, Boundary::Target, &[])
        .unwrap()
        .attach(&scalar, Boundary::Target, &[])
        .unwrap();
    let l = three
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            Some(Bias::Higher),
            &sig,
        )
        .unwrap()
        .target();
    // l is .
    //      . .
    let into_middle = l.clone().identity().contract(
        Boundary::Target.into(),
        &mut [],
        0,
        Direction::Forward,
        Some(Bias::Lower),
        &sig,
    );
    assert!(into_middle.is_ok());
    let into_left = l.identity().contract(
        Boundary::Target.into(),
        &mut [],
        0,
        Direction::Forward,
        Some(Bias::Higher),
        &sig,
    );
    assert!(into_left.is_ok());
}

#[test]
fn braids() {
    // make braids by contraction
    let (sig, crossing) = examples::crossing();
    assert_debug_snapshot!(
        "crossing",
        crossing
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )
            .unwrap()
            .target()
    );

    let (sig, touching) = examples::touching();
    assert_debug_snapshot!(
        "touching",
        touching
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )
            .unwrap()
            .target()
    );
}

#[test]
fn beads() {
    let (sig, diagram) = examples::two_beads();

    let contracted = diagram
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )
        .unwrap();

    assert_debug_snapshot!("beads", contracted);
}

#[test]
fn stacks() {
    let (sig, diagram) = examples::stacks();

    let contracted = diagram
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )
        .unwrap();

    assert_debug_snapshot!("stacks", contracted);
}

#[test]
fn inverses_1d() {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let f = sig.add(x, x).unwrap();
    let f_inverse = f.inverse();
    let f_then_inverse = DiagramN::new(f.source(), vec![f.cospans(), f_inverse.cospans()].concat());
    assert_debug_snapshot!(
        "1-endomorphism_then_inverse",
        f_then_inverse
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )
            .unwrap()
            .target()
    );

    let inverse_then_f = DiagramN::new(
        f_inverse.source(),
        vec![f.cospans(), f_inverse.cospans()].concat(),
    );
    assert_debug_snapshot!(
        "1-inverse_then_endomorphism",
        inverse_then_f
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )
            .unwrap()
            .target()
    );
}

#[test]
fn inverses_2d() {
    let (sig, e) = examples::two_endomorphism();
    let e_inverse = e.inverse();

    let e_then_e = DiagramN::new(e.source(), [e.cospans(), e.cospans()].concat());
    assert!(e_then_e
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig
        )
        .is_err());

    let e_then_inverse = DiagramN::new(e.source(), [e.cospans(), e_inverse.cospans()].concat());
    assert_debug_snapshot!(
        "2-endomorphism_then_inverse",
        e_then_inverse
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )
            .unwrap()
            .target()
    );

    let inverse_then_e = DiagramN::new(
        e_inverse.source(),
        [e_inverse.cospans(), e.cospans()].concat(),
    );
    assert_debug_snapshot!(
        "2-inverse_then_endomorphism",
        inverse_then_e
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )
            .unwrap()
            .target()
    );
}

#[test]
fn snakerator() {
    let (sig, snake) = examples::snake();

    assert_eq!(
        snake
            .clone()
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )
            .expect("failed to contract snake")
            .target(),
        Diagram::from(snake.source().identity())
    );
}

#[test]
fn bubble_pop_2d() {
    let (sig, bubble) = examples::bubble();
    assert_eq!(
        bubble
            .clone()
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )
            .expect("failed to contract bubble")
            .target(),
        Diagram::from(bubble.source().identity())
    );
}

// cone-wise smoothing should smooth out the cap next to a wire
// | |
// | p⁻¹    | |
// | |   ⤳  q |
// q p      | |
// | |
#[test]
#[allow(clippy::many_single_char_names)]
fn bead_tensor_bead_and_inverse() -> anyhow::Result<()> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let f = sig.add(x, x)?;
    let g = sig.add(x, x)?;
    let p = sig.add(f.clone(), f.clone())?;
    let q = sig.add(g.clone(), g.clone())?;
    let p_then_inverse = p.attach(&p.inverse(), Boundary::Target, &[0])?;
    let g_then_p_then_inverse = p_then_inverse.attach(&g, Boundary::Source, &[])?;
    let q_then_p_then_inverse = g_then_p_then_inverse.attach(&q, Boundary::Source, &[0])?;
    let q_tensor_p_then_inverse = q_then_p_then_inverse
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target();
    assert_eq!(
        q_tensor_p_then_inverse
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )?
            .target(),
        q.attach(&f, Boundary::Target, &[])?.into()
    );
    Ok(())
}

// cone-wise smoothing should handle non-trivial indices
//  |  |
//  |  e⁻¹    |
//  |  |   ⤳  m
//  m  e     / \
// / \ |
#[test]
fn monoid_tensor_bead_and_inverse() -> anyhow::Result<()> {
    let (mut sig, monoid) = examples::two_monoid();
    let f = DiagramN::try_from(
        sig.generator_info(Generator::new(1, 1))
            .unwrap()
            .diagram()
            .clone(),
    )?;
    let e = sig.add(f.clone(), f.clone())?;
    let e_then_inverse = e.attach(&e.inverse(), Boundary::Target, &[])?;
    let monoid_wire = monoid.attach(&f, Boundary::Target, &[])?;
    let monoid_tensor_e_then_inverse = monoid_wire
        .attach(&e_then_inverse, Boundary::Target, &[1])?
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target();
    assert_eq!(
        monoid_tensor_e_then_inverse
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )?
            .target(),
        monoid_wire.into()
    );

    let wire_monoid = monoid.attach(&f, Boundary::Source, &[])?;
    let e_then_inverse_tensor_monoid = wire_monoid
        .attach(&e_then_inverse, Boundary::Target, &[])?
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target();
    assert_eq!(
        e_then_inverse_tensor_monoid
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )?
            .target(),
        wire_monoid.into()
    );

    Ok(())
}

// cone-wise smoothing should only happen at the top level, never in a recursive level
// braidings of:
// |
// |  -
// | / \
// (this diagram is 3D)
#[test]
fn cap_braid() -> anyhow::Result<()> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let s = sig.add(x.identity(), x.identity())?;
    let two_cap: DiagramN = s
        .attach(&s.inverse(), Boundary::Target, &[])?
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            Some(Bias::Same),
            &sig,
        )?;
    let wire_then_cap = two_cap.attach(&s, Boundary::Source, &[0])?;
    let wire_over_cap: DiagramN = wire_then_cap
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Lower),
            &sig,
        )?
        .target()
        .try_into()?;
    assert_eq!(wire_over_cap.cospans().len(), 1);
    let wire_under_cap: DiagramN = wire_then_cap
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Higher),
            &sig,
        )?
        .target()
        .try_into()?;
    assert_eq!(wire_under_cap.cospans().len(), 1);

    // also check wire can be separated from cap
    assert_eq!(
        wire_over_cap
            .identity()
            .expand(
                Boundary::Target.into(),
                &mut [Height::Singular(0)],
                [Height::Singular(0), Height::Singular(0)],
                Direction::Backward,
                &sig
            )?
            .target(),
        two_cap.attach(&s, Boundary::Source, &[])?.into()
    );
    assert_eq!(
        wire_under_cap
            .identity()
            .expand(
                Boundary::Target.into(),
                &mut [Height::Singular(0)],
                [Height::Singular(0), Height::Singular(0)],
                Direction::Forward,
                &sig
            )?
            .target(),
        two_cap.attach(&s, Boundary::Source, &[])?.into()
    );

    Ok(())
}

// ensure that
//  -
// / \
// \ /
//  /
// / \
// does not contract
#[test]
fn no_reidemeister_1() -> anyhow::Result<()> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let s = sig.add(x.identity(), x.identity())?;
    let two_cap: DiagramN = s
        .attach(&s.inverse(), Boundary::Target, &[])?
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            Some(Bias::Same),
            &sig,
        )?;
    let r1_source = two_cap
        .contract(
            Boundary::Source.into(),
            &mut [],
            0,
            Direction::Forward,
            Some(Bias::Higher),
            &sig,
        )?
        .expand(
            Boundary::Source.into(),
            &mut [],
            [Height::Singular(0), Height::Singular(1)],
            Direction::Forward,
            &sig,
        )?
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target();
    assert!(r1_source
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig
        )
        .is_err());
    Ok(())
}

// contractions like:
// | | |
// e | |     | \ /
// | \ /  ⤳  e  -
// |  -      |
// |
// etc.
#[test]
fn bead_through_cap_cup() -> anyhow::Result<()> {
    let (_, cap) = examples::cap();
    let (mut sig, cup) = examples::cup();
    let f = DiagramN::try_from(
        sig.generator_info(Generator::new(1, 1))
            .unwrap()
            .diagram()
            .clone(),
    )?;
    let e = sig.add(f.clone(), f.clone())?;

    // cup contractions
    let wire_tensor_cup = cup.attach(&f, Boundary::Source, &[0])?;
    let cup_tensor_wire = cup.attach(&f, Boundary::Target, &[0])?;

    let e_above_left_cup = wire_tensor_cup.attach(&e, Boundary::Target, &[])?;
    let e_tensor_cup = e_above_left_cup
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target();

    let e_below_left_cup = wire_tensor_cup.attach(&e, Boundary::Source, &[])?;
    assert_eq!(
        e_below_left_cup
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )?
            .target(),
        e_tensor_cup
    );

    let e_above_right_cup = cup_tensor_wire.attach(&e, Boundary::Target, &[2])?;
    let cup_tensor_e = e_above_right_cup
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target();

    let e_below_right_cup = cup_tensor_wire.attach(&e, Boundary::Source, &[])?;
    assert_eq!(
        e_below_right_cup
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )?
            .target(),
        cup_tensor_e
    );

    // cap contractions
    let wire_tensor_cap = cap.attach(&f, Boundary::Source, &[0])?;
    let cap_tensor_wire = cap.attach(&f, Boundary::Target, &[0])?;

    let e_above_left_cap = wire_tensor_cap.attach(&e, Boundary::Target, &[])?;
    let e_tensor_cap = e_above_left_cap
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target();

    let e_below_left_cap = wire_tensor_cap.attach(&e, Boundary::Source, &[])?;
    assert_eq!(
        e_below_left_cap
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )?
            .target(),
        e_tensor_cap
    );

    let e_above_right_cap = cap_tensor_wire.attach(&e, Boundary::Target, &[])?;
    let cap_tensor_e = e_above_right_cap
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target();

    let e_below_right_cap = cap_tensor_wire.attach(&e, Boundary::Source, &[2])?;
    assert_eq!(
        e_below_right_cap
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig
            )?
            .target(),
        cap_tensor_e
    );

    Ok(())
}

#[test]
fn three_dimensional_scalar_across_wire_preserves_label_neighbourhood() -> anyhow::Result<()> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let two = sig.add(x.identity(), x.identity())?;
    let three = sig.add(x.identity().identity(), x.identity().identity())?;
    // two_then_three:
    // 2
    // 2 3
    // 2
    let two_then_three = three.attach(&two, Boundary::Source, &[0])?;
    // contracted:
    // 2
    // 3
    // 2
    let contracted = two_then_three
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Higher),
            &sig,
        )?
        .target();
    // expanded:
    //   2
    // 3 2
    //   2
    let expanded: DiagramN = contracted
        .identity()
        .expand(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            [Height::Singular(0), Height::Singular(0)],
            Direction::Backward,
            &sig,
        )?
        .target()
        .try_into()?;
    // chop off the right part
    let source: Diagram = {
        let extended_source: DiagramN = expanded.source().try_into()?;
        assert_eq!(extended_source.cospans().len(), 1);
        DiagramN::new(extended_source.source(), vec![]).into()
    };
    assert_eq!(
        three,
        DiagramN::new(source, vec![expanded.cospans()[0].clone()])
    );
    Ok(())
}

// ensure
//  -   -
// / \ / \
// contracts (horizontally)
#[test]
fn contract_two_caps() -> anyhow::Result<()> {
    let (mut sig, cap) = examples::cap();
    let x = sig
        .generator_info(Generator::new(0, 0))
        .unwrap()
        .diagram()
        .clone();
    let f = DiagramN::try_from(
        sig.generator_info(Generator::new(1, 1))
            .unwrap()
            .diagram()
            .clone(),
    )?;
    let cap_tensor_cap = DiagramN::try_from(
        cap.attach(&f, Boundary::Target, &[])?
            .attach(&f.inverse(), Boundary::Target, &[])?
            .attach(&cap, Boundary::Target, &[])?
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig,
            )?
            .target(),
    )?;
    assert!(cap_tensor_cap
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            None,
            &sig
        )
        .is_ok());

    let algebraic_cap = sig.add(f.attach(&f.inverse(), Boundary::Target, &[])?, x.identity())?;
    let acap_tensor_acap = DiagramN::try_from(
        algebraic_cap
            .attach(&f, Boundary::Target, &[])?
            .attach(&f.inverse(), Boundary::Target, &[])?
            .attach(&algebraic_cap, Boundary::Target, &[])?
            .identity()
            .contract(
                Boundary::Target.into(),
                &mut [],
                0,
                Direction::Forward,
                None,
                &sig,
            )?
            .target(),
    )?;
    assert!(acap_tensor_acap
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            None,
            &sig
        )
        .is_err());

    Ok(())
}

// ensure
//  -
// / \
// \ /
//  h
// / \
// \ /
//  -
//  does not contract (3D hourglass)
#[test]
fn hourglass_no_absorb() -> anyhow::Result<()> {
    let (sig, endomorphism) = endomorphism_on_ring()?;
    let hourglass = endomorphism
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .contract(
            Boundary::Source.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?;
    assert!(hourglass
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            Some(Bias::Same),
            &sig
        )
        .is_err());
    assert!(hourglass
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            1,
            Direction::Forward,
            Some(Bias::Same),
            &sig
        )
        .is_err());
    Ok(())
}

// contractions like
//   |    |
// c | ⤳  c
// | |   /|
// etc. (in 3D)
#[test]
fn counit_braid() -> anyhow::Result<()> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let s = sig.add(x.identity(), x.identity())?;
    let counit = sig.add(s.clone(), x.identity().identity())?;
    let counit_tensor_wire = counit.attach(&s, Boundary::Target, &[])?;
    let wire_tensor_counit = counit.attach(&s, Boundary::Source, &[])?;

    let counit_braid_above_left = counit_tensor_wire.clone().identity().contract(
        Boundary::Target.into(),
        &mut [Height::Singular(0)],
        0,
        Direction::Forward,
        Some(Bias::Lower),
        &sig,
    );
    assert!(counit_braid_above_left.is_ok());
    let counit_braid_below_left = counit_tensor_wire.identity().contract(
        Boundary::Target.into(),
        &mut [Height::Singular(0)],
        0,
        Direction::Forward,
        Some(Bias::Higher),
        &sig,
    );
    assert!(counit_braid_below_left.is_ok());
    let counit_braid_below_right = wire_tensor_counit.clone().identity().contract(
        Boundary::Target.into(),
        &mut [Height::Singular(0)],
        0,
        Direction::Forward,
        Some(Bias::Lower),
        &sig,
    );
    assert!(counit_braid_below_right.is_ok());
    let counit_braid_above_right = wire_tensor_counit.identity().contract(
        Boundary::Target.into(),
        &mut [Height::Singular(0)],
        0,
        Direction::Forward,
        Some(Bias::Higher),
        &sig,
    );
    assert!(counit_braid_above_right.is_ok());
    Ok(())
}

// ensure that
// | |   | |   | |   | |   | |
// | b   | |   | |   | |   a |
// | | ⤳ a b ⬿ a b ⤳ a b ⬿ | |
// a |   | |   | |   | |   | b
// | |   | |   | |   | |   | |
// contracts
#[test]
fn bead_interchanger() -> anyhow::Result<()> {
    let (sig, beads) = examples::two_beads();

    let contracted = beads.clone().identity().contract(
        Boundary::Target.into(),
        &mut [],
        0,
        Direction::Forward,
        None,
        &sig,
    )?;
    let expanded_forwards = contracted.expand(
        Boundary::Target.into(),
        &mut [],
        [Height::Singular(0), Height::Singular(1)],
        Direction::Forward,
        &sig,
    )?;
    assert_eq!(
        DiagramN::try_from(
            expanded_forwards
                .identity()
                .contract(
                    Boundary::Target.into(),
                    &mut [],
                    0,
                    Direction::Forward,
                    None,
                    &sig
                )?
                .target()
        )
        .expect("failed to contract expanded interchanger")
        .target(),
        beads.into()
    );
    let expanded_backwards = contracted.expand(
        Boundary::Target.into(),
        &mut [],
        [Height::Singular(0), Height::Singular(1)],
        Direction::Backward,
        &sig,
    )?;
    assert!(expanded_backwards
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig
        )
        .is_ok());
    Ok(())
}

//  ensure this contraction+expansion produces the correct result:
//  |   |
//   \ /    |   |
//    \      \ /    | |
//   / \  ⤳   u   ⤳ u |
//  |   |      \      |
//  |   u       |
//  |
#[test]
fn pull_through_braid() -> anyhow::Result<()> {
    let (mut sig, braid) = examples::crossing();
    let braid: DiagramN = braid
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target()
        .try_into()?;
    let x = sig
        .generator_info(Generator::new(0, 0))
        .unwrap()
        .diagram()
        .clone();
    let s = DiagramN::try_from(
        sig.generator_info(Generator::new(1, 2))
            .unwrap()
            .diagram()
            .clone(),
    )?;
    let unit = sig.add(x.identity().identity(), s.clone())?;
    let unit_then_braid = braid.attach(&unit, Boundary::Source, &[1])?;
    let unit_over_braid: DiagramN = unit_then_braid
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target()
        .try_into()?;
    let unit_through_braid = unit_over_braid
        .identity()
        .expand(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            [Height::Singular(0), Height::Singular(1)],
            Direction::Forward,
            &sig,
        )?
        .target();

    assert_eq!(
        unit_through_braid,
        unit.attach(&s, Boundary::Target, &[])?.into()
    );
    Ok(())
}

// ensure that
// |   |
// e   |
// |   |
//  \ /
//   -
// and
//     |
// c   |
// |   |
//  \ /
//   -
// don't contract
#[test]
fn no_bend_cup() -> anyhow::Result<()> {
    let (_sig, cup) = examples::cup();
    let (mut sig, e) = examples::bead_series(1);
    let cup_then_e = cup.attach(&e, Boundary::Target, &[])?;
    assert!(cup_then_e
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig
        )
        .is_err());

    let x = sig
        .generator_info(Generator::new(0, 0))
        .unwrap()
        .diagram()
        .clone();
    let f = DiagramN::try_from(
        sig.generator_info(Generator::new(1, 1))
            .unwrap()
            .diagram()
            .clone(),
    )?;
    let counit = sig.add(f, x.identity())?;
    let cup_then_counit = cup.attach(&counit, Boundary::Target, &[])?;
    assert!(cup_then_counit
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig
        )
        .is_err());
    Ok(())
}

// all combinations of contracting
// | | |
// | | |
// | | |
// to make simultaneous braids
#[test]
fn double_braid() -> anyhow::Result<()> {
    let (sig, s) = examples::scalar();
    let three_wires = s
        .attach(&s, Boundary::Target, &[])?
        .attach(&s, Boundary::Target, &[])?
        .identity();
    let over_left = three_wires
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Regular(0)],
            0,
            Direction::Forward,
            Some(Bias::Lower),
            &sig,
        )?
        .target();
    assert!(over_left
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Higher),
            &sig
        )
        .is_ok());
    assert!(over_left
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Lower),
            &sig
        )
        .is_ok());

    let under_left = three_wires
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Regular(0)],
            0,
            Direction::Forward,
            Some(Bias::Higher),
            &sig,
        )?
        .target();
    assert!(under_left
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Higher),
            &sig
        )
        .is_ok());
    assert!(under_left
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Lower),
            &sig
        )
        .is_ok());

    let over_right = three_wires
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Regular(0)],
            1,
            Direction::Forward,
            Some(Bias::Lower),
            &sig,
        )?
        .target();
    assert!(over_right
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Higher),
            &sig
        )
        .is_ok());
    assert!(over_right
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Lower),
            &sig
        )
        .is_ok());

    let under_right = three_wires
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Regular(0)],
            1,
            Direction::Forward,
            Some(Bias::Higher),
            &sig,
        )?
        .target();
    assert!(under_right
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Higher),
            &sig
        )
        .is_ok());
    assert!(under_right
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Lower),
            &sig
        )
        .is_ok());
    Ok(())
}

// contractions like:
// | | |   | | |
// e | | ⤳ e  >
// | | |   | | |
#[test]
fn braid_next_to_endomorphism() -> anyhow::Result<()> {
    let (mut sig, s) = examples::scalar();
    let e = sig.add(s.clone(), s.clone())?;
    let e_wire_wire = e
        .attach(&s, Boundary::Target, &[])?
        .attach(&s, Boundary::Target, &[])?;
    assert!(e_wire_wire
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            1,
            Direction::Forward,
            Some(Bias::Lower),
            &sig
        )
        .is_ok());
    assert!(e_wire_wire
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            1,
            Direction::Forward,
            Some(Bias::Higher),
            &sig
        )
        .is_ok());
    let wire_wire_e = e
        .attach(&s, Boundary::Source, &[])?
        .attach(&s, Boundary::Source, &[])?;
    assert!(wire_wire_e
        .clone()
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Lower),
            &sig
        )
        .is_ok());
    assert!(wire_wire_e
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [Height::Singular(0)],
            0,
            Direction::Forward,
            Some(Bias::Higher),
            &sig
        )
        .is_ok());
    Ok(())
}

fn endomorphism_on_weak_unit() -> anyhow::Result<(impl Signature, DiagramN)> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let weak_x = Diagram::from(x).weak_identity();
    let f = sig.add(weak_x.clone(), weak_x)?;
    Ok((sig, f))
}

fn counit_on_endomorphism() -> anyhow::Result<(impl Signature, DiagramN)> {
    let (mut sig, endomorphism) = examples::bead_series(1);
    let f = sig
        .generator_info(Generator::new(1, 1))
        .unwrap()
        .diagram()
        .clone();
    let counit = sig.add(endomorphism, f.identity())?;
    Ok((sig, counit))
}

fn counit_on_scalar() -> anyhow::Result<(impl Signature, DiagramN)> {
    let (mut sig, scalar) = examples::scalar();
    let x = sig
        .generator_info(Generator::new(0, 0))
        .unwrap()
        .diagram()
        .clone();
    let counit = sig.add(scalar, x.identity().identity())?;
    Ok((sig, counit))
}

fn endomorphism_on_braid() -> anyhow::Result<(impl Signature, DiagramN)> {
    let (mut sig, braid) = examples::crossing();
    let endomorphism = sig.add(braid.clone(), braid)?;
    Ok((sig, endomorphism))
}

fn endomorphism_on_ring() -> anyhow::Result<(impl Signature, DiagramN)> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let f = sig.add(x, x)?;
    let cap = f
        .attach(&f.inverse(), Boundary::Target, &[])?
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?;
    let cup = f
        .attach(&f.inverse(), Boundary::Target, &[])?
        .identity()
        .contract(
            Boundary::Source.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?;
    let ring = cup.attach(&cap, Boundary::Target, &[])?;
    let endomorphism = sig.add(ring.clone(), ring)?;
    Ok((sig, endomorphism))
}

fn endomorphism_on_algebraic_ring() -> anyhow::Result<(impl Signature, DiagramN)> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let f = sig.add(x, x)?;
    let algebraic_cap = sig.add(f.attach(&f.inverse(), Boundary::Target, &[])?, x.identity())?;
    let algebraic_cup = sig.add(x.identity(), f.attach(&f.inverse(), Boundary::Target, &[])?)?;
    let ring = algebraic_cup.attach(&algebraic_cap, Boundary::Target, &[])?;
    let endomorphism = sig.add(ring.clone(), ring)?;
    Ok((sig, endomorphism))
}

fn endomorphism_on_half_algebraic_ring() -> anyhow::Result<(impl Signature, DiagramN)> {
    let mut sig = SignatureBuilder::default();
    let x = sig.add_zero();
    let f = sig.add(x, x)?;
    let algebraic_cap = sig.add(f.attach(&f.inverse(), Boundary::Target, &[])?, x.identity())?;
    let cup = f
        .attach(&f.inverse(), Boundary::Target, &[])?
        .identity()
        .contract(
            Boundary::Source.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?;
    let ring = cup.attach(&algebraic_cap, Boundary::Target, &[])?;
    let endomorphism = sig.add(ring.clone(), ring)?;
    Ok((sig, endomorphism))
}

// ensure that the singular braid on two cups is not smoothed away
#[test]
fn singular_braid_on_two_cups() -> anyhow::Result<()> {
    let (sig, two_cups) = examples::two_cups();

    let braid = two_cups.identity().identity().contract(
        Boundary::Target.into(),
        &mut [Height::Regular(0)],
        0,
        Direction::Forward,
        Some(Bias::Lower),
        &sig,
    )?;

    assert!(braid.cospans()[0].backward.is_identity());

    Ok(())
}

// everything should contract when composed with a weak unit, on either side
#[test_case(examples::associator())]
#[test_case(examples::two_endomorphism())]
#[test_case(examples::two_monoid())]
#[test_case(examples::scalar())]
#[test_case(examples::two_scalars())]
#[test_case(examples::touching())]
#[test_case(examples::crossing())]
#[test_case(examples::two_beads())]
#[test_case(examples::three_beads())]
#[test_case(examples::stacks())]
#[test_case(examples::matchsticks())]
#[test_case(examples::bead_series(1))]
#[test_case(examples::bead_series(2))]
#[test_case(examples::bead_series(3))]
#[test_case(examples::monoid_unit())]
#[test_case(examples::scalar_and_beads())]
#[test_case(examples::algebraic_snake())]
#[test_case(examples::bubble())]
#[test_case(examples::cap())]
#[test_case(examples::cup())]
#[test_case(examples::snake())]
#[test_case(examples::lips())]
#[test_case(examples::pants_unit())]
#[test_case(endomorphism_on_weak_unit().expect("failed to create endomorphism on weak unit"))]
#[test_case(counit_on_endomorphism().expect("failed to create counit on endomorphism"))]
#[test_case(counit_on_scalar().expect("failed to create counit on scalar"))]
#[test_case(endomorphism_on_braid().expect("failed to create endomorphism on braid"))]
#[test_case(endomorphism_on_ring().expect("failed to create endomorphism on homotopy ring"))]
#[test_case(endomorphism_on_algebraic_ring().expect("failed to create endomorphism on algebraic ring"))]
#[test_case(endomorphism_on_half_algebraic_ring().expect("failed to create endomorphism on half algebraic ring"))]
fn contract_with_weak_id((sig, diagram): (impl Signature, DiagramN)) -> anyhow::Result<()> {
    let pre_weak = diagram
        .source()
        .weak_identity()
        .attach(&diagram, Boundary::Target, &[])?;
    let contracted: DiagramN = pre_weak
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            0,
            Direction::Forward,
            None,
            &sig,
        )?
        .target()
        .try_into()?;
    assert_eq!(
        &contracted, &diagram,
        "precompose with weak id and contract"
    );
    let post_weak = diagram
        .target()
        .weak_identity()
        .attach(&diagram, Boundary::Source, &[])?;
    let contracted: DiagramN = post_weak
        .identity()
        .contract(
            Boundary::Target.into(),
            &mut [],
            diagram.size() - 1,
            Direction::Forward,
            None,
            &sig,
        )?
        .target()
        .try_into()?;
    assert_eq!(
        &contracted, &diagram,
        "postcompose with weak id and contract"
    );
    Ok(())
}

#[allow(clippy::wildcard_imports)]
use crate::*;
use crate::{
    rewrite::Cone,
    signature::{Signature, SignatureBuilder},
    Boundary::{Source, Target},
    Height::{Regular, Singular},
    SliceIndex::{Boundary, Interior},
};

//    |       |
//    m       m
//   / \     / \
//  |   | → |   |
//  m   |   |   m
// / \  |   |  / \
pub fn associator() -> (SignatureBuilder, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let f = sig.add(x.clone(), x).unwrap();
    let ff = f.attach(&f, Target, &[]).unwrap();
    let m = sig.add(ff, f).unwrap();
    let left = m.attach(&m, Source, &[0]).unwrap();
    let right = m.attach(&m, Source, &[1]).unwrap();
    let associator = sig.add(left, right).unwrap();
    (sig, associator)
}

// x
pub fn one_zero_cell() -> (impl Signature, Diagram) {
    let mut sig = SignatureBuilder::new();
    let x = sig.add_zero();
    (sig, x)
}

// |
// e
// |
pub fn two_endomorphism() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let space = sig.add_zero();
    let wire = sig.add(space.clone(), space).unwrap();
    let e = sig.add(wire.clone(), wire).unwrap();
    (sig, e)
}

//  |
//  m
// / \
pub fn two_monoid() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let space = sig.add_zero();
    let wire = sig.add(space.clone(), space).unwrap();
    let wirewire = wire.attach(&wire, Target, &[]).unwrap();
    let m = sig.add(wirewire, wire).unwrap();
    (sig, m)
}

//  s
pub fn scalar() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let s = sig.add(x.identity(), x.identity()).unwrap();
    (sig, s)
}

//  t
//
//  s
pub fn two_scalars() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let s = sig.add(x.identity(), x.identity()).unwrap();
    let t = sig.add(x.identity(), x.identity()).unwrap();
    (sig, s.attach(&t, Target, &[]).unwrap())
}

//  |   |
//   \ /
//    >
//   / \
//  |   |
pub fn touching() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let x_generator = Generator::new(0, 0);
    let s = sig.add(x.identity(), x.identity()).unwrap();
    let s_generator = Generator::new(1, 2);
    let s_s = s.attach(&s, Target, &[]).unwrap();
    let rewrite = |coord| {
        Rewrite0::new(
            x_generator,
            s_generator,
            (s_generator, coord).into(),
            Orientation::Positive,
        )
        .into()
    };
    let fwd = rewrite(vec![Interior(Singular(0)), Boundary(Source)]);
    let bwd = rewrite(vec![Interior(Singular(0)), Boundary(Target)]);
    let up = rewrite(vec![Boundary(Source), Interior(Regular(0))]);
    let down = rewrite(vec![Boundary(Target), Interior(Regular(0))]);
    let s_internal = Cospan {
        forward: fwd,
        backward: bwd,
    };
    let up_cone = Cone::new(0, vec![], s_internal.clone(), vec![up.clone()], vec![]);
    let down_cone = Cone::new(0, vec![], s_internal.clone(), vec![down], vec![]);
    let s_tensor_s_cospan = Cospan {
        forward: RewriteN::new(1, vec![up_cone.clone(), up_cone.clone()]).into(),
        backward: RewriteN::new(1, vec![down_cone.clone(), down_cone.clone()]).into(),
    };
    let twist: Rewrite = RewriteN::new(
        2,
        vec![Cone::new(
            0,
            s_s.cospans().to_vec(),
            s_tensor_s_cospan.clone(),
            vec![
                s_tensor_s_cospan.forward,
                RewriteN::new(1, vec![down_cone.clone(), up_cone]).into(),
                s_tensor_s_cospan.backward,
            ],
            vec![
                RewriteN::new(1, vec![Cone::new(1, vec![], s_internal, vec![up], vec![])]).into(),
                RewriteN::new(1, vec![down_cone]).into(),
            ],
        )],
    )
    .into();
    (
        sig,
        DiagramN::new(
            s_s.into(),
            vec![
                Cospan {
                    forward: twist.clone(),
                    backward: Rewrite::identity(2),
                },
                Cospan {
                    forward: Rewrite::identity(2),
                    backward: twist,
                },
            ],
        ),
    )
}

//  |   |
//   \ /
//    \
//   / \
//  |   |
pub fn crossing() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let x_generator = Generator::new(0, 0);
    let s = sig.add(x.identity(), x.identity()).unwrap();
    let s_generator = Generator::new(1, 2);
    let s_s = s.attach(&s, Target, &[]).unwrap();
    let rewrite = |coord| {
        Rewrite0::new(
            x_generator,
            s_generator,
            (s_generator, coord).into(),
            Orientation::Positive,
        )
        .into()
    };
    let fwd = rewrite(vec![Interior(Singular(0)), Boundary(Source)]);
    let bwd = rewrite(vec![Interior(Singular(0)), Boundary(Target)]);
    let up = rewrite(vec![Boundary(Source), Interior(Regular(0))]);
    let down = rewrite(vec![Boundary(Target), Interior(Regular(0))]);
    let s_internal = Cospan {
        forward: fwd,
        backward: bwd,
    };
    let up_cone = Cone::new(0, vec![], s_internal.clone(), vec![up.clone()], vec![]);
    let down_cone = Cone::new(0, vec![], s_internal.clone(), vec![down.clone()], vec![]);
    let s_tensor_s_cospan = Cospan {
        forward: RewriteN::new(1, vec![up_cone.clone(), up_cone.clone()]).into(),
        backward: RewriteN::new(1, vec![down_cone.clone(), down_cone.clone()]).into(),
    };
    let twist: Rewrite = RewriteN::new(
        2,
        vec![Cone::new(
            0,
            s_s.cospans().to_vec(),
            s_tensor_s_cospan.clone(),
            vec![
                s_tensor_s_cospan.forward.clone(),
                RewriteN::new(1, vec![down_cone.clone(), up_cone.clone()]).into(),
                s_tensor_s_cospan.backward.clone(),
            ],
            vec![
                RewriteN::new(
                    1,
                    vec![Cone::new(1, vec![], s_internal.clone(), vec![up], vec![])],
                )
                .into(),
                RewriteN::new(1, vec![down_cone.clone()]).into(),
            ],
        )],
    )
    .into();
    let untwist: Rewrite = RewriteN::new(
        2,
        vec![Cone::new(
            0,
            s_s.cospans().to_vec(),
            s_tensor_s_cospan.clone(),
            vec![
                s_tensor_s_cospan.forward,
                RewriteN::new(1, vec![up_cone.clone(), down_cone]).into(),
                s_tensor_s_cospan.backward,
            ],
            vec![
                RewriteN::new(1, vec![up_cone]).into(),
                RewriteN::new(
                    1,
                    vec![Cone::new(1, vec![], s_internal, vec![down], vec![])],
                )
                .into(),
            ],
        )],
    )
    .into();
    (
        sig,
        DiagramN::new(
            s_s.into(),
            vec![
                Cospan {
                    forward: untwist,
                    backward: Rewrite::identity(2),
                },
                Cospan {
                    forward: Rewrite::identity(2),
                    backward: twist,
                },
            ],
        ),
    )
}

// | |
// | b
// a |
// | |
pub fn two_beads() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let f = sig.add(x.clone(), x).unwrap();
    let a = sig.add(f.clone(), f.clone()).unwrap();
    let b = sig.add(f.clone(), f.clone()).unwrap();
    (
        sig,
        a.attach(&f, Target, &[])
            .unwrap()
            .attach(&b, Target, &[1])
            .unwrap(),
    )
}

// | |
// c |
// | b
// a |
// | |
#[allow(clippy::many_single_char_names)]
pub fn three_beads() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let f = sig.add(x.clone(), x).unwrap();
    let a = sig.add(f.clone(), f.clone()).unwrap();
    let b = sig.add(f.clone(), f.clone()).unwrap();
    let c = sig.add(f.clone(), f.clone()).unwrap();

    (
        sig,
        a.attach(&f, Target, &[])
            .unwrap()
            .attach(&b, Target, &[1])
            .unwrap()
            .attach(&c, Target, &[0])
            .unwrap(),
    )
}

//   m
// m |
// | |
pub fn stacks() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let f = sig.add(x.clone(), x.clone()).unwrap();
    let m = sig.add(f.clone(), x.identity()).unwrap();

    (
        sig,
        m.attach(&f, Target, &[])
            .unwrap()
            .attach(&m, Target, &[])
            .unwrap(),
    )
}

// |
// d
//
// u
// |
pub fn matchsticks() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let f = sig.add(x.clone(), x.clone()).unwrap();
    let up = sig.add(f.clone(), x.identity()).unwrap();
    let down = sig.add(x.identity(), f).unwrap();
    (sig, up.attach(&down, Target, &[]).unwrap())
}

// | |    ...    |
// | |    ...    e
// | |    ...    |
// | | (n times) |
// | |    ...    |
// | e    ...    |
// | |    ...    |
// e |    ...    |
// | |    ...    |
pub fn bead_series(n: usize) -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let f = sig.add(x.clone(), x).unwrap();
    let e = sig.add(f.clone(), f.clone()).unwrap();

    let mut res = e.clone();
    for i in 1..n {
        res = res
            .attach(&f, Target, &[])
            .unwrap()
            .attach(&e, Target, &[i])
            .unwrap();
    }

    (sig, res)
}

//    |
//    m
//   / \
//  |   |
//  |   u
//  |
//  m
// / \
pub fn monoid_unit() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    // 0-cells
    let x = sig.add_zero();

    // 1-cells
    let f = sig.add(x.clone(), x.clone()).unwrap();
    let ff = f.attach(&f, Target, &[]).unwrap();

    // 2-cells
    let m = sig.add(ff, f.clone()).unwrap();
    let u = sig.add(x.identity(), f).unwrap();

    (
        sig,
        m.attach(&u, Source, &[1])
            .unwrap()
            .attach(&m, Source, &[])
            .unwrap(),
    )
}

// |   |
// |   b
// | s |
// a   |
// |   |
#[allow(clippy::many_single_char_names)]
pub fn scalar_and_beads() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    // 0-cells
    let x = sig.add_zero();

    // 1-cells
    let f = sig.add(x.clone(), x.clone()).unwrap();

    // 2-cells
    let a = sig.add(f.clone(), f.clone()).unwrap();
    let b = sig.add(f.clone(), f.clone()).unwrap();
    let s = sig.add(x.identity(), x.identity()).unwrap();

    (
        sig,
        s.attach(&f, Source, &[])
            .unwrap()
            .attach(&f, Target, &[])
            .unwrap()
            .attach(&a, Source, &[0])
            .unwrap()
            .attach(&b, Target, &[1])
            .unwrap(),
    )
}

pub fn snake() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    // 0-cells
    let x = sig.add_zero();

    // 1-cells
    let f = sig.add(x.clone(), x.clone()).unwrap();
    let ff = f.attach(&f, Target, &[]).unwrap();

    // 2-cells
    let cap = sig.add(ff.clone(), x.identity()).unwrap();
    let cup = sig.add(x.identity(), ff).unwrap();
    let snake = cap
        .attach(&f, Target, &[])
        .unwrap()
        .attach(&cup, Source, &[1])
        .unwrap();

    // 3-cells
    let snake_cancel = sig.add(snake, f.identity()).unwrap();

    (sig, snake_cancel)
}

pub fn lips() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    // 0-cells
    let x = sig.add_zero();

    // 1-cells
    let f = sig.add(x.clone(), x.clone()).unwrap();
    let ff = f.attach(&f, Target, &[]).unwrap();

    // 2-cells
    let cap = sig.add(ff.clone(), x.identity()).unwrap();
    let cup = sig.add(x.identity(), ff).unwrap();
    let snake = cap
        .attach(&f, Target, &[])
        .unwrap()
        .attach(&cup, Source, &[1])
        .unwrap();

    // 3-cells
    let snake_death = sig.add(snake.clone(), f.identity()).unwrap();
    let snake_birth = sig.add(f.identity(), snake).unwrap();

    // 4-cells
    let lips = sig
        .add(
            f.identity().identity(),
            snake_birth.attach(&snake_death, Target, &[]).unwrap(),
        )
        .unwrap();

    (sig, lips)
}

pub fn pants_unit() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    // 0-cells
    let x = sig.add_zero();

    // 1-cells
    let f = sig.add(x.clone(), x.clone()).unwrap();
    let ff = f.attach(&f, Target, &[]).unwrap();

    // 2-cells
    let cap = sig.add(ff.clone(), x.identity()).unwrap();
    let cup = sig.add(x.identity(), ff.clone()).unwrap();

    // 3-cells
    let saddle = sig
        .add(cap.attach(&cup, Target, &[]).unwrap(), ff.identity())
        .unwrap();
    let sphere_birth = sig
        .add(
            x.identity().identity(),
            cup.attach(&cap, Target, &[]).unwrap(),
        )
        .unwrap();
    let three_snake = saddle
        .attach(&cap, Target, &[])
        .unwrap()
        .attach(&sphere_birth, Source, &[1])
        .unwrap();

    // 4-cells
    let pants_unit = sig.add(three_snake, cap.identity()).unwrap();

    (sig, pants_unit)
}

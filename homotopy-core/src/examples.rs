use crate::signature::{Signature, SignatureBuilder};
#[allow(clippy::wildcard_imports)]
use crate::*;

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
    let ff = f.attach(&f, Boundary::Target, &[]).unwrap();
    let m = sig.add(ff, f).unwrap();
    let left = m.attach(&m, Boundary::Source, &[0]).unwrap();
    let right = m.attach(&m, Boundary::Source, &[1]).unwrap();
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
    let wirewire = wire.attach(&wire, Boundary::Target, &[]).unwrap();
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
    (sig, s.attach(&t, Boundary::Target, &[]).unwrap())
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
        a.attach(&f, Boundary::Target, &[])
            .unwrap()
            .attach(&b, Boundary::Target, &[1])
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
        a.attach(&f, Boundary::Target, &[])
            .unwrap()
            .attach(&b, Boundary::Target, &[1])
            .unwrap()
            .attach(&c, Boundary::Target, &[0])
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
        m.attach(&f, Boundary::Target, &[])
            .unwrap()
            .attach(&m, Boundary::Target, &[])
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
    (sig, up.attach(&down, Boundary::Target, &[]).unwrap())
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
            .attach(&f, Boundary::Target, &[])
            .unwrap()
            .attach(&e, Boundary::Target, &[i])
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
    let ff = f.attach(&f, Boundary::Target, &[]).unwrap();

    // 2-cells
    let m = sig.add(ff, f.clone()).unwrap();
    let u = sig.add(x.identity(), f).unwrap();

    (
        sig,
        m.attach(&u, Boundary::Source, &[1])
            .unwrap()
            .attach(&m, Boundary::Source, &[])
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
        s.attach(&f, Boundary::Source, &[])
            .unwrap()
            .attach(&f, Boundary::Target, &[])
            .unwrap()
            .attach(&a, Boundary::Source, &[0])
            .unwrap()
            .attach(&b, Boundary::Target, &[1])
            .unwrap(),
    )
}

pub fn snake() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    // 0-cells
    let x = sig.add_zero();

    // 1-cells
    let f = sig.add(x.clone(), x.clone()).unwrap();
    let ff = f.attach(&f, Boundary::Target, &[]).unwrap();

    // 2-cells
    let cap = sig.add(ff.clone(), x.identity()).unwrap();
    let cup = sig.add(x.identity(), ff).unwrap();
    let snake = cap
        .attach(&f, Boundary::Target, &[])
        .unwrap()
        .attach(&cup, Boundary::Source, &[1])
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
    let ff = f.attach(&f, Boundary::Target, &[]).unwrap();

    // 2-cells
    let cap = sig.add(ff.clone(), x.identity()).unwrap();
    let cup = sig.add(x.identity(), ff).unwrap();
    let snake = cap
        .attach(&f, Boundary::Target, &[])
        .unwrap()
        .attach(&cup, Boundary::Source, &[1])
        .unwrap();

    // 3-cells
    let snake_death = sig.add(snake.clone(), f.identity()).unwrap();
    let snake_birth = sig.add(f.identity(), snake).unwrap();

    // 4-cells
    let lips = sig
        .add(
            f.identity().identity(),
            snake_birth
                .attach(&snake_death, Boundary::Target, &[])
                .unwrap(),
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
    let ff = f.attach(&f, Boundary::Target, &[]).unwrap();

    // 2-cells
    let cap = sig.add(ff.clone(), x.identity()).unwrap();
    let cup = sig.add(x.identity(), ff.clone()).unwrap();

    // 3-cells
    let saddle = sig
        .add(
            cap.attach(&cup, Boundary::Target, &[]).unwrap(),
            ff.identity(),
        )
        .unwrap();
    let sphere_birth = sig
        .add(
            x.identity().identity(),
            cup.attach(&cap, Boundary::Target, &[]).unwrap(),
        )
        .unwrap();
    let three_snake = saddle
        .attach(&cap, Boundary::Target, &[])
        .unwrap()
        .attach(&sphere_birth, Boundary::Source, &[1])
        .unwrap();

    // 4-cells
    let pants_unit = sig.add(three_snake, cap.identity()).unwrap();

    (sig, pants_unit)
}

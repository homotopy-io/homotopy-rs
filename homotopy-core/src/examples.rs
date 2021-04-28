use crate::signature::{Signature, SignatureBuilder};
#[allow(clippy::wildcard_imports)]
use crate::*;

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

// project out 1 dimension:
//   m
// m |
// | |
pub fn stacks() -> (impl Signature, DiagramN) {
    let mut sig = SignatureBuilder::new();

    let x = sig.add_zero();
    let f = sig.add(x.identity(), x.identity()).unwrap();
    let m = sig.add(f.clone(), x.identity().identity()).unwrap();

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

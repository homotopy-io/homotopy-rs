use contraction::contraction;
use criterion::criterion_main;
use expansion::expansion;

mod contraction;
mod expansion;

criterion_main!(contraction, expansion);

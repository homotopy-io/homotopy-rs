mod contraction;
mod expansion;
use contraction::contraction;
use criterion::criterion_main;
use expansion::expansion;

criterion_main!(contraction, expansion);

use collapse::collapse;
use contraction::contraction;
use criterion::criterion_main;
use expansion::expansion;

mod collapse;
mod contraction;
mod expansion;

criterion_main!(collapse, contraction, expansion);

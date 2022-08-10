use std::fmt::Write;

use homotopy_core::{Generator, Orientation};

use crate::style::{GeneratorStyle, SignatureStyleData};

pub mod render;
pub mod shape;

pub fn stylesheet(styles: &impl SignatureStyleData) -> String {
    let mut stylesheet = String::new();

    writeln!(stylesheet, ".inverse--wire {{ stroke-dasharray: 2; }}").unwrap();

    writeln!(stylesheet, ".inverse--point {{ stroke: #ffffff; }}").unwrap();

    for (generator, style) in styles.as_pairs() {
        writeln!(
            stylesheet,
            ".{name} {{ fill: {color}; stroke: {color}; }}",
            name = raw_generator_class(generator, "surface"),
            color = &style.color().lighten(0.1).hex()
        )
        .unwrap();
        writeln!(
            stylesheet,
            ".{name} {{ stroke: {color}; }}",
            name = raw_generator_class(generator, "wire"),
            color = &style.color().lighten(0.05).hex()
        )
        .unwrap();
        writeln!(
            stylesheet,
            ".{name} {{ fill: {color}; }}",
            name = raw_generator_class(generator, "point"),
            color = &style.color().hex()
        )
        .unwrap();
    }

    stylesheet
}

pub fn raw_generator_class(generator: Generator, suffix: &str) -> String {
    format!(
        "generator__{}-{}--{}",
        generator.id, generator.dimension, suffix
    )
}

pub fn generator_class(generator: Generator, orientation: Orientation, suffix: &str) -> String {
    let class = raw_generator_class(generator, suffix);
    if orientation == Orientation::Positive {
        class
    } else {
        format!("{} inverse--{}", class, suffix)
    }
}

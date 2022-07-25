use std::fmt::Write;

use homotopy_core::Generator;

use crate::style::{GeneratorStyle, SignatureStyleData};

pub mod render;
pub mod shape;

pub fn stylesheet(styles: &impl SignatureStyleData) -> String {
    let mut stylesheet = String::new();

    for (generator, style) in styles.as_pairs() {
        writeln!(
            stylesheet,
            ".{name} {{ fill: {color}; stroke: {color}; }}",
            name = generator_class(generator, "surface"),
            color = &style.color().lighten(0.1).hex()
        )
        .unwrap();
        writeln!(
            stylesheet,
            ".{name} {{ stroke: {color}; }}",
            name = generator_class(generator, "wire"),
            color = &style.color().lighten(0.05).hex()
        )
        .unwrap();
        writeln!(
            stylesheet,
            ".{name} {{ fill: {color}; }}",
            name = generator_class(generator, "point"),
            color = &style.color().hex()
        )
        .unwrap();
    }

    stylesheet
}

pub fn generator_class(generator: Generator, suffix: &str) -> String {
    format!(
        "generator__{}-{}--{}",
        generator.id, generator.dimension, suffix
    )
}

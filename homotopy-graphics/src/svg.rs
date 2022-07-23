use std::fmt::Write;

use homotopy_core::Generator;

use crate::style::{Color, GeneratorStyle, SignatureStyleData};

pub mod render;
pub mod shape;

pub fn stylesheet(styles: &impl SignatureStyleData, prefix: &str) -> String {
    let mut stylesheet = String::new();

    for generator in styles.generators() {
        let style = styles.generator_style(generator).unwrap();

        writeln!(
            stylesheet,
            ".{name} {{ fill: {color}; stroke: {color}; }}",
            name = class(prefix, generator, "surface"),
            color = hex(&style.color().lighten(0.1))
        )
        .unwrap();
        writeln!(
            stylesheet,
            ".{name} {{ stroke: {color}; }}",
            name = class(prefix, generator, "wire"),
            color = hex(&style.color().lighten(0.05))
        )
        .unwrap();
        writeln!(
            stylesheet,
            ".{name} {{ fill: {color}; }}",
            name = class(prefix, generator, "point"),
            color = hex(&style.color())
        )
        .unwrap();
    }

    stylesheet
}

pub fn class(prefix: &str, generator: Generator, suffix: &str) -> String {
    format!(
        "{}__{}-{}--{}",
        prefix, generator.id, generator.dimension, suffix
    )
}

fn hex(color: &Color) -> String {
    format!("#{:X}", color.0)
}

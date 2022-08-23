use std::fmt::Write;

use homotopy_core::{Generator, Orientation};

use crate::style::{GeneratorStyle, SignatureStyleData};

pub mod render;
pub mod shape;

macro_rules! write_style_for {
    (@fmt_str "surface") => {
        ".{name}{orientation} {{ fill: {color}; stroke: {color}; }}"
    };
    (@fmt_str "wire") => {
        ".{name}{orientation} {{ stroke: {color}; }}"
    };
    (@fmt_str "point") => {
        ".{name}{orientation} {{ fill: {color}; }}"
    };
    ($generator:expr, $stylesheet:expr, $orientation:literal, $codimension:tt, $color:expr) => {{
        writeln!(
            $stylesheet,
            write_style_for!(@fmt_str $codimension),
            name = generator_class($generator, $codimension),
            orientation = format!(".{}--{}", $orientation, $codimension),
            color = $color.hex(),
        )
        .unwrap()
    }};
    ($generator:expr, $stylesheet:expr, $codimension:tt, $color:expr) => {{
        writeln!(
            $stylesheet,
            write_style_for!(@fmt_str $codimension),
            name = generator_class($generator, $codimension),
            orientation = "",
            color = $color.hex(),
        )
        .unwrap()
    }};
}

pub fn stylesheet(styles: &impl SignatureStyleData) -> String {
    let mut stylesheet = String::new();

    for (generator, style) in styles.as_pairs() {
        write_style_for!(
            generator,
            stylesheet,
            "inverse",
            "point",
            style.color().with_lightness(0.15)
        );
        write_style_for!(
            generator,
            stylesheet,
            "zero",
            "point",
            style.color().with_lightness(0.23)
        );
        write_style_for!(
            generator,
            stylesheet,
            "point",
            style.color().with_lightness(0.31)
        );

        write_style_for!(
            generator,
            stylesheet,
            "inverse",
            "wire",
            style.color().with_lightness(0.39)
        );
        write_style_for!(
            generator,
            stylesheet,
            "zero",
            "wire",
            style.color().with_lightness(0.47)
        );
        write_style_for!(
            generator,
            stylesheet,
            "wire",
            style.color().with_lightness(0.55)
        );

        write_style_for!(
            generator,
            stylesheet,
            "inverse",
            "surface",
            style.color().with_lightness(0.63)
        );
        write_style_for!(
            generator,
            stylesheet,
            "zero",
            "surface",
            style.color().with_lightness(0.71)
        );
        write_style_for!(
            generator,
            stylesheet,
            "surface",
            style.color().with_lightness(0.79)
        );
    }

    stylesheet
}

pub fn generator_class(generator: Generator, suffix: &str) -> String {
    match generator.orientation {
        Orientation::Positive => format!(
            "generator__{}-{}--{}",
            generator.id, generator.dimension, suffix
        ),
        Orientation::Zero => format!(
            "generator__{}-{}--{} zero--{}",
            generator.id, generator.dimension, suffix, suffix
        ),
        Orientation::Negative => format!(
            "generator__{}-{}--{} inverse--{}",
            generator.id, generator.dimension, suffix, suffix
        ),
    }
}

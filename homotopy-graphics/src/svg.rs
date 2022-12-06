use std::fmt::Write;

use homotopy_core::{Generator, Orientation};

use crate::style::{GeneratorRepresentation, GeneratorStyle, SignatureStyleData};

pub mod render;
pub mod shape;

macro_rules! write_styles_for {
    (
        @c_r
        $c:expr,
        $orientation:expr,
        $generator:expr,
        $color:expr,
        $stylesheet:expr
     ) => {{
        writeln!(
            $stylesheet,
            ".{name} {{ fill: {color}; stroke: {color}; }}",
            name = generator_class($generator, $c, $orientation),
            color = $color.lighten($c, $orientation).hex(),
        )
        .unwrap()
    }};
    (
        $generator:expr,
        $styles:expr,
        $stylesheet:expr
    ) => {{
        let color = $styles.generator_style($generator).unwrap().color();

        for c in 0..3 {
            write_styles_for!(@c_r c, Orientation::Positive, $generator, color, $stylesheet);
            write_styles_for!(@c_r c, Orientation::Zero, $generator, color, $stylesheet);
            write_styles_for!(@c_r c, Orientation::Negative, $generator, color, $stylesheet);
        }
    }};
}

pub fn stylesheet(styles: &impl SignatureStyleData) -> String {
    let mut stylesheet = String::new();

    writeln!(
        stylesheet,
        ".wire {{ fill: none !important; }} .point {{ stroke: none !important; }}",
    )
    .unwrap();

    for generator in styles.generators() {
        write_styles_for!(generator, styles, stylesheet);
    }

    stylesheet
}

#[inline]
pub fn generator_class_from_diagram_dim(
    generator: Generator,
    orientation: Orientation,
    diagram_dimension: usize,
    representation: GeneratorRepresentation,
) -> String {
    let d = diagram_dimension;
    let n = generator.dimension;
    let k = representation as usize;

    let c = d.saturating_sub(n + k) % 3;

    format!(
        "{} {}",
        generator_class(generator, c, orientation),
        representation,
    )
}

#[inline]
fn generator_class(generator: Generator, c: usize, orientation: Orientation) -> String {
    format!(
        "generator__{}-{}--{}-{}",
        generator.id,
        generator.dimension,
        c,
        match orientation {
            Orientation::Positive => "pos",
            Orientation::Negative => "neg",
            Orientation::Zero => "zer",
        }
    )
}

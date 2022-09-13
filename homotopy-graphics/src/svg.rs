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
        $style:expr,
        $stylesheet:expr,
    ) => {{
        let color = $style.color();

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

    for (generator, style) in styles.as_pairs() {
        write_styles_for!(generator, style, stylesheet,);
    }

    stylesheet
}

#[inline]
pub fn generator_class_from_diagram_dim(
    generator: Generator,
    diagram_dimension: usize,
    representation: GeneratorRepresentation,
) -> String {
    let d = diagram_dimension;
    let n = generator.dimension;
    let k = representation as usize;

    let c = d.saturating_sub(n + k);

    format!(
        "{} {}",
        generator_class(generator, c, generator.orientation),
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

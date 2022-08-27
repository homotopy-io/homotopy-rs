use std::fmt::Write;

use homotopy_core::{Generator, Orientation};

use crate::style::{GeneratorStyle, SignatureStyleData};

pub mod render;
pub mod shape;

macro_rules! write_styles_for {
    (
        @offset_at
        $offset:expr,
        $generator:expr,
        $color:expr,
        $stylesheet:expr
     ) => {{
        writeln!(
            $stylesheet,
            ".{name} {{ fill: {color}; stroke: {color}; }}",
            name = generator_class_from_offset($generator, $offset),
            color = $color.lighten_from_offset($offset).hex(),
        )
        .unwrap()
    }};
    (
        $generator:expr,
        $style:expr,
        $stylesheet:expr,
    ) => {{
        let color = $style.color();

        for offset in 0..9 {
            write_styles_for!(@offset_at offset, $generator, color, $stylesheet);
        }
    }};
}

pub fn stylesheet(
    styles: &impl SignatureStyleData,
    lightness1: f32,
    lightness2: f32,
    lightness3: f32,
    lightness4: f32,
    lightness5: f32,
    lightness6: f32,
    lightness7: f32,
    lightness8: f32,
    lightness9: f32,
) -> String {
    let mut stylesheet = String::new();

    writeln!(
        stylesheet,
        ".wire {{ fill: none !important; }} .point {{ stroke: none !important; }}",
    )
    .unwrap();


    for (generator, style) in styles.as_pairs() {
        write_styles_for!(
            generator,
            style,
            stylesheet,
        );
    }

    stylesheet
}

#[inline]
pub fn generator_class_from_diagram_dim(generator: Generator, diagram_dimension: usize) -> String {
    let orientation = match generator.orientation {
        Orientation::Positive => 0,
        Orientation::Zero => 1,
        Orientation::Negative => 2,
    };
    let offset = 3 * orientation + (diagram_dimension - generator.dimension.min(diagram_dimension)).min(2);

    let codimension = match diagram_dimension - generator.dimension.min(diagram_dimension) {
        0 => "point",
        1 => "wire",
        _ => "",
    };

    format!("{} {}", generator_class_from_offset(generator, offset % 9), codimension)
}

#[inline]
fn generator_class_from_offset(generator: Generator, offset: usize) -> String {
    format!("generator__{}-{}--{}", generator.id, generator.dimension, offset)
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

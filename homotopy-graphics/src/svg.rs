use std::fmt::Write;

use homotopy_core::{Generator, Orientation};

use crate::style::{GeneratorStyle, SignatureStyleData};

pub mod render;
pub mod shape;

macro_rules! write_styles_for {
    (
        @c_r
        $c:expr,
        $r:expr,
        $generator:expr,
        $color:expr,
        $stylesheet:expr
     ) => {{
        writeln!(
            $stylesheet,
            ".{name} {{ fill: {color}; stroke: {color}; }}",
            name = generator_class_from_c_r($generator, $c, $r),
            color = $color.lighten_from_c_r($c, $r).hex(),
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
            for r in -1..=1 {
                write_styles_for!(@c_r c, r, $generator, color, $stylesheet);
            }
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
    k: usize,
    diagram_dimension: usize,
) -> String {
    let r = match generator.orientation {
        Orientation::Positive => 1,
        Orientation::Zero => 0,
        Orientation::Negative => -1,
    };
    let d = diagram_dimension as isize;
    let n = generator.dimension as isize;
    let k = k as isize;

    let c = (d - n - k).max(0);

    format!(
        "{} {}",
        generator_class_from_c_r(generator, c, r),
        match k {
            0 => "point",
            1 => "wire",
            _ => "",
        },
    )
}

#[inline]
fn generator_class_from_c_r(generator: Generator, c: isize, r: isize) -> String {
    format!(
        "generator__{}-{}--{}-{}",
        generator.id,
        generator.dimension,
        c,
        match r {
            1 => "pos",
            -1 => "neg",
            _ => "zer",
        }
    )
}

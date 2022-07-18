use std::fmt::Write;

use euclid::default::Point2D;
use homotopy_common::hash::FastHashMap;
use homotopy_core::{
    common::DimensionError,
    complex::make_complex,
    layout::Layout,
    projection::{Depths, Projection},
    Diagram, Generator,
};
use itertools::Itertools;
use lyon_path::{Event, Path};

use crate::{
    path_util::{offset, simplify_graphic},
    style::{GeneratorStyle, SignatureStyleData, VertexShape},
    svg::render::GraphicElement,
};

const OCCLUSION_DELTA: f32 = 0.2;

pub fn color(generator: Generator) -> String {
    format!("generator-{}-{}", generator.id, generator.dimension)
}

pub fn render(
    diagram: &Diagram,
    stylesheet: &str,
    signature_styles: &impl SignatureStyleData,
) -> Result<String, DimensionError> {
    let layout = Layout::<2>::new(diagram)?;
    let complex = make_complex(diagram);
    let depths = Depths::<2>::new(diagram)?;
    let projection = Projection::<2>::new(diagram, &layout, &depths)?;
    let graphic = simplify_graphic(&GraphicElement::build(
        &complex,
        &layout,
        &projection,
        &depths,
    ));

    let mut surfaces = Vec::default();
    let mut wires: FastHashMap<usize, Vec<(Generator, Path)>> = FastHashMap::default();
    let mut points = Vec::default();
    for element in graphic {
        match element {
            GraphicElement::Surface(g, path) => surfaces.push((g, path)),
            GraphicElement::Wire(g, depth, path, _) => {
                wires.entry(depth).or_default().push((g, path));
            }
            GraphicElement::Point(g, point) => points.push((g, point)),
        }
    }

    let mut tikz = String::new();
    writeln!(tikz, "\\begin{{tikzpicture}}").unwrap();
    tikz.push_str(stylesheet);

    // We only worry with masking if it's actually needed.
    if wires.len() > 1 {
        writeln!(tikz, "% Rendering with masked").unwrap(); //TODO remove
        tikz.push_str(&render_masked(surfaces, wires));
    } else {
        writeln!(tikz, "% Rendering with unmasked").unwrap(); //TODO remove
        tikz.push_str(&render_unmasked(surfaces, wires));
    }

    // Points are unchanged
    for (g, point) in points {
        let vertex = render_vertex(signature_styles.generator_style(g).unwrap(), point);
        writeln!(tikz, "\\fill[{}] {}", color(g), vertex).unwrap();
    }

    writeln!(tikz, "\\end{{tikzpicture}}").unwrap();

    Ok(tikz)
}

// Simpler renderer in case masking is not needed.
fn render_unmasked(
    surfaces: Vec<(Generator, Path)>,
    wires: FastHashMap<usize, Vec<(Generator, Path)>>,
) -> String {
    let mut tikz = String::new();

    // Surfaces
    for (g, path) in surfaces {
        write!(
            tikz,
            "\\fill[{}!75, name={}-{}] ",
            color(g),
            g.id,
            g.dimension
        )
        .unwrap();
        tikz.push_str(&render_path(&path));
        writeln!(tikz, ";").unwrap();
    }

    for (_, layer) in wires.into_iter().sorted_by_cached_key(|(k, _)| *k).rev() {
        for (g, path) in &layer {
            write!(tikz, "\\draw[{}!80, line width=5pt] ", color(*g)).unwrap();
            tikz.push_str(&render_path(path));
            writeln!(tikz, ";").unwrap();
        }
    }

    tikz
}

fn render_masked(
    surfaces: Vec<(Generator, Path)>,
    wires: FastHashMap<usize, Vec<(Generator, Path)>>,
) -> String {
    let mut tikz = String::new();

    // Surfaces
    writeln!(tikz, "\\newcommand*\\Background{{").unwrap();
    for (g, path) in surfaces {
        write!(
            tikz,
            "\\fill[{}!75, name={}-{}] ",
            color(g),
            g.id,
            g.dimension
        )
        .unwrap();
        tikz.push_str(&render_path(&path));
        writeln!(tikz, ";").unwrap();
    }
    writeln!(tikz, "}}").unwrap();
    writeln!(tikz, "\\Background").unwrap();

    // Wires
    for (i, (_, layer)) in wires
        .into_iter()
        .sorted_by_cached_key(|(k, _)| *k)
        .rev()
        .enumerate()
    {
        // Background
        if i > 0 {
            writeln!(tikz, "\\begin{{scope}}").unwrap();
            write!(tikz, "\\clip ").unwrap();
            for (_, path) in &layer {
                tikz.push_str(&offset_multiple(OCCLUSION_DELTA, path));
            }
            writeln!(tikz, ";").unwrap();
            writeln!(tikz, "\\Background").unwrap();
            writeln!(tikz, "\\end{{scope}}").unwrap();
        }

        for (g, path) in &layer {
            write!(tikz, "\\draw[{}!80, line width=5pt] ", color(*g)).unwrap();
            tikz.push_str(&render_path(path));
            writeln!(tikz, ";").unwrap();
        }
    }

    tikz
}

fn render_point(point: Point2D<f32>) -> String {
    let x = (point.x * 100.0).round() / 100.0;
    let y = (point.y * 100.0).round() / 100.0;
    format!("({}, {})", x, y)
}

fn render_vertex(generator_style: &impl GeneratorStyle, point: Point2D<f32>) -> String {
    use VertexShape::{Circle, Square};

    const CIRCLE_RADIUS: f32 = 0.14; // r = 4pt
    const SQUARE_SIDELENGTH: f32 = 0.28; // 8pt x 8pt

    let shape = generator_style.shape().unwrap_or_default();
    let shape_str = match shape {
        Circle => "circle",
        Square => "rectangle",
    };
    let (xo, yo) = match shape {
        Circle => (0.0, 0.0),
        Square => (-SQUARE_SIDELENGTH / 2., -SQUARE_SIDELENGTH / 2.),
    };
    let x1 = ((xo + point.x) * 100.0).round() / 100.0;
    let y1 = ((yo + point.y) * 100.0).round() / 100.0;
    let sz = match shape {
        Circle => vec![CIRCLE_RADIUS],
        Square => vec![SQUARE_SIDELENGTH + x1, SQUARE_SIDELENGTH + y1],
    }
    .iter()
    .map(|&s| s.to_string())
    .collect::<Vec<String>>()
    .join(", ");
    format!("({},{}) {} ({});", x1, y1, shape_str, sz)
}

fn render_path(path: &Path) -> String {
    let mut result = String::new();
    for event in path {
        match event {
            Event::Begin { at } => result.push_str(&render_point(at)),
            Event::Line { to, .. } => write!(result, " -- {}", render_point(to)).unwrap(),
            Event::Quadratic { ctrl, to, .. } => write!(
                result,
                " .. controls {} .. {}",
                render_point(ctrl),
                render_point(to)
            )
            .unwrap(),
            Event::Cubic {
                ctrl1, ctrl2, to, ..
            } => write!(
                result,
                " .. controls {} and {} .. {}",
                render_point(ctrl1),
                render_point(ctrl2),
                render_point(to),
            )
            .unwrap(),
            Event::End { close, .. } => {
                if close {
                    write!(result, " -- cycle").unwrap();
                }
            }
        }
    }
    result
}

//TODO move to path_util once the dependency on render_path is removed.
fn offset_multiple(delta: f32, path: &Path) -> String {
    let mut tikz = String::new();
    let mut builder = Path::builder();
    for event in path {
        match event {
            Event::End { .. } => {
                builder.path_event(event);
                let segment = builder.build();
                let left = offset(-delta, &segment)
                    .reversed()
                    .with_attributes()
                    .into_path();
                let right = offset(delta, &segment);
                tikz.push_str(&render_path(&right));
                tikz.push_str(" -- ");
                tikz.push_str(&render_path(&left));
                tikz.push_str(" -- cycle");
                builder = Path::builder();
            }
            _ => {
                builder.path_event(event);
            }
        }
    }
    tikz
}

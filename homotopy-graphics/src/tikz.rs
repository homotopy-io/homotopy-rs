use std::fmt::Write;

use euclid::default::Point2D;
use homotopy_common::hash::{FastHashMap, FastHashSet};
use homotopy_core::{
    common::DimensionError,
    complex::make_complex,
    diagram::Diagram0,
    layout::Layout,
    projection::{Depths, Projection},
    Diagram, Generator, Orientation,
};
use itertools::Itertools;
use lyon_path::{Event, Path};

use crate::{
    path_util::simplify_graphic,
    style::{Color, GeneratorRepresentation, GeneratorStyle, SignatureStyleData, VertexShape},
    svg::render::GraphicElement,
};

#[allow(clippy::implicit_hasher)]
pub fn stylesheet(
    styles: &impl SignatureStyleData,
    dimension: usize,
    diagrams: &FastHashSet<(Diagram0, GeneratorRepresentation)>,
) -> String {
    let mut stylesheet = String::new();

    for diagram in diagrams {
        let generator = diagram.0.generator;
        let orientation = diagram.0.orientation;
        let representation = diagram.1;
        let selector = color_selector(diagram.0, dimension, representation);
        let color = styles.generator_style(generator).unwrap().color();
        writeln!(
            stylesheet,
            "\\definecolor{{{generator}}}{color}",
            generator = name(generator, selector, orientation),
            color = rgb(color.lighten(selector, orientation).clone()),
        )
        .unwrap();
    }

    stylesheet
}

#[inline]
pub fn color_selector(
    diagram: Diagram0,
    diagram_dimension: usize,
    representation: GeneratorRepresentation,
) -> usize {
    let d = diagram_dimension;
    let n = diagram.generator.dimension;
    let k = representation as usize;

    d.saturating_sub(n + k) % 3
}

#[inline]
pub fn name_from_diagram_dim(
    diagram: Diagram0,
    diagram_dimension: usize,
    representation: GeneratorRepresentation,
) -> String {
    name(
        diagram.generator,
        color_selector(diagram, diagram_dimension, representation),
        diagram.orientation,
    )
}

fn name(generator: Generator, c: usize, orientation: Orientation) -> String {
    format!(
        "generator-{}-{}-{c}-{}",
        generator.id,
        generator.dimension,
        match orientation {
            Orientation::Positive => "pos",
            Orientation::Negative => "neg",
            Orientation::Zero => "zer",
        }
    )
}

fn rgb(color: Color) -> String {
    let (r, g, b) = color.into_components::<u8>();
    format!("{{RGB}}{{{r}, {g}, {b}}}")
}

pub fn render(
    diagram: &Diagram,
    signature_styles: &impl SignatureStyleData,
    leftright_mode: bool,
    show_braids: bool,
) -> Result<String, DimensionError> {
    let dimension = diagram.dimension();
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
    let mut wires: FastHashMap<usize, Vec<(Diagram0, Path)>> = FastHashMap::default();
    let mut points = Vec::default();
    let mut diagrams: FastHashSet<_> = Default::default();
    for element in graphic {
        match element {
            GraphicElement::Surface(g, path) => {
                diagrams.insert((g, GeneratorRepresentation::Surface));
                surfaces.push((g, path));
            }
            GraphicElement::Wire(g, depth, path, _) => {
                diagrams.insert((g, GeneratorRepresentation::Wire));
                wires.entry(depth).or_default().push((g, path));
            }
            GraphicElement::Point(g, point) => {
                diagrams.insert((g, GeneratorRepresentation::Point));
                points.push((g, point));
            }
        }
    }

    let mut tikz = String::new();
    writeln!(tikz, "\\begin{{tikzpicture}}").unwrap();
    tikz.push_str(&stylesheet(signature_styles, dimension, &diagrams));

    tikz.push_str(&render_inner(
        &surfaces,
        wires,
        leftright_mode,
        show_braids,
        dimension,
    ));

    // Points are unchanged
    for (d, point) in points {
        let vertex = render_vertex(
            signature_styles.generator_style(d.generator).unwrap(),
            point,
            leftright_mode,
        );
        writeln!(
            tikz,
            "\\fill[{}] {}",
            name_from_diagram_dim(d, dimension, GeneratorRepresentation::Point),
            vertex
        )
        .unwrap();
    }

    writeln!(tikz, "\\end{{tikzpicture}}").unwrap();

    Ok(tikz)
}

// This contains all the "magic" commands we need to inject
// in the case we want to show braidings.
const MAGIC_MACRO: &str = "\n\\newcommand{\\wire}[2]{
  \\ifdefined\\recolor\\draw[color=\\recolor, line width=10pt]\\else\\draw[color=#1, line width=5pt]\\fi #2;
}
\\newcommand{\\clipped}[3]{
\\begin{scope}
  \\newcommand{\\recolor}{#1}
  \\clip#3;
  #2
\\end{scope}
}\n\n";

fn render_inner(
    surfaces: &[(Diagram0, Path)],
    wires: FastHashMap<usize, Vec<(Diagram0, Path)>>,
    leftright_mode: bool,
    show_braids: bool,
    diagram_dimension: usize,
) -> String {
    let mut tikz = String::new();

    let needs_masking = wires.len() > 1 && show_braids;

    if needs_masking {
        tikz.push_str(MAGIC_MACRO);
        // The transparency group does nothing in terms of our masking strategy,
        // but it is useful in case users lower the opacity. It changes the
        // opacity rules to hide our masking shenanigans.
        // If you want to see it in action, add [opacity=.5] at
        // at \begin{tikzpicture} and remove [transparency group].
        tikz.push_str("\\begin{scope}[transparency group]\n");
    } else {
        tikz.push_str("\\begin{scope}\n");
    }

    tikz.push_str("% Background surfaces\n");
    for (g, path) in surfaces.iter() {
        writeln!(
            tikz,
            "\\fill[{color}] {path};",
            color = name_from_diagram_dim(*g, diagram_dimension, GeneratorRepresentation::Surface),
            path = &render_path(path, leftright_mode)
        )
        .unwrap();
    }

    // Since we always clip with respect to the same background paths,
    // might as well make a macro for it and have TeX do the CTRL+V for us.
    if needs_masking {
        writeln!(tikz, "\\newcommand{{\\layer}}[1]{{",).unwrap();
        for (g, path) in surfaces.iter() {
            writeln!(
                tikz,
                "  \\clipped{{{color}}}{{#1}}{{{path}}}",
                color =
                    name_from_diagram_dim(*g, diagram_dimension, GeneratorRepresentation::Surface),
                path = &render_path(path, leftright_mode)
            )
            .unwrap();
        }
        tikz.push_str("  #1\n}\n\n");
    }

    // The masking logic mostly concerns the wires.
    // Unlike the background, wires do not all share the same depth.
    // Here we create a series of "layer" commands, which
    // either render their own paths or recourse to the successive layer.
    //
    // The \wire command checks if \recolor is defined,
    // which allows to override the colour into something else.
    // We use this to switch between drawing the wire in "mask mode"
    // and "normal mode". The if/else statement is run on TeX side,
    // and the logic is in MAGIC_MACRO!
    tikz.push_str("% Wire layers\n");
    for (i, (_, layer)) in wires
        .into_iter()
        .sorted_by_cached_key(|(k, _)| *k)
        .rev()
        .enumerate()
    {
        if i > 0 && needs_masking {
            tikz.push_str("\\layer{\n");
        }
        for (g, path) in &layer {
            if needs_masking {
                // We pass the geometry of the wire directly to the current layer.
                // This is to avoid naming annoyances.
                writeln!(
                    tikz,
                    "\\wire{{{color}}}{{{path}}};",
                    color =
                        name_from_diagram_dim(*g, diagram_dimension, GeneratorRepresentation::Wire),
                    path = &render_path(path, leftright_mode)
                )
                .unwrap();
            } else {
                writeln!(
                    tikz,
                    "\\draw[color={color}, line width=5pt]{path};",
                    color =
                        name_from_diagram_dim(*g, diagram_dimension, GeneratorRepresentation::Wire),
                    path = &render_path(path, leftright_mode)
                )
                .unwrap();
            }
        }
        if i > 0 && needs_masking {
            tikz.push_str("}\n");
        }
    }

    tikz.push_str("\\end{scope}\n");

    tikz
}

fn render_point(point: Point2D<f32>, leftright_mode: bool) -> String {
    let x = (point.x * 100.0).round() / 100.0;
    let y = (point.y * 100.0).round() / 100.0;
    if leftright_mode {
        format!("({y},{})", -x)
    } else {
        format!("({x},{y})")
    }
}

fn render_vertex(
    generator_style: &impl GeneratorStyle,
    point: Point2D<f32>,
    leftright_mode: bool,
) -> String {
    use VertexShape::{Circle, Square};

    const CIRCLE_RADIUS: f32 = 0.14; // r = 4pt
    const SQUARE_SIDELENGTH: f32 = 0.28; // 8pt x 8pt

    let shape = generator_style.shape();
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

    if leftright_mode {
        format!("({y1},{}) {shape_str} ({sz});", -x1)
    } else {
        format!("({x1},{y1}) {shape_str} ({sz});")
    }
}

fn render_path(path: &Path, leftright_mode: bool) -> String {
    let mut result = String::new();
    for event in path {
        match event {
            Event::Begin { at } => result.push_str(&render_point(at, leftright_mode)),
            Event::Line { to, .. } => {
                write!(result, " -- {}", render_point(to, leftright_mode)).unwrap();
            }
            Event::Quadratic { ctrl, to, .. } => write!(
                result,
                " .. controls {} .. {}",
                render_point(ctrl, leftright_mode),
                render_point(to, leftright_mode)
            )
            .unwrap(),
            Event::Cubic {
                ctrl1, ctrl2, to, ..
            } => write!(
                result,
                " .. controls {} and {} .. {}",
                render_point(ctrl1, leftright_mode),
                render_point(ctrl2, leftright_mode),
                render_point(to, leftright_mode),
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

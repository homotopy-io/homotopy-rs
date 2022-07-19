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
use numerals::roman::Roman;

use crate::{
    path_util::simplify_graphic,
    style::{GeneratorStyle, SignatureStyleData, VertexShape},
    svg::render::GraphicElement,
};

pub fn color(generator: Generator) -> String {
    format!("generator-{}-{}", generator.id, generator.dimension)
}

// Idea: we want to define custom commands to access our bulky geometrical data for each
// generator.
//
// Problem: TeX doesn't allow either numbers or underscores/dashes in command names
// without recourse to some exceptionally ugly hacks.
//
// Solution: we use Roman numerals, which are just letters!
//
// Remark: technically, TeX does have a way to produce its own Roman numerals
// but to make it happen in the places we need, i.e. \newcommand,
// we risk accidentally invoking Cthulhu.
pub fn name(generator: Generator, depth: usize) -> String {
    format!(
        "\\gen{:x}O{:x}O{:x}",
        Roman::from((generator.id + 1) as i16),
        Roman::from((generator.dimension + 1) as i16),
        Roman::from((depth + 1) as i16)
    )
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
        tikz.push_str(&render_masked(&surfaces, wires));
    } else {
        tikz.push_str(&render_unmasked(&surfaces, wires));
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
    surfaces: &[(Generator, Path)],
    wires: FastHashMap<usize, Vec<(Generator, Path)>>,
) -> String {
    let mut tikz = String::new();

    // Surfaces
    for (g, path) in surfaces.iter() {
        writeln!(tikz, "\\fill[{}!75]{};", color(*g), render_path(path)).unwrap();
    }

    for (_, layer) in wires.into_iter().sorted_by_cached_key(|(k, _)| *k).rev() {
        for (g, path) in &layer {
            writeln!(
                tikz,
                "\\draw[{}!80, line width=5pt]{};",
                color(*g),
                render_path(path)
            )
            .unwrap();
        }
    }

    tikz
}

// This contains all the "magic" commands we need to inject.
// No formatting needed, it's the same all the time.
const MAGIC_MACRO: &str = "\n\\newcommand{\\layered}[4]{
  \\ifnum#3>0 \\draw[color=#4!75, line width=10pt]\\else\\draw[color=#2!80, line width=5pt]\\fi #1;
}%
\\newcommand{\\clipped}[3]{%
  \\begin{scope}
    \\clip#1;
    #3{1}{#2}
  \\end{scope}
}%\n\n";

fn render_masked(
    surfaces: &[(Generator, Path)],
    wires: FastHashMap<usize, Vec<(Generator, Path)>>,
) -> String {
    let mut tikz = String::new();

    let mut gen_counts: FastHashMap<Generator, usize> = FastHashMap::default();
    let wire_layers = wires.len();

    tikz.push_str(MAGIC_MACRO);
    // The transparency group is useful in case users lower
    // opacity. It changes the opacity rules to hide our
    // masking strategy.
    // If you want to see it in action, add [opacity=.5] at
    // at \begin{tikzpicture}.
    tikz.push_str("\\begin{scope}[transparency group]\n");

    tikz.push_str("% Surfaces\n");
    for (g, path) in surfaces.iter() {
        let counts = gen_counts.entry(*g).or_default();
        writeln!(
            tikz,
            "\\newcommand{{{name}}}{{{path}}}",
            name = name(*g, *counts),
            path = &render_path(path)
        )
        .unwrap();
        *counts += 1;
    }

    // The masking logic mostly concerns the wires.
    // Unlike the background, wires do not all share the same depth.
    // Here we create a series of "layer" commands, which
    // either render their own paths or recourse to the successive layer.
    tikz.push_str("% Wires\n");
    let mut layer_defs = String::new();
    for (i, (_, layer)) in wires
        .into_iter()
        .sorted_by_cached_key(|(k, _)| *k)
        .rev()
        .enumerate()
    {
        let mut layer_def = String::new();
        for (g, path) in &layer {
            let counts = gen_counts.entry(*g).or_default();
            // Here we save the actual path geometry into a \newcommand.
            writeln!(
                tikz,
                "\\newcommand{{{name}}}{{{path}}}",
                name = name(*g, *counts),
                path = &render_path(path)
            )
            .unwrap();
            // And then instruct the current layer as to how to get that geometry.
            writeln!(
                layer_def,
                "  \\layered{{{name}}}{{{color}}}{{#1}}{{#2}};",
                name = name(*g, *counts),
                color = color(*g),
            )
            .unwrap();
            // Finally we make a note of how may times we saw the path.
            *counts += 1;
        }
        // Interpolate the layer definition into a global layer section.
        // This looks better than alternating "this is a path" and "here is its layer".
        writeln!(
            layer_defs,
            "\\newcommand{{\\layer{i:x}}}[2]{{\n\
                {layer}\
                }}",
            i = Roman::from((i + 1) as i16),
            layer = layer_def
        )
        .unwrap();
    }

    tikz.push_str("% Layer defs\n");
    tikz.push_str(&layer_defs);

    // Since we always clip with respect to the same background paths,
    // might as well make a macro for it and have TeX do the CTRL+V for us.
    writeln!(tikz, "\\newcommand{{\\clippedlayer}}[1]{{",).unwrap();
    for (g, _) in surfaces.iter() {
        writeln!(
            tikz,
            "  \\clipped{{{name}}}{{{color}}}{{#1}}",
            name = name(*g, 0),
            color = color(*g),
        )
        .unwrap();
    }
    tikz.push_str("  #1{0}{}\n}\n\n");
    // The layer command takes an extra two arguments
    // which allow to override the color into something else.
    // We use this to switch between drawing the wire in "mask mode"
    // and "normal mode".
    // The if/else statement is run on TeX side, and the logic is in MAGIC!

    tikz.push_str("% Background\n");
    for (g, _) in surfaces.iter() {
        writeln!(
            tikz,
            "\\fill[{color}!75]{name};",
            color = color(*g),
            name = name(*g, 0),
        )
        .unwrap();
    }

    for i in 0..wire_layers {
        if i > 0 {
            writeln!(
                tikz,
                "\\clippedlayer{{\\layer{i:x}}}",
                i = Roman::from((i + 1) as i16)
            )
            .unwrap();
        } else {
            writeln!(
                tikz,
                "\\layer{i:x}{{0}}{{-}}",
                i = Roman::from((i + 1) as i16)
            )
            .unwrap();
        }
    }

    tikz.push_str("\\end{scope}\n");

    tikz
}

fn render_point(point: Point2D<f32>) -> String {
    let x = (point.x * 100.0).round() / 100.0;
    let y = (point.y * 100.0).round() / 100.0;
    format!("({},{})", x, y)
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

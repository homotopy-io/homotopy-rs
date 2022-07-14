use std::fmt::Write;

use euclid::{default::Point2D, Vector2D};
use homotopy_common::hash::FastHashMap;
use homotopy_core::{
    common::DimensionError,
    complex::make_complex,
    layout::Layout,
    projection::{Depths, Projection},
    Diagram, Generator,
};
use itertools::Itertools;
use lyon_geom::{CubicBezierSegment, Line, LineSegment};
use lyon_path::{Event, Path};

use crate::{
    style::{GeneratorStyle, GeneratorStyles, VertexShape},
    svg::render::GraphicElement,
};

const OCCLUSION_DELTA: f32 = 0.2;

trait TikzRenderVertex {
    fn render(&self, point: Point2D<f32>) -> String;
}

impl<T: GeneratorStyle> TikzRenderVertex for T {
    fn render(&self, point: Point2D<f32>) -> String {
        const CIRCLE_RADIUS: f32 = 0.14; // r = 4pt
        const SQUARE_SIDELENGTH: f32 = 0.28; // 8pt x 8pt

        use VertexShape::{Circle, Square};
        let shape = self.shape().unwrap_or_default();
        let shape_str = match shape {
            Circle => "circle",
            Square => "square",
        };
        let (xo, yo) = match shape {
            Circle => (0.0, 0.0),
            Square => (-SQUARE_SIDELENGTH / 2., -SQUARE_SIDELENGTH / 2.),
        };
        let x1 = (point.x * 100.0 + xo).round() / 100.0;
        let y1 = (point.y * 100.0 + yo).round() / 100.0;
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
}

pub fn color(generator: Generator) -> String {
    format!("generator-{}-{}", generator.id, generator.dimension)
}

pub fn render<T, S>(
    diagram: &Diagram,
    stylesheet: &str,
    generator_styles: Option<&T>,
) -> Result<String, DimensionError>
where
    T: GeneratorStyles<S>,
    S: GeneratorStyle,
{
    let layout = Layout::<2>::new(diagram)?;
    let complex = make_complex(diagram);
    let depths = Depths::<2>::new(diagram)?;
    let projection = Projection::<2>::new(diagram, &layout, &depths)?;
    let graphic = GraphicElement::build(&complex, &layout, &projection, &depths);

    let mut surfaces = Vec::default();
    let mut wires: FastHashMap<usize, Vec<(Generator, Path)>> = FastHashMap::default();
    let mut points = Vec::default();
    for element in graphic {
        match element {
            GraphicElement::Surface(g, path) => surfaces.push((g, path)),
            GraphicElement::Wire(g, path, mask) => {
                wires.entry(mask.len()).or_default().push((g, path));
            }
            GraphicElement::Point(g, point) => points.push((g, point)),
        }
    }

    let mut tikz = String::new();
    writeln!(tikz, "\\begin{{tikzpicture}}").unwrap();
    tikz.push_str(stylesheet);

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
                let left = offset(-OCCLUSION_DELTA, path).reversed();
                let right = offset(OCCLUSION_DELTA, path);
                tikz.push_str(&render_path(&right));
                tikz.push_str(" -- ");
                tikz.push_str(&render_path(&left));
                tikz.push_str(" -- cycle");
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

    // Points
    // TODO(thud): this `default_shape` should not be hardcoded here
    let default_shape = |point| format!("{} circle (4pt)", render_point(point));
    for (g, point) in points {
        write!(tikz, "\\fill[{}] ", color(g)).unwrap();
        let vertex = generator_styles
            .map(|styles| styles.generator_style(g))
            .map_or_else(
                || Some(default_shape(point)),
                |style| style.map(|s| s.render(point)),
            )
            .unwrap();
        writeln!(tikz, " {}", vertex).unwrap();
    }

    writeln!(tikz, "\\end{{tikzpicture}}").unwrap();

    Ok(tikz)
}

fn render_point(point: Point2D<f32>) -> String {
    let x = (point.x * 100.0).round() / 100.0;
    let y = (point.y * 100.0).round() / 100.0;
    format!("({}, {})", x, y)
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

// Offsetting a curve.
// TOOD(@calintat): Move somewhere else.
fn offset(delta: f32, path: &Path) -> Path {
    let mut builder = Path::builder();

    for event in path {
        match event {
            Event::Cubic {
                from,
                ctrl1,
                ctrl2,
                to,
            } => {
                let segment = offset_cubical(
                    delta,
                    CubicBezierSegment {
                        from,
                        ctrl1,
                        ctrl2,
                        to,
                    },
                );
                builder.begin(segment.from);
                builder.cubic_bezier_to(segment.ctrl1, segment.ctrl2, segment.to);
                builder.end(false);
                return builder.build();
            }
            Event::Line { from, to } => {
                let segment = offset_linear(delta, LineSegment { from, to });
                builder.begin(segment.from);
                builder.line_to(segment.to);
                builder.end(false);
                return builder.build();
            }
            _ => (),
        }
    }

    panic!("Cannot offset a path made of multiple segments")
}

fn perp<U>(v: Vector2D<f32, U>) -> Vector2D<f32, U> {
    Vector2D::new(v.y, -v.x).normalize()
}

fn offset_linear(delta: f32, segment: LineSegment<f32>) -> LineSegment<f32> {
    let v = perp(segment.to - segment.from);
    LineSegment {
        from: segment.from + v * delta,
        to: segment.to + v * delta,
    }
}

fn offset_cubical(delta: f32, segment: CubicBezierSegment<f32>) -> CubicBezierSegment<f32> {
    if segment.from == segment.ctrl1
        || segment.ctrl1 == segment.ctrl2
        || segment.ctrl2 == segment.to
    {
        let leg = offset_linear(
            delta,
            LineSegment {
                from: segment.from,
                to: segment.to,
            },
        );
        CubicBezierSegment {
            from: leg.from,
            ctrl1: leg.from,
            ctrl2: leg.to,
            to: leg.to,
        }
    } else {
        let leg1 = offset_linear(
            delta,
            LineSegment {
                from: segment.from,
                to: segment.ctrl1,
            },
        );
        let leg2 = offset_linear(
            delta,
            LineSegment {
                from: segment.ctrl1,
                to: segment.ctrl2,
            },
        );
        let leg3 = offset_linear(
            delta,
            LineSegment {
                from: segment.ctrl2,
                to: segment.to,
            },
        );

        let from = leg1.from;

        let line1 = Line {
            point: leg1.from,
            vector: leg1.to - leg1.from,
        };
        let line2 = Line {
            point: leg2.from,
            vector: leg2.to - leg2.from,
        };
        let line3 = Line {
            point: leg3.from,
            vector: leg3.to - leg3.from,
        };

        let ctrl1 = line1.intersection(&line2).unwrap_or(leg1.to);
        let ctrl2 = line2.intersection(&line3).unwrap_or(leg2.to);

        let to = leg3.to;

        CubicBezierSegment {
            from,
            ctrl1,
            ctrl2,
            to,
        }
    }
}

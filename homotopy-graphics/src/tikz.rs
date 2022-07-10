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

use crate::svg::render::GraphicElement;

const OCCLUSION_DELTA: f32 = 0.2;

pub fn color(generator: Generator) -> String {
    format!("generator-{}-{}", generator.id, generator.dimension)
}

pub fn render(diagram: &Diagram, stylesheet: &str) -> Result<String, DimensionError> {
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

    // Points
    for (g, point) in points {
        write!(tikz, "\\fill[{}] ", color(g)).unwrap();
        tikz.push_str(&render_point(point));
        writeln!(tikz, " circle (4pt);").unwrap();
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

fn offset_multiple(delta: f32, path: &Path) -> String {
    let mut tikz = String::new();
    let mut builder = Path::builder();
    for event in path {
        match event {
            Event::End { .. } => {
                builder.path_event(event);
                let segment = builder.build();
                let left = offset(-delta, &segment).reversed();
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

// Offsetting a curve.
// TOOD(@calintat): Move somewhere else.
fn offset(delta: f32, path: &Path) -> Path {
    let mut flag = false;
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
                if !flag {
                    builder.begin(segment.from);
                    flag = true;
                }
                builder.cubic_bezier_to(segment.ctrl1, segment.ctrl2, segment.to);
            }
            // TODO handle Quadratic properly
            Event::Line { from, to } | Event::Quadratic { from, to, .. } => {
                let segment = offset_linear(delta, LineSegment { from, to });
                if !flag {
                    builder.begin(segment.from);
                    flag = true;
                }
                builder.line_to(segment.to);
            }
            _ => (),
        }
    }
    builder.end(false);
    builder.build()
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

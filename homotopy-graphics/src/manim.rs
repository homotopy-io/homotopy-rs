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
const INDENT: &str = "    ";

pub fn color(generator: Generator) -> String {
    format!("generator_{}_{}", generator.id, generator.dimension)
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

    // Needed for working out translations/scalings
    let mut max_point = Point2D::<f32>::zero();
    for element in graphic {
        match element {
            GraphicElement::Surface(g, path) => {
                max_point = max_point.max(max_point_path(&path));
                surfaces.push((g, path));
            }
            GraphicElement::Wire(g, path, mask) => {
                max_point = max_point.max(max_point_path(&path));
                wires.entry(mask.len()).or_default().push((g, path));
            }
            GraphicElement::Point(g, point) => {
                max_point = max_point.max(point);
                points.push((g, point));
            }
        }
    }

    let mut manim = String::new();
    manim.push_str("# Uncomment line below if needed\n");
    manim.push_str("#from manim import *\n");
    manim.push_str("#import numpy as np\n");

    writeln!(
        manim,
        concat!(
            "\nclass HomotopyIoManim(Scene):\n",
            "{ind}def get_colors(self):\n",
            "{ind}{ind}colors = {{\n",
            "{stylesheet}",
            "{ind}{ind}}}\n",
            "{ind}{ind}return colors\n",
        ),
        ind = INDENT,
        stylesheet = stylesheet
    )
    .unwrap();

    // Surfaces
    writeln!(
        manim,
        concat!(
            "{ind}# Surfaces\n",
            "{ind}def get_surfaces(self):\n",
            "{ind}{ind}C = self.get_colors()\n",
            "{ind}{ind}surfaces = VGroup()"
        ),
        ind = INDENT
    )
    .unwrap();
    for (g, path) in surfaces {
        writeln!(
            manim,
            "{ind}{ind}surfaces.add(VMobject(stroke_width=1).set_fill(C[\"{color}\"],0.75){path}) # path_{id}_{dim}",
            ind=INDENT,
            color=color(g),
            id=g.id,
            dim=g.dimension,
            path=&render_path(&path)
        )
        .unwrap();
    }

    // Wires
    writeln!(
        manim,
        concat!(
            "{ind}{ind}return surfaces\n\n",
            "{ind}# Wires\n",
            "{ind}def get_wires(self, surfaces):\n",
            "{ind}{ind}C = self.get_colors()\n",
            "{ind}{ind}wires = VGroup()"
        ),
        ind = INDENT
    )
    .unwrap();
    for (i, (_, layer)) in wires
        .into_iter()
        .sorted_by_cached_key(|(k, _)| *k)
        .rev()
        .enumerate()
    {
        // Background
        if i > 0 {
            writeln!(manim, "{ind}{ind}# Begin scope", ind = INDENT).unwrap();
            for (g, path) in &layer {
                let left = offset(-OCCLUSION_DELTA, path).reversed();
                let right = offset(OCCLUSION_DELTA, path);
                writeln!(manim, concat!("{ind}{ind}wires.add(Intersection(surfaces,",
                         "VMobject().set_fill(BLACK, 1.0){path_right}{path_left},color=C[\"generator_{id}_{dim}\"],fill_opacity=0.8)) # path_{id}_{dim}"),
                         ind=INDENT,
                         id=g.id,
                         dim=g.dimension,path_left=&render_path(&left),
                         path_right=&render_path(&right)
                ).unwrap();
            }
            writeln!(manim, "{ind}{ind}# End scope", ind = INDENT).unwrap();
        }

        for (g, path) in &layer {
            writeln!(manim, "{ind}{ind}wires.add(VMobject(stroke_color=C[\"{color}\"],stroke_width=5){path}) # path_{id}_{dim}",
                ind=INDENT,
                color=color(*g),
                id=g.id,
                dim=g.dimension,
                path=&render_path(path)
            ).unwrap();
        }
    }

    // Points
    writeln!(
        manim,
        concat!(
            "{ind}{ind}return wires\n\n",
            "{ind}# Points\n",
            "{ind}def get_points(self):\n",
            "{ind}{ind}C = self.get_colors()\n",
            "{ind}{ind}points = VGroup()"
        ),
        ind = INDENT
    )
    .unwrap();
    //TODO work out right radius for circles to match SVG/tikz export.
    for (g, point) in points {
        writeln!(manim, "{ind}{ind}points.add(Circle(radius=0.125,color=C[\"{color}\"],fill_opacity=1).move_to({pt})) # circle_{id}_{dim}",
            ind=INDENT,
            id=g.id,
            dim=g.dimension,
            color=color(g),
            pt=&render_point(point)
        ).unwrap();
    }
    //TODO work out good scaling automatically
    writeln!(
        manim,
        concat!("{ind}{ind}return points\n\n",
            "{ind}# We now put everything together\n",
            "{ind}def construct(self):\n",
            "{ind}{ind}#C = self.get_colors()\n",
            "{ind}{ind}surfaces = self.get_surfaces()\n",
            "{ind}{ind}wires = self.get_wires(surfaces)\n",
            "{ind}{ind}points = self.get_points()\n",
            "{ind}{ind}# Root\n",
            "{ind}{ind}root = VGroup(surfaces,wires,points).shift({x}*LEFT+{y}*DOWN).scale(0.125)\n",
            "{ind}{ind}# Static output (low rendering times)\n",
            "{ind}{ind}#self.add(root)\n",
            "{ind}{ind}# Animated output\n",
            "{ind}{ind}self.play(DrawBorderThenFill(surfaces))\n",
            "{ind}{ind}self.play(Create(root))\n",
            "{ind}{ind}text = MarkupText(\"Homotopy.io\", color=BLUE).next_to(root, 2*DOWN)\n",
            "{ind}{ind}self.play(Write(text))\n",
            "{ind}{ind}self.wait(5)\n",
        ),
        ind = INDENT,
        x = max_point.x * 0.5,
        y = max_point.y * 0.5,
    )
    .unwrap();

    Ok(manim)
}

fn render_point(point: Point2D<f32>) -> String {
    let x = ((point.x) * 100.0).round() / 100.0;
    let y = ((point.y) * 100.0).round() / 100.0;
    format!("[{},{},0]", x, y)
}

fn max_point_path(path: &Path) -> Point2D<f32> {
    let mut max_point = Point2D::zero();
    for event in path {
        match event {
            Event::Line { to, .. } | Event::Quadratic { to, .. } | Event::Cubic { to, .. } => {
                max_point = max_point.max(to);
            }
            _ => {}
        }
    }
    max_point
}

fn render_path(path: &Path) -> String {
    let mut result = String::new();
    for event in path {
        match event {
            Event::Begin { at } => write!(
                result,
                ".set_points_as_corners([np.array({pt})]*2)",
                pt = render_point(at)
            )
            .unwrap(),
            Event::Line { to, .. } => write!(
                result,
                ".add_line_to(np.array({pt}))",
                pt = render_point(to)
            )
            .unwrap(),
            Event::Quadratic { ctrl, to, .. } => write!(
                result,
                ".add_quadratic_bezier_curve_to(np.array({}),np.array({}))",
                render_point(ctrl),
                render_point(to)
            )
            .unwrap(),
            Event::Cubic {
                ctrl1, ctrl2, to, ..
            } => write!(
                result,
                ".add_cubic_bezier_curve_to(np.array({}),np.array({}),np.array({}))",
                render_point(ctrl1),
                render_point(ctrl2),
                render_point(to),
            )
            .unwrap(),
            Event::End { .. } => {}
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

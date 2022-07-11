use euclid::Vector2D;
use homotopy_common::hash::FastHashMap;
use homotopy_core::common::Generator;
use lyon_geom::{CubicBezierSegment, Line, LineSegment};
use lyon_path::{builder::PathBuilder, Event, Path};

use crate::svg::{geom::Point, render::GraphicElement};

pub fn simplify_graphic<const N: usize>(graphic: &[GraphicElement<N>]) -> Vec<GraphicElement<N>> {
    let mut new_graphic = Vec::with_capacity(graphic.len());
    let mut point_elements = Vec::new();

    // (depth, gen) -> Vec<path>
    let mut grouped_wires =
        FastHashMap::<(usize, Generator), Vec<lyon_path::path::Builder>>::default();

    for element in graphic {
        match element {
            GraphicElement::Surface(g, path) => {
                new_graphic.push(GraphicElement::Surface(*g, simplify_path(path)));
            }
            GraphicElement::Wire(g, depth, path, _) => {
                let entry = grouped_wires.entry((*depth, *g)).or_default();
                // Recycle the builder if possible!
                //TODO handle reversal case
                // Use builder.current_position
                if let Some(last) = entry.last_mut() {
                    last.extend(path.iter());
                } else {
                    let mut builder = Path::builder();
                    builder.extend(path.iter());
                    entry.push(builder);
                }
            }
            GraphicElement::Point { .. } => {
                point_elements.push(element.clone());
            }
        }
    }

    for ((depth, g), wires) in grouped_wires {
        for path in wires {
            new_graphic.push(GraphicElement::Wire(
                g,
                depth,
                simplify_path(&path.build()),
                Vec::new(),
            ));
        }
    }
    new_graphic.extend(point_elements);
    new_graphic
}

// Test collinearity with dot product formula up to precision
fn points_collinear(p0: Point, p1: Point, p2: Point) -> bool {
    ((p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x)).abs() <= 0.00005
}

pub fn simplify_path(path: &Path) -> Path {
    //TODO either make a hot-path for Begin - UselessBezierCubic -- End, or do not call on unmerged wires.
    let mut builder = Path::builder();
    let mut it = path.iter();
    let mut under_cons: Option<lyon_path::PathEvent> = it.next();
    let mut peek_head: Option<lyon_path::PathEvent> = it.next();
    loop {
        //  Do not assume under_cons == peek_head of previous iteration.
        //  We want to rewrite it.
        match (under_cons, peek_head) {
            // Get rid of peek_head == None cases first
            (None, None) => {
                break;
            }
            (Some(ev), None) => {
                builder.path_event(ev);
                break;
            }
            // Now can assume there is a next element!
            (None, Some(_)) => {
                // I don't think this should ever happen
                // But we won't issue a warning to avoid I/O
                // in potential rendering loop.
                under_cons = peek_head;
                peek_head = it.next();
            }
            // Collinear lines can be merged
            (
                Some(lyon_path::Event::Line {
                    from: from1,
                    to: to1,
                }),
                Some(lyon_path::Event::Line {
                    from: from2,
                    to: to2,
                }),
            ) if to1 == from2 && points_collinear(from1, to1, to2) => {
                under_cons = Some(lyon_path::Event::Line {
                    from: from1,
                    to: to2,
                });
                peek_head = it.next();
            }
            // Collinear Beziers can be transformed to lines
            (_, Some(lyon_path::Event::Quadratic { from, ctrl, to }))
                if points_collinear(from, ctrl, to) =>
            {
                peek_head = Some(lyon_path::Event::Line { from, to });
            }
            (
                _,
                Some(lyon_path::Event::Cubic {
                    from,
                    ctrl1,
                    ctrl2,
                    to,
                }),
            ) if points_collinear(ctrl1, ctrl2, to) => {
                peek_head = Some(lyon_path::Event::Line { from, to });
            }
            // Needless End -- Begin can be removed
            (
                Some(lyon_path::Event::End {
                    last, close: false, ..
                }),
                Some(lyon_path::Event::Begin { at }),
            ) if last == at => {
                under_cons = it.next();
                peek_head = it.next();
            }
            (Some(ev), _) => {
                builder.path_event(ev);
                under_cons = peek_head;
                peek_head = it.next();
            }
        };
    }
    builder.build()
}

// Offsetting a curve.
pub fn offset(delta: f32, path: &Path) -> Path {
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

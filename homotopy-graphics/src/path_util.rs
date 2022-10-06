use euclid::approxeq::ApproxEq;
use homotopy_common::hash::FastHashMap;
use homotopy_core::diagram::Diagram0;
use lyon_path::{path::Builder, Path};

use crate::svg::{render::GraphicElement, shape::Point};

pub fn simplify_graphic<const N: usize>(graphic: &[GraphicElement<N>]) -> Vec<GraphicElement<N>> {
    let mut new_graphic = Vec::with_capacity(graphic.len());
    let mut point_elements = Vec::new();

    // (depth, gen) -> Vec<(path, start, end)>
    let mut grouped_wires =
        FastHashMap::<(usize, Diagram0), Vec<(Builder, Point, Point)>>::default();

    for element in graphic {
        match element {
            GraphicElement::Surface(g, path) => {
                new_graphic.push(GraphicElement::Surface(*g, simplify_path(path)));
            }
            GraphicElement::Wire(g, depth, path, _, _) => {
                let entry = grouped_wires.entry((*depth, *g)).or_default();

                let extremes = path_extremes(path).unwrap();

                // The following loop is the core algo wire merging.
                // The idea is: if we see a segment that shares an endpoint
                // with a previously seen segment, concatenate them,
                // and otherwise, just push it as is.
                //
                // Then everything is concatenated end-to-end and passed to
                // the simplifier. The more continuous segments we can catch,
                // the more the simplifier can cut out!
                //
                // We do this by checking whether the extremes of a wire
                // under consideration match those of any earlier wire.
                // This gives four cases, in theory, but two of them would
                // require reversing wires, which is logically undesirable and
                // geometrically unnecessary given that geometry extraction
                // ensures for us that "wires only go up".
                // So, in practice, we are reduced to checking two possible cases.
                //
                // If no matches work, we move to the next seen wire.
                let mut it = entry.iter_mut();
                loop {
                    if let Some((builder, from, to)) = it.next() {
                        match (extremes.0.approx_eq(to), extremes.1.approx_eq(from)) {
                            (true, _) => {
                                builder.extend_from_paths(&[path.as_slice()]);
                                *to = extremes.1;
                            }
                            (_, true) => {
                                let mut new_builder = Path::builder();
                                new_builder.extend_from_paths(&[
                                    path.as_slice(),
                                    builder.clone().build().as_slice(),
                                ]);
                                *builder = new_builder;
                                *from = extremes.0;
                            }
                            (_, _) => {
                                continue;
                            }
                        }
                    } else {
                        let mut builder = Path::builder();
                        builder.extend_from_paths(&[path.as_slice()]);
                        entry.push((builder, extremes.0, extremes.1));
                    }
                    break;
                }
            }
            GraphicElement::Point { .. } => {
                point_elements.push(element.clone());
            }
        }
    }

    for ((depth, g), wires) in grouped_wires {
        let mut merged_path = Path::builder();
        for (builder, _, _) in wires {
            merged_path.extend_from_paths(&[builder.build().as_slice()]);
        }
        // TODO(thud): arrows will have to be merged and returned here.
        new_graphic.push(GraphicElement::Wire(
            g,
            depth,
            simplify_path(&merged_path.build()),
            Vec::new(),
            None,
        ));
    }
    new_graphic.extend(point_elements);
    new_graphic
}

// This function computes the effective Being and End of Path.
fn path_extremes(path: &Path) -> Option<(Point, Point)> {
    match (path.iter().next(), path.iter().last()) {
        // Cannot assume that End refers to same Begin
        // as path could be made of multiple segments.
        (Some(lyon_path::Event::Begin { at }), Some(lyon_path::Event::End { last, .. })) => {
            Some((at, last))
        }
        (_, _) => None,
    }
}

// Points are collinear if area of triangle is zero
fn points_collinear(p0: Point, p1: Point, p2: Point) -> bool {
    (p0.x * (p1.y - p2.y) + p1.x * (p2.y - p0.y) + p2.x * (p0.y - p1.y)).approx_eq(&0.0)
}

// Simple peep-hole simplifier for paths.
// Churns through the wire step by step and checks if local
// simplifications can be performed.
pub fn simplify_path(path: &Path) -> Path {
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
            // Collinear Beziers can be transformed to lines
            (Some(lyon_path::Event::Quadratic { from, ctrl, to }), _)
                if points_collinear(from, ctrl, to) =>
            {
                under_cons = Some(lyon_path::Event::Line { from, to });
            }
            (_, Some(lyon_path::Event::Quadratic { from, ctrl, to }))
                if points_collinear(from, ctrl, to) =>
            {
                peek_head = Some(lyon_path::Event::Line { from, to });
            }
            (
                Some(lyon_path::Event::Cubic {
                    from,
                    ctrl1,
                    ctrl2,
                    to,
                }),
                _,
            ) if points_collinear(from, ctrl1, ctrl2) && points_collinear(ctrl1, ctrl2, to) => {
                under_cons = Some(lyon_path::Event::Line { from, to });
            }
            (
                _,
                Some(lyon_path::Event::Cubic {
                    from,
                    ctrl1,
                    ctrl2,
                    to,
                }),
            ) if points_collinear(from, ctrl1, ctrl2) && points_collinear(ctrl1, ctrl2, to) => {
                peek_head = Some(lyon_path::Event::Line { from, to });
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
            ) if to1.approx_eq(&from2) && points_collinear(from1, to1, to2) => {
                under_cons = Some(lyon_path::Event::Line {
                    from: from1,
                    to: to2,
                });
                peek_head = it.next();
            }
            // Needless End -- Begin can be removed
            (
                Some(_),
                Some(lyon_path::Event::End {
                    last, close: false, ..
                }),
            ) => {
                // We can handle this a bit differently
                // since End is a bit "special" as a drawing command,
                // in the sense that not much is going on after it.
                //
                // If it has a useless Begin after, we skip both.
                // Otherwise, just pass to next iteration.
                let next = it.next();
                match next {
                    Some(lyon_path::Event::Begin { at }) if last.approx_eq(&at) => {
                        peek_head = it.next();
                    }
                    _ => {
                        builder.path_event(under_cons.unwrap());
                        builder.path_event(peek_head.unwrap());
                        under_cons = next;
                        peek_head = it.next();
                    }
                }
            }
            (None, Some(_)) => {
                // I don't think this should ever happen
                // But we won't issue a warning to avoid I/O
                // in potential rendering loop.
                under_cons = peek_head;
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

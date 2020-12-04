use euclid::default::Point2D;
use lyon_algorithms::hit_test::hit_test_path;
use lyon_geom::{CubicBezierSegment, QuadraticBezierSegment};
use lyon_path::Path;
use std::fmt::Write;

pub type Point = Point2D<f32>;

pub struct Fill {
    pub path: Path,
}

impl Fill {
    pub fn contains_point(&self, point: Point, tolerance: f32) -> bool {
        hit_test_path(
            &point,
            self.path.into_iter(),
            lyon_path::FillRule::NonZero,
            tolerance,
        )
    }
}

pub struct Stroke {
    pub path: Path,
    pub width: f32,
}

impl Stroke {
    pub fn contains_point(&self, point: Point, tolerance: f32) -> bool {
        for event in self.path.into_iter() {
            use lyon_path::Event;
            match event {
                Event::Begin { .. } => {}
                Event::Line { from, to } => {
                    if distance_to_line_segment(point, from, to) < self.width {
                        return true;
                    }
                }
                Event::Quadratic { from, ctrl, to } => {
                    let flattened = QuadraticBezierSegment { from, ctrl, to };
                    let mut current = from;

                    for next in flattened.flattened(tolerance) {
                        if distance_to_line_segment(point, current, next) < self.width * 0.5 {
                            return true;
                        }

                        current = next;
                    }
                }
                Event::Cubic {
                    from,
                    ctrl1,
                    ctrl2,
                    to,
                } => {
                    let segment = CubicBezierSegment {
                        from,
                        ctrl1,
                        ctrl2,
                        to,
                    };
                    let mut current = from;

                    for next in segment.flattened(tolerance) {
                        if distance_to_line_segment(point, current, next) < self.width * 0.5 {
                            return true;
                        }

                        current = next;
                    }
                }
                Event::End { last, first, close } => {
                    if close && distance_to_line_segment(point, last, first) < self.width * 0.5 {
                        return true;
                    }
                }
            }
        }

        false
    }
}

fn distance_to_line_segment(point: Point, from: Point, to: Point) -> f32 {
    let vec = to - from;
    let square_length = vec.square_length();

    if square_length == 0.0 {
        (from - point).length()
    } else {
        let t = (point - from).dot(vec) / square_length;
        let t = f32::max(0.0, f32::min(1.0, t));
        let projected = from + vec * t;
        (projected - point).length()
    }
}

pub struct Circle {
    pub center: Point,
    pub radius: f32,
}

impl Circle {
    pub fn contains_point(&self, point: Point) -> bool {
        (self.center - point).square_length() < self.radius * self.radius
    }
}

pub fn path_to_svg(path: Path) -> String {
    let mut svg = String::new();

    for event in path.into_iter() {
        use lyon_path::Event;
        match event {
            Event::Begin { at } => {
                write!(&mut svg, "M {} {}", at.x, at.y).unwrap();
            }
            Event::Line { to, .. } => {
                write!(&mut svg, "L {} {}", to.x, to.y).unwrap();
            }
            Event::Quadratic { ctrl, to, .. } => {
                write!(&mut svg, "Q {} {}, {} {}", ctrl.x, ctrl.y, to.x, to.y).unwrap();
            }
            Event::Cubic {
                ctrl1, ctrl2, to, ..
            } => {
                write!(
                    &mut svg,
                    "C {} {}, {} {}, {} {}",
                    ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y
                )
                .unwrap();
            }
            Event::End { close, .. } => {
                if close {
                    write!(&mut svg, "Z").unwrap();
                }
            }
        }
    }

    svg
}

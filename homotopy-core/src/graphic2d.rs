use crate::common::*;
use crate::diagram::*;
use crate::layout::Layout;
use crate::rewrite::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub enum Element {
    CellPoint(Point),
    CellWire(Vec<Point>),
    CellSurface(Vec<Point>),
    IdentityWire(Vec<Point>),
    IdentitySurface(Vec<(Point, Point)>),
}

pub type Point = (SliceIndex, SliceIndex);

pub fn make_svg(diagram: &DiagramN, layout: &Layout, generators: &Generators) -> String {
    let mut svg = String::new();

    let mut elements = make_elements(diagram);
    let colors = &["lightgray", "gray", "black"];
    let scale: f32 = 50.0;

    let (width, height) = layout
        .get(Boundary::Target.into(), Boundary::Target.into())
        .unwrap();

    svg += &format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\n",
        width * scale,
        height * scale
    );

    elements.sort_by_key(|element| match element {
        Element::CellPoint(_) => 2,
        Element::IdentityWire(_) => 1,
        Element::CellWire(_) => 1,
        Element::IdentitySurface(_) => 0,
        Element::CellSurface(_) => 0,
    });

    // TODO: Clean this up.

    for element in elements {
        match element {
            Element::CellPoint(element) => {
                let generator = generators.get(element.0, element.1).unwrap();
                let position = layout.get(element.0, element.1).unwrap();

                svg += &format!(
                    "<circle r=\"5\" cx=\"{}\" cy=\"{}\" fill=\"{}\" />\n",
                    position.0 * scale,
                    position.1 * scale,
                    colors[generator.id]
                );
            }
            Element::CellWire(element) => {
                let mut path = String::new();

                // TODO: Bezier curves
                for (i, (x, y)) in element.iter().enumerate() {
                    let (x, y) = layout.get(*x, *y).unwrap();

                    if i == 0 {
                        path += &format!("M {} {}", x * scale, y * scale);
                    } else {
                        path += &format!("L {} {}", x * scale, y * scale);
                    }
                }

                let generator = generators.get(element[0].0, element[0].1).unwrap();

                svg += &format!(
                    "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"4\" />\n",
                    path, colors[generator.id]
                );
            }
            Element::CellSurface(element) => {
                let mut path = String::new();

                // TODO: Bezier curves
                for (i, (x, y)) in element.iter().enumerate() {
                    let (x, y) = layout.get(*x, *y).unwrap();

                    if i == 0 {
                        path += &format!("M {} {}", x * scale, y * scale);
                    } else {
                        path += &format!("L {} {}", x * scale, y * scale);
                    }
                }

                let generator = generators.get(element[0].0, element[0].1).unwrap();

                svg += &format!(
                    "<path d=\"{}\" fill=\"{}\" stroke=\"{}\" strokeWidth=\"1\" />\n",
                    path, colors[generator.id], colors[generator.id]
                );
            }
            Element::IdentityWire(element) => {
                let mut path = String::new();

                for (i, (x, y)) in element.iter().enumerate() {
                    let (x, y) = layout.get(*x, *y).unwrap();

                    if i == 0 {
                        path += &format!("M {} {}", x * scale, y * scale);
                    } else {
                        path += &format!("L {} {}", x * scale, y * scale);
                    }
                }

                let generator = generators.get(element[0].0, element[0].1).unwrap();

                svg += &format!(
                    "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"4\" />\n",
                    path, colors[generator.id]
                );
            }
            Element::IdentitySurface(element) => {
                let mut points = Vec::new();
                points.extend(element.iter().map(|(p, _)| *p));
                points.extend(element.iter().rev().map(|(_, p)| *p));

                let mut path = String::new();

                // TODO: Bezier curves?
                for (i, (x, y)) in points.iter().enumerate() {
                    let (x, y) = layout.get(*x, *y).unwrap();

                    if i == 0 {
                        path += &format!("M {} {}", x * scale, y * scale);
                    } else {
                        path += &format!("L {} {}", x * scale, y * scale);
                    }
                }

                let generator = generators.get(points[0].0, points[0].1).unwrap();

                svg += &format!(
                    "<path d=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1\" />\n",
                    path, colors[generator.id], colors[generator.id]
                );
            }
        }
    }

    svg += "</svg>";
    svg
}

enum Block {
    Cell(SingularHeight, SingularHeight),
    Identity(Identity),
}

struct Identity {
    start: SliceIndex,
    xs: Vec<SingularHeight>,
}

fn analyze(diagram: &DiagramN) -> Vec<Block> {
    use Height::*;

    let slices: Vec<DiagramN> = diagram
        .slices()
        .into_iter()
        .map(|slice| slice.to_n().unwrap().clone())
        .collect();
    let cospans = diagram.cospans();

    let mut blocks = Vec::new();
    let mut identities: HashMap<SingularHeight, Identity> = (0..slices.first().unwrap().size())
        .map(|i| {
            (
                i,
                Identity {
                    start: Boundary::Source.into(),
                    xs: vec![i, i],
                },
            )
        })
        .collect();

    for y in 0..diagram.size() {
        let forward = cospans[y].forward.to_n().unwrap();
        let backward = cospans[y].backward.to_n().unwrap();

        let mut targets = forward.targets();
        targets.extend(backward.targets());
        targets.sort();
        targets.dedup();

        let size = slices[Singular(y).to_int()].size();

        let mut new_identities = HashMap::new();

        for x in 0..size {
            if !targets.iter().any(|t| *t == x) {
                let source_x = forward.singular_preimage(x).start;
                let target_x = backward.singular_preimage(x).start;

                match identities.remove(&source_x) {
                    None => {
                        new_identities.insert(
                            target_x,
                            Identity {
                                start: Regular(y).into(),
                                xs: vec![source_x, x, target_x],
                            },
                        );
                    }
                    Some(mut id) => {
                        id.xs.push(x);
                        id.xs.push(target_x);
                        new_identities.insert(target_x, id);
                    }
                };
            } else {
                blocks.push(Block::Cell(x, y));
            }
        }

        blocks.extend(identities.drain().map(|(_, id)| Block::Identity(id)));
        std::mem::replace(&mut identities, new_identities);
    }

    for x in 0..slices.last().unwrap().size() {
        match identities.remove(&x) {
            None => {
                blocks.push(Block::Identity(Identity {
                    start: Regular(diagram.size()).into(),
                    xs: vec![x, x],
                }));
            }
            Some(mut id) => {
                id.xs.push(x);
                blocks.push(Block::Identity(id));
            }
        }
    }

    blocks
}

type Elements = Vec<Element>;

fn make_elements(diagram: &DiagramN) -> Vec<Element> {
    let cospans = diagram.cospans();
    let mut elements = Vec::new();
    let blocks = analyze(diagram);

    for block in blocks {
        match block {
            Block::Identity(id) => {
                make_identity(id, diagram.size(), &mut elements);
            }
            Block::Cell(x, y) => {
                let cospan = &cospans[y];
                let forward = cospan.forward.to_n().unwrap();
                let backward = cospan.backward.to_n().unwrap();

                make_cell(
                    (x, y),
                    //(regular0, singular, regular1),
                    (forward, backward),
                    &mut elements,
                );
            }
        }
    }

    elements
}

fn make_cell(position: (usize, usize), rewrites: (&RewriteN, &RewriteN), elements: &mut Elements) {
    let (x, y) = position;
    let (forward, backward) = rewrites;

    let x_source = forward.singular_preimage(x);
    let x_target = backward.singular_preimage(x);

    use Height::*;

    // Point
    elements.push(Element::CellPoint((Singular(x).into(), Singular(y).into())));

    // Wires
    for (wire_xs, wire_y) in &[(x_source.clone(), y), (x_target.clone(), y + 1)] {
        for wire_x in wire_xs.clone() {
            elements.push(Element::CellWire(vec![
                (Singular(wire_x).into(), Regular(*wire_y).into()),
                (Singular(x).into(), Singular(y).into()),
            ]));
        }
    }

    // Surfaces
    for (wire_xs, wire_y) in &[(x_source.clone(), y), (x_target.clone(), y + 1)] {
        for (i, wire_x) in wire_xs.clone().enumerate() {
            elements.push(Element::CellSurface(vec![
                (Regular(wire_x).into(), Regular(*wire_y).into()),
                (Singular(wire_x).into(), Regular(*wire_y).into()),
                (Singular(x).into(), Singular(y).into()),
            ]));

            elements.push(Element::CellSurface(vec![
                (Regular(wire_x + 1).into(), Regular(*wire_y).into()),
                (Singular(wire_x).into(), Regular(*wire_y).into()),
                (Singular(x).into(), Singular(y).into()),
            ]));
        }
    }

    elements.push(Element::CellSurface(vec![
        (Regular(x).into(), Singular(y).into()),
        (Singular(x).into(), Singular(y).into()),
        (Regular(x_source.start).into(), Regular(y).into()),
    ]));

    elements.push(Element::CellSurface(vec![
        (Regular(x + 1).into(), Singular(y).into()),
        (Singular(x).into(), Singular(y).into()),
        (Regular(x_source.end).into(), Regular(y).into()),
    ]));

    elements.push(Element::CellSurface(vec![
        (Regular(x).into(), Singular(y).into()),
        (Singular(x).into(), Singular(y).into()),
        (Regular(x_target.start).into(), Regular(y + 1).into()),
    ]));

    elements.push(Element::CellSurface(vec![
        (Regular(x + 1).into(), Singular(y).into()),
        (Singular(x).into(), Singular(y).into()),
        (Regular(x_target.end).into(), Regular(y + 1).into()),
    ]));
}

#[derive(Debug, Clone, Serialize)]
pub struct Generators(Vec<Vec<Generator>>);

impl Generators {
    pub fn new(diagram: &DiagramN) -> Self {
        if diagram.dimension() < 2 {
            // TODO: Make this into an error
            panic!();
        }

        // TODO: Projection
        Generators(
            diagram
                .slices()
                .map(|slice| {
                    slice
                        .to_n()
                        .unwrap()
                        .slices()
                        .map(|p| p.max_generator())
                        .collect()
                })
                .collect(),
        )
    }

    pub fn get(&self, x: SliceIndex, y: SliceIndex) -> Option<Generator> {
        let slice = match y {
            SliceIndex::Boundary(Boundary::Source) => self.0.first()?,
            SliceIndex::Boundary(Boundary::Target) => self.0.last()?,
            SliceIndex::Interior(height) => self.0.get(height.to_int())?,
        };

        match x {
            SliceIndex::Boundary(Boundary::Source) => slice.first().cloned(),
            SliceIndex::Boundary(Boundary::Target) => slice.last().cloned(),
            SliceIndex::Interior(height) => slice.get(height.to_int()).cloned(),
        }
    }
}

fn make_identity(id: Identity, diagram_size: usize, elements: &mut Elements) {
    use Height::*;

    let mut left: Vec<(Point, Point)> = Vec::new();
    let mut right: Vec<(Point, Point)> = Vec::new();
    let mut wire: Vec<Point> = Vec::new();

    for (i, x) in id.xs.iter().enumerate() {
        let height = SliceIndex::from_int(id.start.to_int(diagram_size) + i as isize, diagram_size);
        left.push(((Regular(*x).into(), height), (Singular(*x).into(), height)));
        right.push((
            (Regular(*x + 1).into(), height),
            (Singular(*x).into(), height),
        ));
        wire.push((Singular(*x).into(), height));
    }

    elements.push(Element::IdentitySurface(left));
    elements.push(Element::IdentitySurface(right));
    elements.push(Element::IdentityWire(wire));
}

mod test {
    use super::*;
    use crate::layout;

    fn example_assoc() -> DiagramN {
        let x = Generator {
            id: 0,
            dimension: 0,
        };
        let f = Generator {
            id: 1,
            dimension: 1,
        };
        let m = Generator {
            id: 2,
            dimension: 2,
        };

        let fd = DiagramN::new(f, x, x);
        let ffd = fd.attach(fd.clone(), Boundary::Target, &[]).unwrap();
        let md = DiagramN::new(m, ffd, fd);

        let mut result = md.clone();

        for _ in 0..10 {
            result = result.attach(md.clone(), Boundary::Source, &[0]).unwrap();
        }

        result
    }

    fn test() {
        let diagram = benchmark("construction", || example_assoc());
        let generators = Generators::new(&diagram);
        let mut solver = layout::Solver::new(diagram.clone()).unwrap();
        benchmark("layout", || solver.solve(10000));
        let layout = solver.finish();
        let svg = make_svg(&diagram, &layout, &generators);
    }

    fn benchmark<F, A>(name: &str, f: F) -> A
    where
        F: FnOnce() -> A,
    {
        let start = std::time::Instant::now();
        let result = f();
        let duration = std::time::Instant::now().duration_since(start);
        println!("[{}] {}us", name, duration.as_micros());
        result
    }
}

use crate::common::*;
use crate::diagram::DiagramN;
use crate::layout::Layout;
use crate::projection::Generators;
use crate::rewrite::RewriteN;
use petgraph::unionfind::UnionFind;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hash;

/// An element to draw in the 2d graphic of a diagram.
///
/// Connected components of surfaces and wires for the same generator are merged for efficiency
/// reasons. This gives produces vastly fewer path elements or draw instructions, but does not
/// distinguish between regions in the diagram that map to different actions when interacted with.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphicElement {
    Surface(Generator, VectorPath),
    Wire(Generator, VectorPath),
    Point(Generator, (f32, f32)),
}

/// An action region in the diagram.
///
/// Adjacent regions for the same generator are not merged so that they can be used to determine
/// the precise logical location in the diagram a user interacts with. While these regions could
/// also be used for drawing the diagram, surfaces and wires are subdivided into numerous parts
/// which can slow down drawing or increase the size of generated vector images unneccessarily.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionRegion {
    Surface(Coordinate, Coordinate, Coordinate, VectorPath),
    Wire(Coordinate, Coordinate, VectorPath),
    Point(Coordinate, (f32, f32)),
}

/// The 2-dimensional geometry of a diagram. 
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    /// The elements to draw for a diagram's 2-dimensional graphic. Has to be drawn in order for
    /// correct results.
    pub graphic: Vec<GraphicElement>,

    /// Regions in the graphic that map to a logical location for interactivity. Hit tests
    /// have to be performed in order for correct results.
    pub actions: Vec<ActionRegion>,
}

pub type VectorPath = Vec<VectorCommand>;

/// Vector path commands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorCommand {
    /// Move to a location.
    MoveTo((f32, f32)),
    /// Draw a line.
    LineTo((f32, f32)),
    /// Draw a cubic Bezier curve.
    CurveTo((f32, f32), (f32, f32), (f32, f32)),
}

impl VectorCommand {
    /// The SVG path command string.
    pub fn to_str(&self) -> String {
        match self {
            VectorCommand::MoveTo((x, y)) => format!("M {} {}", x, y),
            VectorCommand::LineTo((x, y)) => format!("L {} {}", x, y),
            VectorCommand::CurveTo((x0, y0), (x1, y1), (x2, y2)) => {
                format!("C {} {}, {} {}, {} {}", x0, y0, x1, y1, x2, y2)
            }
        }
    }
}

impl Geometry {
    pub fn new(diagram: &DiagramN, layout: &Layout, generators: &Generators) -> Self {
        let complex = generate_complex(diagram);
        let actions = complex_to_actions(&complex, layout);
        let graphic = complex_to_graphic(&complex, layout, generators);
        Geometry { actions, graphic }
    }
}

/// Construct the action regions from the simplicial complex of a diagram.
fn complex_to_actions(complex: &[Simplex], layout: &Layout) -> Vec<ActionRegion> {
    let mut regions: Vec<_> = complex
        .iter()
        .map(|simplex| match simplex {
            Simplex::Surface(p0, p1, p2) => {
                ActionRegion::Surface(*p0, *p1, *p2, to_curve(&[*p0, *p1, *p2], true, layout))
            }
            Simplex::Wire(p0, p1) => {
                ActionRegion::Wire(*p0, *p1, to_curve(&[*p0, *p1], false, layout))
            }
            Simplex::Point(p) => ActionRegion::Point(*p, layout.get(p.0, p.1).unwrap()),
        })
        .collect();

    regions.sort_by_key(|region| match region {
        ActionRegion::Surface(_, _, _, _) => 0,
        ActionRegion::Wire(_, _, _) => 1,
        ActionRegion::Point(_, _) => 2,
    });

    regions
}

/// Construct the graphical elements from the simplicial complex of a diagram.
fn complex_to_graphic(
    complex: &[Simplex],
    layout: &Layout,
    generators: &Generators,
) -> Vec<GraphicElement> {
    let mut elements = Vec::new();
    let mut grouped_surfaces: HashMap<Generator, Vec<Vec<Coordinate>>> = HashMap::new();

    for simplex in complex {
        match simplex {
            Simplex::Surface(p0, p1, p2) => {
                let generator = generators.get(p0.0, p0.1).unwrap();
                let (o0, o1, o2) = orient_surface((*p0, *p1, *p2));
                grouped_surfaces
                    .entry(generator)
                    .or_default()
                    .push(vec![o0, o1, o2]);
            }
            Simplex::Wire(p0, p1) => {
                let generator = generators.get(p0.0, p0.1).unwrap();
                elements.push(GraphicElement::Wire(
                    generator,
                    to_curve(&[*p0, *p1], false, layout),
                ));
            }
            Simplex::Point(p) => {
                let generator = generators.get(p.0, p.1).unwrap();
                elements.push(GraphicElement::Point(
                    generator,
                    layout.get(p.0, p.1).unwrap(),
                ));
            }
        }
    }

    for (generator, surfaces) in grouped_surfaces {
        for merged in merge_surfaces(surfaces.into_iter()) {
            elements.push(GraphicElement::Surface(
                generator,
                to_curve(&merged, true, layout),
            ));
        }
    }

    // TODO: Group and merge wires as well.

    elements.sort_by_key(|element| match element {
        GraphicElement::Surface(_, _) => 0,
        GraphicElement::Wire(_, _) => 1,
        GraphicElement::Point(_, _) => 2,
    });

    elements
}

/// Creates a vector path for between the specified points, using Bezier curves
/// and straight paths where appropriate.
fn to_curve(points: &[Coordinate], closed: bool, layout: &Layout) -> VectorPath {
    assert!(points.len() > 0);

    let mut commands = Vec::with_capacity(points.len() + 1);

    let start = layout.get(points[0].0, points[0].1).unwrap();
    commands.push(VectorCommand::MoveTo((start.0, start.1)));

    for i in 1..points.len() {
        commands.push(to_curve_segment(points[i - 1], points[i], layout));
    }

    if closed {
        commands.push(to_curve_segment(*points.last().unwrap(), points[0], layout));
    }

    commands
}

/// Creates a single vector command in a path.
fn to_curve_segment(start: Coordinate, end: Coordinate, layout: &Layout) -> VectorCommand {
    use self::Height::*;
    use self::SliceIndex::*;

    let layout_start = layout.get(start.0, start.1).unwrap();
    let layout_end = layout.get(end.0, end.1).unwrap();

    match (start, end) {
        (
            (Interior(Singular(_)), Interior(Regular(_))),
            (Interior(Singular(_)), Interior(Singular(_))),
        ) => VectorCommand::CurveTo(
            (layout_start.0, (layout_end.1 + layout_start.1) / 2.0),
            (layout_start.0, layout_end.1),
            (layout_end.0, layout_end.1),
        ),
        (
            (Interior(Singular(_)), Interior(Singular(_))),
            (Interior(Singular(_)), Interior(Regular(_))),
        ) => VectorCommand::CurveTo(
            (layout_end.0, layout_start.1),
            (layout_end.0, (layout_end.1 + layout_start.1) / 2.0),
            (layout_start.0, layout_start.1),
        ),
        _ => VectorCommand::LineTo(layout_end),
    }
}

/// Logical coordinates in the diagram.
pub type Coordinate = (SliceIndex, SliceIndex);

/// Orients a surface so it has consistent orientation with the rest.
fn orient_surface(
    surface: (Coordinate, Coordinate, Coordinate),
) -> (Coordinate, Coordinate, Coordinate) {
    fn ordering_to_int(ordering: Ordering) -> isize {
        match ordering {
            Ordering::Less => (-1),
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    let (p0, p1, p2) = surface;

    let a0 = ordering_to_int(p1.0.cmp(&p0.0));
    let a1 = ordering_to_int(p1.1.cmp(&p0.1));
    let b0 = ordering_to_int(p2.0.cmp(&p1.0));
    let b1 = ordering_to_int(p2.1.cmp(&p1.1));

    if a0 * b1 - a1 * b0 < 0 {
        (p1, p0, p2)
    } else {
        (p0, p1, p2)
    }
}

// Merges surfaces together. Assumes a consistent orientation.
fn merge_surfaces<P, I>(surfaces: I) -> Vec<Vec<P>>
where
    P: Hash + Eq + Copy,
    I: ExactSizeIterator<Item = Vec<P>>,
{
    // Find a representative for each connected component
    let count = surfaces.len();
    let mut repr = UnionFind::<usize>::new(count);
    let mut edge_to_surface: HashMap<(P, P), usize> = HashMap::new();

    for (surface_index, surface) in surfaces.enumerate() {
        for i in 0..surface.len() {
            let source = surface[i];
            let target = surface[(i + 1) % surface.len()];

            match edge_to_surface.remove(&(target, source)) {
                Some(other_index) => {
                    repr.union(surface_index, other_index);
                }
                None => {
                    edge_to_surface.insert((source, target), surface_index);
                }
            }
        }
    }

    // Gather the connected components
    let components: Vec<_> = {
        let mut components = vec![vec![]; count];

        for (edge, index) in edge_to_surface {
            components[repr.find(index)].push(edge);
        }

        components
            .into_iter()
            .filter(|edges| !edges.is_empty())
            .collect()
    };

    // Orient each connected component
    let parts: Vec<_> = {
        let mut parts = vec![];

        for component in components {
            let next: HashMap<P, P> = component.iter().map(|(s, t)| (*s, *t)).collect();
            let mut part = vec![component[0].0];
            let mut end = component[0].1;

            while end != part[0] {
                part.push(end);
                end = next[&end];
            }

            parts.push(part);
        }

        parts
    };

    parts
}

#[derive(Debug, Clone)]
enum Simplex {
    Surface(Coordinate, Coordinate, Coordinate),
    Wire(Coordinate, Coordinate),
    Point(Coordinate),
}

/// Generate a 2-dimensional simplicial complex for a diagram.
fn generate_complex(diagram: &DiagramN) -> Vec<Simplex> {
    use Height::*;
    let mut complex = Vec::new();

    let slices: Vec<DiagramN> = diagram
        .slices()
        .map(|slice| slice.try_into().unwrap())
        .collect();

    let cospans = diagram.cospans();

    for y in 0..diagram.size() {
        let slice = &slices[Singular(y).to_int()];
        let forward = cospans[y].forward.to_n().unwrap();
        let backward = cospans[y].backward.to_n().unwrap();

        let targets = {
            let mut targets = forward.targets();
            targets.extend(backward.targets());
            targets
        };

        for x in 0..slice.size() {
            generate_cell(x, y, y, forward, &mut complex);
            generate_cell(x, y, y + 1, backward, &mut complex);

            if targets.iter().any(|t| *t == x) {
                complex.push(Simplex::Point((Singular(x).into(), Singular(y).into())));
            }
        }
    }

    complex
}

fn generate_cell(sx: usize, sy: usize, ry: usize, rewrite: &RewriteN, complex: &mut Vec<Simplex>) {
    use Height::*;

    let rxs = rewrite.singular_preimage(sx);

    for rx in rxs.clone() {
        // Surface to the left of a wire
        complex.push(Simplex::Surface(
            (Regular(rx).into(), Regular(ry).into()),
            (Singular(rx).into(), Regular(ry).into()),
            (Singular(sx).into(), Singular(sy).into()),
        ));

        // Surface to the right of a wire
        complex.push(Simplex::Surface(
            (Regular(rx + 1).into(), Regular(ry).into()),
            (Singular(rx).into(), Regular(ry).into()),
            (Singular(sx).into(), Singular(sy).into()),
        ));

        // Wire
        complex.push(Simplex::Wire(
            (Singular(rx).into(), Regular(ry).into()),
            (Singular(sx).into(), Singular(sy).into()),
        ));
    }

    // Surface to the left
    complex.push(Simplex::Surface(
        (Regular(rxs.start).into(), Regular(ry).into()),
        (Regular(sx).into(), Singular(sy).into()),
        (Singular(sx).into(), Singular(sy).into()),
    ));

    // Surface to the right
    complex.push(Simplex::Surface(
        (Regular(rxs.end).into(), Regular(ry).into()),
        (Regular(sx + 1).into(), Singular(sy).into()),
        (Singular(sx).into(), Singular(sy).into()),
    ));
}

// mod test {
//     use super::*;
//     use crate::diagram::Diagram;

//     #[test]
//     fn test() {
//         let x = Generator::new(0, 0);
//         let y = Generator::new(1, 0);
//         let z = Generator::new(2, 0);
//         let f = Generator::new(3, 1);
//         let g = Generator::new(4, 1);
//         let h = Generator::new(5, 1);
//         let m = Generator::new(6, 2);

//         let d_f = DiagramN::new(f, x, y).unwrap();
//         let d_g = DiagramN::new(g, y, z).unwrap();
//         let d_h = DiagramN::new(h, x, z).unwrap();
//         let d_fg = d_f.attach(d_g.clone(), Boundary::Target, &[]).unwrap();
//         let d_m = DiagramN::new(m, d_fg, d_h).unwrap();

//         let generators = Generators::new(&d_m);
//         let complex = generate_complex(&d_m);

//         // for surface in complex.surfaces {
//         //     let surface = surface.clone().orient(surface.orientation().unwrap());
//         //     println!("{:?} {:?}", surface.orientation(), surface);
//         // }

//         let mut surfaces_grouped: HashMap<Generator, Vec<Surface>> = HashMap::new();

//         for surface in complex.surfaces {
//             let orientation = surface.orientation().unwrap();
//             let generator = generators.get(surface.0 .0, surface.0 .1).unwrap();
//             let surface = surface.orient(orientation);
//             surfaces_grouped.entry(generator).or_default().push(surface);
//         }

//         let result: Vec<_> = surfaces_grouped
//             .into_iter()
//             .map(|(generator, surfaces)| (generator, merge_surfaces(surfaces.into_iter())))
//             .collect();

//         println!("{:#?}", result);
//     }
// }

use crate::geometry::{Circle, Fill, Point, Shape, Stroke};
use crate::layout2d::Layout;
use euclid::default::Transform2D;
use homotopy_core::complex::Simplex;
use homotopy_core::projection::Generators;
use homotopy_core::{
    common::{Generator, Height, SliceIndex},
    projection::Depths,
};
use lyon_path::Path;
use petgraph::unionfind::UnionFind;
use seahash::SeaHasher;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

type Coordinate = (SliceIndex, SliceIndex);

/// An action region in the diagram.
///
/// Adjacent regions for the same generator are not merged so that they can be used to determine
/// the precise logical location in the diagram a user interacts with. While these regions could
/// also be used for drawing the diagram, surfaces and wires are subdivided into numerous parts
/// which can slow down drawing or increase the size of generated vector images unneccessarily.
#[derive(Debug, Clone)]
pub enum ActionRegion {
    Surface([Coordinate; 3], Path),
    Wire([Coordinate; 2], Path),
    Point([Coordinate; 1], Point),
}

impl ActionRegion {
    /// Apply an affine coordinate transformation to the region.
    pub fn transformed(&self, transform: &Transform2D<f32>) -> Self {
        use ActionRegion::{Point, Surface, Wire};
        match self {
            Surface(cs, path) => Surface(*cs, path.transformed(transform)),
            Wire(cs, path) => Wire(*cs, path.transformed(transform)),
            Point(cs, point) => Point(*cs, transform.transform_point(*point)),
        }
    }

    /// Construct action regions from the simplicial complex of a diagram.
    ///
    /// THe regions have to be tested for hits in the order they are returned to
    /// ahieve the correct result since they may overlap.
    ///
    /// This function can panic or produce undefined results if the simplicial complex and the
    /// layout have not come from the same diagram.
    pub fn build(complex: &[Simplex], layout: &Layout) -> Vec<Self> {
        let mut region_surfaces = Vec::new();
        let mut region_wires = Vec::new();
        let mut region_points = Vec::new();

        for simplex in complex {
            match simplex {
                Simplex::Surface(ps) => {
                    let path = make_path(ps, true, layout);
                    region_surfaces.push(Self::Surface(*ps, path));
                }
                Simplex::Wire(ps) => {
                    let path = make_path(ps, false, layout);
                    region_wires.push(Self::Wire(*ps, path));
                }
                Simplex::Point([p]) => {
                    let center = layout.get(p.0, p.1).unwrap();
                    region_points.push(Self::Point([*p], center));
                }
            }
        }

        let mut regions = region_points;
        regions.extend(region_wires);
        regions.extend(region_surfaces);
        regions
    }

    pub fn to_shape(&self, wire_thickness: f32, point_radius: f32) -> Shape {
        match self {
            ActionRegion::Surface(_, path) => Fill::new(path.clone()).into(),
            ActionRegion::Wire(_, path) => Stroke::new(path.clone(), wire_thickness).into(),
            ActionRegion::Point(_, point) => Circle::new(*point, point_radius).into(),
        }
    }
}

impl From<&ActionRegion> for Simplex {
    fn from(ar: &ActionRegion) -> Self {
        match ar {
            ActionRegion::Surface(ps, _) => Self::Surface(*ps),
            ActionRegion::Wire(ps, _) => Self::Wire(*ps),
            ActionRegion::Point(ps, _) => Self::Point(*ps),
        }
    }
}

/// An element to draw in the 2-dimensional graphic of a diagram.
///
/// Connected components of surfaces and wires for the same generator are merged for efficiency
/// reasons. This gives produces vastly fewer path elements or draw instructions, but does not
/// distinguish between regions in the diagram that map to different actions when interacted with.
#[derive(Debug, Clone)]
pub enum GraphicElement {
    /// A surface given by a closed path to be filled.
    Surface(Generator, Path),
    /// A wire given by a path to be stroked.
    Wire(Generator, Path, Vec<Path>),
    /// A point that is drawn as a circle.
    Point(Generator, Point),
}

type FastHashMap<K, V> = HashMap<K, V, std::hash::BuildHasherDefault<SeaHasher>>;

impl GraphicElement {
    /// Apply an affine coordinate transformation to the element.
    pub fn transformed(&self, transform: &Transform2D<f32>) -> Self {
        use GraphicElement::{Point, Surface, Wire};
        match self {
            Surface(g, path) => Surface(*g, path.transformed(transform)),
            Wire(g, path, mask) => {
                let path = path.transformed(transform);
                let mask = mask
                    .iter()
                    .map(|mask| mask.transformed(transform))
                    .collect();
                Wire(*g, path, mask)
            }
            Point(g, point) => Point(*g, transform.transform_point(*point)),
        }
    }

    pub fn generator(&self) -> Generator {
        use GraphicElement::{Point, Surface, Wire};
        match self {
            Surface(generator, _) | Wire(generator, _, _) | Point(generator, _) => *generator,
        }
    }

    /// Construct graphical elements from the simplicial complex of a diagram.
    ///
    /// The elements have to be drawn in the order they are returned to achieve the correct effect
    /// since they may overlap.
    ///
    /// This function can panic or produce undefined results if the simplicial complex, the layout
    /// and the projected generators have not come from the same diagram.
    pub fn build(
        complex: &[Simplex],
        layout: &Layout,
        generators: &Generators,
        depths: &Depths,
    ) -> Vec<Self> {
        let mut wire_elements = Vec::new();
        let mut surface_elements = Vec::new();
        let mut point_elements = Vec::new();

        let mut grouped_surfaces = FastHashMap::<Generator, Vec<[Coordinate; 3]>>::default();

        for simplex in complex {
            match simplex {
                Simplex::Surface(ps) => {
                    let generator = generators.get(ps[0].0, ps[0].1).unwrap();
                    grouped_surfaces
                        .entry(generator)
                        .or_default()
                        .push(orient_surface(ps));
                }
                Simplex::Wire(ps) => {
                    let generator = generators.get(ps[0].0, ps[0].1).unwrap();

                    let mask = match depths.edge_depth([ps[0].1, ps[0].0], [ps[1].1, ps[1].0]) {
                        Some(depth) => depths
                            .edges_above(depth, [ps[1].1, ps[1].0])
                            .into_iter()
                            .map(|s| make_path(&[(s[1], s[0]), ps[1]], false, layout))
                            .collect(),
                        None => vec![],
                    };

                    wire_elements.push(Self::Wire(generator, make_path(ps, false, layout), mask));
                }
                Simplex::Point([p]) => {
                    let generator = generators.get(p.0, p.1).unwrap();
                    point_elements.push(Self::Point(generator, layout.get(p.0, p.1).unwrap()));
                }
            }
        }

        for (generator, surfaces) in grouped_surfaces {
            for merged in merge_surfaces(surfaces.into_iter()) {
                surface_elements.push(Self::Surface(generator, make_path(&merged, true, layout)));
            }
        }

        // TODO: Group and merge wires as well.

        let mut elements = surface_elements;
        elements.extend(wire_elements);
        elements.extend(point_elements);
        elements
    }
}

fn orient_surface(surface: &[Coordinate; 3]) -> [Coordinate; 3] {
    fn ordering_to_int(ordering: Ordering) -> isize {
        match ordering {
            Ordering::Less => (-1),
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    let a0 = ordering_to_int(surface[1].0.cmp(&surface[0].0));
    let a1 = ordering_to_int(surface[1].1.cmp(&surface[0].1));
    let b0 = ordering_to_int(surface[2].0.cmp(&surface[1].0));
    let b1 = ordering_to_int(surface[2].1.cmp(&surface[1].1));

    if a0 * b1 - a1 * b0 < 0 {
        [surface[1], surface[0], surface[2]]
    } else {
        *surface
    }
}

fn merge_surfaces<P, I>(surfaces: I) -> Vec<Vec<P>>
where
    P: Hash + Eq + Copy,
    I: ExactSizeIterator<Item = [P; 3]>,
{
    // Find a representative for each connected component.
    let count = surfaces.len();
    let mut repr = UnionFind::<usize>::new(count);
    let mut edge_to_surface = FastHashMap::<(P, P), usize>::default();

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

    // Gather the connected components.
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

    // Find edge ordering for each connected component.
    let parts: Vec<_> = {
        let mut parts = vec![];

        for component in components {
            let next: FastHashMap<P, P> = component.iter().map(|(s, t)| (*s, *t)).collect();
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

fn make_path(points: &[Coordinate], closed: bool, layout: &Layout) -> Path {
    let mut builder = Path::builder();

    let start = layout.get(points[0].0, points[0].1).unwrap();
    builder.move_to(start);

    for i in 1..points.len() {
        make_path_segment(points[i - 1], points[i], layout, &mut builder);
    }

    if closed {
        make_path_segment(*points.last().unwrap(), points[0], layout, &mut builder);
    }

    builder.build()
}

fn make_path_segment(
    start: Coordinate,
    end: Coordinate,
    layout: &Layout,
    builder: &mut lyon_path::Builder,
) {
    use self::Height::{Regular, Singular};
    use self::SliceIndex::Interior;

    let layout_start = layout.get(start.0, start.1).unwrap();
    let layout_end = layout.get(end.0, end.1).unwrap();

    match (start, end) {
        ((_, Interior(Regular(_))), (Interior(Singular(_)), Interior(Singular(_)))) => builder
            .cubic_bezier_to(
                (layout_start.x, 0.8 * layout_end.y + 0.2 * layout_start.y).into(),
                (layout_start.x, layout_end.y).into(),
                layout_end,
            ),
        ((Interior(Singular(_)), Interior(Singular(_))), (_, Interior(Regular(_)))) => builder
            .cubic_bezier_to(
                (layout_end.x, layout_start.y).into(),
                (layout_end.x, 0.2 * layout_end.y + 0.8 * layout_start.y).into(),
                layout_end,
            ),
        _ => builder.line_to(layout_end),
    };
}

use std::{cmp::Ordering, hash::Hash};

use euclid::default::Transform2D;
use homotopy_common::hash::FastHashMap;
use homotopy_core::{
    common::{Height, SliceIndex},
    complex::Simplex,
    diagram::Diagram0,
    layout::Layout,
    projection::{Depths, Homotopy, Projection},
    Generator, Orientation,
};
use lyon_path::{builder::NoAttributes, Path};

use super::shape::project_2d;
use crate::{
    path_util::simplify_path,
    style::GeneratorRepresentation,
    svg::shape::{Circle, Fill, Point, Shape, Stroke},
};

type Coordinate<const N: usize> = [SliceIndex; N];

/// An action region in the diagram.
///
/// Adjacent regions for the same generator are not merged so that they can be used to determine
/// the precise logical location in the diagram a user interacts with. While these regions could
/// also be used for drawing the diagram, surfaces and wires are subdivided into numerous parts
/// which can slow down drawing or increase the size of generated vector images unneccessarily.
#[derive(Debug, Clone)]
pub enum ActionRegion<const N: usize> {
    Surface([Coordinate<N>; 3], Path),
    Wire([Coordinate<N>; 2], Path),
    Point([Coordinate<N>; 1], Point),
}

impl<const N: usize> ActionRegion<N> {
    /// Apply an affine coordinate transformation to the region.
    #[must_use]
    pub fn transformed(&self, transform: &Transform2D<f32>) -> Self {
        use ActionRegion::{Point, Surface, Wire};
        match self {
            Surface(cs, path) => Surface(*cs, path.clone().transformed(transform)),
            Wire(cs, path) => Wire(*cs, path.clone().transformed(transform)),
            Point(cs, point) => Point(*cs, transform.transform_point(*point)),
        }
    }

    /// Construct action regions from the simplicial complex of a diagram.
    ///
    /// The regions have to be tested for hits in the order they are returned to
    /// achieve the correct result since they may overlap.
    ///
    /// This function can panic or produce undefined results if the simplicial complex and the
    /// layout have not come from the same diagram.
    pub fn build(
        complex: &[(Simplex<N>, bool)],
        layout: &Layout<N>,
        projection: &Projection<N>,
    ) -> Vec<Self> {
        let mut region_surfaces = Vec::new();
        let mut region_wires = Vec::new();
        let mut region_points = Vec::new();

        for (simplex, _) in complex
            .iter()
            .filter(|(simplex, visible)| *visible || matches!(simplex, Simplex::Point(_)))
        {
            match simplex {
                Simplex::Surface(ps) => {
                    let path = build_path(ps, true, layout, projection);
                    region_surfaces.push(Self::Surface(*ps, path));
                }
                Simplex::Wire(ps) => {
                    let path = build_path(ps, false, layout, projection);
                    region_wires.push(Self::Wire(*ps, path));
                }
                Simplex::Point([p]) => {
                    let center = project_2d(layout[p]).into();
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

impl<const N: usize> From<&ActionRegion<N>> for Simplex<N> {
    fn from(ar: &ActionRegion<N>) -> Self {
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
pub enum GraphicElement<const N: usize> {
    /// A surface given by a closed path to be filled.
    Surface(Diagram0, Path),
    /// A wire given by a depth and a path to be stroked.
    Wire(Diagram0, usize, Path, Vec<Path>),
    /// A point that is drawn as determined by its vertex_shape
    Point(Diagram0, Point),
}

impl<const N: usize> GraphicElement<N> {
    /// Apply an affine coordinate transformation to the element.
    #[must_use]
    pub fn transformed(&self, transform: &Transform2D<f32>) -> Self {
        use GraphicElement::{Point, Surface, Wire};
        match self {
            Surface(g, path) => Surface(*g, path.clone().transformed(transform)),
            Wire(g, depth, path, mask) => {
                let path = path.clone().transformed(transform);
                let mask = mask
                    .iter()
                    .map(|mask| mask.clone().transformed(transform))
                    .collect();
                Wire(*g, *depth, path, mask)
            }
            Point(g, point) => Point(*g, transform.transform_point(*point)),
        }
    }

    pub fn generator(&self) -> Generator {
        use GraphicElement::{Point, Surface, Wire};
        match self {
            Surface(d, _) | Wire(d, _, _, _) | Point(d, _) => d.generator,
        }
    }

    pub fn orientation(&self) -> Orientation {
        use GraphicElement::{Point, Surface, Wire};
        match self {
            Surface(d, _) | Wire(d, _, _, _) | Point(d, _) => d.orientation,
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
        complex: &[(Simplex<N>, bool)],
        layout: &Layout<N>,
        projection: &Projection<N>,
        depths: &Depths<N>,
    ) -> Vec<Self> {
        let mut wire_elements = Vec::new();
        let mut surface_elements = Vec::new();
        let mut point_elements = Vec::new();

        let mut grouped_surfaces = FastHashMap::<Diagram0, Vec<[Coordinate<N>; 3]>>::default();

        for (simplex, _) in complex.iter().filter(|(_, visible)| *visible) {
            match simplex {
                Simplex::Surface(ps) => {
                    let generator = projection.generator(ps[0]);
                    grouped_surfaces
                        .entry(generator)
                        .or_default()
                        .push(orient_surface(ps));
                }
                Simplex::Wire(ps) => {
                    let (generator, _) = projection.front_generator(ps[0]);

                    let (depth, mask) = match depths.edge_depth(ps[0], ps[1]) {
                        Some(depth) => (
                            depth + 1,
                            depths
                                .edges_above(depth, ps[1])
                                .into_iter()
                                .map(|s| build_path(&[s, ps[1]], false, layout, projection))
                                .collect(),
                        ),
                        None => (0, vec![]),
                    };
                    wire_elements.push(Self::Wire(
                        generator,
                        depth,
                        build_path(&orient_wire(ps), false, layout, projection),
                        mask,
                    ));
                }
                Simplex::Point([p]) => {
                    let (generator, is_identity) = projection.front_generator(*p);
                    if !is_identity {
                        point_elements.push(Self::Point(generator, project_2d(layout[p]).into()));
                    }
                }
            }
        }

        for (generator, surfaces) in grouped_surfaces {
            let mut path_builder = NoAttributes::new().with_svg();

            for points in merge_simplices(surfaces) {
                make_path(&points, true, layout, projection, &mut path_builder);
            }

            // Quick enough to do it every time
            let path = simplify_path(&path_builder.build());

            surface_elements.push(Self::Surface(generator, path));
        }

        let mut elements = surface_elements;
        elements.extend(wire_elements);
        elements.extend(point_elements);
        elements
    }

    pub fn to_shape(&self, wire_thickness: f32, point_radius: f32) -> Shape {
        match self {
            GraphicElement::Surface(_, path) => Fill::new(path.clone()).into(),
            GraphicElement::Wire(_, _, path, _) => Stroke::new(path.clone(), wire_thickness).into(),
            GraphicElement::Point(_, point) => Circle::new(*point, point_radius).into(),
        }
    }
}

// TODO(@calintat): These functions can be removed if we record the parity of each simplex.
fn orient_wire<const N: usize>(wire: &[Coordinate<N>; 2]) -> [Coordinate<N>; 2] {
    let [a, b] = *wire;
    if a[0] < b[0] {
        [a, b]
    } else {
        [b, a]
    }
}

fn orient_surface<const N: usize>(surface: &[Coordinate<N>; 3]) -> [Coordinate<N>; 3] {
    fn ordering_to_int(ordering: Ordering) -> isize {
        match ordering {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    let a0 = ordering_to_int(surface[1][1].cmp(&surface[0][1]));
    let a1 = ordering_to_int(surface[1][0].cmp(&surface[0][0]));
    let b0 = ordering_to_int(surface[2][1].cmp(&surface[1][1]));
    let b1 = ordering_to_int(surface[2][0].cmp(&surface[1][0]));

    if a0 * b1 - a1 * b0 < 0 {
        [surface[1], surface[0], surface[2]]
    } else {
        *surface
    }
}

/// Merge a collection of simplices to a sequence of closed curves such that rendering the
/// curves with the SVG evenodd fill rule will yield the
fn merge_simplices<P, I>(simplices: I) -> Vec<Vec<P>>
where
    P: Hash + Eq + Copy,
    I: IntoIterator<Item = [P; 3]>,
{
    #[derive(Debug, Clone, Copy)]
    struct EdgeData<P> {
        source: P,
        prev: usize,
        next: usize,
    }

    // Doubly linked circular lists of edges. This will be initialised with all the edges along the
    // boundaries of the input simplices. After that edges found to be in the interior of the final
    // surface are removed and the lists relinked appropriately.
    let mut edges = Vec::<Option<EdgeData<P>>>::new();

    // Pairs of indices of edges that overlap going in opposite directions. Edges in this list are
    // in the interior of the surface so they will be removed from the final surface boundaries.
    let mut interior_pairs = Vec::<(usize, usize)>::new();

    // Map that allows to look up an edge index by the pair of its source and target points. Using
    // the invariant that no edge can be doubled we optimize this by removing any edge that is
    // found to be in the interior.
    let mut edge_to_index = FastHashMap::<(P, P), usize>::default();

    // Iterate over the simplices and add the edges.
    for simplex in simplices {
        let base_index = edges.len();

        for i in 0..3 {
            let source = simplex[i];
            let target = simplex[(i + 1) % 3];
            let edge_index = base_index + i;

            if let Some(rev_index) = edge_to_index.remove(&(target, source)) {
                // If we already added the opposite edge, this new edge is in the interior.
                interior_pairs.push((edge_index, rev_index));
            } else {
                edge_to_index.insert((source, target), edge_index);
            }

            edges.push(Some(EdgeData {
                source,
                prev: i.checked_sub(1).unwrap_or(2) + base_index,
                next: (i + 1) % 3 + base_index,
            }));
        }
    }

    // Remove the interior edges.
    for (a, b) in std::mem::take(&mut interior_pairs) {
        let a_data = edges[a].unwrap();
        let b_data = edges[b].unwrap();

        edges[a_data.prev].as_mut().unwrap().next = b_data.next;
        edges[b_data.next].as_mut().unwrap().prev = a_data.prev;

        edges[a_data.next].as_mut().unwrap().prev = b_data.prev;
        edges[b_data.prev].as_mut().unwrap().next = a_data.next;

        edges[a] = None;
        edges[b] = None;
    }

    // Extract the remaining circular paths.
    let mut paths = Vec::new();

    for start in 0..edges.len() {
        let mut path = Vec::new();
        let mut current = start;

        while let Some(data) = std::mem::take(&mut edges[current]) {
            path.push(data.source);
            current = data.next;
        }

        if !path.is_empty() {
            paths.push(path);
        }
    }

    paths
}

fn build_path<const N: usize>(
    points: &[Coordinate<N>],
    closed: bool,
    layout: &Layout<N>,
    projection: &Projection<N>,
) -> Path {
    let mut builder = NoAttributes::new().with_svg();
    make_path(points, closed, layout, projection, &mut builder);
    builder.build()
}

fn make_path<const N: usize>(
    points: &[Coordinate<N>],
    closed: bool,
    layout: &Layout<N>,
    projection: &Projection<N>,
    builder: &mut lyon_path::builder::WithSvg<lyon_path::path::Builder>,
) {
    let start = project_2d(layout[&points[0]]).into();
    builder.move_to(start);

    for i in 1..points.len() {
        make_path_segment(points[i - 1], points[i], layout, projection, builder);
    }

    if closed {
        make_path_segment(
            *points.last().unwrap(),
            points[0],
            layout,
            projection,
            builder,
        );
    }
}

fn make_path_segment<const N: usize>(
    start: Coordinate<N>,
    end: Coordinate<N>,
    layout: &Layout<N>,
    projection: &Projection<N>,
    builder: &mut lyon_path::builder::WithSvg<lyon_path::path::Builder>,
) {
    use self::{
        Height::{Regular, Singular},
        SliceIndex::Interior,
    };

    let layout_start: Point = project_2d(layout[&start]).into();
    let layout_end: Point = project_2d(layout[&end]).into();

    match ((start.first(), start.get(1)), (end.first(), end.get(1))) {
        (
            (Some(Interior(Regular(_))), _),
            (Some(Interior(Singular(_))), Some(Interior(Singular(_)))),
        ) => {
            match projection.homotopy(end) {
                // Vertical tangent
                Some(Homotopy::HalfBraid) => builder.cubic_bezier_to(
                    (layout_start.x, 0.2 * layout_start.y + 0.8 * layout_end.y).into(),
                    (layout_end.x, 0.4 * layout_start.y + 0.6 * layout_end.y).into(),
                    layout_end,
                ),
                // Horizontal tangent
                _ => builder.cubic_bezier_to(
                    (layout_start.x, 0.2 * layout_start.y + 0.8 * layout_end.y).into(),
                    (0.6 * layout_start.x + 0.4 * layout_end.x, layout_end.y).into(),
                    layout_end,
                ),
            }
        }
        (
            (Some(Interior(Singular(_))), Some(Interior(Singular(_)))),
            (Some(Interior(Regular(_))), _),
        ) => {
            match projection.homotopy(start) {
                // Vertical tangent
                Some(Homotopy::HalfBraid) => builder.cubic_bezier_to(
                    (layout_start.x, 0.4 * layout_end.y + 0.6 * layout_start.y).into(),
                    (layout_end.x, 0.2 * layout_end.y + 0.8 * layout_start.y).into(),
                    layout_end,
                ),
                // Horizontal tangent
                _ => builder.cubic_bezier_to(
                    (0.6 * layout_end.x + 0.4 * layout_start.x, layout_start.y).into(),
                    (layout_end.x, 0.2 * layout_end.y + 0.8 * layout_start.y).into(),
                    layout_end,
                ),
            }
        }
        _ => builder.line_to(layout_end),
    };
}

impl<const N: usize> From<GraphicElement<N>> for GeneratorRepresentation {
    fn from(element: GraphicElement<N>) -> Self {
        match element {
            GraphicElement::Point(_, _) => Self::Point,
            GraphicElement::Wire(_, _, _, _) => Self::Wire,
            GraphicElement::Surface(_, _) => Self::Surface,
        }
    }
}

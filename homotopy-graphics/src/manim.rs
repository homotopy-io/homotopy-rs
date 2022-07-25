use std::fmt::Write;

use euclid::default::Point2D;
use homotopy_common::hash::FastHashMap;
use homotopy_core::{
    common::DimensionError,
    complex::make_complex,
    layout::Layout,
    projection::{Depths, Projection},
    Diagram, Generator,
};
use itertools::Itertools;
use lyon_path::{Event, Path};

use crate::{
    path_util::simplify_graphic,
    style::{GeneratorStyle, SignatureStyleData, VertexShape},
    svg::render::GraphicElement,
};

const INDENT: &str = "    ";

pub fn stylesheet(styles: &impl SignatureStyleData) -> String {
    let mut stylesheet = String::new();

    for (generator, style) in styles.as_pairs() {
        writeln!(
            stylesheet,
            "            \"{generator}\": \"{color}\",",
            generator = name(generator),
            color = &style.color().hex()
        )
        .unwrap();
    }

    stylesheet
}

fn name(generator: Generator) -> String {
    format!("generator_{}_{}", generator.id, generator.dimension)
}

pub fn render(
    diagram: &Diagram,
    signature_styles: &impl SignatureStyleData,
    stylesheet: &str,
) -> Result<String, DimensionError> {
    let layout = Layout::<2>::new(diagram)?;
    let complex = make_complex(diagram);
    let depths = Depths::<2>::new(diagram)?;
    let projection = Projection::<2>::new(diagram, &layout, &depths)?;
    let graphic = simplify_graphic(&GraphicElement::build(
        diagram,
        &complex,
        &layout,
        &projection,
        &depths,
    ));

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
            GraphicElement::Wire(g, depth, path, _mask) => {
                max_point = max_point.max(max_point_path(&path));
                wires.entry(depth).or_default().push((g, path));
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
            "{ind}{ind}surfaces.add(VMobject(){path}.set_stroke(width=1).set_fill(C[\"{color}\"],0.75)) # path_{id}_{dim}",
            ind=INDENT,
            color=name(g),
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
                writeln!(manim, concat!("{ind}{ind}wires.add(Intersection(surfaces,",
                         "VMobject(){path}.set_stroke(width=10),color=C[\"generator_{id}_{dim}\"],fill_opacity=0.8)) # path_{id}_{dim}"),
                         ind=INDENT,
                         id=g.id,
                         dim=g.dimension,
                         path=&render_path(path)
                ).unwrap();
            }
            writeln!(manim, "{ind}{ind}# End scope", ind = INDENT).unwrap();
        }

        for (g, path) in &layer {
            writeln!(manim, "{ind}{ind}wires.add(VMobject(){path}.set_stroke(color=C[\"{color}\"],width=5)) # path_{id}_{dim}",
                ind=INDENT,
                color=name(*g),
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
        let vertex = render_vertex(signature_styles.generator_style(g).unwrap(), &name(g));
        writeln!(
            manim,
            "{ind}{ind}points.add({vertex}.move_to({pt})) # circle_{id}_{dim}",
            ind = INDENT,
            id = g.id,
            dim = g.dimension,
            vertex = vertex,
            pt = &render_point(point)
        )
        .unwrap();
    }

    writeln!(
        manim,
        concat!("{ind}{ind}return points\n\n",
            "{ind}# We now put everything together\n",
            "{ind}def construct(self):\n",
            "{ind}{ind}#C = self.get_colors()\n",
            "{ind}{ind}surfaces = self.get_surfaces()\n",
            "{ind}{ind}wires = self.get_wires(surfaces)\n",
            "{ind}{ind}points = self.get_points()\n",
            "{ind}{ind}# Background (for rendering consistency, set color=BLACK if unwanted)\n",
            "{ind}{ind}bg = Rectangle(width={x}*2,height={y}*2,color=WHITE,fill_opacity=1).move_to(surfaces)\n",
            "{ind}{ind}# Root\n",
            "{ind}{ind}scale_factor = max(config.frame_size[0]/{x},config.frame_size[1]/{x})*0.002 # Magic number\n",
            "{ind}{ind}root = VGroup(bg,surfaces,wires,points).shift({x}*LEFT+{y}*DOWN).scale(scale_factor)\n",
            "{ind}{ind}# Static output (low rendering times)\n",
            "{ind}{ind}#self.add(root)\n",
            "{ind}{ind}# Animated output\n",
            "{ind}{ind}self.play(DrawBorderThenFill(VGroup(bg,surfaces)))\n",
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
    format!("np.array([{},{},0])", x, y)
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

fn render_vertex(generator_style: &impl GeneratorStyle, color: &str) -> String {
    use VertexShape::{Circle, Square};
    const CIRCLE_RADIUS: f32 = 0.125;
    const SQUARE_SIDELENGTH: f32 = 0.125 / 2.;

    match generator_style.shape().unwrap_or_default() {
        Circle => format!(
            "Circle(radius={radius},color=C[\"{color}\"],fill_opacity=1)",
            radius = CIRCLE_RADIUS,
            color = color,
        ),
        Square => format!(
            "Square(side_length={side_length},color=C[\"{color}\"],fill_opacity=1)",
            side_length = SQUARE_SIDELENGTH,
            color = color,
        ),
    }
}

fn render_path(path: &Path) -> String {
    let mut result = String::new();
    for event in path {
        match event {
            Event::Begin { at } => {
                write!(result, ".start_new_path({})", render_point(at)).unwrap();
            }
            Event::Line { to, .. } => {
                write!(result, ".add_line_to({})", render_point(to)).unwrap();
            }
            Event::Quadratic { ctrl, to, .. } => write!(
                result,
                ".add_quadratic_bezier_curve_to({},{})",
                render_point(ctrl),
                render_point(to)
            )
            .unwrap(),
            Event::Cubic {
                ctrl1, ctrl2, to, ..
            } => write!(
                result,
                ".add_cubic_bezier_curve_to({},{},{})",
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

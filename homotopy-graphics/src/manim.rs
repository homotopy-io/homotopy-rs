use std::fmt::Write;

use euclid::default::Point2D;
use homotopy_common::hash::FastHashMap;
use homotopy_core::{
    common::DimensionError,
    complex::make_complex,
    diagram::Diagram0,
    layout::Layout,
    projection::{Depths, Projection},
    Diagram, Generator, Orientation,
};
use itertools::Itertools;
use lyon_path::{Event, Path};
use serde::Serialize;

use crate::{
    path_util::simplify_graphic,
    style::{GeneratorRepresentation, GeneratorStyle, SignatureStyleData, VertexShape},
    svg::render::GraphicElement,
};

const INDENT: &str = "    ";

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize)]
pub struct ManimOptions {
    /// Whether to use the OpenGL renderer.
    pub use_opengl: bool,
}

pub fn stylesheet(styles: &impl SignatureStyleData) -> String {
    let mut stylesheet = String::new();

    for generator in styles.generators() {
        let color = styles.generator_style(generator).unwrap().color();
        for c in 0..3 {
            for orientation in [
                Orientation::Positive,
                Orientation::Zero,
                Orientation::Negative,
            ] {
                writeln!(
                    stylesheet,
                    "            \"{generator}\": \"{color}\",",
                    generator = name(generator, c, orientation),
                    color = color.lighten(c, orientation).hex()
                )
                .unwrap();
            }
        }
    }

    stylesheet
}

#[inline]
#[must_use]
pub fn name_from_diagram_dim(
    diagram: Diagram0,
    diagram_dimension: usize,
    representation: GeneratorRepresentation,
) -> String {
    let d = diagram_dimension;
    let n = diagram.generator.dimension;
    let k = representation as usize;

    let c = d.saturating_sub(n + k) % 3;

    name(diagram.generator, c, diagram.orientation)
}

#[inline]
fn name(generator: Generator, c: usize, orientation: Orientation) -> String {
    format!(
        "generator_{}_{}_{c}_{}",
        generator.id,
        generator.dimension,
        match orientation {
            Orientation::Positive => "pos",
            Orientation::Negative => "neg",
            Orientation::Zero => "zer",
        }
    )
}

pub fn render(
    diagram: &Diagram,
    dimension: u8,
    signature_styles: &impl SignatureStyleData,
    stylesheet: &str,
    options: ManimOptions,
) -> Result<String, DimensionError> {
    match dimension {
        0 => render_generic::<0>(diagram, signature_styles, stylesheet, options),
        1 => render_generic::<1>(diagram, signature_styles, stylesheet, options),
        2 => render_generic::<2>(diagram, signature_styles, stylesheet, options),
        _ => Err(DimensionError),
    }
}

fn render_generic<const N: usize>(
    diagram: &Diagram,
    signature_styles: &impl SignatureStyleData,
    stylesheet: &str,
    options: ManimOptions,
) -> Result<String, DimensionError> {
    let layout = Layout::<N>::new(diagram)?;
    let complex = make_complex(diagram);
    let depths = Depths::<N>::new(diagram)?;
    let projection = Projection::<N>::new(diagram, &layout, &depths)?;
    let graphic = simplify_graphic(&GraphicElement::build(
        &complex,
        &layout,
        &projection,
        &depths,
    ));

    let mut surfaces = Vec::default();
    let mut wires: FastHashMap<usize, Vec<(Diagram0, Path)>> = FastHashMap::default();
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
    if options.use_opengl {
        manim.push_str(
            "# Render with 'manim --format mp4 --renderer=opengl homotopy_io_export.py'\n",
        );
    } else {
        manim.push_str(
            "# Render with 'manim --format mp4 --renderer=cairo homotopy_io_export.py'\n",
        );
    }
    manim.push_str("import numpy as np\n");
    manim.push_str("from manim import *\n");
    if options.use_opengl {
        manim.push_str(
            "from manim.mobject.opengl.opengl_vectorized_mobject import OpenGLVMobject\n",
        );
    }

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

    let vmobj = if options.use_opengl {
        "OpenGLVMobject"
    } else {
        "VMobject"
    };
    writeln!(
        manim,
        concat!(
            "{ind}def build_path(self, geom, **kwargs):\n",
            "{ind}{ind}obj = {vmobj}()\n",
            "{ind}{ind}obj.set_stroke(**kwargs)\n",
            "{ind}{ind}for c in geom:\n",
            "{ind}{ind}{ind}if c[0] == 0:\n",
            "{ind}{ind}{ind}{ind}obj.start_new_path(c[1])\n",
            "{ind}{ind}{ind}elif c[0] == 1:\n",
            "{ind}{ind}{ind}{ind}obj.add_line_to(c[1])\n",
            "{ind}{ind}{ind}elif c[0] == 2:\n",
            "{ind}{ind}{ind}{ind}obj.add_quadratic_bezier_curve_to(c[1],c[2])\n",
            "{ind}{ind}{ind}else:\n",
            "{ind}{ind}{ind}{ind}obj.add_cubic_bezier_curve_to(c[1],c[2],c[3])\n",
            "{ind}{ind}return obj\n",
        ),
        ind = INDENT,
        vmobj = vmobj,
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
    for (d, path) in surfaces {
        writeln!(
            manim,
            "{ind}{ind}surfaces.add(self.build_path({path},width=1).set_fill(C[\"{color}\"],1)) # path_{id}_{dim}",
            ind=INDENT,
            color=name_from_diagram_dim(d, diagram.dimension(), GeneratorRepresentation::Surface),
            id=d.generator.id,
            dim=d.generator.dimension,
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
            writeln!(manim, "{INDENT}{INDENT}# Begin scope").unwrap();
            for (d, path) in &layer {
                writeln!(manim, "{INDENT}{INDENT}wires.add(Intersection(surfaces,self.build_path({path},width=20),color=C[\"generator_{id}_{dim}\"]))",
                         id=d.generator.id,
                         dim=d.generator.dimension,
                         path=&render_path(path)
                ).unwrap();
            }
            writeln!(manim, "{INDENT}{INDENT}# End scope").unwrap();
        }

        for (d, path) in &layer {
            writeln!(manim, "{INDENT}{INDENT}wires.add(self.build_path({path},width=20,color=C[\"{color}\"])) # path_{id}_{dim}",
                color=name_from_diagram_dim(*d, diagram.dimension(), GeneratorRepresentation::Wire),
                id=d.generator.id,
                dim=d.generator.dimension,
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
    for (d, point) in points {
        let vertex = render_vertex(
            signature_styles.generator_style(d.generator).unwrap(),
            &name_from_diagram_dim(d, diagram.dimension(), GeneratorRepresentation::Point),
        );
        writeln!(
            manim,
            "{ind}{ind}points.add({vertex}.move_to(np.array([{ptx},{pty},1])) # circle_{id}_{dim}",
            ind = INDENT,
            id = d.generator.id,
            dim = d.generator.dimension,
            vertex = vertex,
            ptx = point.x,
            pty = point.y
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
            "{ind}{ind}bg = Rectangle(width={x}*2,height={y}*2,color=WHITE).move_to(surfaces)\n",
            "{ind}{ind}# Root\n",
            "{ind}{ind}scale_factor = max(config.frame_size[0]/{x},config.frame_size[1]/{x})*0.002 # Magic number\n",
            "{ind}{ind}root = VGroup(bg,surfaces,wires,points).shift({x}*LEFT+{y}*DOWN).scale(scale_factor)\n",
            "{ind}{ind}# Static output (low rendering times)\n",
            "{ind}{ind}#self.add(root)\n",
            "{ind}{ind}# Animated output\n",
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
    format!("np.array([{x},{y},0])")
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
    const CIRCLE_RADIUS: f32 = 0.125 / 2.0;
    const SQUARE_SIDELENGTH: f32 = 0.125 / 2.;

    match generator_style.shape() {
        Circle => format!("Circle(radius={CIRCLE_RADIUS},color=C[\"{color}\"],fill_opacity=1)",),
        Square => {
            format!("Square(side_length={SQUARE_SIDELENGTH},color=C[\"{color}\"],fill_opacity=1)",)
        }
    }
}

fn render_path(path: &Path) -> String {
    let mut result = String::new();
    write!(result, "[").unwrap();
    for event in path {
        match event {
            Event::Begin { at } => {
                write!(result, "(0,{}),", render_point(at)).unwrap();
            }
            Event::Line { to, .. } => {
                write!(result, "(1,{}),", render_point(to)).unwrap();
            }
            Event::Quadratic { ctrl, to, .. } => {
                write!(result, "(2,{},{}),", render_point(ctrl), render_point(to)).unwrap();
            }
            Event::Cubic {
                ctrl1, ctrl2, to, ..
            } => write!(
                result,
                "(3,{},{},{}),",
                render_point(ctrl1),
                render_point(ctrl2),
                render_point(to),
            )
            .unwrap(),
            Event::End { .. } => {}
        }
    }
    write!(result, "]").unwrap();
    result
}

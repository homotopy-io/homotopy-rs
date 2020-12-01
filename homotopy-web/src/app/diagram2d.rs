use closure::closure;
use homotopy_core::graphic2d::*;
use homotopy_core::layout::Layout;
use homotopy_core::{Boundary, DiagramN, Generator, SliceIndex};
use yew::prelude::*;

pub struct Diagram2D {
    props: Props,
    computed: Computed,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub diagram: DiagramN,
    #[prop_or_default]
    pub style: Style,
    #[prop_or_default]
    pub on_select: Callback<(SliceIndex, SliceIndex)>,
}

#[derive(Clone, PartialEq, Copy)]
pub struct Style {
    scale: f32,
    wire_thickness: f32,
    point_radius: f32,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            scale: 40.0,
            wire_thickness: 8.0,
            point_radius: 6.0,
        }
    }
}

// TODO: Generator -> Color map in props
// TODO: Drag callbacks in props
// TODO: Highlights in props

pub enum Message {}

/// The computed properties of a diagram that are potentially expensive to compute but can be
/// cached if the diagram does not change.
struct Computed {
    elements: Vec<Element>,
    generators: Generators,
    layout: Layout,
}

impl Computed {
    fn new(diagram: &DiagramN) -> Self {
        assert!(diagram.dimension() >= 2);

        let mut elements = make_elements(&diagram);
        elements.sort_by_key(Element::codimension);
        let generators = Generators::new(&diagram);
        let layout = Layout::new(diagram.clone(), 2000).unwrap();
        Computed {
            elements,
            generators,
            layout,
        }
    }
}

impl Component for Diagram2D {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let computed = Computed::new(&props.diagram);
        Diagram2D { props, computed }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.diagram != props.diagram {
            self.props = props;
            self.computed = Computed::new(&self.props.diagram);
            true
        } else if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let (width, height) = self.dimensions();

        // TODO: Boundaries

        html! {
            <svg xmlns={"http://www.w3.org/2000/svg"} width={width} height={height}>
                {self.computed.elements.iter().map(|e| self.view_element(e)).collect::<Html>()}
            </svg>
        }
    }
}

impl Diagram2D {
    fn transform_coordinates(&self, c: (f32, f32)) -> (f32, f32) {
        let x = (c.0 + 1.0) * self.props.style.scale;
        let y = self.dimensions().1 - c.1 * self.props.style.scale;
        (x, y)
    }

    fn generator_color(&self, generator: Generator) -> &str {
        let colors = &["lightgray", "gray", "black"];
        colors[generator.id]
    }

    /// The width and height of the diagram in pixels.
    fn dimensions(&self) -> (f32, f32) {
        let (width, height) = self.computed
            .layout
            .get(Boundary::Target.into(), Boundary::Target.into())
            .unwrap();
        let width = width * self.props.style.scale;
        let height = height * self.props.style.scale;
        (width, height)
    }

    fn view_element(&self, element: &Element) -> Html {
        match element {
            Element::CellPoint(point) => self.view_cell_point(*point),
            Element::IdentityWire(points) => self.view_identity_wire(points),
            Element::CellWire(points) => self.view_cell_wire(points),
            Element::CellSurface(points) => self.view_cell_surface(points),
            Element::IdentitySurface(points) => self.view_identity_surface(points),
        }
    }

    fn view_cell_point(&self, point: Point) -> Html {
        let generator = self.computed.generators.get(point.0, point.1).unwrap();
        let color = self.generator_color(generator);
        let (x, y) =
            self.transform_coordinates(self.computed.layout.get(point.0, point.1).unwrap());

        let onclick = self
            .props
            .on_select
            .reform(closure!(clone point, |_| point));

        html! {
            <circle r={self.props.style.point_radius} cx={x} cy={y} fill={color} onclick={onclick} />
        }
    }

    fn view_cell_wire(&self, points: &[Point]) -> Html {
        // TODO: Masks for wires that pass over each other

        let generator = self
            .computed
            .generators
            .get(points[0].0, points[0].1)
            .unwrap();
        let color = self.generator_color(generator);

        let path = {
            let mut path = String::new();

            // TODO: Bezier curves
            for (i, (x, y)) in points.iter().enumerate() {
                let (x, y) = self.transform_coordinates(self.computed.layout.get(*x, *y).unwrap());

                if i == 0 {
                    path += &format!("M {} {}", x, y);
                } else {
                    path += &format!("L {} {}", x, y);
                }
            }
            path
        };

        let onclick = {
            let first_point = &points[0];
            self.props
                .on_select
                .reform(closure!(clone first_point, |_| first_point))
        };

        html! {
            <path d={path} stroke={color} strokeWidth={self.props.style.wire_thickness} onclick={onclick} />
        }
    }

    fn view_cell_surface(&self, points: &[Point]) -> Html {
        let generator = self
            .computed
            .generators
            .get(points[0].0, points[0].1)
            .unwrap();
        let color = self.generator_color(generator);

        let path = {
            let mut path = String::new();

            // TODO: Bezier curves
            for (i, (x, y)) in points.iter().enumerate() {
                let (x, y) = self.transform_coordinates(self.computed.layout.get(*x, *y).unwrap());

                if i == 0 {
                    path += &format!("M {} {}", x, y);
                } else {
                    path += &format!("L {} {}", x, y);
                }
            }

            path
        };

        let onclick = {
            let first_point = &points[0];
            self.props
                .on_select
                .reform(closure!(clone first_point, |_| first_point))
        };

        html! {
            <path d={path} fill={color} stroke={color} strokeWidth={1} onclick={onclick} />
        }
    }

    fn view_identity_wire(&self, points: &[Point]) -> Html {
        let generator = self
            .computed
            .generators
            .get(points[0].0, points[0].1)
            .unwrap();
        let color = self.generator_color(generator);

        let path = {
            let mut path = String::new();

            // TODO: Bezier curves
            for (i, (x, y)) in points.iter().enumerate() {
                let (x, y) = self.transform_coordinates(self.computed.layout.get(*x, *y).unwrap());

                if i == 0 {
                    path += &format!("M {} {}", x, y);
                } else {
                    path += &format!("L {} {}", x, y);
                }
            }

            path
        };

        // TODO: Calculate click position using height

        html! {
            <path d={path} stroke={color} strokeWidth={self.props.style.wire_thickness} />
        }
    }

    fn view_identity_surface(&self, element: &[(Point, Point)]) -> Html {
        let points = {
            let mut points = Vec::new();
            points.extend(element.iter().map(|(p, _)| *p));
            points.extend(element.iter().rev().map(|(_, p)| *p));
            points
        };

        let generator = self
            .computed
            .generators
            .get(points[0].0, points[0].1)
            .unwrap();
        let color = self.generator_color(generator);

        let path = {
            let mut path = String::new();

            // TODO: Bezier curves?
            for (i, (x, y)) in points.iter().enumerate() {
                let (x, y) = self.transform_coordinates(self.computed.layout.get(*x, *y).unwrap());

                if i == 0 {
                    path += &format!("M {} {}", x, y);
                } else {
                    path += &format!("L {} {}", x, y);
                }
            }
            path
        };

        // TODO: Calculate click position using height

        html! {
            <path d={path} fill={color} stroke={color} strokeWidth={1} />
        }
    }
}

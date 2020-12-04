use euclid::default::{Point2D, Size2D, Transform2D};
use homotopy_core::complex::{make_complex, Simplex};
use homotopy_core::projection::Generators;
use homotopy_core::{Boundary, DiagramN, Generator};
use homotopy_graphics::geometry;
use homotopy_graphics::geometry::path_to_svg;
use homotopy_graphics::graphic2d::*;
use homotopy_graphics::layout2d::Layout;
use web_sys::Element;
use yew::prelude::*;

pub struct Diagram2D {
    props: Props,
    computed: Computed,
    link: ComponentLink<Self>,
    node_ref: NodeRef,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub diagram: DiagramN,
    #[prop_or_default]
    pub style: Style,
    #[prop_or_default]
    pub on_select: Callback<Simplex>,
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

pub enum Message {
    /// The diagram has been clicked at a point in screen coordinates.
    OnClick(Point2D<f32>),
}

/// The computed properties of a diagram that are potentially expensive to compute but can be
/// cached if the diagram does not change.
struct Computed {
    layout: Layout,
    graphic: Vec<GraphicElement>,
    actions: Vec<ActionRegion>,
}

impl Computed {
    fn new(diagram: &DiagramN) -> Self {
        assert!(diagram.dimension() >= 2);

        let generators = Generators::new(&diagram);
        let layout = Layout::new(diagram.clone(), 2000).unwrap();
        let complex = make_complex(&diagram);
        let graphic = GraphicElement::build(&complex, &layout, &generators);
        let actions = ActionRegion::build(&complex, &layout);
        Computed {
            layout,
            graphic,
            actions,
        }
    }
}

impl Component for Diagram2D {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let computed = Computed::new(&props.diagram);
        let node_ref = NodeRef::default();
        Diagram2D {
            props,
            computed,
            link,
            node_ref,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::OnClick(point) => {
                let point = self.transform_screen_to_image().transform_point(point);
                match self.find_region(point) {
                    Some(region) => self.props.on_select.emit(region.into()),
                    None => {}
                }
                false
            }
        }
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
        let size = self.dimensions();

        let onclick = {
            let link = self.link.clone();
            Callback::from(move |e: MouseEvent| {
                let x = e.client_x() as f32;
                let y = e.client_y() as f32;
                link.send_message(Message::OnClick((x, y).into()));
            })
        };

        html! {
            <svg
                xmlns={"http://www.w3.org/2000/svg"}
                width={size.width}
                height={size.height}
                onclick={onclick}
                ref={self.node_ref.clone()}
            >
                {self.computed.graphic.iter().map(|e| self.view_element(e)).collect::<Html>()}
            </svg>
        }
    }
}

impl Diagram2D {
    /// Transform coordinates in the diagram layout to coordinates in the SVG image. In
    /// particular, the vertical direction is flipped so that diagrams are read from
    /// bottom to top.
    fn transform(&self) -> Transform2D<f32> {
        let scale = self.props.style.scale;
        Transform2D::scale(scale, -scale).then_translate((0.0, self.dimensions().height).into())
    }

    /// Transform coordinates on the screen (such as those in `MouseEvent`s) to coordinates in the
    /// SVG image. This incorporates translation and zoom of the diagram component.
    fn transform_screen_to_image(&self) -> Transform2D<f32> {
        let rect = self
            .node_ref
            .cast::<Element>()
            .unwrap()
            .get_bounding_client_rect();

        let screen_size = Size2D::new(rect.width() as f32, rect.height() as f32);
        let image_size = self.dimensions();

        Transform2D::translation(-rect.left() as f32, -rect.top() as f32).then_scale(
            image_size.width / screen_size.width,
            image_size.height / screen_size.height,
        )
    }

    /// Find the action region corresponding to a point in image coordinates.
    /// To use this to react to mouse events, the location of the mouse first needs
    /// to be translated into image coordinates using `transform_screen_to_image`.
    fn find_region(&self, point: Point2D<f32>) -> Option<&ActionRegion> {
        // TODO: Cache the hit test geometry, so we can precompute bounding rectangles
        let result = self.computed.actions.iter().find(|region| {
            let region = region.transformed(&self.transform());

            match region {
                ActionRegion::Surface(_, path) => {
                    geometry::Fill::new(path).contains_point(point, 1.0)
                }
                ActionRegion::Wire(_, path) => {
                    let width = self.props.style.wire_thickness;
                    geometry::Stroke::new(path, width).contains_point(point, 1.0)
                }
                ActionRegion::Point(_, center) => {
                    let radius = self.props.style.point_radius;
                    geometry::Circle::new(center, radius).contains_point(point)
                }
            }
        });

        result
    }

    fn generator_color(&self, generator: Generator) -> &str {
        let colors = &["lightgray", "gray", "black"];
        colors[generator.id]
    }

    /// The width and height of the diagram image in pixels.
    ///
    /// This is not the size of the diagram as it appears on the screen, since
    /// it might be zoomed by any parent component.
    fn dimensions(&self) -> Size2D<f32> {
        let size = self
            .computed
            .layout
            .get(Boundary::Target.into(), Boundary::Target.into())
            .unwrap()
            .to_vector()
            .to_size();

        size * self.props.style.scale
    }

    /// Creates the SVG elements for the diagram.
    fn view_element(&self, element: &GraphicElement) -> Html {
        let generator = element.generator();
        let color = self.generator_color(generator);

        match element {
            GraphicElement::Surface(_, path) => {
                let path = path_to_svg(path.transformed(&self.transform()));
                html! {
                    <path d={path} fill={color} stroke={color} stroke-width={1} />
                }
            }
            GraphicElement::Wire(_, path) => {
                let path = path_to_svg(path.transformed(&self.transform()));
                html! {
                    <path d={path} stroke={color} stroke-width={self.props.style.wire_thickness} fill="none" />
                }
            }
            GraphicElement::Point(_, point) => {
                let point = self.transform().transform_point(*point);
                html! {
                    <circle r={self.props.style.point_radius} cx={point.x} cy={point.y} fill={color} />
                }
            }
        }
    }
}

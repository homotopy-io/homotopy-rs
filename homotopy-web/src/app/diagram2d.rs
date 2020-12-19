use crate::app::signature_stylesheet::SignatureStylesheet;
use crate::model::RenderStyle;
use euclid::default::{Point2D, Size2D, Transform2D, Vector2D};
use homotopy_core::complex::{make_complex, Simplex};
use homotopy_core::projection::Generators;
use homotopy_core::{Boundary, Diagram, DiagramN, Generator, Height, SliceIndex};
use homotopy_graphics::geometry;
use homotopy_graphics::geometry::path_to_svg;
use homotopy_graphics::graphic2d::*;
use homotopy_graphics::layout2d::Layout;
use web_sys::Element;
use yew::prelude::*;

pub struct Diagram2D {
    props: Props2D,
    diagram: PreparedDiagram,
    link: ComponentLink<Self>,
    node_ref: NodeRef,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props2D {
    pub diagram: DiagramN,
    #[prop_or_default]
    pub style: RenderStyle,
    #[prop_or_default]
    pub on_select: Callback<Vec<Vec<SliceIndex>>>,
    #[prop_or_default]
    pub highlight: Option<Highlight2D>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Highlight2D {
    pub from: [SliceIndex; 2],
    pub to: [SliceIndex; 2],
}

// TODO: Drag callbacks in props
// TODO: Highlights in props

pub enum Message2D {
    /// The diagram has been clicked at a point in screen coordinates.
    OnClick(Point2D<f32>),
}

/// The computed properties of a diagram that are potentially expensive to compute but can be
/// cached if the diagram does not change.
struct PreparedDiagram {
    graphic: Vec<GraphicElement>,
    actions: Vec<(Simplex, geometry::Shape)>,
    layout: Layout,

    /// The width and height of the diagram image in pixels.
    ///
    /// This is not the size of the diagram as it appears on the screen, since
    /// it might be zoomed by any parent component.
    dimensions: Size2D<f32>,

    /// Transform coordinates in the diagram layout to coordinates in the SVG image. In
    /// particular, the vertical direction is flipped so that diagrams are read from
    /// bottom to top.
    transform: Transform2D<f32>,
}

impl PreparedDiagram {
    fn new(diagram: DiagramN, style: RenderStyle) -> Self {
        assert!(diagram.dimension() >= 2);

        let generators = Generators::new(&diagram);
        let layout = Layout::new(diagram.clone(), 2000).unwrap();
        let complex = make_complex(&diagram);
        let graphic = GraphicElement::build(&complex, &layout, &generators);
        let actions = ActionRegion::build(&complex, &layout);

        let dimensions = layout
            .get(Boundary::Target.into(), Boundary::Target.into())
            .unwrap()
            .to_vector()
            .to_size()
            * style.scale;

        let transform = Transform2D::scale(style.scale, -style.scale)
            .then_translate((0.0, dimensions.height).into());

        let actions = actions
            .into_iter()
            .map(|action| {
                let shape = action
                    .transformed(&transform)
                    .to_shape(style.wire_thickness, style.point_radius);
                ((&action).into(), shape)
            })
            .collect();

        PreparedDiagram {
            layout,
            graphic,
            actions,
            dimensions,
            transform,
        }
    }
}

impl Component for Diagram2D {
    type Message = Message2D;
    type Properties = Props2D;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let diagram = PreparedDiagram::new(props.diagram.clone(), props.style);
        let node_ref = NodeRef::default();
        Diagram2D {
            props,
            link,
            node_ref,
            diagram,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message2D::OnClick(point) => {
                let point = self.transform_screen_to_image().transform_point(point);
                let result = self
                    .diagram
                    .actions
                    .iter()
                    .find(|(_, shape)| shape.contains_point(point, 0.01))
                    .map(|(simplex, _)| simplex.clone());
                match result {
                    Some(simplex) => self
                        .props
                        .on_select
                        .emit(simplex.into_iter().map(|(x, y)| vec![y, x]).collect()),
                    None => {}
                }
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if (self.props.diagram != props.diagram) || (self.props.style != props.style) {
            self.diagram = PreparedDiagram::new(props.diagram.clone(), props.style);
            self.props = props;
            true
        } else if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let size = self.diagram.dimensions;

        let onclick = {
            let link = self.link.clone();
            Callback::from(move |e: MouseEvent| {
                if !e.ctrl_key() {
                    let x = e.client_x() as f32;
                    let y = e.client_y() as f32;
                    link.send_message(Message2D::OnClick((x, y).into()));
                }
            })
        };

        // TODO: Do not redraw diagram when highlight changes!
        log::info!("redrawing diagram");

        html! {
            <svg
                xmlns={"http://www.w3.org/2000/svg"}
                width={size.width}
                height={size.height}
                onclick={onclick}
                ref={self.node_ref.clone()}
            >
                {self.diagram.graphic.iter().map(|e| self.view_element(e)).collect::<Html>()}
                {self.view_highlight()}
            </svg>
        }
    }
}

impl Diagram2D {
    /// Transform coordinates on the screen (such as those in `MouseEvent`s) to coordinates in the
    /// SVG image. This incorporates translation and zoom of the diagram component.
    fn transform_screen_to_image(&self) -> Transform2D<f32> {
        let rect = self
            .node_ref
            .cast::<Element>()
            .unwrap()
            .get_bounding_client_rect();

        let screen_size = Size2D::new(rect.width() as f32, rect.height() as f32);
        let image_size = self.diagram.dimensions;

        Transform2D::translation(-rect.left() as f32, -rect.top() as f32).then_scale(
            image_size.width / screen_size.width,
            image_size.height / screen_size.height,
        )
    }

    /// Creates the SVG elements for the diagram.
    fn view_element(&self, element: &GraphicElement) -> Html {
        let generator = element.generator();

        match element {
            GraphicElement::Surface(_, path) => {
                let class = SignatureStylesheet::name("generator", generator, "surface");
                let path = path_to_svg(path.transformed(&self.diagram.transform));
                html! {
                    <path d={path} class={class} stroke-width={1} />
                }
            }
            GraphicElement::Wire(_, path) => {
                let class = SignatureStylesheet::name("generator", generator, "wire");
                let path = path_to_svg(path.transformed(&self.diagram.transform));
                html! {
                    <path d={path} class={class} stroke-width={self.props.style.wire_thickness} fill="none" />
                }
            }
            GraphicElement::Point(_, point) => {
                let class = SignatureStylesheet::name("generator", generator, "point");
                let point = self.diagram.transform.transform_point(*point);
                html! {
                    <circle r={self.props.style.point_radius} cx={point.x} cy={point.y} class={class} />
                }
            }
        }
    }

    fn view_highlight(&self) -> Html {
        let highlight = match self.props.highlight {
            Some(highlight) => highlight,
            None => {
                return Default::default();
            }
        };
        
        let padding = self.props.style.scale * 0.25;

        let from = self.position(highlight.from) + Vector2D::new(padding, padding);
        let to = self.position(highlight.to) - Vector2D::new(padding, padding);

        let path = format!(
            "M {from_x} {from_y} L {from_x} {to_y} L {to_x} {to_y} L {to_x} {from_y} Z",
            from_x = from.x,
            from_y = from.y,
            to_x = to.x,
            to_y = to.y
        );

        html! {
            <path
                d={path}
                class="diagram-svg__highlight"
            />
        }
    }

    fn position(&self, point: [SliceIndex; 2]) -> Point2D<f32> {
        let point = self.diagram.layout.get(point[0], point[1]).unwrap();
        let point = self.diagram.transform.transform_point(point);
        point
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props1D {
    pub diagram: DiagramN,
    #[prop_or_default]
    pub style: RenderStyle,
    #[prop_or_default]
    pub on_select: Callback<Vec<Vec<SliceIndex>>>,
}

pub enum Message1D {}

pub struct Diagram1D {
    props: Props1D,
}

impl Component for Diagram1D {
    type Message = Message1D;
    type Properties = Props1D;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Diagram1D { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let size = self.dimensions();
        let generators: Vec<_> = self
            .props
            .diagram
            .slices()
            .map(|slice| slice.max_generator())
            .collect();

        let mut points = Vec::new();
        let mut wires = Vec::new();

        for height in 0..self.props.diagram.size() {
            wires.push(self.view_wire(
                generators[Height::Regular(height).to_int()],
                Height::Regular(height).into(),
                Height::Singular(height).into(),
            ));

            wires.push(self.view_wire(
                generators[Height::Regular(height + 1).to_int()],
                Height::Regular(height + 1).into(),
                Height::Singular(height).into(),
            ));

            points.push(self.view_point(
                generators[Height::Singular(height).to_int()],
                Height::Singular(height).into(),
            ));
        }

        wires.push(self.view_wire(
            generators[0],
            Height::Regular(0).into(),
            Boundary::Source.into(),
        ));

        wires.push(self.view_wire(
            *generators.last().unwrap(),
            Height::Regular(self.props.diagram.size()).into(),
            Boundary::Target.into(),
        ));

        html! {
            <svg
                xmlns={"http://www.w3.org/2000/svg"}
                 width={size.width}
                 height={size.height}
            >
                {wires.into_iter().collect::<Html>()}
                {points.into_iter().collect::<Html>()}
            </svg>
        }
    }
}

impl Diagram1D {
    fn dimensions(&self) -> Size2D<f32> {
        let style = &self.props.style;
        let width = f32::max(style.point_radius, style.wire_thickness) * 2.0;
        let height = (self.props.diagram.size() as f32 + 1.0) * 2.0 * style.scale;
        Size2D::new(width, height)
    }

    fn to_y(&self, index: SliceIndex) -> f32 {
        use self::Boundary::*;
        use self::SliceIndex::*;

        let scale = self.props.style.scale;
        let size = self.dimensions();

        match index {
            Boundary(Source) => size.height,
            Boundary(Target) => 0.0,
            Interior(height) => size.height - (height.to_int() as f32 + 1.0) * scale,
        }
    }

    fn view_wire(&self, generator: Generator, from: SliceIndex, to: SliceIndex) -> Html {
        let path = format!(
            "M {x} {from} L {x} {to}",
            from = self.to_y(from),
            to = self.to_y(to),
            x = self.dimensions().width * 0.5
        );
        let class = SignatureStylesheet::name("generator", generator, "wire");
        let style = &self.props.style;

        let onselect = self.props.on_select.clone();
        let onclick = Callback::from(move |e: MouseEvent| {
            if !e.ctrl_key() {
                onselect.emit(vec![vec![from], vec![to]]);
            }
        });

        html! {
            <path
                d={path}
                class={class}
                stroke-width={style.wire_thickness}
                fill="none"
                onclick={onclick}
            />
        }
    }

    fn view_point(&self, generator: Generator, point: SliceIndex) -> Html {
        let class = SignatureStylesheet::name("generator", generator, "point");
        let style = &self.props.style;

        let onselect = self.props.on_select.clone();
        let onclick = Callback::from(move |e: MouseEvent| {
            if !e.ctrl_key() {
                onselect.emit(vec![vec![point]]);
            }
        });

        html! {
            <circle
                cx={self.dimensions().width * 0.5}
                cy={self.to_y(point)}
                r={style.point_radius}
                class={class}
                onclick={onclick}
            />
        }
    }
}

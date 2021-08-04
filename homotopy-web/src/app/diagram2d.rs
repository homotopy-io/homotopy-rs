use std::convert::{From, Into, TryInto};
use std::f32::consts::PI;

use web_sys::Element;

use yew::prelude::*;

use euclid::default::{Point2D, Size2D, Transform2D, Vector2D};
use euclid::Angle;

use homotopy_core::common::Direction;
use homotopy_core::complex::{make_complex, Simplex};
use homotopy_core::contraction::Bias;
use homotopy_core::projection::{Depths, Generators};
use homotopy_core::rewrite::RewriteN;
use homotopy_core::{Boundary, DiagramN, Generator, Height, SliceIndex};

use homotopy_graphics::geometry;
use homotopy_graphics::geometry::path_to_svg;
use homotopy_graphics::graphic2d::{ActionRegion, GraphicElement};
use homotopy_graphics::layout2d::Layout;

use crate::app::signature_stylesheet::SignatureStylesheet;
use crate::model::proof::homotopy::{Contract, Expand, Homotopy};
use crate::model::proof::RenderStyle;

use crate::components::{read_touch_list_abs, Finger};

pub struct Diagram2D {
    props: Props2D,
    diagram: PreparedDiagram,
    link: ComponentLink<Self>,
    node_ref: NodeRef,
    drag_start: Option<Point2D<f32>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props2D {
    pub diagram: DiagramN,
    pub id: String,
    #[prop_or_default]
    pub style: RenderStyle,
    #[prop_or_default]
    pub on_select: Callback<Vec<Vec<SliceIndex>>>,
    #[prop_or_default]
    pub on_homotopy: Callback<Homotopy>,
    #[prop_or_default]
    pub highlight: Option<Highlight2D>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HighlightKind {
    Attach,
    Slice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Highlight2D {
    pub from: [SliceIndex; 2],
    pub to: [SliceIndex; 2],
    pub kind: HighlightKind,
}

// TODO: Drag callbacks in props
// TODO: Highlights in props

#[allow(clippy::enum_variant_names)]
pub enum Message2D {
    OnMouseDown(Point2D<f32>),
    OnMouseMove(Point2D<f32>),
    OnMouseUp,
    OnTouchUpdate(Vec<(Finger, Point2D<f32>)>),
    OnTouchMove(Vec<(Finger, Point2D<f32>)>),
}

/// The computed properties of a diagram that are potentially expensive to compute but can be
/// cached if the diagram does not change.
struct PreparedDiagram {
    graphic: Vec<GraphicElement>,
    actions: Vec<(Simplex, geometry::Shape)>,
    depths: Depths,
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
    fn new(diagram: &DiagramN, style: RenderStyle) -> Self {
        assert!(diagram.dimension() >= 2);

        let generators = Generators::new(diagram);
        let layout = Layout::new(diagram, 2000).unwrap();
        let complex = make_complex(diagram);
        let depths = Depths::new(diagram);
        let graphic = GraphicElement::build(&complex, &layout, &generators, &depths);
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

        let depths = Depths::new(diagram);

        Self {
            graphic,
            actions,
            depths,
            layout,
            dimensions,
            transform,
        }
    }
}

impl Component for Diagram2D {
    type Message = Message2D;
    type Properties = Props2D;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let diagram = PreparedDiagram::new(&props.diagram, props.style);
        let node_ref = NodeRef::default();
        let drag_start = Default::default();
        Self {
            props,
            diagram,
            link,
            node_ref,
            drag_start,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message2D::OnMouseDown(point) => {
                self.drag_start = Some(point);
                false
            }
            Message2D::OnMouseMove(point) => {
                self.pointer_move(point);
                false
            }
            Message2D::OnMouseUp => {
                self.pointer_stop();
                false
            }
            Message2D::OnTouchUpdate(touches) => {
                if self.drag_start.is_none() && touches.len() == 1 {
                    self.drag_start = Some(touches[0].1);
                } else if touches.is_empty() {
                    self.drag_start = None;
                }
                false
            }
            Message2D::OnTouchMove(touches) => {
                if touches.len() == 1 {
                    self.pointer_move(touches[0].1);
                }
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if (self.props.diagram != props.diagram) || (self.props.style != props.style) {
            self.diagram = PreparedDiagram::new(&props.diagram, props.style);
            self.props = props;
            true
        } else if self.props == props {
            false
        } else {
            self.props = props;
            true
        }
    }

    fn view(&self) -> Html {
        let size = self.diagram.dimensions;

        let on_mouse_down = {
            let link = self.link.clone();
            Callback::from(move |e: MouseEvent| {
                if !e.alt_key() {
                    let x = e.client_x() as f32;
                    let y = e.client_y() as f32;
                    link.send_message(Message2D::OnMouseDown((x, y).into()));
                }
            })
        };

        let on_mouse_move = {
            let link = self.link.clone();
            Callback::from(move |e: MouseEvent| {
                if !e.alt_key() {
                    let x = e.client_x() as f32;
                    let y = e.client_y() as f32;
                    link.send_message(Message2D::OnMouseMove((x, y).into()));
                }
            })
        };

        let on_mouse_up = {
            let link = self.link.clone();
            Callback::from(move |_e: MouseEvent| {
                link.send_message(Message2D::OnMouseUp);
            })
        };

        let on_touch_move = {
            let link = self.link.clone();
            Callback::from(move |e: TouchEvent| {
                let touches = read_touch_list_abs(&e.touches())
                    .map(|(finger, point)| (finger, point.cast()))
                    .collect();
                link.send_message(Message2D::OnTouchMove(touches));
            })
        };

        let on_touch_update = {
            let link = self.link.clone();
            Callback::from(move |e: TouchEvent| {
                let touches = read_touch_list_abs(&e.touches())
                    .map(|(finger, point)| (finger, point.cast()))
                    .collect();
                link.send_message(Message2D::OnTouchUpdate(touches));
            })
        };

        // TODO: Do not redraw diagram when highlight changes!
        // TODO: Do not redraw diagram for drags.

        log::info!("redrawing diagram");

        html! {
            <svg
                xmlns={"http://www.w3.org/2000/svg"}
                width={size.width.to_string()}
                height={size.height.to_string()}
                onmousedown={on_mouse_down}
                onmouseup={on_mouse_up}
                onmousemove={on_mouse_move}
                ontouchmove={on_touch_move}
                ontouchstart={on_touch_update}
                ontouchend={on_touch_update.clone()}
                ontouchcancel={on_touch_update.clone()}
                ref={self.node_ref.clone()}
            >
                {self.diagram.graphic.iter().enumerate().map(|(i, e)| self.view_element(i, e)).collect::<Html>()}
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
    fn view_element(&self, index: usize, element: &GraphicElement) -> Html {
        let generator = element.generator();

        match element {
            GraphicElement::Surface(_, path) => {
                let class = SignatureStylesheet::name("generator", generator, "surface");
                let path = path_to_svg(&path.clone().transformed(&self.diagram.transform));
                html! {
                    <path d={path} class={class} stroke-width={1} />
                }
            }
            GraphicElement::Wire(_, path, mask) => {
                let class = SignatureStylesheet::name("generator", generator, "wire");
                let path = path_to_svg(&path.clone().transformed(&self.diagram.transform));

                if mask.is_empty() {
                    html! {
                        <path
                            d={path}
                            class={class}
                            stroke-width={self.props.style.wire_thickness.to_string()}
                            fill="none"
                        />
                    }
                } else {
                    let mask_paths: Html = mask
                        .iter()
                        .map(|mask_path| {
                            html! {
                                <path
                                    d={path_to_svg(&mask_path.clone().transformed(&self.diagram.transform))}
                                    stroke-width={(self.props.style.wire_thickness * 2.0).to_string()}
                                    fill="none"
                                    stroke="black"
                                    stroke-linecap="round"
                                />
                            }
                        })
                        .collect();

                    let mask_id = format!("{}-mask-{}", self.props.id, index);

                    html! {
                        <>
                            <defs>
                                <mask maskUnits="userSpaceOnUse" id={mask_id.clone()}>
                                    <rect width="100%" height="100%" fill="white" />
                                    {mask_paths}
                                </mask>
                            </defs>
                            <path
                                d={path}
                                class={class}
                                stroke-width={self.props.style.wire_thickness.to_string()}
                                fill="none"
                                mask={format!("url(#{})", mask_id)}
                            />
                        </>
                    }
                }
            }
            GraphicElement::Point(_, point) => {
                let class = SignatureStylesheet::name("generator", generator, "point");
                let point = self.diagram.transform.transform_point(*point);
                html! {
                    <circle r={self.props.style.point_radius.to_string()} cx={point.x.to_string()} cy={point.y.to_string()} class={class} />
                }
            }
        }
    }

    fn view_highlight(&self) -> Html {
        let highlight = if let Some(highlight) = self.props.highlight {
            highlight
        } else {
            return Default::default();
        };

        let padding = match highlight.kind {
            HighlightKind::Attach => {
                let padding = self.props.style.scale * 0.25;
                Vector2D::new(padding, padding)
            }
            HighlightKind::Slice => Vector2D::new(0.0, self.props.style.scale * 0.5),
        };

        let from = self.position(highlight.from) + padding;
        let to = self.position(highlight.to) - padding;

        let path = format!(
            "M {from_x} {from_y} L {from_x} {to_y} L {to_x} {to_y} L {to_x} {from_y} Z",
            from_x = from.x,
            from_y = from.y,
            to_x = to.x,
            to_y = to.y
        );

        let class = match highlight.kind {
            HighlightKind::Attach => "diagram-svg__attach-highlight",
            HighlightKind::Slice => "diagram-svg__slice-highlight",
        };

        html! {
            <path d={path} class={class}/>
        }
    }

    fn position(&self, point: [SliceIndex; 2]) -> Point2D<f32> {
        let point = self.diagram.layout.get(point[0], point[1]).unwrap();
        self.diagram.transform.transform_point(point)
    }

    fn simplex_at(&self, point: Point2D<f32>) -> Option<Simplex> {
        let point = self.transform_screen_to_image().transform_point(point);
        self.diagram
            .actions
            .iter()
            .find(|(_, shape)| shape.contains_point(point, 0.01))
            .map(|(simplex, _)| simplex.clone())
    }

    fn pointer_move(&mut self, point: Point2D<f32>) {
        if let Some(start) = self.drag_start {
            let diff: Vector2D<f32> = point - start;
            let distance = self.props.style.scale * 0.5;

            if diff.square_length() < distance * distance {
                return;
            }

            let angle = diff.angle_from_x_axis();
            self.drag_start = None;

            let simplex = match self.simplex_at(start) {
                Some(simplex) => simplex,
                None => return,
            };

            let homotopy = drag_to_homotopy(
                angle,
                &simplex,
                self.props.diagram.clone(),
                &self.diagram.depths,
            );

            if let Some(homotopy) = homotopy {
                log::info!("Homotopy: {:?}", homotopy);
                self.props.on_homotopy.emit(homotopy);
            } else {
                log::info!("No homotopy");
            }
        }
    }

    fn pointer_stop(&mut self) {
        // If the mouse button is released without having travelled a distance great enough
        // to indicate a drag, it should be interpreted as a click.  This is preferrable to
        // a separate onclick handler since drags aren't interpreted as clicks anymore.
        if let Some(point) = self.drag_start {
            self.drag_start = None;
            if let Some(simplex) = self.simplex_at(point) {
                self.props
                    .on_select
                    .emit(simplex.into_iter().map(|(x, y)| vec![y, x]).collect());
            }
        }
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
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props == props {
            false
        } else {
            self.props = props;
            true
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
                 width={size.width.to_string()}
                 height={size.height.to_string()}
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
        use self::Boundary::{Source, Target};
        use self::SliceIndex::{Boundary, Interior};

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
                stroke-width={style.wire_thickness.to_string()}
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
                cx={(self.dimensions().width * 0.5).to_string()}
                cy={self.to_y(point).to_string()}
                r={style.point_radius.to_string()}
                class={class}
                onclick={onclick}
            />
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props0D {
    pub diagram: Generator,
    #[prop_or_default]
    pub style: RenderStyle,
}

pub enum Message0D {}

pub struct Diagram0D {
    props: Props0D,
}

impl Component for Diagram0D {
    type Message = Message0D;
    type Properties = Props0D;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props == props {
            false
        } else {
            self.props = props;
            true
        }
    }

    fn view(&self) -> Html {
        let size = self.dimensions();

        html! {
            <svg
                xmlns={"http://www.w3.org/2000/svg"}
                 width={size.width.to_string()}
                 height={size.height.to_string()}
            >
                {self.view_point(self.props.diagram)}
            </svg>
        }
    }
}

impl Diagram0D {
    const RADIUS_SCALE: f32 = 3.0;

    fn dimensions(&self) -> Size2D<f32> {
        let style = &self.props.style;
        let dimension = style.point_radius * 2.0 * Self::RADIUS_SCALE;
        Size2D::new(dimension, dimension)
    }

    fn view_point(&self, generator: Generator) -> Html {
        let class = SignatureStylesheet::name("generator", generator, "point");
        let style = &self.props.style;

        html! {
            <circle
                cx={(self.dimensions().width * 0.5).to_string()}
                cy={(self.dimensions().height * 0.5).to_string()}
                r={(style.point_radius * Self::RADIUS_SCALE).to_string()}
                class={class}
            />
        }
    }
}

fn drag_to_homotopy(
    angle: Angle<f32>,
    simplex: &Simplex,
    diagram: DiagramN,
    depths: &Depths,
) -> Option<Homotopy> {
    use Height::{Regular, Singular};
    use SliceIndex::{Boundary, Interior};

    let abs_radians = angle.radians.abs();
    let horizontal = !(PI / 4.0..(3.0 * PI) / 4.0).contains(&abs_radians);

    let (point, boundary) = match simplex {
        Simplex::Surface([p0, _, _]) => (p0, false),
        Simplex::Wire([_, p1 @ (_, Boundary(_))]) => (p1, true),
        Simplex::Wire([p0, _]) => (p0, false),
        Simplex::Point([p0]) => (p0, false),
    };

    // Handle horizontal and vertical drags
    let (prefix, y, x, diagram) = if horizontal || boundary {
        let depth = match point.0 {
            Interior(Singular(_)) => Height::Singular(depths.node_depth([point.1, point.0])?),
            _ => return None,
        };

        let diagram: DiagramN = diagram.slice(point.1)?.try_into().ok()?;
        (Some(point.1), point.0, depth.into(), diagram)
    } else {
        (None, point.1, point.0, diagram)
    };

    // TODO: Are there valid homotopies on boundary coordinates?
    let y = match y {
        Interior(y) => y,
        Boundary(_) => return None,
    };

    let x = match x {
        Interior(y) => y,
        Boundary(_) => return None,
    };

    // Decide if the drag is an expansion or a contraction
    let expansion = match y {
        Regular(_) => true,
        Singular(height) => {
            let cospan = &diagram.cospans()[height];
            let forward: &RewriteN = (&cospan.forward).try_into().unwrap();
            let backward: &RewriteN = (&cospan.backward).try_into().unwrap();

            // TODO: This should probably be a method on Cospan.
            let mut targets: Vec<_> = forward.targets();
            targets.extend(backward.targets());
            targets.sort_unstable();
            targets.dedup();
            targets.len() > 1
        }
    };

    let direction = if horizontal || boundary {
        if (-0.5 * PI..0.5 * PI).contains(&angle.radians) {
            Direction::Forward
        } else {
            Direction::Backward
        }
    } else if angle.radians <= 0.0 {
        Direction::Forward
    } else {
        Direction::Backward
    };

    if expansion {
        let mut location: Vec<_> = prefix.into_iter().collect();
        location.push(y.into());
        location.push(x.into());

        Some(Homotopy::Expand(Expand {
            location,
            direction,
        }))
    } else {
        let bias = if horizontal || boundary || abs_radians >= PI / 2.0 {
            Bias::Lower
        } else {
            Bias::Higher
        };

        let bias = Some(bias);

        let height = match y {
            Regular(_) => unreachable!(),
            Singular(height) => height,
        };

        Some(Homotopy::Contract(Contract {
            bias,
            location: prefix.into_iter().collect(),
            height,
            direction,
        }))
    }
}

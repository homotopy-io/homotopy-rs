use std::{
    convert::{From, Into, TryInto},
    f32::consts::PI,
};

use euclid::{
    default::{Point2D, Size2D, Transform2D, Vector2D},
    Angle,
};
use homotopy_core::{
    common::Direction,
    complex::{make_complex, Simplex},
    contraction::Bias,
    layout::Layout,
    projection::{Depths, Projection},
    rewrite::RewriteN,
    Boundary, Diagram, DiagramN, Height, SliceIndex,
};
use homotopy_graphics::{
    style::VertexShape,
    svg::{
        generator_class,
        render::{ActionRegion, GraphicElement},
        shape::{path_to_svg, project_2d, Point, Shape},
    },
};
use web_sys::Element;
use yew::prelude::*;

use crate::{
    components::{read_touch_list_abs, Finger},
    model::proof::{
        homotopy::{Contract, Expand, Homotopy},
        RenderStyle, Signature,
    },
};

pub struct DiagramSvg<const N: usize> {
    props: DiagramSvgProps<N>,
    prepared: PreparedDiagram<N>,
    node_ref: NodeRef,
    drag_start: Option<Point2D<f32>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct DiagramSvgProps<const N: usize> {
    pub diagram: Diagram,
    pub id: String,
    pub signature: Signature,
    #[prop_or_default]
    pub style: RenderStyle,
    #[prop_or_default]
    pub on_select: Callback<Vec<Vec<SliceIndex>>>,
    #[prop_or_default]
    pub on_homotopy: Callback<Homotopy>,
    #[prop_or_default]
    pub highlight: Option<HighlightSvg<N>>,
    #[prop_or_default]
    pub max_width: Option<f32>,
    #[prop_or_default]
    pub max_height: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HighlightKind {
    Attach,
    Slice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HighlightSvg<const N: usize> {
    pub from: [SliceIndex; N],
    pub to: [SliceIndex; N],
    pub kind: HighlightKind,
}

// TODO: Drag callbacks in props
// TODO: Highlights in props

#[allow(clippy::enum_variant_names)]
pub enum DiagramSvgMessage {
    OnMouseDown(Point2D<f32>),
    OnMouseMove(Point2D<f32>),
    OnMouseUp,
    OnTouchUpdate(Vec<(Finger, Point2D<f32>)>),
    OnTouchMove(Vec<(Finger, Point2D<f32>)>),
}

/// The computed properties of a diagram that are potentially expensive to compute but can be
/// cached if the diagram does not change.
struct PreparedDiagram<const N: usize> {
    graphic: Vec<GraphicElement<N>>,
    actions: Vec<(Simplex<N>, Shape)>,
    depths: Depths<N>,
    layout: Layout<N>,

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

impl<const N: usize> PreparedDiagram<N> {
    fn new(diagram: &Diagram, style: RenderStyle) -> Self {
        assert!(diagram.dimension() >= N);

        let performance = web_sys::window().unwrap().performance().unwrap();
        performance.mark("startPrepareDiagram").unwrap();

        let layout = Layout::new(diagram).unwrap();
        let complex = make_complex(diagram);
        let depths = Depths::new(diagram).unwrap();
        let projection = Projection::new(diagram, &layout, &depths).unwrap();
        let graphic = GraphicElement::build(diagram, &complex, &layout, &projection, &depths);
        let actions = ActionRegion::build(&complex, &layout, &projection);

        let dimensions = Point::from(project_2d(layout.get([Boundary::Target.into(); N])))
            .max((1.0, 1.0).into())
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

        performance.mark("stopPrepareDiagram").unwrap();
        performance
            .measure_with_start_mark_and_end_mark(
                "prepareDiagram",
                "startPrepareDiagram",
                "stopPrepareDiagram",
            )
            .unwrap();
        log::info!(
            "preparing diagram for rendering took {}ms",
            js_sys::Reflect::get(
                &performance
                    .get_entries_by_name_with_entry_type("prepareDiagram", "measure")
                    .get(0),
                &wasm_bindgen::JsValue::from_str("duration")
            )
            .unwrap()
            .as_f64()
            .unwrap()
        );

        performance.clear_marks();
        performance.clear_measures();

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

impl<const N: usize> Component for DiagramSvg<N> {
    type Message = DiagramSvgMessage;
    type Properties = DiagramSvgProps<N>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props().clone();
        let prepared = PreparedDiagram::new(&props.diagram, props.style);
        let node_ref = NodeRef::default();
        let drag_start = Default::default();
        Self {
            props,
            prepared,
            node_ref,
            drag_start,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DiagramSvgMessage::OnMouseDown(point) => {
                self.drag_start = Some(point);
                false
            }
            DiagramSvgMessage::OnMouseMove(point) => {
                self.pointer_move(ctx, point);
                false
            }
            DiagramSvgMessage::OnMouseUp => {
                self.pointer_stop(ctx);
                false
            }
            DiagramSvgMessage::OnTouchUpdate(touches) => {
                if self.drag_start.is_none() && touches.len() == 1 {
                    self.drag_start = Some(touches[0].1);
                } else if touches.is_empty() {
                    self.drag_start = None;
                }
                false
            }
            DiagramSvgMessage::OnTouchMove(touches) => {
                if touches.len() == 1 {
                    self.pointer_move(ctx, touches[0].1);
                }
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        // self.props contains the old props
        if &self.props == ctx.props() {
            false
        } else {
            if self.props.diagram != ctx.props().diagram || self.props.style != ctx.props().style {
                // re-layout
                self.prepared = PreparedDiagram::new(&ctx.props().diagram, ctx.props().style);
            }
            self.props = ctx.props().clone();
            true
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let size = self.prepared.dimensions;

        let width = match self.props.max_width {
            Some(width) => width.min(size.width),
            None => size.width,
        };

        let height = match self.props.max_height {
            Some(height) => height.min(size.height),
            None => size.height,
        };

        let on_mouse_down = {
            let link = ctx.link().clone();
            Callback::from(move |e: MouseEvent| {
                if !e.alt_key() {
                    let x = e.client_x() as f32;
                    let y = e.client_y() as f32;
                    link.send_message(DiagramSvgMessage::OnMouseDown((x, y).into()));
                }
            })
        };

        let on_mouse_move = {
            let link = ctx.link().clone();
            Callback::from(move |e: MouseEvent| {
                if !e.alt_key() {
                    let x = e.client_x() as f32;
                    let y = e.client_y() as f32;
                    link.send_message(DiagramSvgMessage::OnMouseMove((x, y).into()));
                }
            })
        };

        let on_mouse_up = {
            let link = ctx.link().clone();
            Callback::from(move |_e: MouseEvent| {
                link.send_message(DiagramSvgMessage::OnMouseUp);
            })
        };

        let on_touch_move = {
            let link = ctx.link().clone();
            Callback::from(move |e: TouchEvent| {
                let touches = read_touch_list_abs(&e.touches())
                    .map(|(finger, point)| (finger, point.cast()))
                    .collect();
                link.send_message(DiagramSvgMessage::OnTouchMove(touches));
            })
        };

        let on_touch_update = {
            let link = ctx.link().clone();
            Callback::from(move |e: TouchEvent| {
                let touches = read_touch_list_abs(&e.touches())
                    .map(|(finger, point)| (finger, point.cast()))
                    .collect();
                link.send_message(DiagramSvgMessage::OnTouchUpdate(touches));
            })
        };

        // TODO: Do not redraw diagram when highlight changes!
        // TODO: Do not redraw diagram for drags.

        log::info!("redrawing diagram");

        html! {
            <svg
                xmlns={"http://www.w3.org/2000/svg"}
                width={width.to_string()}
                height={height.to_string()}
                viewBox={format!("0 0 {} {}", size.width, size.height)}
                onmousedown={on_mouse_down}
                onmouseup={on_mouse_up}
                onmousemove={on_mouse_move}
                ontouchmove={on_touch_move}
                ontouchstart={on_touch_update}
                ontouchend={on_touch_update.clone()}
                ontouchcancel={on_touch_update.clone()}
                ref={self.node_ref.clone()}
            >
                {self.prepared.graphic.iter().enumerate().map(|(i, e)| self.view_element(ctx, i, e)).collect::<Html>()}
                {self.view_highlight(ctx)}
            </svg>
        }
    }
}

impl<const N: usize> DiagramSvg<N> {
    /// Transform coordinates on the screen (such as those in `MouseEvent`s) to coordinates in the
    /// SVG image. This incorporates translation and zoom of the diagram component.
    fn transform_screen_to_image(&self) -> Transform2D<f32> {
        let rect = self
            .node_ref
            .cast::<Element>()
            .unwrap()
            .get_bounding_client_rect();

        let screen_size = Size2D::new(rect.width() as f32, rect.height() as f32);
        let image_size = self.prepared.dimensions;

        Transform2D::translation(-rect.left() as f32, -rect.top() as f32).then_scale(
            image_size.width / screen_size.width,
            image_size.height / screen_size.height,
        )
    }

    /// Creates the SVG elements for the diagram.
    fn view_element(&self, ctx: &Context<Self>, index: usize, element: &GraphicElement<N>) -> Html {
        let generator = element.generator();

        match element {
            GraphicElement::Surface(_, path) => {
                let class = generator_class(generator, "surface");
                let path = path_to_svg(&path.clone().transformed(&self.prepared.transform));
                html! {
                    <path d={path} class={class} stroke-width={1} />
                }
            }
            GraphicElement::Wire(_, _, path, mask) => {
                let class = generator_class(generator, "wire");
                let path = path_to_svg(&path.clone().transformed(&self.prepared.transform));

                if mask.is_empty() {
                    html! {
                        <path
                            d={path}
                            class={class}
                            stroke-width={ctx.props().style.wire_thickness.to_string()}
                            fill="none"
                        />
                    }
                } else {
                    let mask_paths: Html = mask
                        .iter()
                        .map(|mask_path| {
                            html! {
                                <path
                                    d={path_to_svg(&mask_path.clone().transformed(&self.prepared.transform))}
                                    stroke-width={(ctx.props().style.wire_thickness * 2.0).to_string()}
                                    fill="none"
                                    stroke="black"
                                    stroke-linecap="round"
                                />
                            }
                        })
                        .collect();

                    let mask_id = format!("{}-mask-{}", ctx.props().id, index);

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
                                stroke-width={ctx.props().style.wire_thickness.to_string()}
                                fill="none"
                                mask={format!("url(#{})", mask_id)}
                            />
                        </>
                    }
                }
            }
            GraphicElement::Point(_, point) => {
                use VertexShape::{Circle, Square};
                let class = generator_class(generator, "point");
                let point = self.prepared.transform.transform_point(*point);
                let radius = ctx.props().style.point_radius;
                let shape = if let Some(info) = ctx.props().signature.generator_info(generator) {
                    info.shape.clone()
                } else {
                    Default::default()
                };
                match shape {
                    Circle => html! {
                        <circle
                            r={radius.to_string()}
                            cx={point.x.to_string()}
                            cy={point.y.to_string()}
                            class={class} />
                    },
                    Square => html! {
                        <rect
                            x={(point.x - radius).to_string()}
                            y={(point.y - radius).to_string()}
                            width={(radius * 2.0).to_string()}
                            height={(radius * 2.0).to_string()}
                            class={class} />
                    },
                }
            }
        }
    }

    fn view_highlight(&self, ctx: &Context<Self>) -> Html {
        let highlight = if let Some(highlight) = ctx.props().highlight {
            highlight
        } else {
            return Default::default();
        };

        let padding = match highlight.kind {
            HighlightKind::Attach => {
                let padding = ctx.props().style.scale * 0.25;
                Vector2D::new(padding, padding)
            }
            HighlightKind::Slice => Vector2D::new(0.0, ctx.props().style.scale * 0.5),
        };

        let from = self.position(highlight.from) + padding;
        let to = self.position(highlight.to) - padding;

        let path = format!(
            "M {from_x} {from_y} L {from_x} {to_y} L {to_x} {to_y} L {to_x} {from_y} Z",
            from_x = if N == 1 { 0.0 } else { from.x },
            from_y = from.y,
            to_x = if N == 1 {
                ctx.props().style.scale
            } else {
                to.x
            },
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

    fn position(&self, point: [SliceIndex; N]) -> Point2D<f32> {
        let point = project_2d(self.prepared.layout.get(point)).into();
        self.prepared.transform.transform_point(point)
    }

    fn simplex_at(&self, point: Point2D<f32>) -> Option<Simplex<N>> {
        let point = self.transform_screen_to_image().transform_point(point);
        self.prepared
            .actions
            .iter()
            .find(|(_, shape)| shape.contains_point(point, 0.01))
            .map(|(simplex, _)| simplex.clone())
    }

    fn pointer_move(&mut self, ctx: &Context<Self>, point: Point2D<f32>) {
        if let Some(start) = self.drag_start {
            let diff: Vector2D<f32> = point - start;
            let distance = ctx.props().style.scale * 0.5;

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
                ctx.props().diagram.clone(),
                &self.prepared.depths,
            );

            if let Some(homotopy) = homotopy {
                log::info!("Homotopy: {:?}", homotopy);
                ctx.props().on_homotopy.emit(homotopy);
            } else {
                log::info!("No homotopy");
            }
        }
    }

    fn pointer_stop(&mut self, ctx: &Context<Self>) {
        // If the mouse button is released without having travelled a distance great enough
        // to indicate a drag, it should be interpreted as a click.  This is preferrable to
        // a separate onclick handler since drags aren't interpreted as clicks anymore.
        if let Some(point) = self.drag_start {
            self.drag_start = None;
            if let Some(simplex) = self.simplex_at(point) {
                ctx.props()
                    .on_select
                    .emit(simplex.into_iter().map(|p| p.to_vec()).collect());
            }
        }
    }
}

fn drag_to_homotopy<const N: usize>(
    angle: Angle<f32>,
    simplex: &Simplex<N>,
    diagram: Diagram,
    depths: &Depths<N>,
) -> Option<Homotopy> {
    use Height::{Regular, Singular};
    use SliceIndex::{Boundary, Interior};

    let abs_radians = angle.radians.abs();
    let horizontal = !(PI / 4.0..(3.0 * PI) / 4.0).contains(&abs_radians);

    let (point, boundary) = match simplex {
        Simplex::Surface([p0, _, _]) => (p0, false),
        Simplex::Wire([_, p1]) if matches!(p1[0], Boundary(_)) => (p1, true),
        Simplex::Wire([p0, _]) => (p0, false),
        Simplex::Point([p0]) => (p0, false),
    };

    match N {
        1 => {
            let height = match point[0] {
                Boundary(_) => return None,
                Interior(height) => height,
            };

            let direction = if angle.radians <= 0.0 {
                Direction::Forward
            } else {
                Direction::Backward
            };

            Some(match height {
                Regular(_) => Homotopy::Expand(Expand {
                    location: point.to_vec(),
                    direction,
                }),
                Singular(i) => Homotopy::Contract(Contract {
                    bias: None,
                    location: Default::default(),
                    height: i,
                    direction,
                }),
            })
        }
        2 => {
            let diagram: DiagramN = diagram.try_into().ok()?;

            // Handle horizontal and vertical drags
            log::debug!("Point: {:?}", point);
            let (prefix, y, x, diagram) = if horizontal || boundary {
                let depth = match point[1] {
                    Interior(Singular(_)) =>
                    /* TODO: cancellation moves by invertibility */
                    {
                        Height::Singular(depths.node_depth(*point).unwrap_or_default())
                    }
                    _ => return None,
                };

                let diagram: DiagramN = diagram.slice(point[0])?.try_into().ok()?;
                (Some(point[0]), point[1], depth.into(), diagram)
            } else {
                (None, point[0], point[1], diagram)
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
                    if diagram.dimension() == 1 {
                        false
                    } else {
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
                let bias = if horizontal || boundary || abs_radians >= 2.0 * PI / 3.0 {
                    Some(Bias::Lower)
                } else if abs_radians <= PI / 3.0 {
                    Some(Bias::Higher)
                } else {
                    None
                };

                let bias = bias;

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
        _ => unreachable!(),
    }
}

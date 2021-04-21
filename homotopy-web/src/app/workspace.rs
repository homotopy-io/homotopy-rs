mod path_control;
mod slice_control;

use crate::app::diagram2d::{Diagram1D, Diagram2D, Highlight2D};
use crate::app::panzoom;
use crate::model::proof::homotopy::Homotopy;
use crate::model::proof::{Action, Signature, Workspace};
use homotopy_core::{
    attach::BoundaryPath,
    common::{Boundary, Height, SliceIndex},
};
use homotopy_core::{Diagram, DiagramN};

use path_control::PathControl;
use slice_control::SliceControl;
use std::convert::{Into, TryFrom, TryInto};
use yew::prelude::*;

// TODO: Workspace rerendering when panzoom is changed needs to be smoother.

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub workspace: Workspace,
    pub dispatch: Callback<Action>,
    pub signature: Signature,
}

pub enum Message {
    PanZoom(panzoom::Message),
}

pub struct WorkspaceView {
    props: Props,
    panzoom: panzoom::PanZoom,
    on_select: Callback<Vec<Vec<SliceIndex>>>,
    on_homotopy: Callback<Homotopy>,
}

impl Component for WorkspaceView {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let panzoom_callback = link.callback(Message::PanZoom);
        let panzoom = panzoom::PanZoom::new(NodeRef::default(), &panzoom_callback);
        let on_select = props.dispatch.reform(Action::SelectPoints);
        let on_homotopy = props.dispatch.reform(Action::Homotopy);
        Self {
            props,
            panzoom,
            on_select,
            on_homotopy,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::PanZoom(msg) => self.panzoom.update(msg),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        // TODO: Ensure that panzoom is centered initially when the diagram
        // is changed.

        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let slice_buttons = match self.visible_diagram() {
            Diagram::Diagram0(_) => Default::default(),
            Diagram::DiagramN(d) => html! {
                <SliceControl
                    translate={self.panzoom.translate().y}
                    scale={self.panzoom.scale()}
                    number_slices={d.size()}
                    descend_slice={self.props.dispatch.reform(Action::DescendSlice)}
                />
            },
        };

        let path_control = html! {
            <PathControl
                path={self.props.workspace.path.clone()}
                ascend_slice={self.props.dispatch.reform(Action::AscendSlice)}
            />
        };

        html! {
            <content
                class="workspace"
                onmousemove={self.panzoom.on_mouse_move()}
                onmouseup={self.panzoom.on_mouse_up()}
                onmousedown={self.panzoom.on_mouse_down()}
                onwheel={self.panzoom.on_wheel()}
                ontouchmove={self.panzoom.on_touch_move()}
                ontouchstart={self.panzoom.on_touch_update()}
                ontouchend={self.panzoom.on_touch_update()}
                ontouchcancel={self.panzoom.on_touch_update()}
                ref={self.panzoom.node_ref()}
            >
                {path_control}
                {slice_buttons}
                {self.view_diagram()}
            </content>
        }
    }
}

impl WorkspaceView {
    fn visible_diagram(&self) -> Diagram {
        // TODO: This should not be recomputed every view
        let mut diagram = self.props.workspace.diagram.clone();
        for index in self.props.workspace.path.iter() {
            diagram = DiagramN::try_from(diagram).unwrap().slice(*index).unwrap();
        }
        diagram
    }

    fn view_diagram(&self) -> Html {
        match self.visible_diagram() {
            Diagram::Diagram0(_generator) => {
                html! {
                    <div>{"todo: 0-dimensional diagram"}</div>
                }
            }
            Diagram::DiagramN(diagram) if diagram.dimension() == 1 => {
                html! {
                    <div class="workspace__diagram" style={self.diagram_style()}>
                        <Diagram1D
                            diagram={diagram.clone()}
                            on_select={self.on_select.clone()}
                        />
                    </div>
                }
            }
            Diagram::DiagramN(diagram) => {
                let highlight = highlight_2d(&self.props.workspace, &self.props.signature);

                html! {
                    <div class="workspace__diagram" style={self.diagram_style()}>
                        <Diagram2D
                            diagram={diagram.clone()}
                            id="workspace__diagram"
                            on_select={self.on_select.clone()}
                            on_homotopy={self.on_homotopy.clone()}
                            highlight={highlight}
                        />
                    </div>
                }
            }
        }
    }

    fn diagram_style(&self) -> String {
        let translate = self.panzoom.translate();
        let scale = self.panzoom.scale();

        format!(
            r#"
                transform-origin: 0px 0px;
                transform: translate({x}px, {y}px) scale({s});
            "#,
            x = translate.x,
            y = translate.y,
            s = scale
        )
    }
}

fn highlight_2d(workspace: &Workspace, signature: &Signature) -> Option<Highlight2D> {
    use Height::Regular;

    let attach_option = workspace.highlight.as_ref()?;

    let info = signature.get(&attach_option.generator).unwrap();
    let needle: DiagramN = info.diagram.clone().try_into().unwrap();

    let mut boundary_path = attach_option.boundary_path.clone();
    let mut embedding = attach_option.embedding.clone();

    if let Some(BoundaryPath(boundary, depth)) = boundary_path {
        if depth >= workspace.path.len() {
            boundary_path = Some(BoundaryPath(boundary, depth - workspace.path.len()));
        } else {
            boundary_path = None;
            embedding = embedding.skip(workspace.path.len() - depth - 1);
        }
    } else {
        embedding = embedding.skip(workspace.path.len());
    }

    match boundary_path {
        None => {
            // Note: An empty boundary path implies that `needle` is one dimension
            // higher than the currently displayed diagram. Since this function
            // computes highlights for 2d diagrams, the `needle` diagram is at
            // least three-dimensional.
            let needle_s: DiagramN = needle.source().try_into().unwrap();
            let needle_st: DiagramN = needle_s.target().try_into().unwrap();

            Some(Highlight2D {
                from: [Regular(embedding[1]).into(), Regular(embedding[0]).into()],
                to: [
                    Regular(embedding[1] + needle_st.size()).into(),
                    Regular(embedding[0] + needle_s.size()).into(),
                ],
            })
        }
        Some(bp) if bp.depth() == 0 => {
            let slice: DiagramN = needle
                .slice(bp.boundary().flip())
                .unwrap()
                .try_into()
                .unwrap();
            let size = slice.size();
            Some(Highlight2D {
                from: [Regular(embedding[0]).into(), bp.boundary().into()],
                to: [Regular(embedding[0] + size).into(), bp.boundary().into()],
            })
        }
        Some(bp) => Some(Highlight2D {
            from: [bp.boundary().into(), Boundary::Source.into()],
            to: [bp.boundary().into(), Boundary::Target.into()],
        }),
    }
}

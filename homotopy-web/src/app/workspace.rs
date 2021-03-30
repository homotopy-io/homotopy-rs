use crate::app::diagram2d::{Diagram1D, Diagram2D, Highlight2D};
use crate::app::icon::Icon;
use crate::app::panzoom;
use crate::model::proof::homotopy::Homotopy;
use crate::model::proof::{Action, GeneratorInfo, Workspace};
use closure::closure;
use homotopy_core::common::{Boundary, Generator, Height, SliceIndex};
use homotopy_core::{Diagram, DiagramN};
use std::convert::{Into, TryFrom, TryInto};
use yew::prelude::*;

// TODO: Workspace rerendering when panzoom is changed needs to be smoother.

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub workspace: Workspace,
    pub dispatch: Callback<Action>,
    pub signature: im::HashMap<Generator, GeneratorInfo>,
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
                {self.view_path_controls()}
                {self.view_slice_buttons()}
                {self.view_diagram()}
            </content>
        }
    }
}

impl WorkspaceView {
    fn view_path_controls(&self) -> Html {
        let path = &self.props.workspace.path;
        let path_len = path.len();
        let dispatch = &self.props.dispatch;

        let steps: Html = path
            .iter()
            .map(|step| match step {
                SliceIndex::Boundary(Boundary::Source) => "S".to_owned(),
                SliceIndex::Boundary(Boundary::Target) => "T".to_owned(),
                SliceIndex::Interior(Height::Singular(h)) => format!("s{}", h),
                SliceIndex::Interior(Height::Regular(h)) => format!("r{}", h),
            })
            .enumerate()
            .map(closure!(clone dispatch, |(index, step)| {
                html! {
                    <span
                        class="workspace__path-step"
                        onclick={dispatch.reform(move |_| Action::AscendSlice(path_len - index - 1))}
                    >
                        {step}
                    </span>
                }
            }))
            .collect();

        let class = format!(
            "workspace__path {}",
            if path.is_empty() {
                "workspace__path--empty"
            } else {
                ""
            }
        );

        html! {
            <div class={class}>
                <span
                    class="workspace__path-home"
                    onclick={dispatch.reform(move |_| Action::AscendSlice(path_len))}
                >
                    <Icon name="menu" />
                </span>
                {steps}
            </div>
        }
    }

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
                let highlight = self.highlight_2d();
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

    fn view_slice_buttons(&self) -> Html {
        let slices = match self.visible_diagram() {
            Diagram::Diagram0(_generator) => {
                return Default::default();
            }
            Diagram::DiagramN(diagram) => diagram.size(),
        };

        let mut buttons = vec![self.view_slice_button(Boundary::Target.into())];
        buttons.extend(
            (0..(slices * 2 + 1))
                .rev()
                .map(|i| self.view_slice_button(Height::from_int(i).into())),
        );
        buttons.push(self.view_slice_button(Boundary::Source.into()));

        let buttons: Html = buttons.into_iter().collect();

        let style = format!(
            r#"
                transform-origin: 0px 0px;
                transform: translate(0px, {y}px)
            "#,
            y = self.panzoom.translate().y - (0.5 * 40.0 * self.panzoom.scale())
        );

        html! {
            <div class="workspace__slice-buttons" style={style}>
                {buttons}
            </div>
        }
    }

    fn view_slice_button(&self, index: SliceIndex) -> Html {
        let button_style = format!(
            r#"
                height: {height}px;
            "#,
            height = self.panzoom.scale() * 40.0
        );

        let label = match index {
            SliceIndex::Boundary(Boundary::Source) => "Source".to_owned(),
            SliceIndex::Boundary(Boundary::Target) => "Target".to_owned(),
            SliceIndex::Interior(Height::Regular(i)) => format!("Regular {}", i),
            SliceIndex::Interior(Height::Singular(i)) => format!("Singular {}", i),
        };

        html! {
            <div
                class="workspace__slice-button tooltip tooltip--left"
                data-tooltip={label}
                style={&button_style}
                onclick={self.props.dispatch.reform(move |_| Action::DescendSlice(index))}
            >
                <Icon name="arrow_right" />
            </div>
        }
    }

    fn highlight_2d(&self) -> Option<Highlight2D> {
        use Height::Regular;

        let option = self.props.workspace.highlight.as_ref()?;

        let info = self.props.signature.get(&option.generator).unwrap();
        let needle: DiagramN = info.diagram.clone().try_into().unwrap();
        let embedding = &option.embedding;

        match &option.boundary_path {
            None => {
                let target: DiagramN = needle.target().try_into().unwrap();
                Some(Highlight2D {
                    from: [Regular(embedding[0]).into(), Regular(embedding[1]).into()],
                    to: [
                        Regular(embedding[0] + target.size()).into(),
                        Regular(embedding[1] + needle.size()).into(),
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

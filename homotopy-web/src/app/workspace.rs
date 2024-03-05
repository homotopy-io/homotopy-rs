use homotopy_core::SliceIndex;
use path_control::PathControl;
use slice_control::SliceControl;
use view_control::ViewControl;
use yew::prelude::*;

use crate::{
    app::{
        diagram_gl::DiagramGl,
        diagram_svg::{
            highlight::{highlight_attachment, highlight_slice},
            DiagramSvg,
        },
        info::get_onboarding_message,
        tex::TexSpan,
        AppSettings,
    },
    components::panzoom::PanZoomComponent,
    model::{
        proof::{self, AttachOption, Metadata, Signature, Workspace},
        Action,
    },
};

mod path_control;
mod slice_control;
mod view_control;

// TODO: Workspace rerendering when panzoom is changed needs to be smoother.

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub workspace: Option<Workspace>,
    pub dispatch: Callback<Action>,
    pub signature: Signature,
    pub metadata: Metadata,
    pub attachment_highlight: Option<AttachOption>,
    pub slice_highlight: Option<SliceIndex>,
}

#[function_component]
pub fn WorkspaceView(props: &Props) -> Html {
    let diagram_ref = NodeRef::default();
    let workspace = &props.workspace;
    let diagram_view = if workspace.is_some() {
        view_diagram(props, diagram_ref.clone())
    } else {
        // Show onboarding info if workspace and signature is empty
        get_onboarding_message()
    };
    let project_title = html! {
        <TexSpan
            class="workspace__project-title"
            error_color="#c004"
            raw_tex={props.metadata.title.as_ref().map(Clone::clone).unwrap_or_default()}
        />
    };
    let slice_buttons = match workspace {
        Some(ref ws) if matches!(ws.view.dimension(), 1 | 2) => {
            let diagram = ws.visible_diagram();
            html! {
                <SliceControl
                    number_slices={diagram.size().unwrap()}
                    descend_slice={props.dispatch.reform(Action::Proof).reform(proof::Action::DescendSlice)}
                    diagram_ref={diagram_ref}
                    on_hover={props.dispatch.reform(Action::HighlightSlice)}
                />
            }
        }
        _ => Default::default(),
    };
    let toolbar = workspace.as_ref().map_or_else(
        Default::default,
        |ws| {
            html! {
                <div class="workspace__toolbar">
                    <PathControl
                        path={ws.path.clone()}
                        view={ws.view}
                        ascend_slice={props.dispatch.reform(Action::Proof).reform(proof::Action::AscendSlice)}
                        increase_view={props.dispatch.reform(Action::Proof).reform(proof::Action::IncreaseView)}
                        decrease_view={props.dispatch.reform(Action::Proof).reform(proof::Action::DecreaseView)}
                        dimension={ws.diagram.dimension()}
                    />
                    <ViewControl />
                </div>
            }
        },
    );

    html! {
        <div class="workspace">
            {diagram_view}
            {slice_buttons}
            <div class="workspace__overlay-top">
                {project_title}
                {toolbar}
            </div>
        </div>
    }
}

fn view_diagram(props: &Props, diagram_ref: NodeRef) -> Html {
    if let Some(ref ws) = props.workspace {
        match ws.view.dimension() {
            0 => view_diagram_svg::<0>(props, diagram_ref),
            1 => view_diagram_svg::<1>(props, diagram_ref),
            2 => view_diagram_svg::<2>(props, diagram_ref),
            _ => {
                html! {
                    <DiagramGl
                        diagram={ws.visible_diagram()}
                        signature={props.signature.clone()}
                        view={ws.view}
                    />
                }
            }
        }
    } else {
        Default::default()
    }
}

fn view_diagram_svg<const N: usize>(props: &Props, diagram_ref: NodeRef) -> Html {
    let on_select = props
        .dispatch
        .reform(|p| Action::SelectPoint(p, AppSettings::get_weak_units()));
    let on_homotopy = props
        .dispatch
        .reform(|homotopy| Action::Proof(proof::Action::Homotopy(homotopy)));

    if let Some(ref ws) = props.workspace {
        let attachment_highlight = props
            .attachment_highlight
            .as_ref()
            .map(|option| highlight_attachment::<N>(ws.path.len(), option));
        let slice_highlight = props.slice_highlight.map(highlight_slice::<N>);
        let highlight = attachment_highlight.or(slice_highlight);
        html! {
            <PanZoomComponent on_scroll={props.dispatch.reform(Action::Proof).reform(proof::Action::SwitchSlice)}>
                <DiagramSvg<N>
                    diagram={ws.visible_diagram()}
                    id="workspace__diagram"
                    signature={props.signature.clone()}
                    on_select={on_select}
                    on_homotopy={on_homotopy}
                    highlight={highlight}
                    diagram_ref={diagram_ref}
                />
            </PanZoomComponent>
        }
    } else {
        Default::default()
    }
}

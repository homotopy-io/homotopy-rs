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
        proof::{self, homotopy::Homotopy, AttachOption, Metadata, Signature, Workspace},
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

pub struct WorkspaceView {
    on_select: Callback<Vec<SliceIndex>>,
    on_homotopy: Callback<Homotopy>,
    diagram_ref: NodeRef,
}

impl Component for WorkspaceView {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let on_select = ctx
            .props()
            .dispatch
            .reform(|p| Action::SelectPoint(p, AppSettings::get_weak_units()));
        let on_homotopy = ctx
            .props()
            .dispatch
            .reform(|homotopy| Action::Proof(proof::Action::Homotopy(homotopy)));
        Self {
            on_select,
            on_homotopy,
            diagram_ref: Default::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let workspace = &ctx.props().workspace;
        let diagram_view = if workspace.is_some() {
            self.view_diagram(ctx)
        } else {
            // Show onboarding info if workspace and signature is empty
            get_onboarding_message()
        };
        let project_title = html! {
            <TexSpan
                class="workspace__project-title"
                error_color="#c004"
                raw_tex={ctx.props().metadata.title.as_ref().map(Clone::clone).unwrap_or_default()}
            />
        };
        let slice_buttons = match workspace {
            Some(ref ws) if matches!(ws.view.dimension(), 1 | 2) => {
                let diagram = ws.visible_diagram();
                html! {
                    <SliceControl
                        number_slices={diagram.size().unwrap()}
                        descend_slice={ctx.props().dispatch.reform(Action::Proof).reform(proof::Action::DescendSlice)}
                        diagram_ref={self.diagram_ref.clone()}
                        on_hover={ctx.props().dispatch.reform(Action::HighlightSlice)}
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
                            ascend_slice={ctx.props().dispatch.reform(Action::Proof).reform(proof::Action::AscendSlice)}
                            increase_view={ctx.props().dispatch.reform(Action::Proof).reform(proof::Action::IncreaseView)}
                            decrease_view={ctx.props().dispatch.reform(Action::Proof).reform(proof::Action::DecreaseView)}
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
}

impl WorkspaceView {
    fn view_diagram(&self, ctx: &Context<Self>) -> Html {
        if let Some(ref ws) = ctx.props().workspace {
            match ws.view.dimension() {
                0 => Self::view_diagram_svg::<0>(self, ctx),
                1 => Self::view_diagram_svg::<1>(self, ctx),
                2 => Self::view_diagram_svg::<2>(self, ctx),
                _ => {
                    html! {
                        <DiagramGl
                            diagram={ws.visible_diagram()}
                            signature={ctx.props().signature.clone()}
                            view={ws.view}
                        />
                    }
                }
            }
        } else {
            Default::default()
        }
    }

    #[allow(clippy::let_underscore_untyped)]
    fn view_diagram_svg<const N: usize>(&self, ctx: &Context<Self>) -> Html {
        if let Some(ref ws) = ctx.props().workspace {
            let attachment_highlight = ctx
                .props()
                .attachment_highlight
                .as_ref()
                .map(|option| highlight_attachment::<N>(ws.path.len(), option));
            let slice_highlight = ctx.props().slice_highlight.map(highlight_slice::<N>);
            let highlight = attachment_highlight.or(slice_highlight);
            html! {
                <PanZoomComponent on_scroll={ctx.props().dispatch.reform(Action::Proof).reform(proof::Action::SwitchSlice)}>
                    <DiagramSvg<N>
                        diagram={ws.visible_diagram()}
                        id="workspace__diagram"
                        signature={ctx.props().signature.clone()}
                        on_select={self.on_select.clone()}
                        on_homotopy={self.on_homotopy.clone()}
                        highlight={highlight}
                        diagram_ref={self.diagram_ref.clone()}
                    />
                </PanZoomComponent>
            }
        } else {
            Default::default()
        }
    }
}

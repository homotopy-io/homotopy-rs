use std::convert::{Into, TryInto};

use homotopy_core::{
    common::{Boundary, Height, SliceIndex},
    DiagramN,
};
use path_control::PathControl;
use slice_control::SliceControl;
use view_control::ViewControl;
use yew::prelude::*;

use crate::{
    app::{
        diagram_gl::DiagramGl,
        diagram_svg::{DiagramSvg, HighlightKind, HighlightSvg},
        tex::TexSpan,
    },
    components::panzoom::PanZoomComponent,
    model::proof::{homotopy::Homotopy, Action, Metadata, Signature, Workspace},
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
}

pub enum Message {}

pub struct WorkspaceView {
    on_select: Callback<Vec<Vec<SliceIndex>>>,
    on_homotopy: Callback<Homotopy>,
    diagram_ref: NodeRef,
}

impl Component for WorkspaceView {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let on_select = ctx.props().dispatch.reform(Action::SelectPoints);
        let on_homotopy = ctx.props().dispatch.reform(Action::Homotopy);
        Self {
            on_select,
            on_homotopy,
            diagram_ref: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let workspace = &ctx.props().workspace;
        let diagram_view = if workspace.is_some() {
            self.view_diagram(ctx)
        } else {
            // TODO: Show onboarding info if workspace and signature is empty
            html! {
                <content class="workspace__empty-diagram"></content>
            }
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
                        descend_slice={ctx.props().dispatch.reform(Action::DescendSlice)}
                        diagram_ref={self.diagram_ref.clone()}
                        on_hover={ctx.props().dispatch.reform(Action::HighlightSlice)}
                    />
                }
            }
            _ => Default::default(),
        };
        #[allow(clippy::let_unit_value)]
        let toolbar = workspace.as_ref().map_or_else(
            || html! {},
            |ws| {
                html! {
                    <div class="workspace__toolbar">
                        <PathControl
                            path={ws.path.clone()}
                            view={ws.view}
                            ascend_slice={ctx.props().dispatch.reform(Action::AscendSlice)}
                            update_view={ctx.props().dispatch.reform(Action::UpdateView)}
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

    fn view_diagram_svg<const N: usize>(&self, ctx: &Context<Self>) -> Html {
        if let Some(ref ws) = ctx.props().workspace {
            let highlight = highlight_attachment::<N>(ws, &ctx.props().signature)
                .or_else(|| highlight_slice::<N>(ws));
            html! {
                <PanZoomComponent on_scroll={ctx.props().dispatch.reform(Action::SwitchSlice)}>
                    <DiagramSvg<N>
                        diagram={ws.visible_diagram()}
                        id="workspace__diagram"
                        signature={ctx.props().signature.clone()}
                        on_select={self.on_select.clone()}
                        on_homotopy={self.on_homotopy.clone()}
                        highlight={highlight}
                        ref={self.diagram_ref.clone()}
                    />
                </PanZoomComponent>
            }
        } else {
            Default::default()
        }
    }
}

// TODO: highlighting needs better documentation and maybe a refactor

fn highlight_attachment<const N: usize>(
    workspace: &Workspace,
    signature: &Signature,
) -> Option<HighlightSvg<N>> {
    use Height::Regular;

    fn extend<const N: usize>(slices: [SliceIndex; 2]) -> [SliceIndex; N] {
        let mut extension = [SliceIndex::Interior(Regular(0)); N];
        for (i, &slice) in slices.iter().enumerate() {
            if let Some(x) = extension.get_mut(i) {
                *x = slice;
            }
        }
        extension
    }

    let attach_option = workspace.attachment_highlight.as_ref()?;
    let needle: DiagramN = if attach_option.inverse {
        DiagramN::try_from(
            signature
                .generator_info(attach_option.generator)?
                .diagram
                .clone(),
        )
        .ok()?
        .inverse()
        .ok()?
    } else {
        DiagramN::try_from(
            signature
                .generator_info(attach_option.generator)?
                .diagram
                .clone(),
        )
        .ok()?
    };

    let boundary_path = attach_option.boundary_path;
    let embedding = &attach_option.embedding;

    match boundary_path {
        Some(bp) if bp.depth() == workspace.path.len() => {
            let slice: Result<DiagramN, _> = needle.slice(bp.boundary().flip()).unwrap().try_into();
            let size = slice.map(|slice| slice.size()).unwrap_or_default();
            Some(HighlightSvg {
                from: extend([
                    bp.boundary().into(),
                    Regular(embedding.get(0).copied().unwrap_or_default()).into(),
                ]),
                to: extend([
                    bp.boundary().into(),
                    Regular(embedding.get(0).copied().unwrap_or_default() + size).into(),
                ]),
                kind: HighlightKind::Attach,
            })
        }
        Some(bp) if bp.depth() > workspace.path.len() => Some(HighlightSvg {
            from: extend([Boundary::Source.into(), bp.boundary().into()]),
            to: extend([Boundary::Target.into(), bp.boundary().into()]),
            kind: HighlightKind::Attach,
        }),
        Some(bp) if bp.boundary() == Boundary::Source => {
            let embedding = embedding.skip(workspace.path.len() - bp.depth() - 1);
            Some(HighlightSvg {
                from: extend([
                    Regular(embedding.get(0).copied().unwrap_or_default()).into(),
                    Regular(embedding.get(1).copied().unwrap_or_default()).into(),
                ]),
                to: extend([
                    Regular(embedding.get(0).copied().unwrap_or_default()).into(),
                    Regular(embedding.get(1).copied().unwrap_or_default() + 1).into(),
                ]),
                kind: HighlightKind::Attach,
            })
        }
        _ => {
            // Note: An empty boundary path implies that `needle` is one dimension higher than the
            // currently displayed diagram. Since this function computes highlights for at least 1D
            // diagrams, the `needle` diagram is at least two-dimensional. When it computes a
            // highlight for a 2D diagram, `needle` is at least three-dimensional.
            let depth = boundary_path.map_or(0, |bp| bp.depth() + 1);
            let embedding = embedding.skip(workspace.path.len() - depth);
            let needle_s: DiagramN = needle.source().try_into().unwrap();
            let needle_st_size: usize = needle_s
                .target()
                .try_into()
                .map(|d: DiagramN| d.size())
                .unwrap_or_default();

            Some(HighlightSvg {
                from: extend([
                    Regular(embedding.get(0).copied().unwrap_or_default()).into(),
                    Regular(embedding.get(1).copied().unwrap_or_default()).into(),
                ]),
                to: extend([
                    Regular(embedding.get(0).copied().unwrap_or_default() + needle_s.size()).into(),
                    Regular(embedding.get(1).copied().unwrap_or_default() + needle_st_size).into(),
                ]),
                kind: HighlightKind::Attach,
            })
        }
    }
}

fn highlight_slice<const N: usize>(workspace: &Workspace) -> Option<HighlightSvg<N>> {
    let slice = workspace.slice_highlight.as_ref()?;
    let mut from = [Boundary::Source.into(); N];
    from[0] = *slice;
    let mut to = [Boundary::Target.into(); N];
    to[0] = *slice;

    Some(HighlightSvg {
        from,
        to,
        kind: HighlightKind::Slice,
    })
}

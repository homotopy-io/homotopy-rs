use std::convert::{Into, TryInto};

use homotopy_core::{
    common::{Boundary, Height, SliceIndex},
    Diagram, DiagramN,
};
use path_control::PathControl;
use slice_control::SliceControl;
use view_control::ViewControl;
use yew::prelude::*;

use crate::{
    app::{
        diagram_gl::GlDiagram,
        diagram_svg::{Diagram0D, DiagramSvg, HighlightKind, HighlightSvg},
    },
    components::panzoom::PanZoomComponent,
    model::proof::{homotopy::Homotopy, Action, Signature, Workspace},
};

mod path_control;
mod slice_control;
mod view_control;

// TODO: Workspace rerendering when panzoom is changed needs to be smoother.

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub workspace: Workspace,
    pub dispatch: Callback<Action>,
    pub signature: Signature,
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
        let slice_buttons = match Self::visible_diagram(ctx) {
            Diagram::DiagramN(d) if ctx.props().workspace.view.dimension() == 2 => html! {
                <SliceControl
                    number_slices={d.size()}
                    descend_slice={ctx.props().dispatch.reform(Action::DescendSlice)}
                    diagram_ref={self.diagram_ref.clone()}
                    on_hover={ctx.props().dispatch.reform(Action::HighlightSlice)}
                />
            },
            _ => Default::default(),
        };

        html! {
            <div class="workspace">
                {self.view_diagram(ctx)}
                {slice_buttons}
                <div class="workspace__toolbar">
                    <PathControl
                        path={ctx.props().workspace.path.clone()}
                        view={ctx.props().workspace.view}
                        ascend_slice={ctx.props().dispatch.reform(Action::AscendSlice)}
                        update_view={ctx.props().dispatch.reform(Action::UpdateView)}
                        dimension={ctx.props().workspace.diagram.dimension()}
                    />
                    <ViewControl />
                </div>
            </div>
        }
    }
}

impl WorkspaceView {
    fn visible_diagram(ctx: &Context<Self>) -> Diagram {
        // TODO: This should not be recomputed every view
        ctx.props().workspace.visible_diagram()
    }

    fn view_diagram(&self, ctx: &Context<Self>) -> Html {
        match Self::visible_diagram(ctx) {
            Diagram::Diagram0(generator) => {
                html! {
                    <PanZoomComponent
                        on_scroll={ctx.props().dispatch.reform(Action::SwitchSlice)}
                    >
                        <Diagram0D
                            diagram={generator}
                            ref={self.diagram_ref.clone()}
                        />
                    </PanZoomComponent>
                }
            }
            Diagram::DiagramN(diagram) if diagram.dimension() == 1 => {
                let highlight =
                    highlight_attachment::<1>(&ctx.props().workspace, &ctx.props().signature)
                        .or_else(|| highlight_slice::<1>(&ctx.props().workspace));
                html! {
                    <PanZoomComponent
                        on_scroll={ctx.props().dispatch.reform(Action::SwitchSlice)}
                    >
                        <DiagramSvg<1>
                            diagram={diagram}
                            id="workspace__diagram"
                            on_select={self.on_select.clone()}
                            on_homotopy={self.on_homotopy.clone()}
                            highlight={highlight}
                            ref={self.diagram_ref.clone()}
                        />
                    </PanZoomComponent>
                }
            }
            Diagram::DiagramN(diagram) => match ctx.props().workspace.view.dimension() {
                3 | 4 => html! {
                    <GlDiagram
                        diagram={diagram}
                        signature={ctx.props().signature.clone()}
                        view={ctx.props().workspace.view}
                    />
                },
                _ => {
                    let highlight =
                        highlight_attachment::<2>(&ctx.props().workspace, &ctx.props().signature)
                            .or_else(|| highlight_slice::<2>(&ctx.props().workspace));

                    html! {
                        <PanZoomComponent
                            on_scroll={ctx.props().dispatch.reform(Action::SwitchSlice)}
                        >
                            <DiagramSvg<2>
                                diagram={diagram}
                                id="workspace__diagram"
                                on_select={self.on_select.clone()}
                                on_homotopy={self.on_homotopy.clone()}
                                highlight={highlight}
                                ref={self.diagram_ref.clone()}
                            />
                        </PanZoomComponent>
                    }
                }
            },
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
    let needle: DiagramN = signature
        .generator_info(attach_option.generator)?
        .diagram
        .clone()
        .try_into()
        .unwrap();

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

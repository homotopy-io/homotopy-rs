use yew::{callback::Callback, prelude::*};

use super::{DrawerViewSize, Sidebar, SidebarButton, SidebarDrawer, SidebarMsg};
#[cfg(debug_assertions)]
use crate::app::debug::DebugView;
use crate::{
    app::{project::ProjectView, settings::SettingsView, signature::SignatureView},
    components::Visible,
    model::{
        self,
        proof::{Action, SignatureEdit},
        Proof,
    },
};

macro_rules! declare_sidebar_drawers {
    ($(
        $(#[cfg($cfg:meta)])?
        $name:ident {
            $title:literal,
            $class:literal,
            $icon:literal,
            $body:expr,
            $(min_width: $min_width:expr,)?
            $(top_icon: $top_icon:literal,
              top_icon_action: $action:expr,)?
        }
    )*) => {
        #[allow(unused)]
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum NavDrawer {
            $(
                $(#[cfg($cfg)])*
                $name
            ),*
        }

        impl NavDrawer {
            pub(super) fn view(
                self,
                model_dispatch: &Callback<model::Action>,
                sidebar_dispatch: &Callback<SidebarMsg>,
                proof: &Proof,
                initial_width: i32,
                drawer_view_size: DrawerViewSize,
            ) -> Html {
                match self {
                    $(
                        $(#[cfg($cfg)])?
                        NavDrawer::$name => {
                            let body = $body;
                            html! {
                                <SidebarDrawer
                                    title={$title}
                                    class={$class}
                                    initial_width={initial_width}
                                    $(min_width={$min_width})?
                                    model_dispatch={model_dispatch}
                                    sidebar_dispatch={sidebar_dispatch}
                                    drawer_view_size={drawer_view_size}
                                    $(icon={$top_icon})?
                                    $(on_click={
                                        let action = $action;
                                        action(proof)
                                    })?
                                >
                                    {body(model_dispatch, proof, drawer_view_size)}
                                </SidebarDrawer>
                            }
                        }
                    ),*
                }
            }
        }

        impl Sidebar {
            pub(super) fn nav(&self, ctx: &Context<Self>) -> Html {
                html! {
                    <nav class="sidebar__nav">
                    $({{
                        $(#[cfg($cfg)])?
                        html! {
                            <SidebarButton
                                label={$title}
                                icon={$icon}
                                action={SidebarMsg::Toggle(NavDrawer::$name)}
                                shortcut={None}
                                dispatch={ctx.link().callback(|x| x)}
                                visibility={Visible}
                            />
                        }

                        $(
                            #[cfg(not($cfg))]
                            html! {}
                        )?
                    }})*
                    </nav>
                }
            }
        }
    }
}

declare_sidebar_drawers! {
    DRAWER_PROJECT {
        "Project",
        "project",
        "info",
        |dispatch, proof: &Proof, _| html! {
            <ProjectView
                dispatch={dispatch}
                metadata={proof.metadata().clone()}
            />
        },
    }

    DRAWER_SETTINGS {
        "Settings",
        "settings",
        "settings",
        |_, _, _| html! {
            <SettingsView />
        },
        min_width: 250,
    }

    DRAWER_SIGNATURE {
        "Signature",
        "signature",
        "list",
        |dispatch: &Callback<model::Action>, proof: &Proof, drawer_view_size: DrawerViewSize| html! {
            <SignatureView
                signature={proof.signature().clone()}
                dispatch={dispatch.reform(model::Action::Proof)}
                drawer_view_size={drawer_view_size}
            />
        },
        top_icon: "create_new_folder",
        top_icon_action: |proof: &Proof| model::Action::Proof(Action::EditSignature(SignatureEdit::NewFolder(proof.signature().as_tree().root()))),
    }

    #[cfg(debug_assertions)]
    DRAWER_DEBUG {
        "Debug",
        "debug",
        "bug_report",
        |_, proof: &Proof, _| html! {
            <DebugView proof={proof.clone()} />
        },
    }
}

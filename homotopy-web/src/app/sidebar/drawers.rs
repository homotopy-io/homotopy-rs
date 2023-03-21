use yew::{callback::Callback, prelude::*};

use super::{DrawerViewSize, Sidebar, SidebarButton, SidebarDrawer, SidebarMsg};
#[cfg(any(debug_assertions, feature = "show_debug_panel"))]
use crate::app::debug::DebugView;
use crate::{
    app::{
        image_export::ImageExportView, project::ProjectView, settings::SettingsView,
        signature::SignatureView,
    },
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
            $(top_icon: $top_icon:expr,
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
                                action={SidebarMsg::Toggle(Some(NavDrawer::$name))}
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
    // DRAWER_LOGIN {
    //     "Account",
    //     "account",
    //     "account_circle",
    //     |_, _, _| html! {
    //         <AccountView
    //         />
    //     },
    //     min_width: 250,
    // }

    DRAWER_PROJECT {
        "Project",
        "project",
        "info",
        |dispatch, proof: &Proof, _| html! {
            <ProjectView
                dispatch={dispatch}
                metadata={proof.metadata.clone()}
            />
        },
        min_width: 250,
    }

    DRAWER_SIGNATURE {
        "Signature",
        "signature",
        "list",
        |dispatch: &Callback<model::Action>, proof: &Proof, drawer_view_size: DrawerViewSize| html! {
            <SignatureView
                signature={proof.signature.clone()}
                dispatch={dispatch.clone()}
                drawer_view_size={drawer_view_size}
            />
        },
        min_width: 250,
        top_icon: "create_new_folder",
        top_icon_action: |proof: &Proof| model::Action::Proof(Action::EditSignature(SignatureEdit::NewFolder(proof.signature.as_tree().root()))),
    }

    DRAWER_IMAGE_EXPORT {
        "Image export",
        "ImageExport",
        "output",
        |dispatch, proof: &Proof, _| html! {
            <ImageExportView
                dispatch={dispatch}
                view_dim={proof.workspace.as_ref().map_or(0, |ws| ws.view.dimension())}
            />
        },
        min_width: 250,
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

    #[cfg(any(debug_assertions, feature = "show_debug_panel"))]
    DRAWER_DEBUG {
        "Debug",
        "debug",
        "bug_report",
        |dispatch, proof: &Proof, _| html! {
            <DebugView proof={proof.clone()} dispatch={dispatch} />
        },
        min_width: 250,
    }
}

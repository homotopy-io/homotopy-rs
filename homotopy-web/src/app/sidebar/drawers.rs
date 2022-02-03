use yew::{callback::Callback, prelude::*};

use super::{Sidebar, SidebarButton, SidebarDrawer, SidebarMsg};
#[cfg(debug_assertions)]
use crate::app::debug::DebugView;
use crate::{
    app::{project::ProjectView, settings::SettingsView, signature::SignatureView},
    components::Visible,
    model::{self, Proof},
};

macro_rules! declare_sidebar_drawers {
    ($(
        $(#[$attrib:meta])*
        $name:ident {
            $title:literal,
            $class:literal,
            $icon:literal,
            $body:expr,
        }
    )*) => {
        #[allow(unused)]
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum NavDrawer {
            $(
                $(#[$attrib])*
                $name
            ),*
        }

        impl NavDrawer {
            pub(super) fn view(
                self,
                dispatch: &Callback<model::Action>,
                proof: &Proof,
            ) -> Html {
                match self {
                    $(
                        $(#[$attrib])*
                        NavDrawer::$name => {
                            let body = $body;
                            html! {
                                <SidebarDrawer
                                    title={$title}
                                    class={$class}
                                    dispatch={dispatch}
                                >
                                    {body(dispatch, proof)}
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
                    $({
                        $(#[$attrib])*
                        html!{
                            <SidebarButton
                                label={$title}
                                icon={$icon}
                                action={SidebarMsg::Toggle(NavDrawer::$name)}
                                shortcut={None}
                                dispatch={ctx.link().callback(|x| x)}
                                visibility={Visible}
                            />
                        }
                    })*
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
        |dispatch, _| html! {
            <ProjectView dispatch={dispatch} />
        },
    }

    DRAWER_SETTINGS {
        "Settings",
        "settings",
        "settings",
        |_, _| html! {
            <SettingsView />
        },
    }

    DRAWER_SIGNATURE {
        "Signature",
        "signature",
        "list",
        |dispatch: &Callback<model::Action>, proof: &Proof| html! {
            <SignatureView
                signature={proof.signature().clone()}
                dispatch={dispatch.reform(model::Action::Proof)}
            />
        },
    }

    #[cfg(debug_assertions)]
    DRAWER_DEBUG {
        "Debug",
        "debug",
        "bug_report",
        |_, proof: &Proof| html! {
            <DebugView proof={proof.clone()} />
        },
    }
}

use yew::callback::Callback;
use yew::prelude::*;

use crate::app::project::ProjectView;
use crate::app::settings::SettingsView;
use crate::app::signature::SignatureView;

use crate::components::Visible;
use crate::model::{self, Proof};

use super::{Sidebar, SidebarButton, SidebarDrawer, SidebarMsg};

macro_rules! declare_sidebar_drawers {
    ($($name:ident {
        $title:literal,
        $class:literal,
        $icon:literal,
        $body:expr,
    })*) => {
        #[allow(unused)]
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum NavDrawer {
            $($name),*
        }

        impl NavDrawer {
            pub(super) fn view(
                self,
                dispatch: &Callback<model::Action>,
                proof: &Proof,
            ) -> Html {
                match self {
                    $(NavDrawer::$name => {
                        let body = $body;
                        html! {
                            <SidebarDrawer
                                title={$title}
                                class={$class}
                            >
                                {body(dispatch, proof)}
                            </SidebarDrawer>
                        }
                    }),*
                }
            }
        }

        impl Sidebar {
            pub(super) fn nav(&self) -> Html {
                html! {
                    <nav class="sidebar__nav">
                        $(<SidebarButton
                            label={$title}
                            icon={$icon}
                            action={SidebarMsg::Toggle(NavDrawer::$name)}
                            shortcut={None}
                            dispatch={self.link.callback(|x| x)}
                            visibility={Visible}
                        />)*
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
}

mod diagram2d;
mod panzoom;
use closure::closure;
use diagram2d::Diagram2D;
use homotopy_core::*;
use panzoom::PanZoom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;
use yew::prelude::*;
use yew_functional::*;
use yew_functional_macro::functional_component;
use yew_mdc::components::*;

#[derive(Default, Clone, Debug, PartialEq, Properties)]
pub struct Props {}

#[derive(Default, Clone, Debug, PartialEq)]
struct State {
    signature: Signature,
    project: Project,
    view: Option<View>,
    workspace: Option<Workspace>,
    boundary: Option<(Boundary, Diagram)>,
}

#[derive(Clone, Debug, PartialEq)]
struct Workspace {
    diagram: Diagram,
    path: Vec<SliceIndex>,
}

#[derive(Default, Clone, Debug, PartialEq)]
struct Signature {
    generators: HashMap<Generator, GeneratorInfo>,
}

#[derive(Clone, Debug, PartialEq)]
struct GeneratorInfo {
    name: String,
    colour: String,
    diagram: Diagram,
}

#[derive(Default, Clone, Debug, PartialEq)]
struct Project {
    name: String,
    abs: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
enum View {
    ViewSignature,
    ViewProject,
    ViewUser,
}

enum Action {
    ToggleView(View),
    RenameGenerator(Generator, String),
    RecolourGenerator(Generator, String),
    RemoveGenerator(Generator),
    SelectGenerator(Generator),
    DescendSlice(SliceIndex),
    AscendSlice(usize),
    ClearWorkspace,
    IdentityDiagram,
    MakeBoundary(Boundary),
    MakeGenerator,
    RestrictDiagram,
}

#[functional_component]
pub fn app(props: &Props) -> Html {
    let (state, dispatch) = use_reducer(
        |prev: Rc<State>, action: Action| -> State {
            match action {
                Action::ToggleView(v) => State {
                    view: if prev.view == Some(v) { None } else { Some(v) },
                    ..State::clone(&prev)
                },
                Action::RenameGenerator(id, name) => State {
                    signature: Signature {
                        generators: {
                            let mut generators = prev.signature.generators.clone();
                            generators.entry(id).and_modify(|e| e.name = name);
                            generators
                        },
                    },
                    ..State::clone(&prev)
                },
                Action::RecolourGenerator(id, colour) => State {
                    signature: Signature {
                        generators: {
                            let mut generators = prev.signature.generators.clone();
                            generators.entry(id).and_modify(|e| e.colour = colour);
                            generators
                        },
                    },
                    ..State::clone(&prev)
                },
                Action::RemoveGenerator(id) => State {
                    signature: Signature {
                        generators: {
                            let mut generators = prev.signature.generators.clone();
                            generators.remove(&id);
                            generators
                        },
                    },
                    ..State::clone(&prev)
                },
                Action::SelectGenerator(_) => State {
                    /* TODO */
                    ..State::clone(&prev)
                },
                Action::DescendSlice(index) => State {
                    workspace: {
                        prev.workspace.clone().map(|mut w| {
                            w.path.push(index);
                            w
                        })
                    },
                    ..State::clone(&prev)
                },
                Action::AscendSlice(count) => State {
                    workspace: {
                        prev.workspace.clone().map(|mut w| {
                            w.path.truncate(w.path.len() - count);
                            w
                        })
                    },
                    ..State::clone(&prev)
                },
                Action::ClearWorkspace => State {
                    workspace: Default::default(),
                    ..State::clone(&prev)
                },
                Action::IdentityDiagram => State {
                    /* TODO */
                    ..State::clone(&prev)
                },
                Action::MakeBoundary(_) => State {
                    /* TODO */
                    ..State::clone(&prev)
                },
                Action::MakeGenerator => State {
                    /* TODO */
                    ..State::clone(&prev)
                },
                Action::RestrictDiagram => State {
                    /* TODO */
                    ..State::clone(&prev)
                },
            }
        },
        Default::default(),
    );

    let example = {
        let x = Generator {
            id: 0,
            dimension: 0,
        };
        let f = Generator {
            id: 1,
            dimension: 1,
        };
        let m = Generator {
            id: 2,
            dimension: 2,
        };

        let fd = DiagramN::new(f, x, x).unwrap();
        let ffd = fd.attach(fd.clone(), Boundary::Target, &[]).unwrap();
        let md = DiagramN::new(m, ffd, fd).unwrap();

        let mut result = md.clone();

        for _ in 0..2 {
            result = result.attach(md.clone(), Boundary::Source, &[0]).unwrap();
        }

        result
    };

    html! {
        <>
        <TopAppBar id="app-bar">
            <section class="mdc-top-app-bar__section mdc-top-app-bar__section--align-start">
                <IconButton classes="material-icons mdc-top-app-bar__navigation-icon"
                    togglable=true toggle_on={state.view == Some(View::ViewSignature)}
                    onclick=Callback::from(closure!(clone dispatch,
                                                    |_: MouseEvent| dispatch(Action::ToggleView(View::ViewSignature))))
                >
                        {"functions"}
                </IconButton>
                <IconButton classes="material-icons mdc-top-app-bar__navigation-icon"
                    togglable=true toggle_on={state.view == Some(View::ViewProject)}
                    onclick=Callback::from(closure!(clone dispatch,
                                                    |_: MouseEvent| dispatch(Action::ToggleView(View::ViewProject))))
                >
                        {"description"}
                </IconButton>
                <IconButton classes="material-icons mdc-top-app-bar__navigation-icon"
                    togglable=true toggle_on={state.view == Some(View::ViewUser)}
                    onclick=Callback::from(closure!(clone dispatch,
                                                    |_: MouseEvent| dispatch(Action::ToggleView(View::ViewUser))))
                >
                        {"person"}
                </IconButton>
            </section>
            <section class="mdc-top-app-bar__section mdc-top-app-bar__section--align-end">
                <img src="/logo.svg" alt="logo" width="32" height="32"/>
            </section>
        </TopAppBar>

        <Drawer id="drawer"
            // classes="mdc-top-app-bar--fixed-adjust"
            // style=yew_mdc::components::drawer::Style::Dismissible
            // open={state.view.is_some()}
        >
            <DrawerHeader>
                <h3 class="mdc-drawer__title">{match state.view {
                    Some(View::ViewSignature) => "Signature",
                    Some(View::ViewProject) => "Project",
                    Some(View::ViewUser) => "User",
                    None => ""
                }}</h3>
            </DrawerHeader>
            <DrawerContent>
                {"TODO"}
            </DrawerContent>
        </Drawer>

        <div class="mdc-drawer-app-content mdc-top-app-bar--fixed-adjust">
            <main class="main-content" id="main-content">
                <PanZoom>
                    <Diagram2D diagram={example} on_select={Callback::from(|pt| {
                        log::info!("clicked! {:?}", pt);
                    })} />
                </PanZoom>
            </main>
        </div>
        </>
    }
}

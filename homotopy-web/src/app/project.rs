use closure::closure;

use yew::html::ChangeData::Files;
use yew::prelude::*;
use yew_functional::function_component;
use yew_functional::use_state;
use yew_services::{reader::FileData, ReaderService};

use crate::components::Drawer;
use crate::model::Action;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub dispatch: Callback<Action>,
}

#[function_component(ProjectView)]
pub fn project_view(props: &Props) -> Html {
    let export = props.dispatch.reform(|_| Action::ExportProof);
    let dispatch = &props.dispatch;
    let (_, set_reader_task) = use_state(|| None);
    let import: Callback<ChangeData> = Callback::from(closure!(clone dispatch, |evt| {
        if let Files(filelist) = evt {
            let file = filelist.get(0).unwrap();
            let callback = Callback::from(
                closure!(clone dispatch, clone set_reader_task, |fd: FileData| {
                    dispatch.emit(Action::ImportProof(fd.content.into()));
                    set_reader_task(None);
                }),
            );
            let task = ReaderService::read_file(file, callback).expect("failed to read file");
            set_reader_task(Some(task));
        }
    }));
    html! {
        <Drawer title="Project" class="project">
            <button onclick=export>{"Export"}</button>
                <label for="import">
                    {"Import"}
                </label>
                <input type="file" accept="application/msgpack,.hom" class="visually-hidden" id="import" onchange=import/>
        </Drawer>
    }
}

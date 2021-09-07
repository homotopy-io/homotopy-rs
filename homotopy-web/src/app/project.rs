use closure::closure;
use yew::{functional::use_ref, html::ChangeData::Files, prelude::*};
use yew_macro::function_component;

use crate::model::Action;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub dispatch: Callback<Action>,
}

#[function_component(ProjectView)]
pub fn project_view(props: &Props) -> Html {
    let export = props.dispatch.reform(|_| Action::ExportProof);
    let dispatch = &props.dispatch;
    let reader_task = use_ref(|| None);
    let import: Callback<ChangeData> = Callback::from(closure!(clone dispatch, |evt| {
        if let Files(filelist) = evt {
            let file = filelist.get(0).unwrap();
            let task = gloo::file::callbacks::read_as_bytes(&file.into(), closure!(clone dispatch, |res| {
                dispatch.emit(Action::ImportProof(res.expect("failed to read file").into()));
            }));
            *reader_task.borrow_mut() = Some(task);
        }
    }));
    html! {
        <>
            <button onclick={export}>{"Export"}</button>
            <label for="import">
                {"Import"}
            </label>
            <input type="file" accept="application/msgpack,.hom" class="visually-hidden" id="import" onchange={import}/>
        </>
    }
}

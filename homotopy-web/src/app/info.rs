use yew::prelude::*;

use crate::components::icon::{Icon, IconSize};

pub fn get_help_message() -> Html {
    html! {
        <div class="help">
            <p>
                {"For instructions on how to use the tool, visit the "}<a href="https://github.com/homotopy-io/homotopy-rs/blob/master/TUTORIAL.md">{"tutorial"}</a>{"."}
            </p>
            <table>
                <tr>
                    <td class="help-action"><Icon name="touch_app" size={IconSize::Icon24}/></td>
                    <td class="help-description">{"Attach"}</td>
                </tr>
                <tr>
                    <td class="help-action"><Icon name="swipe" size={IconSize::Icon24}/></td>
                    <td class="help-description">{"Homotopy"}</td>
                </tr>
                <tr>
                    <td class="help-action">{"Hold  "}<kbd class="kbc-button">{"SHIFT"}</kbd></td>
                    <td class="help-description">{"Cancel inverses"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"U"}</kbd></td>
                    <td class="help-description">{"Undo"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"Y"}</kbd></td>
                    <td class="help-description">{"Redo"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"A"}</kbd></td>
                    <td class="help-description">{"Add 0-cell"}</td>
                </tr>
                <tr>
                    <td class="help-action">
                        <kbd class="kbc-button">{"S"}</kbd>
                        {" "}
                        <kbd class="kbc-button">{"T"}</kbd>
                    </td>
                    <td class="help-description">{"Set source/target"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"I"}</kbd></td>
                    <td class="help-description">{"Take identity"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"C"}</kbd></td>
                    <td class="help-description">{"Clear workspace"}</td>
                </tr>
                <tr>
                    <td class="help-action">
                        <kbd class="kbc-button">{"↑"}</kbd>
                        {" "}
                        <kbd class="kbc-button">{"↓"}</kbd>
                    </td>
                    <td class="help-description">{"Switch slice"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"←"}</kbd></td>
                    <td class="help-description">{"Ascend slice"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"→"}</kbd></td>
                    <td class="help-description">{"Descend slice"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"D"}</kbd></td>
                    <td class="help-description">{"Behead"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"F"}</kbd></td>
                    <td class="help-description">{"Befoot"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"V"}</kbd></td>
                    <td class="help-description">{"Invert"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"R"}</kbd></td>
                    <td class="help-description">{"Restrict"}</td>
                </tr>
                <tr>
                    <td class="help-action"><kbd class="kbc-button">{"H"}</kbd></td>
                    <td class="help-description">{"Theorem"}</td>
                </tr>
                <tr>
                    <td class="help-action">
                        <kbd class="kbc-button">{"1"}</kbd>
                        {" - "}
                        <kbd class="kbc-button">{"9"}</kbd>
                    </td>
                    <td class="help-description">{"Select generator or attachment"}</td>
                </tr>
            </table>
        </div>
    }
}

pub fn get_about_message() -> Html {
    html! {
        <div>
            <p>
                <a href="https://ncatlab.org/nlab/show/homotopy.io">{"homotopy.io"}</a>
                {": the proof assistant for finitely-presented globular n-categories."}
            </p>
            <p>{"Written by "}
                <a href="https://github.com/doctorn">{"Nathan Corbyn"}</a>
                {", "}
                <a href="https://github.com/zrho">{"Lukas Heidemann"}</a>
                {", "}
                <a href="https://github.com/NickHu">{"Nick Hu"}</a>
                {", "}
                <a href="https://github.com/calintat">{"Calin Tataru"}</a>
                {", "}
                <a href="https://sarti.me">{"Chiara Sarti"}</a>
                {", and "}
                <a href="https://github.com/jamievicary">{"Jamie Vicary"}</a>
                {"."}
            </p>
            <h3>{"License"}</h3>
            <p>{"homotopy.io source code is published under the terms of the BSD 3-Clause License."}</p>
            <pre>{include_str!("../../../LICENSE")}</pre>
            {"homotopy.io documentation is licensed under a "}
            <a rel="license" href="https://creativecommons.org/licenses/by/4.0/">
                {"Creative Commons Attribution 4.0 International License"}
            </a>{"."}
            <br />
            <a rel="license" href="https://creativecommons.org/licenses/by/4.0/">
                <img alt="Creative Commons License" style="border-width:0" src="by.svg" />
            </a>
        </div>
    }
}

pub fn get_panic_message() -> Html {
    html! {
        <div>
            <p>
                {"It appears you have found an unexpected bug in our tool. Many apologies for the poor experience."}
            </p>
            <p>
                {"We would be extremely grateful if you could report this issue."}
            </p>
            <p>
                {"The process is rather straightforward: the button below will download a file containing some debugging information for us, you can attach it in a new issue in our "}
                <a href="https://github.com/homotopy-io/homotopy-rs/issues">{"GitHub tracker"}</a>
                {", alongside a brief description of what your were doing."}
            </p>
            <p>
                {"We'll fix the problem in no time!"}
            </p>
            <button onclick={move |_| {crate::panic::export_dump(false).unwrap();}}>{"Download action logs"}</button>
        </div>
    }
}

pub fn get_onboarding_message() -> Html {
    html! {
        <div class="workspace__empty-diagram">
            <div class="workspace__empty-diagram-content">
                <img src="/logo.svg" alt="Homotopy.io logo" class="workspace__empty-logo" />
                <header class="workspace__empty-header">
                    <h3>{"Don't Panic!"}</h3>
                </header>
                <table>
                    <tr>
                        <td class="workspace__empty-keydesc">{"Add 0-cell"}</td>
                        <td class="workspace__empty-keyicon"><kbd class="kbc-button">{"A"}</kbd></td>
                    </tr>
                    <tr>
                        <td class="workspace__empty-keydesc">{"Set source/target"}</td>
                        <td class="workspace__empty-keyicon">
                            <kbd class="kbc-button">{"S"}</kbd>
                            {" "}
                            <kbd class="kbc-button">{"T"}</kbd>
                        </td>
                    </tr>
                    <tr>
                        <td class="workspace__empty-keydesc">{"Take identity"}</td>
                        <td class="workspace__empty-keyicon"><kbd class="kbc-button">{"I"}</kbd></td>
                    </tr>
                    <tr>
                        <td class="workspace__empty-keydesc">{"Clear workspace"}</td>
                        <td class="workspace__empty-keyicon"><kbd class="kbc-button">{"C"}</kbd></td>
                    </tr>
                    <tr>
                        <td class="workspace__empty-keydesc">{"Help"}</td>
                        <td class="workspace__empty-keyicon"><kbd class="kbc-button">{"?"}</kbd></td>
                    </tr>
                </table>
            </div>
        </div>
    }
}

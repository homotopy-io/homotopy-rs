use yew::prelude::*;

pub fn get_modal_message() -> Html {
    html! {
        <div class="modal-content">
            <header>
                <h2>{"About"}</h2>
            </header>
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
        <div class="modal-content">
            <header>
                <h2>{"Unexpected Crash!"}</h2>
            </header>
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
                <header>
                    <h3>{"Don't Panic!"}</h3>
                </header>
                <table>
                    <tr>
                        <td style="text-align: right">{"Add generators  "}</td>
                        <td style="text-align: left"><kbd class="kbc-button">{"A"}</kbd></td>
                    </tr>
                    <tr>
                        <td style="text-align: right">{"Clear workspace  "}</td>
                        <td style="text-align: left"><kbd class="kbc-button">{"C"}</kbd></td>
                    </tr>
                    <tr>
                        <td style="text-align: right">{"Take source/target  "}</td>
                        <td style="text-align: left">
                            <kbd class="kbc-button">{"S"}</kbd>
                            <kbd class="kbc-button">{"T"}</kbd>
                        </td>
                    </tr>
                    <tr>
                        <td style="text-align: right">{"Take identity  "}</td>
                        <td style="text-align: left"><kbd class="kbc-button">{"I"}</kbd></td>
                    </tr>
                    <tr>
                        <td style="text-align: right">{"Save theorem  "}</td>
                        <td style="text-align: left"><kbd class="kbc-button">{"H"}</kbd></td>
                    </tr>
                </table>
            </div>
        </div>
    }
}

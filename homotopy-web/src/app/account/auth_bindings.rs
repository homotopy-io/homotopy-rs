use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/app/account/account_script.js")]
extern "C" {
    #[wasm_bindgen(js_name = "initializeUI")]
    pub fn initialize_ui();

    #[wasm_bindgen(js_name = "logOut")]
    pub fn log_out();
}

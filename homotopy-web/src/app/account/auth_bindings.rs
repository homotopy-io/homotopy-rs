use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "https://www.gstatic.com/firebasejs/9.7.0/firebase-auth.js")]

extern "C" {
    pub type Auth;
    pub type AuthProvider;
    pub type GoogleAuthProvider;

    #[derive(Debug)]
    #[wasm_bindgen(extends = UserInfo, typescript_type = r#"import("firebase/auth").User"#)]
    pub type User;
    pub type UserMetadata;
    #[derive(Debug)]
    pub type UserInfo;
    pub type UserCredential;

    #[wasm_bindgen(js_name = getAuth)]
    pub fn get_auth() -> Auth;

    // Sign in and sign out
    #[wasm_bindgen(js_name = signInWithPopup)]
    pub fn sign_in_with_popup_google(auth: Auth, provider: GoogleAuthProvider) -> Option<UserCredential>;

    // 
    #[wasm_bindgen(constructor)]
    pub fn new() -> GoogleAuthProvider;

    // #[]
    // pub fn sign_out()
    // User data
}

#[wasm_bindgen(module = "/src/app/account/account_script.js")]
extern "C" {
    #[wasm_bindgen(js_name = "initializeUI")]
    pub fn initialize_ui();
}











#![allow(non_upper_case_globals)]
use std::str;

use axum::{
    body::{Body, Bytes},
    response::{Html, Response},
    routing::get,
    Error, Router,
};
use http::Request;
use tower_service::Service;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn handle_request(uri: &str) -> String {
    let mut router = Router::new().route("/", get(index)).into_service();

    let request: Request<Body> = Request::builder().uri(uri).body("".into()).unwrap();

    let mut response: Response = router.call(request).await.unwrap();
    let data: Option<Result<Bytes, Error>> = http_body::Body::data(response.body_mut()).await;

    let result: Bytes = data.unwrap().unwrap();

    str::from_utf8(&*result).unwrap().to_string()
}

async fn index() -> Html<&'static str> {
    Html("<h1>TODO</h1>")
}

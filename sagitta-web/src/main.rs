#![allow(non_snake_case)]

use dioxus::prelude::*;
use sagitta_remote_api_schema::v2::get_workspaces::{
    V2GetWorkspacesRequest, V2GetWorkspacesResponse,
};
use tracing::Level;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

async fn get_workspaces() -> V2GetWorkspacesResponse {
    let resp = gloo_net::http::Request::post("http://localhost:8512/v2/get-workspaces")
        .json(&V2GetWorkspacesRequest {})
        .unwrap()
        .send()
        .await
        .unwrap();
    let resp: V2GetWorkspacesResponse = resp.json().await.unwrap();
    resp
}

#[component]
fn Home() -> Element {
    let workspaces = use_resource(get_workspaces);
    let list = match workspaces() {
        Some(V2GetWorkspacesResponse::Ok { items }) => {
            rsx! {
                ul {
                    for workspace in items.iter() {
                        li { "{workspace.name.clone()}" }
                    }
                }
            }
        }
        _ => {
            rsx! {
                p { "Failed to get workspaces" }
            }
        }
    };

    rsx! {
        div {
            h1 { "Sagitta Web" }
            h2 { "wokrspaces" }
            { list }
        }
    }
}

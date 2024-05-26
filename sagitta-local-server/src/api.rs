use std::path::PathBuf;

use actix_web::{web, App, HttpServer};
use sagitta_common::clock::Clock;

use crate::api_state::ApiState;

use self::v1::sync::v1_sync;

pub mod v1;

pub struct ServerConfig {
    pub clock: Clock,
    pub port: u16,
    pub local_system_workspace_base_path: PathBuf,
    pub remote_api_base_url: String,
}

pub async fn run_local_api_server(config: ServerConfig) {
    let state = ApiState::new(
        config.clock.clone(),
        config.local_system_workspace_base_path.clone(),
        &config.remote_api_base_url,
    )
    .await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(v1_sync)
    })
    .bind(("0.0.0.0", config.port))
    .unwrap()
    .run()
    .await
    .unwrap();
}

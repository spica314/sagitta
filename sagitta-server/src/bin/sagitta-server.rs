use sagitta_common::clock::Clock;
use sagitta_server::api::run_server;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = sagitta_server::api::ServerConfig {
        base_path: std::path::PathBuf::from("/tmp/sagitta"),
        surreal_uri: "127.0.0.1:28000".to_string(),
        is_main: true,
        clock: Clock::new(),
        port: 8512,
    };
    run_server(config).await;
}

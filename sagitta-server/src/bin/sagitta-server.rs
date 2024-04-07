use sagitta_common::clock::Clock;
use sagitta_server::api::run_server;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = sagitta_server::api::ServerConfig {
        base_path: std::path::PathBuf::from("/tmp/sagitta"),
        clock: Clock::new(),
        port: 8081,
    };
    run_server(config).await;
}

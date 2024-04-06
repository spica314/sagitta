use sagitta_server::api::run_server;

#[tokio::main]
async fn main() {
    let config = sagitta_server::api::ServerConfig {
        base_path: std::path::PathBuf::from("/tmp/sagitta"),
        clock: sagitta_server::tools::Clock::new(),
    };
    run_server(config).await;
}

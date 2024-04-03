use sagitta::api::run_server;

#[tokio::main]
async fn main() {
    let config = sagitta::api::ServerConfig {
        base_path: std::path::PathBuf::from("/tmp/sagitta"),
        clock: sagitta::tools::Clock::new(),
    };
    run_server(config).await;
}

use sagitta_common::clock::Clock;
use sagitta_remote_server::api::{run_server, ServerConfig};

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = ServerConfig {
        base_path: std::path::PathBuf::from("/tmp/sagitta"),
        is_main: true,
        clock: Clock::new(),
        port: 8512,
    };
    run_server(config).await;
}

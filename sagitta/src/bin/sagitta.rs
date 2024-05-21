use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use sagitta::api::ServerConfig;
use sagitta::args::Args;
use sagitta::fs::{run_fs, SagittaConfig};
use sagitta_common::clock::Clock;
use sagitta_remote_api_schema::v2::create_workspace::V2CreateWorkspaceRequest;
use sagitta_remote_api_schema::v2::get_workspaces::{
    V2GetWorkspacesRequest, V2GetWorkspacesResponse,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    eprintln!("args = {:?}", args);

    if let Some(mount) = &args.mount {
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        let config = SagittaConfig {
            base_url: "http://localhost:8512".to_string(),
            mountpoint: mount.to_string(),
            uid,
            gid,
            clock: Clock::new(),
            local_system_workspace_base_path: PathBuf::from_str("./sagitta-test-system").unwrap(),
            debug_sleep_duration: None,
        };
        let api_config = ServerConfig {
            clock: config.clock.clone(),
            port: 8513,
        };
        tokio::spawn(async move {
            sagitta::api::run_local_api_server(api_config).await;
        });
        run_fs(config);
    } else {
        let api_client =
            sagitta::api_client::SagittaApiClient::new("http://localhost:8512".to_string());
        let _local_api_client = sagitta::local_api_client::SagittaLocalApiClient::new(
            "http://localhost:8513".to_string(),
        );
        let command = args.subcommand.unwrap();
        match command {
            sagitta::args::Commands::Workspace { subcommand } => match subcommand.unwrap() {
                sagitta::args::WorkspaceSubcommands::Create { name } => {
                    let list = api_client
                        .v2_get_workspaces(V2GetWorkspacesRequest {})
                        .unwrap();
                    match list {
                        V2GetWorkspacesResponse::Ok { items } => {
                            if items.iter().any(|workspace| workspace.name == name) {
                                eprintln!("Workspace {} already exists", name);
                                return;
                            }
                        }
                        _ => {
                            eprintln!("Failed to get workspaces");
                            return;
                        }
                    }
                    api_client
                        .v2_create_workspace(V2CreateWorkspaceRequest { name })
                        .unwrap();
                }
                sagitta::args::WorkspaceSubcommands::List => {
                    let list = api_client
                        .v2_get_workspaces(V2GetWorkspacesRequest {})
                        .unwrap();
                    match list {
                        V2GetWorkspacesResponse::Ok { items } => {
                            for workspace in items {
                                println!("{}", workspace.name);
                            }
                        }
                        _ => {
                            eprintln!("Failed to get workspaces");
                        }
                    }
                }
            },
        }
    }
}

use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use sagitta::args::Args;
use sagitta::fs::{run_fs, SagittaConfig};
use sagitta_common::clock::Clock;
use sagitta_local_api_schema::v1::sync::V1SyncRequest;
use sagitta_local_server::api::ServerConfig;
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
            local_system_workspace_base_path: config.local_system_workspace_base_path.clone(),
            remote_api_base_url: config.base_url.clone(),
        };
        tokio::spawn(async move {
            sagitta_local_server::api::run_local_api_server(api_config).await;
        });
        run_fs(config);
    } else {
        let api_client =
            sagitta_remote_api_client::SagittaApiClient::new("http://localhost:8512".to_string());
        let local_api_client = sagitta_local_api_client::SagittaLocalApiClient::new(
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
            sagitta::args::Commands::Sync { workspace_id } => {
                local_api_client
                    .v1_sync(V1SyncRequest {
                        workspace_id: workspace_id.clone(),
                    })
                    .unwrap();
            }
        }
    }
}

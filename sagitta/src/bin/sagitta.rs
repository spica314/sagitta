use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use sagitta::args::Args;
use sagitta::fs::{run_fs, SagittaConfig};
use sagitta_common::clock::Clock;

fn main() {
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
        run_fs(config);
    } else {
        let api_client =
            sagitta::api_client::SagittaApiClient::new("http://localhost:8512".to_string());
        let command = args.subcommand.unwrap();
        match command {
            sagitta::args::Commands::Workspace { subcommand } => match subcommand.unwrap() {
                sagitta::args::WorkspaceSubcommands::Create { name } => {
                    let list = api_client.workspace_list().unwrap();
                    if list.workspaces.iter().any(|workspace| workspace == &name) {
                        eprintln!("Workspace {} already exists", name);
                        return;
                    }
                    api_client.workspace_create(&name).unwrap();
                }
                sagitta::args::WorkspaceSubcommands::List => {
                    let list = api_client.workspace_list().unwrap();
                    for workspace in list.workspaces {
                        println!("{}", workspace);
                    }
                }
            },
        }
    }
}

use std::path::PathBuf;

use sagitta_common::clock::Clock;
use sagitta_remote_system_db::RemoteSystemWorkspaceManager;

// use crate::server_files_manager::ServerFilesManager;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub remote_system_workspace_manager: RemoteSystemWorkspaceManager,
    pub clock: Clock,
}

impl ApiState {
    pub fn new(base_path: PathBuf, clock: Clock) -> Self {
        Self {
            remote_system_workspace_manager: RemoteSystemWorkspaceManager::new(base_path),
            clock,
        }
    }
}

use std::path::PathBuf;

use sagitta_common::clock::Clock;
use sagitta_remote_system_db::db::SagittaRemoteSystemDB;
use sagitta_remote_system_workspace::RemoteSystemWorkspaceManager;

#[derive(Clone)]
pub struct ApiState {
    pub remote_system_workspace_manager: RemoteSystemWorkspaceManager,
    pub clock: Clock,
}

impl ApiState {
    pub async fn new(base_path: PathBuf, clock: Clock, db: SagittaRemoteSystemDB) -> Self {
        Self {
            remote_system_workspace_manager: RemoteSystemWorkspaceManager::new(base_path, db).await,
            clock,
        }
    }
}

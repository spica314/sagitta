use std::path::PathBuf;

use sagitta_common::clock::Clock;
use sagitta_local_system_workspace::LocalSystemWorkspaceManager;
use sagitta_remote_api_client::SagittaApiClient;

#[derive(Clone)]
pub struct ApiState {
    pub clock: Clock,
    pub local_system_workspace: LocalSystemWorkspaceManager,
    pub remote_api_client: SagittaApiClient,
}

impl ApiState {
    pub async fn new(
        clock: Clock,
        local_system_workspace_path: PathBuf,
        remote_api_base_url: &str,
    ) -> Self {
        Self {
            clock,
            local_system_workspace: LocalSystemWorkspaceManager::new(local_system_workspace_path),
            remote_api_client: SagittaApiClient::new(remote_api_base_url.to_string()),
        }
    }
}

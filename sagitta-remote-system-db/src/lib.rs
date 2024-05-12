use std::time::SystemTime;

pub mod sqlite;

#[derive(Debug)]
pub struct CreateWorkspaceRequest {
    pub workspace_name: String,
}

#[derive(Debug)]
pub struct CreateWorkspaceResponse {
    pub workspace_id: String,
}

#[derive(Debug)]
pub struct GetWorkspacesRequest {
    pub contains_deleted: bool,
}

#[derive(Debug)]
pub struct GetWorkspacesResponseItem {
    pub workspace_id: String,
    pub workspace_name: String,
    pub created_at: SystemTime,
    pub deleted_at: Option<SystemTime>,
}

#[derive(Debug)]
pub struct GetWorkspacesResponse {
    pub workspaces: Vec<GetWorkspacesResponseItem>,
}

#[derive(Debug)]
pub struct DeleteWorkspaceRequest {
    pub workspace_id: String,
}

#[derive(Debug)]
pub struct DeleteWorkspaceResponse {}

#[derive(Debug)]
pub enum SagittaRemoteSystemDBError {
    WorkspaceAlreadyExists,
    WorkspaceNotFound,
    InternalError,
}

pub trait SagittaRemoteSystemDB {
    fn create_workspace(
        &self,
        request: CreateWorkspaceRequest,
    ) -> Result<CreateWorkspaceResponse, SagittaRemoteSystemDBError>;

    fn get_workspaces(
        &self,
        request: GetWorkspacesRequest,
    ) -> Result<GetWorkspacesResponse, SagittaRemoteSystemDBError>;

    fn delete_workspace(
        &self,
        request: DeleteWorkspaceRequest,
    ) -> Result<DeleteWorkspaceResponse, SagittaRemoteSystemDBError>;
}

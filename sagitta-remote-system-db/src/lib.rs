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
pub struct CreateBlobRequest {
    pub blob_id: String,
    pub hash: String,
    pub size: u64,
}

#[derive(Debug)]
pub struct CreateBlobResponse {}

#[derive(Debug)]
pub struct SearchBlobByHashRequest {
    pub hash: String,
}

#[derive(Debug)]
pub enum SearchBlobByHashResponse {
    Found { blob_id: String, size: u64 },
    NotFound,
}

#[derive(Debug)]
pub struct GetOrCreateFilePathRequest {
    pub path: Vec<String>,
}

#[derive(Debug)]
pub struct GetOrCreateFilePathResponse {
    pub file_path_id: String,
    pub parent: Option<String>,
}

#[derive(Debug)]
pub enum SyncFilesToWorkspaceRequestItem {
    UpsertFile {
        file_path: Vec<String>,
        blob_id: String,
    },
    UpsertDir {
        file_path: Vec<String>,
    },
    DeleteFile {
        file_path: Vec<String>,
    },
    DeleteDir {
        file_path: Vec<String>,
    },
}

#[derive(Debug)]
pub struct SyncFilesToWorkspaceRequest {
    pub workspace_id: String,
    pub items: Vec<SyncFilesToWorkspaceRequestItem>,
}

#[derive(Debug)]
pub struct SyncFilesToWorkspaceResponse {}

#[derive(Debug)]
pub struct GetWorkspaceChangelistRequest {
    pub workspace_id: String,
}

#[derive(Debug)]
pub enum SagittaFileType {
    File,
    Dir,
}

#[derive(Debug)]
pub struct GetWorkspaceChangelistResponseItem {
    pub file_path: String,
    pub blob_id: Option<String>,
    pub deleted: bool,
    pub file_type: SagittaFileType,
}

#[derive(Debug)]
pub struct GetWorkspaceChangelistResponse {
    pub items: Vec<GetWorkspaceChangelistResponseItem>,
}

#[derive(Debug)]
pub struct CommitRequest {
    pub workspace_id: String,
}

#[derive(Debug)]
pub struct CommitResponse {}

#[derive(Debug)]
pub struct GetAllTrunkFilesRequest {}

#[derive(Debug)]
pub struct GetAllTrunkFilesResponseItem {
    pub file_path: String,
    pub blob_id: Option<String>,
    pub deleted: bool,
    pub file_type: SagittaFileType,
}

#[derive(Debug)]
pub struct GetAllTrunkFilesResponse {
    pub items: Vec<GetAllTrunkFilesResponseItem>,
}

#[derive(Debug)]
pub struct GetCommitHistoryRequest {
    pub take: u64,
}

#[derive(Debug)]
pub struct GetCommitHistoryResponseItem {
    pub commit_id: String,
    pub commit_rank: i64,
    pub created_at: SystemTime,
}

#[derive(Debug)]
pub struct GetCommitHistoryResponse {
    pub items: Vec<GetCommitHistoryResponseItem>,
}

#[derive(Debug)]
pub struct ReadDirRequest {
    pub workspace_id: Option<String>,
    pub file_path: Vec<String>,
    pub include_deleted: bool,
}

#[derive(Debug)]
pub struct ReadDirResponseItem {
    pub file_path: String,
    pub file_name: String,
    pub file_type: SagittaFileType,
    pub deleted_at: Option<SystemTime>,
}

#[derive(Debug)]
pub enum ReadDirResponse {
    Found { items: Vec<ReadDirResponseItem> },
    NotFound,
}

#[derive(Debug)]
pub enum SagittaRemoteSystemDBError {
    WorkspaceAlreadyExists,
    WorkspaceNotFound,
    InternalError,
}

pub trait SagittaRemoteSystemDB {
    fn migration(&self) -> Result<(), SagittaRemoteSystemDBError>;

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

    fn create_blob(
        &self,
        request: CreateBlobRequest,
    ) -> Result<CreateBlobResponse, SagittaRemoteSystemDBError>;

    fn search_blob_by_hash(
        &self,
        request: SearchBlobByHashRequest,
    ) -> Result<SearchBlobByHashResponse, SagittaRemoteSystemDBError>;

    fn get_or_create_file_path(
        &self,
        request: GetOrCreateFilePathRequest,
    ) -> Result<GetOrCreateFilePathResponse, SagittaRemoteSystemDBError>;

    fn sync_files_to_workspace(
        &self,
        sync_files_to_workspace_request: SyncFilesToWorkspaceRequest,
    ) -> Result<SyncFilesToWorkspaceResponse, SagittaRemoteSystemDBError>;

    fn get_workspace_changelist(
        &self,
        request: GetWorkspaceChangelistRequest,
    ) -> Result<GetWorkspaceChangelistResponse, SagittaRemoteSystemDBError>;

    fn commit(&self, request: CommitRequest) -> Result<CommitResponse, SagittaRemoteSystemDBError>;

    fn get_all_trunk_files(
        &self,
        request: GetAllTrunkFilesRequest,
    ) -> Result<GetAllTrunkFilesResponse, SagittaRemoteSystemDBError>;

    fn get_commit_history(
        &self,
        request: GetCommitHistoryRequest,
    ) -> Result<GetCommitHistoryResponse, SagittaRemoteSystemDBError>;

    fn read_dir(
        &self,
        request: ReadDirRequest,
    ) -> Result<ReadDirResponse, SagittaRemoteSystemDBError>;
}

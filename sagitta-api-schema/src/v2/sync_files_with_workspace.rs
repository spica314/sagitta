use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum V2SyncFilesWithWorkspaceRequestItem {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2SyncFilesWithWorkspaceRequest {
    pub workspace_id: String,
    pub items: Vec<V2SyncFilesWithWorkspaceRequestItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2SyncFilesWithWorkspaceResponse {}

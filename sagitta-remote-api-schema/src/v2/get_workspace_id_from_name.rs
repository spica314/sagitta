use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2GetWorkspaceIdFromNameRequest {
    pub workspace_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum V2GetWorkspaceIdFromNameResponse {
    Found { workspace_id: String },
    NotFound,
}

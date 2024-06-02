use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V1SyncRequest {
    pub workspace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum V1SyncResponse {
    Ok { upsert_files: Vec<Vec<String>> },
    Err,
}

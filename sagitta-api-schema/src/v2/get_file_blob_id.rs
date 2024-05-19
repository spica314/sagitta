use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2GetFileBlobIdRequest {
    pub path: Vec<String>,
    pub workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum V2GetFileBlobIdResponse {
    Found { blob_id: String },
    NotFound,
}

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2ReadDirRequest {
    pub path: Vec<String>,
    pub workspace_id: Option<String>,
    pub include_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2ReadDirResponseItem {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum V2ReadDirResponse {
    Found { items: Vec<V2ReadDirResponseItem> },
    NotFound,
}

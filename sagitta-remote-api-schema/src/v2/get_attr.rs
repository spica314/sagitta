use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2GetAttrRequest {
    pub path: Vec<String>,
    pub workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum V2GetAttrResponse {
    Found {
        is_dir: bool,
        size: u64,
        modified_at: SystemTime,
        permission: i64,
    },
    NotFound,
}

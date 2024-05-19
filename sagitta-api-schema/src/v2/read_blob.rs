use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2ReadBlobRequest {
    pub blob_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum V2ReadBlobResponse {
    Direct { blob: Vec<u8> },
    NotFound,
}

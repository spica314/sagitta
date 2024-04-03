use sagitta_objects::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobReadRequest {
    pub id: ObjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobReadResponse {
    pub blob: Vec<u8>,
}

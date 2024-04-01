use fuser::FileType;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectId {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagittaBlobObject {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagittaTreeObject {
    pub items: Vec<SagittaTreeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagittaTreeItem {
    pub name: String,
    pub object_id: ObjectId,

    // Metadata for FUSE
    pub size: u64,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
    pub kind: FileType,
    pub perm: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagittaCommitObject {
    pub tree_id: ObjectId,
    pub parent_commit_id: Option<ObjectId>,
    pub message: String,
}

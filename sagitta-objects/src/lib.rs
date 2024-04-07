use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectId {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SagittaTreeObject {
    File(SagittaTreeObjectFile),
    Dir(SagittaTreeObjectDir),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SagittaTreeObjectDir {
    pub items: Vec<(String, ObjectId)>,

    // Metadata for FUSE
    pub size: u64,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
    pub perm: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SagittaTreeObjectFile {
    pub blob_id: ObjectId,

    // Metadata for FUSE
    pub size: u64,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
    pub perm: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SagittaCommitObject {
    pub tree_id: ObjectId,
    pub parent_commit_id: Option<ObjectId>,
    pub message: String,
}

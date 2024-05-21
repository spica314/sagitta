use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2GetWorkspacesRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2GetWorkspacesResponseItem {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum V2GetWorkspacesResponse {
    Ok {
        items: Vec<V2GetWorkspacesResponseItem>,
    },
    Err,
}

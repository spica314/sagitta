use sagitta_objects::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrunkGetHeadRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrunkGetHeadResponse {
    pub id: ObjectId,
}

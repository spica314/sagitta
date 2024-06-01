use actix_web::{post, web, Responder};
use sagitta_remote_api_schema::v2::get_workspace_id_from_name::{
    V2GetWorkspaceIdFromNameRequest, V2GetWorkspaceIdFromNameResponse,
};
use sagitta_remote_system_db::{GetWorkspaceIdFromNameRequest, GetWorkspaceIdFromNameResponse};

use crate::state::ApiState;

#[post("/v2/get-workspace-id-from-name")]
pub async fn v2_get_workspace_id_from_name(
    state: web::Data<ApiState>,
    req: web::Json<V2GetWorkspaceIdFromNameRequest>,
) -> impl Responder {
    let request = GetWorkspaceIdFromNameRequest {
        workspace_name: req.workspace_name.clone(),
    };

    let res = state
        .remote_system_workspace_manager
        .get_workspace_id_from_name(request)
        .unwrap();

    match res {
        GetWorkspaceIdFromNameResponse::Found { workspace_id } => {
            web::Json(V2GetWorkspaceIdFromNameResponse::Found { workspace_id })
        }
        GetWorkspaceIdFromNameResponse::NotFound => {
            web::Json(V2GetWorkspaceIdFromNameResponse::NotFound)
        }
    }
}

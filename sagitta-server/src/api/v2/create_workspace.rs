use actix_web::{post, web, Responder};
use sagitta_api_schema::v2::create_workspace::{
    V2CreateWorkspaceRequest, V2CreateWorkspaceResponse,
};
use sagitta_remote_system_db::CreateWorkspaceRequest;

use crate::state::ApiState;

#[post("/v2/create-workspace")]
pub async fn v2_create_workspace(
    state: web::Data<ApiState>,
    req: web::Json<V2CreateWorkspaceRequest>,
) -> impl Responder {
    let request = CreateWorkspaceRequest {
        workspace_name: req.name.clone(),
    };

    let create_workspace_res = state
        .remote_system_workspace_manager
        .create_workspace(request)
        .unwrap();

    let res = V2CreateWorkspaceResponse::Ok {
        id: create_workspace_res.workspace_id,
    };

    web::Json(res)
}

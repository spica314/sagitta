use actix_web::{post, web, Responder};
use sagitta_api_schema::workspace::create::{WorkspaceCreateRequest, WorkspaceCreateResponse};

use crate::state::ApiState;

#[post("/workspace/create")]
pub async fn workspace_create(
    state: web::Data<ApiState>,
    req: web::Json<WorkspaceCreateRequest>,
) -> impl Responder {
    state
        .remote_system_workspace_manager
        .create_workspace(&req.name)
        .await
        .unwrap();
    let res = WorkspaceCreateResponse { ok: true };
    web::Json(res)
}

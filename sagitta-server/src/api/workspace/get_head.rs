use actix_web::{post, web, Responder};
use sagitta_api_schema::workspace::get_head::{WorkspaceGetHeadRequest, WorkspaceGetHeadResponse};

use crate::state::ApiState;

#[post("/workspace/get-head")]
pub async fn workspace_get_head(
    state: web::Data<ApiState>,
    req: web::Json<WorkspaceGetHeadRequest>,
) -> impl Responder {
    let res = state
        .remote_system_workspace_manager
        .get_head(Some(&req.workspace_id))
        .unwrap();
    let res = WorkspaceGetHeadResponse { id: res };
    web::Json(res)
}

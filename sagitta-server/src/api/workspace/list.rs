use actix_web::{post, web, Responder};
use sagitta_api_schema::workspace::list::{WorkspaceListRequest, WorkspaceListResponse};

use crate::state::ApiState;

#[post("/workspace/list")]
pub async fn workspace_list(
    state: web::Data<ApiState>,
    _req: web::Json<WorkspaceListRequest>,
) -> impl Responder {
    let workspaces = state
        .remote_system_workspace_manager
        .list_workspaces()
        .unwrap();
    let res = WorkspaceListResponse { workspaces };
    web::Json(res)
}

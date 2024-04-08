use actix_web::{post, web, Responder};
use sagitta_api_schema::workspace::list::{WorkspaceListRequest, WorkspaceListResponse};
use sagitta_objects_store::sagitta_objects_store::SagittaObjectsStore;

use crate::state::ApiState;

#[post("/workspace/list")]
pub async fn workspace_list(
    state: web::Data<ApiState>,
    _req: web::Json<WorkspaceListRequest>,
) -> impl Responder {
    let workspaces = state
        .server_files_manager
        .file_store
        .workspace_list()
        .unwrap();
    let res = WorkspaceListResponse { workspaces };
    web::Json(res)
}

use actix_web::{post, web, Responder};
use sagitta_api_schema::workspace::create::{WorkspaceCreateRequest, WorkspaceCreateResponse};
use sagitta_objects_store::sagitta_objects_store::SagittaObjectsStore;

use crate::state::ApiState;

#[post("/workspace/create")]
pub async fn workspace_create(
    state: web::Data<ApiState>,
    req: web::Json<WorkspaceCreateRequest>,
) -> impl Responder {
    state
        .server_files_manager
        .file_store
        .workspace_create(&req.name)
        .unwrap();
    let res = WorkspaceCreateResponse { ok: true };
    web::Json(res)
}

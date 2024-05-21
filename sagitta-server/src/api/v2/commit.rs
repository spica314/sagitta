use actix_web::{post, web, Responder};
use sagitta_remote_api_schema::v2::commit::{V2CommitRequest, V2CommitResponse};
use sagitta_remote_system_db::CommitRequest;

use crate::state::ApiState;

#[post("/v2/commit")]
pub async fn v2_commit(
    state: web::Data<ApiState>,
    req: web::Json<V2CommitRequest>,
) -> impl Responder {
    let request = CommitRequest {
        workspace_id: req.workspace_id.clone(),
    };

    let _commit_res = state
        .remote_system_workspace_manager
        .commit(request)
        .unwrap();

    let res = V2CommitResponse::Ok {};

    web::Json(res)
}

use actix_web::{post, web, Responder};
use sagitta_api_schema::blob::read::{BlobReadRequest, BlobReadResponse};

use crate::state::ApiState;

#[post("/blob/read")]
pub async fn blob_read(
    state: web::Data<ApiState>,
    req: web::Json<BlobReadRequest>,
) -> impl Responder {
    let res = state
        .remote_system_workspace_manager
        .get_object(None, &req.id)
        .unwrap();
    let res = BlobReadResponse { blob: res };
    web::Json(res)
}

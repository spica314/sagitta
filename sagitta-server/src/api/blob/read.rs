use actix_web::{post, web, Responder};
use sagitta_api_schema::blob::read::{BlobReadRequest, BlobReadResponse};

use crate::state::ApiState;

#[post("/blob/read")]
pub async fn blob_read(
    state: web::Data<ApiState>,
    req: web::Json<BlobReadRequest>,
) -> impl Responder {
    let res = state.server_files_manager.get_blob(&req.id);
    let res = BlobReadResponse { blob: res };
    web::Json(res)
}

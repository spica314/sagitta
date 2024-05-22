use actix_web::{post, web, Responder};
use sagitta_common::sha256::calc_sha256_from_slice;
use sagitta_remote_api_schema::v2::write_blob::{V2WriteBlobRequest, V2WriteBlobResponse};
use sagitta_remote_system_db::{CreateOrGetBlobRequest, CreateOrGetBlobResponse};
use sagitta_remote_system_workspace::WriteBlobRequest;

use crate::state::ApiState;

#[post("/v2/write-blob")]
pub async fn v2_write_blob(
    state: web::Data<ApiState>,
    req: web::Json<V2WriteBlobRequest>,
) -> impl Responder {
    let request = CreateOrGetBlobRequest {
        hash: calc_sha256_from_slice(&req.data),
        size: req.data.len() as u64,
    };

    let create_or_get_blob_res = state
        .remote_system_workspace_manager
        .create_or_get_blob(request)
        .unwrap();

    let res = match create_or_get_blob_res {
        CreateOrGetBlobResponse::Created { blob_id } => {
            state
                .remote_system_workspace_manager
                .write_blob(WriteBlobRequest {
                    blob: req.data.clone(),
                    blob_id: blob_id.clone(),
                })
                .unwrap();
            V2WriteBlobResponse { blob_id }
        }
        CreateOrGetBlobResponse::Found { blob_id } => V2WriteBlobResponse { blob_id },
    };

    web::Json(res)
}

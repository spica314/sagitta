use actix_web::{post, web, Responder};
use sagitta_api_schema::v2::get_file_blob_id::{V2GetFileBlobIdRequest, V2GetFileBlobIdResponse};
use sagitta_remote_system_db::{GetFileBlobIdRequest, GetFileBlobIdResponse};

use crate::state::ApiState;

#[post("/v2/get-file-blob-id")]
pub async fn v2_get_file_blob_id(
    state: web::Data<ApiState>,
    req: web::Json<V2GetFileBlobIdRequest>,
) -> impl Responder {
    let request = GetFileBlobIdRequest {
        workspace_id: req.workspace_id.clone(),
        file_path: req.path.clone(),
    };

    let get_file_blob_id_res = state
        .remote_system_workspace_manager
        .get_file_blob_id(request)
        .unwrap();

    let res = match get_file_blob_id_res {
        GetFileBlobIdResponse::Found { blob_id } => V2GetFileBlobIdResponse::Found { blob_id },
        GetFileBlobIdResponse::NotFound => V2GetFileBlobIdResponse::NotFound,
    };

    web::Json(res)
}

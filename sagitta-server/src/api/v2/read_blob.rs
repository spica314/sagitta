use actix_web::{post, web, Responder};
use sagitta_api_schema::v2::read_blob::{V2ReadBlobRequest, V2ReadBlobResponse};
use sagitta_remote_system_workspace::{ReadBlobRequest, ReadBlobResponse};

use crate::state::ApiState;

#[post("/v2/read-blob")]
pub async fn v2_read_blob(
    state: web::Data<ApiState>,
    req: web::Json<V2ReadBlobRequest>,
) -> impl Responder {
    let request = ReadBlobRequest {
        blob_id: req.blob_id.clone(),
    };

    let read_blob_res = state
        .remote_system_workspace_manager
        .read_blob(request)
        .unwrap();

    let res = match read_blob_res {
        ReadBlobResponse::Found { blob } => V2ReadBlobResponse::Direct { blob },
        ReadBlobResponse::NotFound => V2ReadBlobResponse::NotFound,
    };

    web::Json(res)
}

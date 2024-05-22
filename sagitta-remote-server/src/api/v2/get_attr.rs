use actix_web::{post, web, Responder};
use sagitta_remote_api_schema::v2::get_attr::{V2GetAttrRequest, V2GetAttrResponse};
use sagitta_remote_system_db::{GetAttrRequest, GetAttrResponse, SagittaFileType};

use crate::state::ApiState;

#[post("/v2/get-attr")]
pub async fn v2_get_attr(
    state: web::Data<ApiState>,
    req: web::Json<V2GetAttrRequest>,
) -> impl Responder {
    let request = GetAttrRequest {
        workspace_id: req.workspace_id.clone(),
        file_path: req.path.clone(),
    };

    let get_attr_res = state
        .remote_system_workspace_manager
        .get_attr(request)
        .unwrap();

    let res = match get_attr_res {
        GetAttrResponse::Found {
            file_type,
            size,
            modified_at,
        } => V2GetAttrResponse::Found {
            is_dir: file_type == SagittaFileType::Dir,
            size,
            modified_at,
        },
        GetAttrResponse::NotFound => V2GetAttrResponse::NotFound,
    };

    web::Json(res)
}

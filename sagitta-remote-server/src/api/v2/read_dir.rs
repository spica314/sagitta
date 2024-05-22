use actix_web::{post, web, Responder};
use sagitta_remote_api_schema::v2::read_dir::{
    V2ReadDirRequest, V2ReadDirResponse, V2ReadDirResponseItem,
};
use sagitta_remote_system_db::{ReadDirRequest, ReadDirResponse, SagittaFileType};

use crate::state::ApiState;

#[post("/v2/read-dir")]
pub async fn v2_read_dir(
    state: web::Data<ApiState>,
    req: web::Json<V2ReadDirRequest>,
) -> impl Responder {
    let request = ReadDirRequest {
        workspace_id: req.workspace_id.clone(),
        file_path: req.path.clone(),
        include_deleted: req.include_deleted,
    };

    let read_dir_res = state
        .remote_system_workspace_manager
        .read_dir(request)
        .unwrap();

    let res = match read_dir_res {
        ReadDirResponse::Found { items } => {
            let items = items
                .into_iter()
                .map(|item| V2ReadDirResponseItem {
                    name: item.file_name,
                    is_dir: item.file_type == SagittaFileType::Dir,
                    size: item.size,
                    modified_at: item.modified_at,
                })
                .collect();
            V2ReadDirResponse::Found { items }
        }
        ReadDirResponse::NotFound => V2ReadDirResponse::NotFound,
    };

    web::Json(res)
}

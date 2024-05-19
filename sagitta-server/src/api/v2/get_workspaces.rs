use actix_web::{post, web, Responder};
use sagitta_api_schema::v2::get_workspaces::{
    V2GetWorkspacesRequest, V2GetWorkspacesResponse, V2GetWorkspacesResponseItem,
};
use sagitta_remote_system_db::GetWorkspacesRequest;

use crate::state::ApiState;

#[post("/v2/get-workspaces")]
pub async fn v2_get_workspaces(
    state: web::Data<ApiState>,
    _req: web::Json<V2GetWorkspacesRequest>,
) -> impl Responder {
    let request = GetWorkspacesRequest {
        contains_deleted: false,
    };

    let get_workspaces_res = state
        .remote_system_workspace_manager
        .get_workspaces(request)
        .unwrap();

    let res = V2GetWorkspacesResponse::Ok {
        items: get_workspaces_res
            .workspaces
            .into_iter()
            .map(|workspace| V2GetWorkspacesResponseItem {
                id: workspace.workspace_id,
                name: workspace.workspace_name,
            })
            .collect(),
    };

    web::Json(res)
}

use actix_web::{post, web, Responder};
use sagitta_local_api_schema::v1::sync::{V1SyncRequest, V1SyncResponse};
use sagitta_remote_api_schema::v2::{
    sync_files_with_workspace::{
        V2SyncFilesWithWorkspaceRequest, V2SyncFilesWithWorkspaceRequestItem,
    },
    write_blob::V2WriteBlobRequest,
};

use crate::api_state::ApiState;

#[post("/v1/sync")]
pub async fn v1_sync(state: web::Data<ApiState>, req: web::Json<V1SyncRequest>) -> impl Responder {
    let res = V1SyncResponse {};

    let mut sync_request = V2SyncFilesWithWorkspaceRequest {
        workspace_id: req.workspace_id.clone(),
        items: vec![],
    };

    let paths = state
        .local_system_workspace
        .list_cow_files(&req.workspace_id)
        .unwrap();
    for path in &paths {
        let file = state
            .local_system_workspace
            .read_cow_file(&req.workspace_id, path, 0, 4_000_000_000)
            .unwrap();
        let res = state
            .remote_api_client
            .v2_write_blob(V2WriteBlobRequest { data: file })
            .unwrap();
        let blob_id = res.blob_id;

        let sync_item = V2SyncFilesWithWorkspaceRequestItem::UpsertFile {
            file_path: path.clone(),
            blob_id,
        };
        sync_request.items.push(sync_item);
    }

    let _sync_res = state
        .remote_api_client
        .v2_sync_files_with_workspace(sync_request)
        .unwrap();

    state
        .local_system_workspace
        .archive_cow_dir(&req.workspace_id)
        .unwrap();

    web::Json(res)
}

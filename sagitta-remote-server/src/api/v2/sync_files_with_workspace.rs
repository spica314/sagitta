use actix_web::{post, web, Responder};
use sagitta_remote_api_schema::v2::sync_files_with_workspace::{
    V2SyncFilesWithWorkspaceRequest, V2SyncFilesWithWorkspaceRequestItem,
    V2SyncFilesWithWorkspaceResponse,
};
use sagitta_remote_system_db::{SyncFilesToWorkspaceRequest, SyncFilesToWorkspaceRequestItem};

use crate::state::ApiState;

#[post("/v2/sync-files-with-workspace")]
pub async fn v2_sync_files_with_workspace(
    state: web::Data<ApiState>,
    req: web::Json<V2SyncFilesWithWorkspaceRequest>,
) -> impl Responder {
    let request = SyncFilesToWorkspaceRequest {
        workspace_id: req.workspace_id.clone(),
        items: req
            .items
            .iter()
            .map(|item| match item {
                V2SyncFilesWithWorkspaceRequestItem::UpsertFile { file_path, blob_id } => {
                    SyncFilesToWorkspaceRequestItem::UpsertFile {
                        file_path: file_path.clone(),
                        blob_id: blob_id.clone(),
                    }
                }
                V2SyncFilesWithWorkspaceRequestItem::UpsertDir { file_path } => {
                    SyncFilesToWorkspaceRequestItem::UpsertDir {
                        file_path: file_path.clone(),
                    }
                }
                V2SyncFilesWithWorkspaceRequestItem::DeleteFile { file_path } => {
                    SyncFilesToWorkspaceRequestItem::DeleteFile {
                        file_path: file_path.clone(),
                    }
                }
                V2SyncFilesWithWorkspaceRequestItem::DeleteDir { file_path } => {
                    SyncFilesToWorkspaceRequestItem::DeleteDir {
                        file_path: file_path.clone(),
                    }
                }
            })
            .collect(),
    };

    let _res = state
        .remote_system_workspace_manager
        .sync_files_to_workspace(request)
        .unwrap();

    web::Json(V2SyncFilesWithWorkspaceResponse {})
}

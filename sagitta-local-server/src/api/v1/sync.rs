use std::collections::HashMap;

use actix_web::{post, web, Responder};
use log::debug;
use sagitta_config_file::SagittaConfigToml;
use sagitta_local_api_schema::v1::sync::{V1SyncRequest, V1SyncResponse};
use sagitta_remote_api_schema::v2::{
    get_file_blob_id::{V2GetFileBlobIdRequest, V2GetFileBlobIdResponse},
    read_blob::{V2ReadBlobRequest, V2ReadBlobResponse},
    sync_files_with_workspace::{
        V2SyncFilesWithWorkspaceRequest, V2SyncFilesWithWorkspaceRequestItem,
    },
    write_blob::V2WriteBlobRequest,
};

use crate::api_state::ApiState;

#[post("/v1/sync")]
pub async fn v1_sync(state: web::Data<ApiState>, req: web::Json<V1SyncRequest>) -> impl Responder {
    let workspace_id = req.workspace_id.clone();

    let mut sync_request = V2SyncFilesWithWorkspaceRequest {
        workspace_id: workspace_id.clone(),
        items: vec![],
    };

    let paths = state
        .local_system_workspace
        .list_cow_files(&req.workspace_id)
        .unwrap();

    let mut config_cache = HashMap::new();

    let mut upsert_files = vec![];
    for path in &paths {
        // retrieve config files
        for i in 0..path.len() {
            let path = &path[0..i];
            if !config_cache.contains_key(path) {
                let mut config_path = path.to_vec();
                config_path.push(".sagitta.toml".to_string());

                // read cow file
                let file = state.local_system_workspace.read_cow_file(
                    &req.workspace_id,
                    &config_path,
                    0,
                    4_000_000_000,
                );
                if let Ok(file) = file {
                    let toml = std::str::from_utf8(&file).unwrap();
                    let config: SagittaConfigToml = toml::from_str(toml).unwrap();
                    config_cache.insert(path.to_vec(), Some(config));
                    continue;
                }

                // read commited or synced file
                let file = state
                    .remote_api_client
                    .v2_get_file_blob_id(V2GetFileBlobIdRequest {
                        workspace_id: Some(workspace_id.clone()),
                        path: config_path.clone(),
                    })
                    .unwrap();
                match file {
                    V2GetFileBlobIdResponse::Found { blob_id } => {
                        let blob = state
                            .remote_api_client
                            .v2_read_blob_request(V2ReadBlobRequest {
                                blob_id: blob_id.clone(),
                            })
                            .unwrap();
                        match blob {
                            V2ReadBlobResponse::Direct { blob } => {
                                let toml = std::str::from_utf8(&blob).unwrap();
                                let config: SagittaConfigToml = toml::from_str(toml).unwrap();
                                config_cache.insert(path.to_vec(), Some(config));
                            }
                            V2ReadBlobResponse::NotFound => {
                                config_cache.insert(path.to_vec(), None);
                            }
                        }
                    }
                    V2GetFileBlobIdResponse::NotFound => {
                        config_cache.insert(path.to_vec(), None);
                    }
                }
            }
        }

        // check if the file is ignored
        let mut ignored = false;
        for i in 0..path.len() {
            let config_dir_path = &path[0..i];
            let config_target_path = &path[i..];
            if let Some(Some(config)) = config_cache.get(config_dir_path) {
                for ignore in &config.ignores {
                    for config_target_path_chunk in config_target_path {
                        if ignore == config_target_path_chunk {
                            ignored = true;
                            continue;
                        }
                    }
                }
            }
        }
        if ignored {
            debug!("ignored: {:?}", path);
            continue;
        }

        // sync
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
        upsert_files.push(path.clone());
    }

    let _sync_res = state
        .remote_api_client
        .v2_sync_files_with_workspace(sync_request)
        .unwrap();

    state
        .local_system_workspace
        .archive_cow_dir(&req.workspace_id, &upsert_files)
        .unwrap();

    upsert_files.sort();

    web::Json(V1SyncResponse::Ok { upsert_files })
}

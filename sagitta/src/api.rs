use std::path::PathBuf;

use actix_web::{web, App, HttpServer};
use sagitta_objects::{SagittaTreeObject, SagittaTreeObjectFile};
use sagitta_objects_store::sagitta_objects_store::SagittaObjectsStore;

use crate::state::ApiState;

use self::{blob::read::blob_read, trunk::get_head::trunk_get_head};
use crate::tools::Clock;

pub mod blob;
pub mod trunk;

pub struct ServerConfig {
    pub base_path: PathBuf,
    pub clock: Clock,
}

pub async fn run_server(config: ServerConfig) {
    let state = ApiState::new(config.base_path.clone(), config.clock.clone());
    let blob_id = state
        .server_files_manager
        .file_store
        .save_blob(b"Hello, world!\n".as_slice())
        .unwrap();
    let file = SagittaTreeObject::File(SagittaTreeObjectFile {
        name: "hello.txt".to_string(),
        blob_id,
        size: 14,
        mtime: config.clock.now(),
        ctime: config.clock.now(),
        perm: 0o644,
    });
    let file_id = state
        .server_files_manager
        .file_store
        .save_tree(&file)
        .unwrap();
    let tree = SagittaTreeObject::Dir(sagitta_objects::SagittaTreeObjectDir {
        items: vec![file_id],
        size: 4096,
        mtime: config.clock.now(),
        ctime: config.clock.now(),
        perm: 0o755,
    });
    let tree_id = state
        .server_files_manager
        .file_store
        .save_tree(&tree)
        .unwrap();
    let commit = sagitta_objects::SagittaCommitObject {
        tree_id,
        parent_commit_id: None,
        message: "Initial commit".to_string(),
    };
    let commit_id = state
        .server_files_manager
        .file_store
        .save_commit(&commit)
        .unwrap();
    state
        .server_files_manager
        .file_store
        .update_trunk_head(&commit_id)
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(blob_read)
            .service(trunk_get_head)
    })
    .bind(("0.0.0.0", 8081))
    .unwrap()
    .run()
    .await
    .unwrap();
}

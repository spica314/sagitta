use std::path::PathBuf;

use actix_web::{web, App, HttpServer};
use sagitta_common::clock::Clock;
use sagitta_objects::{SagittaTreeObject, SagittaTreeObjectFile};
use sagitta_objects_store::sagitta_objects_store::SagittaObjectsStore;

use crate::state::ApiState;

use self::{blob::read::blob_read, trunk::get_head::trunk_get_head};

pub mod blob;
pub mod trunk;

pub struct ServerConfig {
    pub base_path: PathBuf,
    pub clock: Clock,
    pub port: u16,
}

pub async fn run_server(config: ServerConfig) {
    let state = ApiState::new(config.base_path.clone(), config.clock.clone());

    // root (dir1)
    // - hello.txt (file1)
    // - hello_dir (dir2)
    //     - hello2.txt (file2)

    let file1_blob_id = state
        .server_files_manager
        .file_store
        .save_blob(None, b"Hello, world!\n".as_slice())
        .unwrap();
    let file1 = SagittaTreeObject::File(SagittaTreeObjectFile {
        blob_id: file1_blob_id,
        size: 14,
        mtime: config.clock.now(),
        ctime: config.clock.now(),
        perm: 0o644,
    });
    let file1_id = state
        .server_files_manager
        .file_store
        .save_tree(None, &file1)
        .unwrap();

    let file2_blob_id = state
        .server_files_manager
        .file_store
        .save_blob(None, b"Hello, world!!\n".as_slice())
        .unwrap();
    let file2 = SagittaTreeObject::File(SagittaTreeObjectFile {
        blob_id: file2_blob_id,
        size: 15,
        mtime: config.clock.now(),
        ctime: config.clock.now(),
        perm: 0o644,
    });
    let file2_id = state
        .server_files_manager
        .file_store
        .save_tree(None, &file2)
        .unwrap();

    let tree2 = SagittaTreeObject::Dir(sagitta_objects::SagittaTreeObjectDir {
        items: vec![("hello2.txt".to_string(), file2_id)],
        size: 4096,
        mtime: config.clock.now(),
        ctime: config.clock.now(),
        perm: 0o755,
    });
    let tree2_id = state
        .server_files_manager
        .file_store
        .save_tree(None, &tree2)
        .unwrap();

    let tree1 = SagittaTreeObject::Dir(sagitta_objects::SagittaTreeObjectDir {
        items: vec![
            ("hello.txt".to_string(), file1_id),
            ("hello_dir".to_string(), tree2_id),
        ],
        size: 4096,
        mtime: config.clock.now(),
        ctime: config.clock.now(),
        perm: 0o755,
    });
    let tree1_id = state
        .server_files_manager
        .file_store
        .save_tree(None, &tree1)
        .unwrap();

    let commit = sagitta_objects::SagittaCommitObject {
        tree_id: tree1_id,
        parent_commit_id: None,
        message: "Initial commit".to_string(),
    };
    let commit_id = state
        .server_files_manager
        .file_store
        .save_commit(None, &commit)
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
    .bind(("0.0.0.0", config.port))
    .unwrap()
    .run()
    .await
    .unwrap();
}

use std::path::PathBuf;

use actix_web::{web, App, HttpServer};
use rand::{thread_rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sagitta_common::clock::Clock;
use sagitta_remote_system_db::SagittaRemoteSystemDBTrait;
use sagitta_remote_system_db::{db::SagittaRemoteSystemDB, sqlite::SagittaRemoteSystemDBBySqlite};
use tempfile::NamedTempFile;

use crate::state::ApiState;

use crate::api::v2::read_dir::*;

use self::v2::commit::v2_commit;
use self::v2::create_workspace::v2_create_workspace;
use self::v2::get_attr::v2_get_attr;
use self::v2::get_file_blob_id::v2_get_file_blob_id;
use self::v2::get_workspaces::v2_get_workspaces;
use self::v2::read_blob::v2_read_blob;
use self::v2::sync_files_with_workspace::v2_sync_files_with_workspace;
use self::v2::write_blob::v2_write_blob;

pub mod v2;

pub struct ServerConfig {
    pub base_path: PathBuf,
    pub is_main: bool,
    pub clock: Clock,
    pub port: u16,
}

pub async fn run_server(config: ServerConfig) {
    let rng_crypto = if config.is_main {
        let rng = thread_rng();
        ChaCha20Rng::from_rng(rng).unwrap()
    } else {
        ChaCha20Rng::from_seed([0; 32])
    };

    let file = NamedTempFile::new().unwrap();
    let path = file.into_temp_path();
    let path = path.to_path_buf();

    let sqlite_path = {
        if config.is_main {
            "./sagitta.sqlite"
        } else {
            path.to_str().unwrap()
        }
    };

    let db =
        SagittaRemoteSystemDBBySqlite::new(sqlite_path, rng_crypto, config.clock.clone()).unwrap();
    let db = SagittaRemoteSystemDB::Sqlite(db);
    db.migration().unwrap();

    let state = ApiState::new(config.base_path.clone(), config.clock.clone(), db).await;

    // root (dir1)
    // - hello.txt (file1)
    // - hello_dir (dir2)
    //     - hello2.txt (file2)

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(v2_read_dir)
            .service(v2_get_attr)
            .service(v2_get_file_blob_id)
            .service(v2_read_blob)
            .service(v2_get_workspaces)
            .service(v2_create_workspace)
            .service(v2_write_blob)
            .service(v2_sync_files_with_workspace)
            .service(v2_commit)
    })
    .bind(("0.0.0.0", config.port))
    .unwrap()
    .run()
    .await
    .unwrap();
}

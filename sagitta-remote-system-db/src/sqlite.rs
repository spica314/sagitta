use std::{
    collections::HashSet,
    path::Path,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use rand::RngCore;
use sagitta_common::clock::Clock;

use crate::*;

use base64::{engine::general_purpose::URL_SAFE, Engine};

pub struct SagittaRemoteSystemDBBySqlite<Rng: RngCore> {
    db: Arc<Mutex<rusqlite::Connection>>,
    rng: Arc<Mutex<Rng>>,
    clock: Clock,
}

impl<Rng: RngCore> SagittaRemoteSystemDBBySqlite<Rng> {
    pub fn new<P: AsRef<Path>>(
        sqlite_path: P,
        rng: Rng,
        clock: Clock,
    ) -> Result<Self, rusqlite::Error> {
        let db = rusqlite::Connection::open(sqlite_path)?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            rng: Arc::new(Mutex::new(rng)),
            clock,
        })
    }

    fn generate_id(&self) -> String {
        let mut id = [0u8; 384 / 8];
        let mut rng = self.rng.lock().unwrap();
        rng.fill_bytes(&mut id);

        URL_SAFE.encode(id)
    }

    fn get_or_create_file_path_tx(
        &self,
        request: GetOrCreateFilePathRequest,
        tx: &mut rusqlite::Transaction,
    ) -> Result<GetOrCreateFilePathResponse, SagittaRemoteSystemDBError> {
        let res = {
            let mut stmt = tx
                .prepare("SELECT file_path_id, parent FROM file_path WHERE path = ?")
                .unwrap();
            stmt.query_row(rusqlite::params![request.path.join("/")], |row| {
                Ok(GetOrCreateFilePathResponse {
                    file_path_id: row.get(0)?,
                    parent: row.get(1)?,
                })
            })
        };

        match res {
            Ok(x) => Ok(x),
            Err(_) => {
                let parent = if request.path.len() == 1 {
                    None
                } else {
                    let parent = self
                        .get_or_create_file_path_tx(
                            GetOrCreateFilePathRequest {
                                path: request.path[..request.path.len() - 1].to_vec(),
                            },
                            tx,
                        )
                        .unwrap();
                    Some(parent.file_path_id)
                };
                let id = self.generate_id();
                tx.execute(
                    "INSERT INTO file_path (file_path_id, name, path, parent) VALUES (?, ?, ?, ?)",
                    rusqlite::params![
                        id,
                        request.path.last().unwrap(),
                        request.path.join("/"),
                        parent
                    ],
                )
                .unwrap();

                Ok(GetOrCreateFilePathResponse {
                    file_path_id: id,
                    parent,
                })
            }
        }
    }
}

impl<Rng: RngCore> SagittaRemoteSystemDB for SagittaRemoteSystemDBBySqlite<Rng> {
    fn migration(&self) -> Result<(), SagittaRemoteSystemDBError> {
        let db = self.db.lock().unwrap();
        db.execute(
            "CREATE TABLE IF NOT EXISTS workspace (
                workspace_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                deleted_at TEXT
            )",
            rusqlite::params![],
        )
        .unwrap();

        db.execute(
            "CREATE TABLE IF NOT EXISTS blob (
                blob_id TEXT PRIMARY KEY,
                hash TEXT NOT NULL,
                size INTEGER NOT NULL
            )",
            rusqlite::params![],
        )
        .unwrap();

        db.execute(
            "CREATE TABLE IF NOT EXISTS file_path (
                file_path_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                parent TEXT
            )",
            rusqlite::params![],
        )
        .unwrap();

        db.execute(
            "CREATE TABLE IF NOT EXISTS workspace_file_revision (
                workspace_file_revision_id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                file_path_id TEXT NOT NULL,
                sync_version_number INTEGER NOT NULL,
                blob_id TEXT,
                file_type INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                deleted_at TEXT
            )",
            rusqlite::params![],
        )
        .unwrap();

        Ok(())
    }

    fn create_workspace(
        &self,
        request: CreateWorkspaceRequest,
    ) -> Result<CreateWorkspaceResponse, SagittaRemoteSystemDBError> {
        let id = self.generate_id();
        let now = self.clock.now();
        let now: DateTime<Utc> = now.into();
        let now_str = now.to_rfc3339();

        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT INTO workspace (workspace_id, name, created_at) VALUES (?, ?, ?)",
            rusqlite::params![id, request.workspace_name, now_str],
        )
        .unwrap();

        Ok(CreateWorkspaceResponse { workspace_id: id })
    }

    fn get_workspaces(
        &self,
        request: GetWorkspacesRequest,
    ) -> Result<GetWorkspacesResponse, SagittaRemoteSystemDBError> {
        let db = self.db.lock().unwrap();

        if request.contains_deleted {
            let mut stmt = db
                .prepare("SELECT workspace_id, name, created_at, deleted_at FROM workspace")
                .unwrap();
            let workspaces = stmt
                .query_map(rusqlite::params![], |row| {
                    let created_at: String = row.get(2)?;
                    let deleted_at: Option<String> = row.get(3)?;
                    Ok(GetWorkspacesResponseItem {
                        workspace_id: row.get(0)?,
                        workspace_name: row.get(1)?,
                        created_at: DateTime::parse_from_rfc3339(&created_at).unwrap().into(),
                        deleted_at: deleted_at
                            .map(|x| DateTime::parse_from_rfc3339(&x).unwrap().into()),
                    })
                })
                .unwrap()
                .map(|x| x.unwrap())
                .collect();

            Ok(GetWorkspacesResponse { workspaces })
        } else {
            let mut stmt = db.prepare("SELECT workspace_id, name, created_at, deleted_at FROM workspace WHERE deleted_at IS NULL").unwrap();
            let workspaces = stmt
                .query_map(rusqlite::params![], |row| {
                    let created_at: String = row.get(2)?;
                    let deleted_at: Option<String> = row.get(3)?;
                    Ok(GetWorkspacesResponseItem {
                        workspace_id: row.get(0)?,
                        workspace_name: row.get(1)?,
                        created_at: DateTime::parse_from_rfc3339(&created_at).unwrap().into(),
                        deleted_at: deleted_at
                            .map(|x| DateTime::parse_from_rfc3339(&x).unwrap().into()),
                    })
                })
                .unwrap()
                .map(|x| x.unwrap())
                .collect();

            Ok(GetWorkspacesResponse { workspaces })
        }
    }

    fn delete_workspace(
        &self,
        request: DeleteWorkspaceRequest,
    ) -> Result<DeleteWorkspaceResponse, SagittaRemoteSystemDBError> {
        let now = self.clock.now();
        let now: DateTime<Utc> = now.into();
        let now_str = now.to_rfc3339();

        let db = self.db.lock().unwrap();

        let affected = db
            .execute(
                "UPDATE workspace SET deleted_at = ? WHERE workspace_id = ?",
                rusqlite::params![now_str, request.workspace_id],
            )
            .unwrap();

        if affected == 0 {
            return Err(SagittaRemoteSystemDBError::WorkspaceNotFound);
        }

        Ok(DeleteWorkspaceResponse {})
    }

    fn create_blob(
        &self,
        request: CreateBlobRequest,
    ) -> Result<CreateBlobResponse, SagittaRemoteSystemDBError> {
        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT INTO blob (blob_id, hash, size) VALUES (?, ?, ?)",
            rusqlite::params![request.blob_id, request.hash, request.size],
        )
        .unwrap();

        Ok(CreateBlobResponse {})
    }

    fn search_blob_by_hash(
        &self,
        request: SearchBlobByHashRequest,
    ) -> Result<SearchBlobByHashResponse, SagittaRemoteSystemDBError> {
        let db = self.db.lock().unwrap();

        let mut stmt = db
            .prepare("SELECT blob_id, size FROM blob WHERE hash = ?")
            .unwrap();
        let res = stmt.query_row(rusqlite::params![request.hash], |row| {
            Ok(SearchBlobByHashResponse::Found {
                blob_id: row.get(0)?,
                size: row.get(1)?,
            })
        });

        match res {
            Ok(x) => Ok(x),
            Err(_) => Ok(SearchBlobByHashResponse::NotFound),
        }
    }

    fn get_or_create_file_path(
        &self,
        request: GetOrCreateFilePathRequest,
    ) -> Result<GetOrCreateFilePathResponse, SagittaRemoteSystemDBError> {
        let mut db = self.db.lock().unwrap();
        let mut tx = db.transaction().unwrap();

        let res = self.get_or_create_file_path_tx(request, &mut tx);

        tx.commit().unwrap();

        res
    }

    fn sync_files_to_workspace(
        &self,
        request: SyncFilesToWorkspaceRequest,
    ) -> Result<SyncFilesToWorkspaceResponse, SagittaRemoteSystemDBError> {
        let now = self.clock.now();
        let now: DateTime<Utc> = now.into();
        let now_str = now.to_rfc3339();

        let mut db = self.db.lock().unwrap();
        let mut tx = db.transaction().unwrap();

        let version_number = tx
            .query_row(
                "SELECT MAX(sync_version_number) FROM workspace_file_revision WHERE workspace_id = ?",
                rusqlite::params![request.workspace_id],
                |row| row.get(0),
            )
            .unwrap_or(0) + 1;

        let mut inserted = HashSet::new();

        for item in request.items {
            match item {
                SyncFilesToWorkspaceRequestItem::UpsertFile { file_path, blob_id } => {
                    for i in 1..file_path.len() {
                        let file_path = self
                            .get_or_create_file_path_tx(
                                GetOrCreateFilePathRequest {
                                    path: file_path[..i].to_vec(),
                                },
                                &mut tx,
                            )
                            .unwrap();

                        if inserted.contains(&file_path.file_path_id) {
                            continue;
                        }
                        inserted.insert(file_path.file_path_id.clone());

                        tx
                            .execute(
                                "INSERT INTO workspace_file_revision (workspace_file_revision_id, workspace_id, file_path_id, sync_version_number, file_type, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                                rusqlite::params![self.generate_id(), request.workspace_id, file_path.file_path_id, version_number, 1, now_str],
                            )
                            .unwrap();
                    }

                    let file_path = self
                        .get_or_create_file_path_tx(
                            GetOrCreateFilePathRequest { path: file_path },
                            &mut tx,
                        )
                        .unwrap();
                    tx
                        .execute(
                            "INSERT INTO workspace_file_revision (workspace_file_revision_id, workspace_id, file_path_id, sync_version_number, blob_id, file_type, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                            rusqlite::params![self.generate_id(), request.workspace_id, file_path.file_path_id, version_number, blob_id, 0, now_str],
                        )
                        .unwrap();
                }
                SyncFilesToWorkspaceRequestItem::UpsertDir { file_path } => {
                    for i in 1..=file_path.len() {
                        let file_path = self
                            .get_or_create_file_path_tx(
                                GetOrCreateFilePathRequest {
                                    path: file_path[..i].to_vec(),
                                },
                                &mut tx,
                            )
                            .unwrap();

                        if inserted.contains(&file_path.file_path_id) {
                            continue;
                        }
                        inserted.insert(file_path.file_path_id.clone());

                        tx
                            .execute(
                                "INSERT INTO workspace_file_revision (workspace_file_revision_id, workspace_id, file_path_id, sync_version_number, file_type, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                                rusqlite::params![self.generate_id(), request.workspace_id, file_path.file_path_id, version_number, 1, now_str],
                            )
                            .unwrap();
                    }
                }
                SyncFilesToWorkspaceRequestItem::DeleteFile { file_path } => {
                    for i in 1..file_path.len() {
                        let file_path = self
                            .get_or_create_file_path_tx(
                                GetOrCreateFilePathRequest {
                                    path: file_path[..i].to_vec(),
                                },
                                &mut tx,
                            )
                            .unwrap();

                        if inserted.contains(&file_path.file_path_id) {
                            continue;
                        }
                        inserted.insert(file_path.file_path_id.clone());

                        tx
                            .execute(
                                "INSERT INTO workspace_file_revision (workspace_file_revision_id, workspace_id, file_path_id, sync_version_number, file_type, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                                rusqlite::params![self.generate_id(), request.workspace_id, file_path.file_path_id, version_number, 1, now_str],
                            )
                            .unwrap();
                    }

                    let file_path = self
                        .get_or_create_file_path_tx(
                            GetOrCreateFilePathRequest { path: file_path },
                            &mut tx,
                        )
                        .unwrap();
                    tx
                        .execute(
                            "INSERT INTO workspace_file_revision (workspace_file_revision_id, workspace_id, file_path_id, sync_version_number, file_type, created_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                            rusqlite::params![self.generate_id(), request.workspace_id, file_path.file_path_id, version_number, 0, now_str, now_str],
                        )
                        .unwrap();
                }
                SyncFilesToWorkspaceRequestItem::DeleteDir { file_path } => {
                    for i in 1..file_path.len() {
                        let file_path = self
                            .get_or_create_file_path_tx(
                                GetOrCreateFilePathRequest {
                                    path: file_path[..i].to_vec(),
                                },
                                &mut tx,
                            )
                            .unwrap();

                        if inserted.contains(&file_path.file_path_id) {
                            continue;
                        }
                        inserted.insert(file_path.file_path_id.clone());

                        tx
                            .execute(
                                "INSERT INTO workspace_file_revision (workspace_file_revision_id, workspace_id, file_path_id, sync_version_number, file_type, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                                rusqlite::params![self.generate_id(), request.workspace_id, file_path.file_path_id, version_number, 1, now_str],
                            )
                            .unwrap();
                    }

                    let file_path = self
                        .get_or_create_file_path_tx(
                            GetOrCreateFilePathRequest { path: file_path },
                            &mut tx,
                        )
                        .unwrap();

                    if inserted.contains(&file_path.file_path_id) {
                        continue;
                    }
                    inserted.insert(file_path.file_path_id.clone());

                    tx
                        .execute(
                            "INSERT INTO workspace_file_revision (workspace_file_revision_id, workspace_id, file_path_id, sync_version_number, file_type, created_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                            rusqlite::params![self.generate_id(), request.workspace_id, file_path.file_path_id, version_number, 1, now_str, now_str],
                        )
                        .unwrap();
                }
            }
        }

        tx.commit().unwrap();

        Ok(SyncFilesToWorkspaceResponse {})
    }

    fn get_workspace_changelist(
        &self,
        request: GetWorkspaceChangelistRequest,
    ) -> Result<GetWorkspaceChangelistResponse, SagittaRemoteSystemDBError> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().unwrap();

        let res = {
            let mut stmt = tx
                .prepare(
                    "SELECT file_path.path, workspace_file_revision.blob_id, workspace_file_revision.deleted_at, workspace_file_revision.file_type FROM workspace_file_revision
                    JOIN (
                        SELECT file_path_id, MAX(sync_version_number) AS sync_version_number
                        FROM workspace_file_revision AS workspace_file_revision_2
                        WHERE workspace_file_revision_2.workspace_id = ?
                        GROUP BY workspace_file_revision_2.file_path_id
                    ) AS latest_sync_version
                    ON workspace_file_revision.file_path_id = latest_sync_version.file_path_id AND workspace_file_revision.sync_version_number = latest_sync_version.sync_version_number
                    JOIN file_path ON workspace_file_revision.file_path_id = file_path.file_path_id
                    WHERE workspace_id = ?",
                )
                .unwrap();
            stmt.query_map(
                rusqlite::params![request.workspace_id, request.workspace_id],
                |row| {
                    let deleted_at: Option<String> = row.get(2)?;
                    let file_type: i64 = row.get(3)?;
                    Ok(GetWorkspaceChangelistResponseItem {
                        file_path: row.get(0)?,
                        blob_id: row.get(1)?,
                        deleted: deleted_at.is_some(),
                        file_type: match file_type {
                            0 => SagittaFileType::File,
                            1 => SagittaFileType::Dir,
                            _ => unreachable!(),
                        },
                    })
                },
            )
            .unwrap()
            .map(|x| x.unwrap())
            .collect()
        };

        tx.commit().unwrap();

        Ok(GetWorkspaceChangelistResponse { items: res })
    }
}

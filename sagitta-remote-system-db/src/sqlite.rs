use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use rand::RngCore;
use rand_chacha::ChaCha20Rng;
use sagitta_common::clock::Clock;

use crate::*;

use base64::{engine::general_purpose::URL_SAFE, Engine};

#[derive(Debug, Clone)]
pub struct SagittaRemoteSystemDBBySqlite {
    db: Arc<Mutex<rusqlite::Connection>>,
    rng: Arc<Mutex<ChaCha20Rng>>,
    clock: Clock,
}

impl SagittaRemoteSystemDBBySqlite {
    pub fn new<P: AsRef<Path>>(
        sqlite_path: P,
        rng: ChaCha20Rng,
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
                if request.path.is_empty() {
                    return Err(SagittaRemoteSystemDBError::InternalError);
                }
                let parent = {
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

impl SagittaRemoteSystemDBTrait for SagittaRemoteSystemDBBySqlite {
    fn migration(&self) -> Result<(), SagittaRemoteSystemDBError> {
        let mut db = self.db.lock().unwrap();
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

        db.execute(
            "CREATE TABLE IF NOT EXISTS trunk_file_revision (
                trunk_file_revision_id TEXT PRIMARY KEY,
                file_path_id TEXT NOT NULL,
                commit_id TEXT NOT NULL,
                commit_rank INTEGER NOT NULL,
                blob_id TEXT,
                file_type INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                deleted_at TEXT
            )",
            rusqlite::params![],
        )
        .unwrap();

        db.execute(
            "CREATE TABLE IF NOT EXISTS `commit` (
                commit_id TEXT PRIMARY KEY,
                commit_rank INTEGER NOT NULL,
                created_at TEXT NOT NULL
            )",
            rusqlite::params![],
        )
        .unwrap();

        // create initial commit
        {
            let tx = db.transaction().unwrap();

            let count: i64 = {
                let mut stmt = tx.prepare("SELECT COUNT(*) FROM `commit`").unwrap();
                stmt.query_row(rusqlite::params![], |row| row.get(0))
                    .unwrap()
            };
            if count == 0 {
                let now = self.clock.now();
                let now: DateTime<Utc> = now.into();
                let now_str = now.to_rfc3339();
                tx.execute(
                    "INSERT INTO `commit` (commit_id, commit_rank, created_at) VALUES (?, ?, ?)",
                    rusqlite::params![self.generate_id(), 0, now_str],
                )
                .unwrap();
            }

            tx.commit().unwrap();
        }

        // create root path
        {
            let tx = db.transaction().unwrap();

            let count: i64 = {
                let mut stmt = tx
                    .prepare("SELECT COUNT(*) FROM file_path WHERE path = ''")
                    .unwrap();
                stmt.query_row(rusqlite::params![], |row| row.get(0))
                    .unwrap()
            };
            if count == 0 {
                tx.execute(
                    "INSERT INTO file_path (file_path_id, name, path) VALUES (?, '', '')",
                    rusqlite::params![self.generate_id()],
                )
                .unwrap();
            }

            tx.commit().unwrap();
        }

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

    fn create_or_get_blob(
        &self,
        request: CreateOrGetBlobRequest,
    ) -> Result<CreateOrGetBlobResponse, SagittaRemoteSystemDBError> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().unwrap();

        let blob_id = {
            let res = {
                let mut stmt = tx
                    .prepare("SELECT blob_id FROM blob WHERE hash = ?")
                    .unwrap();
                stmt.query_row(rusqlite::params![request.hash], |row| row.get(0))
            };

            match res {
                Ok(x) => {
                    tx.commit().unwrap();
                    return Ok(CreateOrGetBlobResponse::Found { blob_id: x });
                }
                Err(_) => self.generate_id(),
            }
        };

        tx.execute(
            "INSERT INTO blob (blob_id, hash, size) VALUES (?, ?, ?)",
            rusqlite::params![blob_id, request.hash, request.size],
        )
        .unwrap();

        tx.commit().unwrap();
        Ok(CreateOrGetBlobResponse::Created { blob_id })
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

    fn commit(&self, request: CommitRequest) -> Result<CommitResponse, SagittaRemoteSystemDBError> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().unwrap();

        let now = self.clock.now();
        let now: DateTime<Utc> = now.into();
        let now_str = now.to_rfc3339();

        let commit_id = self.generate_id();
        let commit_rank = tx
            .query_row(
                "SELECT MAX(commit_rank) FROM trunk_file_revision",
                rusqlite::params![],
                |row| row.get(0),
            )
            .unwrap_or(0)
            + 1;

        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO trunk_file_revision (
                        trunk_file_revision_id,
                        file_path_id,
                        commit_id,
                        commit_rank,
                        blob_id,
                        file_type,
                        created_at,
                        deleted_at
                    )
                    SELECT
                        workspace_file_revision_id,
                        workspace_file_revision.file_path_id,
                        ?,
                        ?,
                        blob_id,
                        file_type,
                        created_at,
                        deleted_at
                    FROM workspace_file_revision
                    JOIN (
                        SELECT file_path_id, MAX(sync_version_number) AS sync_version_number
                        FROM workspace_file_revision AS workspace_file_revision_2
                        WHERE workspace_file_revision_2.workspace_id = ?
                        GROUP BY workspace_file_revision_2.file_path_id
                    ) AS latest_sync_version
                    ON workspace_file_revision.file_path_id = latest_sync_version.file_path_id AND workspace_file_revision.sync_version_number = latest_sync_version.sync_version_number
                    WHERE workspace_file_revision.workspace_id = ?",
                )
                .unwrap();
            stmt.execute(rusqlite::params![
                commit_id,
                commit_rank,
                request.workspace_id,
                request.workspace_id
            ])
            .unwrap();
        }

        {
            tx.execute(
                "INSERT INTO `commit` (commit_id, commit_rank, created_at) VALUES (?, ?, ?)",
                rusqlite::params![commit_id, commit_rank, now_str],
            )
            .unwrap();
        }

        {
            let mut stmt = tx
                .prepare("UPDATE workspace SET deleted_at = ? WHERE workspace_id = ?")
                .unwrap();
            stmt.execute(rusqlite::params![now_str, request.workspace_id])
                .unwrap();
        }

        tx.commit().unwrap();

        Ok(CommitResponse {})
    }

    fn get_all_trunk_files(
        &self,
        _request: GetAllTrunkFilesRequest,
    ) -> Result<GetAllTrunkFilesResponse, SagittaRemoteSystemDBError> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().unwrap();

        let res = {
            let mut stmt = tx
                .prepare(
                    "SELECT file_path.path, trunk_file_revision.blob_id, trunk_file_revision.deleted_at, trunk_file_revision.file_type FROM trunk_file_revision
                    JOIN (
                        SELECT file_path_id, MAX(commit_rank) AS commit_rank
                        FROM trunk_file_revision AS trunk_file_revision_2
                        GROUP BY trunk_file_revision_2.file_path_id
                    ) AS latest_commit_version
                    ON trunk_file_revision.file_path_id = latest_commit_version.file_path_id AND trunk_file_revision.commit_rank = latest_commit_version.commit_rank
                    JOIN file_path ON trunk_file_revision.file_path_id = file_path.file_path_id
                    ",
                )
                .unwrap();
            stmt.query_map(rusqlite::params![], |row| {
                let deleted_at: Option<String> = row.get(2)?;
                let file_type: i64 = row.get(3)?;
                Ok(GetAllTrunkFilesResponseItem {
                    file_path: row.get(0)?,
                    blob_id: row.get(1)?,
                    deleted: deleted_at.is_some(),
                    file_type: match file_type {
                        0 => SagittaFileType::File,
                        1 => SagittaFileType::Dir,
                        _ => unreachable!(),
                    },
                })
            })
            .unwrap()
            .map(|x| x.unwrap())
            .collect()
        };

        tx.commit().unwrap();

        Ok(GetAllTrunkFilesResponse { items: res })
    }

    fn get_commit_history(
        &self,
        request: GetCommitHistoryRequest,
    ) -> Result<GetCommitHistoryResponse, SagittaRemoteSystemDBError> {
        let db = self.db.lock().unwrap();

        let mut stmt = db
            .prepare("SELECT commit_id, commit_rank, created_at FROM `commit` ORDER BY commit_rank DESC LIMIT ?")
            .unwrap();
        let res = stmt
            .query_map(rusqlite::params![request.take], |row| {
                let created_at: String = row.get(2)?;
                Ok(GetCommitHistoryResponseItem {
                    commit_id: row.get(0)?,
                    commit_rank: row.get(1)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at).unwrap().into(),
                })
            })
            .unwrap()
            .map(|x| x.unwrap())
            .collect();

        Ok(GetCommitHistoryResponse { items: res })
    }

    fn read_dir(
        &self,
        request: ReadDirRequest,
    ) -> Result<ReadDirResponse, SagittaRemoteSystemDBError> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().unwrap();

        let mut res: BTreeMap<String, ReadDirResponseItem> = BTreeMap::new();

        let parent_id = {
            let mut stmt = tx
                .prepare("SELECT file_path_id FROM file_path WHERE path = ?")
                .unwrap();
            let ids = stmt
                .query_map(rusqlite::params![request.file_path.join("/")], |row| {
                    let id: String = row.get(0)?;
                    Ok(id)
                })
                .unwrap()
                .map(|x| x.unwrap())
                .collect::<Vec<String>>();
            if ids.is_empty() {
                return Ok(ReadDirResponse::NotFound);
            }
            ids[0].clone()
        };

        // check if dir is not deleted (workspace)
        let mut exists_in_workspace = false;
        if !request.file_path.is_empty() && request.workspace_id.is_some() {
            let mut stmt = tx
                .prepare(
                    "SELECT
                        file_path.path,
                        workspace_file_revision.blob_id,
                        workspace_file_revision.deleted_at,
                        workspace_file_revision.file_type,
                        file_path.name,
                        blob.size,
                        workspace_file_revision.created_at
                    FROM workspace_file_revision
                    JOIN (
                        SELECT file_path_id, MAX(sync_version_number) AS sync_version_number
                        FROM workspace_file_revision AS workspace_file_revision_2
                        WHERE workspace_file_revision_2.workspace_id = ?
                        GROUP BY workspace_file_revision_2.file_path_id
                    ) AS latest_sync_version
                    ON workspace_file_revision.file_path_id = latest_sync_version.file_path_id AND workspace_file_revision.sync_version_number = latest_sync_version.sync_version_number
                    JOIN file_path ON workspace_file_revision.file_path_id = file_path.file_path_id
                    LEFT JOIN blob ON workspace_file_revision.blob_id = blob.blob_id
                    WHERE workspace_file_revision.workspace_id = ? AND file_path.file_path_id = ?",
                )
                .unwrap();
            let res_workspace: Vec<ReadDirResponseItem> = stmt
                .query_map(
                    rusqlite::params![request.workspace_id, request.workspace_id, parent_id],
                    |row| {
                        let deleted_at: Option<String> = row.get(2)?;
                        let deleted_at =
                            deleted_at.map(|x| DateTime::parse_from_rfc3339(&x).unwrap().into());
                        let file_type: i64 = row.get(3)?;
                        let file_name: String = row.get(4)?;
                        let size: Option<u64> = row.get(5)?;
                        let created_at: String = row.get(6)?;
                        let created_at: SystemTime =
                            DateTime::parse_from_rfc3339(&created_at).unwrap().into();
                        Ok(ReadDirResponseItem {
                            file_path: row.get(0)?,
                            file_type: match file_type {
                                0 => SagittaFileType::File,
                                1 => SagittaFileType::Dir,
                                _ => unreachable!(),
                            },
                            deleted_at,
                            file_name,
                            size: size.unwrap_or(0),
                            modified_at: created_at,
                        })
                    },
                )
                .unwrap()
                .map(|x| x.unwrap())
                .collect();

            if !res_workspace.is_empty() {
                if res_workspace[0].deleted_at.is_some() {
                    return Ok(ReadDirResponse::NotFound);
                } else {
                    exists_in_workspace = true;
                }
            }
        }

        // check if dir is not deleted (trunk)
        if !request.file_path.is_empty() {
            let mut stmt = tx
                .prepare(
                    "SELECT
                        file_path.path,
                        trunk_file_revision.blob_id,
                        trunk_file_revision.deleted_at,
                        trunk_file_revision.file_type,
                        file_path.name,
                        blob.size,
                        trunk_file_revision.created_at
                    FROM trunk_file_revision
                    JOIN (
                        SELECT file_path_id, MAX(commit_rank) AS commit_rank
                        FROM trunk_file_revision AS trunk_file_revision_2
                        GROUP BY trunk_file_revision_2.file_path_id
                    ) AS latest_commit_version
                    ON trunk_file_revision.file_path_id = latest_commit_version.file_path_id AND trunk_file_revision.commit_rank = latest_commit_version.commit_rank
                    JOIN file_path ON trunk_file_revision.file_path_id = file_path.file_path_id
                    LEFT JOIN blob ON trunk_file_revision.blob_id = blob.blob_id
                    WHERE file_path.file_path_id = ?",
                )
                .unwrap();
            let res_trunk: Vec<ReadDirResponseItem> = stmt
                .query_map(rusqlite::params![parent_id], |row| {
                    let deleted_at: Option<String> = row.get(2)?;
                    let deleted_at =
                        deleted_at.map(|x| DateTime::parse_from_rfc3339(&x).unwrap().into());
                    let file_type: i64 = row.get(3)?;
                    let file_name: String = row.get(4)?;
                    let size: Option<u64> = row.get(5)?;
                    let created_at: String = row.get(6)?;
                    let created_at: SystemTime =
                        DateTime::parse_from_rfc3339(&created_at).unwrap().into();
                    Ok(ReadDirResponseItem {
                        file_path: row.get(0)?,
                        file_type: match file_type {
                            0 => SagittaFileType::File,
                            1 => SagittaFileType::Dir,
                            _ => unreachable!(),
                        },
                        deleted_at,
                        file_name,
                        size: size.unwrap_or(0),
                        modified_at: created_at,
                    })
                })
                .unwrap()
                .map(|x| x.unwrap())
                .collect();

            if !exists_in_workspace {
                if res_trunk.is_empty() {
                    return Ok(ReadDirResponse::NotFound);
                }

                if res_trunk[0].deleted_at.is_some() {
                    return Ok(ReadDirResponse::NotFound);
                }
            }
        }

        // get entries (trunk)
        {
            let mut stmt = tx
                .prepare(
                    "SELECT
                        file_path.path,
                        trunk_file_revision.blob_id,
                        trunk_file_revision.deleted_at,
                        trunk_file_revision.file_type,
                        file_path.name,
                        blob.size,
                        trunk_file_revision.created_at
                    FROM trunk_file_revision
                    JOIN (
                        SELECT file_path_id, MAX(commit_rank) AS commit_rank
                        FROM trunk_file_revision AS trunk_file_revision_2
                        GROUP BY trunk_file_revision_2.file_path_id
                    ) AS latest_commit_version
                    ON trunk_file_revision.file_path_id = latest_commit_version.file_path_id AND trunk_file_revision.commit_rank = latest_commit_version.commit_rank
                    JOIN file_path ON trunk_file_revision.file_path_id = file_path.file_path_id
                    LEFT JOIN blob ON trunk_file_revision.blob_id = blob.blob_id
                    WHERE file_path.parent = ?",
                )
                .unwrap();
            let res_trunk: Vec<ReadDirResponseItem> = stmt
                .query_map(rusqlite::params![parent_id], |row| {
                    let deleted_at: Option<String> = row.get(2)?;
                    let deleted_at =
                        deleted_at.map(|x| DateTime::parse_from_rfc3339(&x).unwrap().into());
                    let file_type: i64 = row.get(3)?;
                    let file_name: String = row.get(4)?;
                    let size: Option<u64> = row.get(5)?;
                    let created_at: String = row.get(6)?;
                    let created_at: SystemTime =
                        DateTime::parse_from_rfc3339(&created_at).unwrap().into();
                    Ok(ReadDirResponseItem {
                        file_path: row.get(0)?,
                        file_type: match file_type {
                            0 => SagittaFileType::File,
                            1 => SagittaFileType::Dir,
                            _ => unreachable!(),
                        },
                        deleted_at,
                        file_name,
                        size: size.unwrap_or(0),
                        modified_at: created_at,
                    })
                })
                .unwrap()
                .map(|x| x.unwrap())
                .collect();

            for item in res_trunk {
                res.insert(item.file_path.clone(), item);
            }
        }

        // get entries (workspace)
        if request.workspace_id.is_some() {
            let mut stmt = tx
                .prepare(
                    "SELECT
                        file_path.path,
                        workspace_file_revision.blob_id,
                        workspace_file_revision.deleted_at,
                        workspace_file_revision.file_type,
                        file_path.name,
                        blob.size,
                        workspace_file_revision.created_at
                    FROM workspace_file_revision
                    JOIN (
                        SELECT file_path_id, MAX(sync_version_number) AS sync_version_number
                        FROM workspace_file_revision AS workspace_file_revision_2
                        WHERE workspace_file_revision_2.workspace_id = ?
                        GROUP BY workspace_file_revision_2.file_path_id
                    ) AS latest_sync_version
                    ON workspace_file_revision.file_path_id = latest_sync_version.file_path_id AND workspace_file_revision.sync_version_number = latest_sync_version.sync_version_number
                    JOIN file_path ON workspace_file_revision.file_path_id = file_path.file_path_id
                    LEFT JOIN blob ON workspace_file_revision.blob_id = blob.blob_id
                    WHERE workspace_file_revision.workspace_id = ? AND file_path.parent = ?",
                )
                .unwrap();
            let res_workspace: Vec<ReadDirResponseItem> = stmt
                .query_map(
                    rusqlite::params![request.workspace_id, request.workspace_id, parent_id],
                    |row| {
                        let deleted_at: Option<String> = row.get(2)?;
                        let deleted_at =
                            deleted_at.map(|x| DateTime::parse_from_rfc3339(&x).unwrap().into());
                        let file_type: i64 = row.get(3)?;
                        let file_name: String = row.get(4)?;
                        let size: Option<u64> = row.get(5)?;
                        let created_at: String = row.get(6)?;
                        let created_at: SystemTime =
                            DateTime::parse_from_rfc3339(&created_at).unwrap().into();
                        Ok(ReadDirResponseItem {
                            file_path: row.get(0)?,
                            file_type: match file_type {
                                0 => SagittaFileType::File,
                                1 => SagittaFileType::Dir,
                                _ => unreachable!(),
                            },
                            deleted_at,
                            file_name,
                            size: size.unwrap_or(0),
                            modified_at: created_at,
                        })
                    },
                )
                .unwrap()
                .map(|x| x.unwrap())
                .collect();

            for item in res_workspace {
                res.insert(item.file_path.clone(), item);
            }
        }

        tx.commit().unwrap();

        let items = res
            .into_iter()
            .filter(|(_, v)| request.include_deleted || v.deleted_at.is_none())
            .map(|(_, v)| v)
            .collect();
        Ok(ReadDirResponse::Found { items })
    }

    fn get_attr(
        &self,
        request: GetAttrRequest,
    ) -> Result<GetAttrResponse, SagittaRemoteSystemDBError> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().unwrap();

        let path_id = {
            let mut stmt = tx
                .prepare("SELECT file_path_id FROM file_path WHERE path = ?")
                .unwrap();
            let ids = stmt
                .query_map(rusqlite::params![request.file_path.join("/")], |row| {
                    let id: String = row.get(0)?;
                    Ok(id)
                })
                .unwrap()
                .map(|x| x.unwrap())
                .collect::<Vec<String>>();
            if ids.is_empty() {
                return Ok(GetAttrResponse::NotFound);
            }
            ids[0].clone()
        };

        if request.workspace_id.is_some() {
            let res_workspace = {
                let mut stmt = tx
                    .prepare(
                        "SELECT
                            workspace_file_revision.blob_id,
                            workspace_file_revision.deleted_at,
                            workspace_file_revision.file_type,
                            blob.size,
                            workspace_file_revision.created_at
                        FROM workspace_file_revision
                        JOIN (
                            SELECT file_path_id, MAX(sync_version_number) AS sync_version_number
                            FROM workspace_file_revision AS workspace_file_revision_2
                            WHERE workspace_file_revision_2.workspace_id = ?
                            GROUP BY workspace_file_revision_2.file_path_id
                        ) AS latest_sync_version
                        ON workspace_file_revision.file_path_id = latest_sync_version.file_path_id AND workspace_file_revision.sync_version_number = latest_sync_version.sync_version_number
                        LEFT JOIN blob ON workspace_file_revision.blob_id = blob.blob_id
                        WHERE workspace_file_revision.workspace_id = ? AND workspace_file_revision.file_path_id = ?
                        "
                    )
                    .unwrap();
                let res_workspace: Vec<GetAttrResponse> = stmt
                    .query_map(
                        rusqlite::params![request.workspace_id, request.workspace_id, path_id],
                        |row| {
                            let deleted_at: Option<String> = row.get(1)?;
                            let file_type: i64 = row.get(2)?;
                            let size: Option<u64> = row.get(3)?;
                            let created_at: String = row.get(4)?;
                            let created_at: SystemTime =
                                DateTime::parse_from_rfc3339(&created_at).unwrap().into();
                            if deleted_at.is_some() {
                                Ok(GetAttrResponse::NotFound)
                            } else {
                                Ok(GetAttrResponse::Found {
                                    file_type: match file_type {
                                        0 => SagittaFileType::File,
                                        1 => SagittaFileType::Dir,
                                        _ => unreachable!(),
                                    },
                                    size: size.unwrap_or(0),
                                    modified_at: created_at,
                                })
                            }
                        },
                    )
                    .unwrap()
                    .map(|x| x.unwrap())
                    .collect();
                res_workspace
            };

            if !res_workspace.is_empty() {
                let item = res_workspace[0].clone();
                tx.commit().unwrap();
                return Ok(item);
            }
        }

        let res_trunk = {
            let mut stmt = tx
                .prepare(
                    "SELECT
                        trunk_file_revision.blob_id,
                        trunk_file_revision.deleted_at,
                        trunk_file_revision.file_type,
                        blob.size,
                        trunk_file_revision.created_at
                    FROM trunk_file_revision
                    JOIN (
                        SELECT file_path_id, MAX(commit_rank) AS commit_rank
                        FROM trunk_file_revision AS trunk_file_revision_2
                        GROUP BY trunk_file_revision_2.file_path_id
                    ) AS latest_commit_version
                    ON trunk_file_revision.file_path_id = latest_commit_version.file_path_id AND trunk_file_revision.commit_rank = latest_commit_version.commit_rank
                    LEFT JOIN blob ON trunk_file_revision.blob_id = blob.blob_id
                    WHERE trunk_file_revision.file_path_id = ?
                    "
                )
                .unwrap();
            let res_trunk: Vec<GetAttrResponse> = stmt
                .query_map(rusqlite::params![path_id], |row| {
                    let deleted_at: Option<String> = row.get(1)?;
                    let file_type: i64 = row.get(2)?;
                    let size: Option<u64> = row.get(3)?;
                    let created_at: String = row.get(4)?;
                    let created_at: SystemTime =
                        DateTime::parse_from_rfc3339(&created_at).unwrap().into();
                    if deleted_at.is_some() {
                        Ok(GetAttrResponse::NotFound)
                    } else {
                        Ok(GetAttrResponse::Found {
                            file_type: match file_type {
                                0 => SagittaFileType::File,
                                1 => SagittaFileType::Dir,
                                _ => unreachable!(),
                            },
                            size: size.unwrap_or(0),
                            modified_at: created_at,
                        })
                    }
                })
                .unwrap()
                .map(|x| x.unwrap())
                .collect();
            res_trunk
        };

        if !res_trunk.is_empty() {
            let item = res_trunk[0].clone();
            tx.commit().unwrap();
            return Ok(item);
        }

        tx.commit().unwrap();
        Ok(GetAttrResponse::NotFound)
    }

    fn get_file_blob_id(
        &self,
        request: GetFileBlobIdRequest,
    ) -> Result<GetFileBlobIdResponse, SagittaRemoteSystemDBError> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().unwrap();

        let path_id = {
            let mut stmt = tx
                .prepare("SELECT file_path_id FROM file_path WHERE path = ?")
                .unwrap();
            let ids = stmt
                .query_map(rusqlite::params![request.file_path.join("/")], |row| {
                    let id: String = row.get(0)?;
                    Ok(id)
                })
                .unwrap()
                .map(|x| x.unwrap())
                .collect::<Vec<String>>();
            if ids.is_empty() {
                return Ok(GetFileBlobIdResponse::NotFound);
            }
            ids[0].clone()
        };

        if request.workspace_id.is_some() {
            let res_workspace = {
                let mut stmt = tx
                    .prepare(
                        "SELECT
                            workspace_file_revision.blob_id
                        FROM workspace_file_revision
                        JOIN (
                            SELECT file_path_id, MAX(sync_version_number) AS sync_version_number
                            FROM workspace_file_revision AS workspace_file_revision_2
                            WHERE workspace_file_revision_2.workspace_id = ?
                            GROUP BY workspace_file_revision_2.file_path_id
                        ) AS latest_sync_version
                        ON workspace_file_revision.file_path_id = latest_sync_version.file_path_id AND workspace_file_revision.sync_version_number = latest_sync_version.sync_version_number
                        WHERE workspace_file_revision.workspace_id = ? AND workspace_file_revision.file_path_id = ?
                        "
                    )
                    .unwrap();
                let res_workspace: Vec<GetFileBlobIdResponse> = stmt
                    .query_map(
                        rusqlite::params![request.workspace_id, request.workspace_id, path_id],
                        |row| {
                            let blob_id: Option<String> = row.get(0)?;
                            if let Some(blob_id) = blob_id {
                                Ok(GetFileBlobIdResponse::Found { blob_id })
                            } else {
                                Ok(GetFileBlobIdResponse::NotFound)
                            }
                        },
                    )
                    .unwrap()
                    .map(|x| x.unwrap())
                    .collect();
                res_workspace
            };

            if !res_workspace.is_empty() {
                let item = res_workspace[0].clone();
                tx.commit().unwrap();
                return Ok(item);
            }
        }

        let res_trunk = {
            let mut stmt = tx
                .prepare(
                    "SELECT
                        trunk_file_revision.blob_id
                    FROM trunk_file_revision
                    JOIN (
                        SELECT file_path_id, MAX(commit_rank) AS commit_rank
                        FROM trunk_file_revision AS trunk_file_revision_2
                        GROUP BY trunk_file_revision_2.file_path_id
                    ) AS latest_commit_version
                    ON trunk_file_revision.file_path_id = latest_commit_version.file_path_id AND trunk_file_revision.commit_rank = latest_commit_version.commit_rank
                    WHERE trunk_file_revision.file_path_id = ?
                    "
                )
                .unwrap();
            let res_trunk: Vec<GetFileBlobIdResponse> = stmt
                .query_map(rusqlite::params![path_id], |row| {
                    let blob_id: Option<String> = row.get(0)?;
                    if let Some(blob_id) = blob_id {
                        Ok(GetFileBlobIdResponse::Found { blob_id })
                    } else {
                        Ok(GetFileBlobIdResponse::NotFound)
                    }
                })
                .unwrap()
                .map(|x| x.unwrap())
                .collect();
            res_trunk
        };

        if !res_trunk.is_empty() {
            let item = res_trunk[0].clone();
            tx.commit().unwrap();
            return Ok(item);
        }

        tx.commit().unwrap();
        Ok(GetFileBlobIdResponse::NotFound)
    }
}

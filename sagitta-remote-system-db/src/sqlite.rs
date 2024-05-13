use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use rand::RngCore;
use sagitta_common::clock::Clock;

use crate::*;

use base64::{engine::general_purpose::URL_SAFE, Engine};

pub struct SagittaRemoteSystemDBBySqlite<Rng: RngCore> {
    db: rusqlite::Connection,
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
            db,
            rng: Arc::new(Mutex::new(rng)),
            clock,
        })
    }

    pub fn migration(&self) {
        self.db
            .execute(
                "CREATE TABLE IF NOT EXISTS workspace (
                workspace_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                deleted_at TEXT
            )",
                rusqlite::params![],
            )
            .unwrap();

        self.db
            .execute(
                "CREATE TABLE IF NOT EXISTS blob (
                blob_id TEXT PRIMARY KEY,
                hash TEXT NOT NULL,
                size INTEGER NOT NULL
            )",
                rusqlite::params![],
            )
            .unwrap();
    }

    fn generate_id(&self) -> String {
        let mut id = [0u8; 384 / 8];
        let mut rng = self.rng.lock().unwrap();
        rng.fill_bytes(&mut id);

        URL_SAFE.encode(id)
    }
}

impl<Rng: RngCore> SagittaRemoteSystemDB for SagittaRemoteSystemDBBySqlite<Rng> {
    fn create_workspace(
        &self,
        request: CreateWorkspaceRequest,
    ) -> Result<CreateWorkspaceResponse, SagittaRemoteSystemDBError> {
        let id = self.generate_id();
        let now = self.clock.now();
        let now: DateTime<Utc> = now.into();
        let now_str = now.to_rfc3339();

        self.db
            .execute(
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
        if request.contains_deleted {
            let mut stmt = self
                .db
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
            let mut stmt = self.db.prepare("SELECT workspace_id, name, created_at, deleted_at FROM workspace WHERE deleted_at IS NULL").unwrap();
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

        let affected = self
            .db
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
        self.db
            .execute(
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
        let mut stmt = self
            .db
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
}

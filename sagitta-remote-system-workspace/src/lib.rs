use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc, time::SystemTime};

use sagitta_common::sha256::calc_sha256_from_slice;
use sagitta_objects::{ObjectId, SagittaCommitObject, SagittaTreeObject};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    sql::Thing,
    Surreal,
};

// file hierarchy
// root
// - trunk
//   - head
//   - objects
//     - 01
//       - 23
//         - 012345...
//     - 03
// - workspace1
//   - head
//   - objects
//     - 01
//       - 23
//         - 012345...
//     - 03
// - workspace2

#[derive(Debug, Clone)]
pub struct RemoteSystemWorkspaceManager {
    base_url: PathBuf,
    conn: Arc<Surreal<Client>>,
}

#[derive(Debug)]
pub enum Error {
    WorkspaceAlreadyExists,
    IOError(std::io::Error),
    SerdeCborError(serde_cbor::error::Error),
    SurrealError(surrealdb::Error),
    Error,
    SurrealQueryError(HashMap<usize, surrealdb::Error>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    pub name: String,
    pub head: String,
    pub head_commit: Option<Thing>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceReadRecord {
    pub id: Thing,
    pub name: String,
    pub head: String,
    pub head_commit: Option<Thing>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilePathReadRecord {
    pub id: Thing,
    pub name: String,
    pub path: String,
    pub parent: Option<Thing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePathWriteRecord {
    pub name: String,
    pub path: String,
    pub parent: Option<Thing>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileRevisionReadRecord {
    pub id: Thing,
    pub path_id: Thing,
    pub file_type: FileType,
    pub commit_id: Option<Thing>,
    pub commit_number: Option<u64>,
    pub workspace_id: Option<Thing>,
    pub blob_id: Option<String>,
    pub ctime: SystemTime,
    pub mtime: SystemTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileRevisionWriteRecord {
    pub path_id: Thing,
    pub file_type: FileType,
    pub commit_id: Option<Thing>,
    pub commit_number: Option<u64>,
    pub workspace_id: Option<Thing>,
    pub blob_id: Option<String>,
    pub ctime: SystemTime,
    pub mtime: SystemTime,
}

impl RemoteSystemWorkspaceManager {
    pub async fn new(base_url: PathBuf, surreal_url: &str, is_main: bool) -> Self {
        let db = Surreal::new::<Ws>(surreal_url)
            .await
            .expect("Failed to connect to SurrealDB.");
        if is_main {
            db.use_ns("main")
                .use_db("db")
                .await
                .expect("Failed to use main/db.");
        } else {
            db.use_ns("test")
                .use_db("db")
                .await
                .expect("Failed to use test/db.");
            db.query("DELETE workspace")
                .query("DELETE file_path")
                .query("DELETE file_revision")
                .query("DELETE commit")
                .await
                .expect("Failed to delete tables.");
        }
        Self {
            base_url: base_url.clone(),
            conn: Arc::new(db),
        }
    }

    pub async fn save_file(
        &self,
        workspace_id: &Thing,
        path: &[String],
        data: &[u8],
    ) -> Result<(), Error> {
        // create file path
        let mut path_id = None;
        let mut parent_id = None;
        for (i, p) in path.iter().enumerate() {
            let mut path_read_res = self
                .conn
                .as_ref()
                .query("SELECT * FROM file_path WHERE name = $name AND parent = $parent;")
                .bind(("name", p))
                .bind(("parent", &parent_id))
                .await
                .map_err(Error::SurrealError)?;
            let path_read: Vec<FilePathReadRecord> =
                path_read_res.take(0).map_err(Error::SurrealError)?;
            if path_read.is_empty() {
                let path_write: Vec<FilePathReadRecord> = self
                    .conn
                    .as_ref()
                    .insert("file_path")
                    .content(vec![FilePathWriteRecord {
                        name: p.clone(),
                        path: path[..=i].join("/"),
                        parent: parent_id.clone(),
                    }])
                    .await
                    .map_err(Error::SurrealError)?;
                path_id = Some(path_write[0].id.clone());
            } else {
                path_id = Some(path_read[0].id.clone());
            }
            parent_id.clone_from(&path_id);
        }

        let mut file_read_res = self
            .conn
            .as_ref()
            .query("SELECT * FROM file_revision WHERE path_id = $path_id AND workspace_id = $workspace_id;")
            .bind(("path_id", path_id.clone()))
            .bind(("workspace_id", workspace_id))
            .await
            .map_err(Error::SurrealError)?;
        let file_read: Vec<FileRevisionReadRecord> =
            file_read_res.take(0).map_err(Error::SurrealError)?;
        let blob_id = calc_sha256_from_slice(data);
        let path_id = path_id.expect("path_id is None.");
        if file_read.is_empty() {
            let _: Vec<FileRevisionReadRecord> = self
                .conn
                .as_ref()
                .create("file_revision")
                .content(FileRevisionWriteRecord {
                    path_id,
                    file_type: FileType::File,
                    commit_id: None,
                    commit_number: None,
                    workspace_id: Some(workspace_id.clone()),
                    blob_id: Some(blob_id),
                    ctime: SystemTime::now(),
                    mtime: SystemTime::now(),
                })
                .await
                .map_err(Error::SurrealError)?;
        } else {
            let _: Option<FileRevisionReadRecord> = self
                .conn
                .as_ref()
                .update(("file_revision", file_read[0].id.clone()))
                .content(FileRevisionWriteRecord {
                    path_id,
                    file_type: FileType::File,
                    commit_id: None,
                    commit_number: None,
                    workspace_id: Some(workspace_id.clone()),
                    blob_id: Some(blob_id),
                    ctime: SystemTime::now(),
                    mtime: SystemTime::now(),
                })
                .await
                .map_err(Error::SurrealError)?;
        }

        Ok(())
    }

    pub async fn initial_commit_if_not_exists(&self) -> Result<(), Error> {
        let mut ret = self
            .conn
            .query("BEGIN TRANSACTION")
            .query("LET $trunk = SELECT * FROM workspace WHERE name = 'trunk'")
            .query(
                "IF array::len($trunk) == 1 THEN {
                THROW 'Workspace already exists.';
            } ELSE {
                LET $new_commit = INSERT INTO commit { number: 0 };
                LET $trunk = (INSERT INTO workspace { name: 'trunk', head: '', head_commit: $new_commit[0].id, deleted: false });
            } END",
            )
            .query("COMMIT TRANSACTION")
            .await
            .map_err(Error::SurrealError)?;
        let errors = ret.take_errors();
        if !errors.is_empty() {
            return Err(Error::SurrealQueryError(errors));
        }
        Ok(())
    }

    pub async fn get_trunk_head_commit(&self) -> Result<String, Error> {
        let mut ret = self
            .conn
            .as_ref()
            .query("SELECT * FROM workspace WHERE name = 'trunk';")
            .await
            .map_err(Error::SurrealError)?;
        let workspace: Vec<WorkspaceRecord> = ret.take(0).expect("trunk not found.");
        let a = workspace
            .first()
            .expect("trunk not found")
            .head_commit
            .clone();
        Ok(a.expect("no head commit").to_string())
    }

    pub async fn commit(&self, workspace_id: &Thing, trunk_head: &str) -> Result<(), Error> {
        let trunk_head = Thing::from_str(trunk_head).expect("Invalid commit id.");
        let mut ret = self
            .conn
            .query("BEGIN TRANSACTION")
            .query("LET $trunks = (SELECT * FROM workspace WHERE name = 'trunk')")
            .query("IF array::len($trunks) != 1 THEN {
                THROW 'Trunk not found. or not unique.';
            } END")
            .query("LET $head_commit = (SELECT * FROM commit WHERE id = $trunks[0].head_commit)")
            .query("LET $new_commit = (INSERT INTO commit { previous_id: $id, number: $head_commit[0].number + 1 })")
            .query("IF $trunks[0].head_commit == $head_id THEN {
                $trunks[0].head_commit = $new_commit[0].id;
            } ELSE {
                THROW 'Head mismatch.';
            } END")
            .query("UPDATE file_revision SET commit_id = $new_commit[0].id, commit_number = $new_commit[0].number WHERE workspace_id = $workspace_id")
            .query("UPDATE workspace SET deleted = true WHERE id = $workspace_id")
            .query("COMMIT TRANSACTION")
            .bind(("workspace_id", workspace_id))
            .bind(("head_id", trunk_head))
            .await
            .map_err(Error::SurrealError)?;
        let errors = ret.take_errors();
        if !errors.is_empty() {
            return Err(Error::SurrealQueryError(errors));
        }
        Ok(())
    }

    pub async fn get_head(&self, workspace_id: Option<&str>) -> Result<ObjectId, Error> {
        let workspace_id = workspace_id.unwrap_or("trunk");
        let mut ret = self
            .conn
            .as_ref()
            .query("SELECT * FROM workspace WHERE name = $name;")
            .bind(("name", workspace_id))
            .await
            .map_err(Error::SurrealError)?;
        let workspace: Vec<WorkspaceRecord> = ret.take(0).expect("No workspace found.");
        Ok(ObjectId {
            id: workspace.first().expect("workspace is None").head.clone(),
        })
    }

    pub async fn update_head(
        &self,
        workspace_id: Option<&str>,
        commit_id: &ObjectId,
    ) -> Result<(), Error> {
        let workspace_name = workspace_id.unwrap_or("trunk");

        self.conn
            .as_ref()
            .query("UPDATE workspace SET name = $workspace, head = $head WHERE name = $workspace;")
            .bind(("head", &commit_id.id))
            .bind(("workspace", workspace_name))
            .await
            .map_err(Error::SurrealError)?;

        Ok(())
    }

    pub fn get_object(&self, workspace_id: Option<&str>, id: &ObjectId) -> Result<Vec<u8>, Error> {
        let workspace_path = match workspace_id {
            Some(workspace_id) => self.base_url.join(workspace_id),
            None => self.base_url.join("trunk"),
        };
        let hierarchy1 = &id.id[0..2];
        let hierarchy2 = &id.id[2..4];
        let object_path = workspace_path
            .join("objects")
            .join(hierarchy1)
            .join(hierarchy2)
            .join(&id.id);
        let r = std::fs::read(object_path).map_err(Error::IOError)?;
        Ok(r)
    }

    pub fn save_object(&self, workspace_id: Option<&str>, data: &[u8]) -> Result<ObjectId, Error> {
        let id = ObjectId {
            id: calc_sha256_from_slice(data),
        };
        let workspace_path = match workspace_id {
            Some(workspace_id) => self.base_url.join(workspace_id),
            None => self.base_url.join("trunk"),
        };
        let hierarchy1 = &id.id[0..2];
        let hierarchy2 = &id.id[2..4];
        let object_parent_path = workspace_path
            .join("objects")
            .join(hierarchy1)
            .join(hierarchy2);
        std::fs::create_dir_all(&object_parent_path).map_err(Error::IOError)?;
        let object_path = object_parent_path.join(&id.id);
        std::fs::write(object_path, data).map_err(Error::IOError)?;
        Ok(id)
    }

    pub fn save_tree(
        &self,
        workspace_id: Option<&str>,
        tree: &SagittaTreeObject,
    ) -> Result<ObjectId, Error> {
        let mut buf = vec![];
        serde_cbor::to_writer(&mut buf, &tree).map_err(Error::SerdeCborError)?;
        let object_id = self.save_object(workspace_id, &buf)?;
        Ok(object_id)
    }

    pub fn save_commit(
        &self,
        workspace_id: Option<&str>,
        commit: &SagittaCommitObject,
    ) -> Result<ObjectId, Error> {
        let mut buf = vec![];
        serde_cbor::to_writer(&mut buf, &commit).map_err(Error::SerdeCborError)?;
        self.save_object(workspace_id, &buf)
    }

    pub async fn create_workspace(&self, workspace_id: &str) -> Result<Thing, Error> {
        let trunk_head = self.get_head(None).await?;

        let created: Vec<WorkspaceReadRecord> = self
            .conn
            .as_ref()
            .create("workspace")
            .content(WorkspaceRecord {
                name: workspace_id.to_string(),
                head: trunk_head.id,
                head_commit: None,
                deleted: false,
            })
            .await
            .map_err(Error::SurrealError)?;

        Ok(created[0].id.clone())
    }

    pub async fn list_workspaces(&self) -> Result<Vec<String>, Error> {
        let mut workspaces = Vec::new();

        let mut workspace_names_res = self
            .conn
            .as_ref()
            .query("SELECT * FROM workspace WHERE deleted = false")
            .await
            .map_err(Error::SurrealError)?;
        let workspace_names: Vec<WorkspaceRecord> =
            workspace_names_res.take(0).map_err(Error::SurrealError)?;
        for workspace in workspace_names {
            if workspace.name != "trunk" {
                workspaces.push(workspace.name);
            }
        }
        Ok(workspaces)
    }
}

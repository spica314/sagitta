use std::{path::PathBuf, sync::Arc};

use sagitta_common::sha256::calc_sha256_from_slice;
use sagitta_objects::{ObjectId, SagittaCommitObject, SagittaTreeObject};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    pub name: String,
    pub head: String,
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
            db.query("DELETE workspace;")
                .await
                .expect("Failed to delete workspace.");
        }
        Self {
            base_url: base_url.clone(),
            conn: Arc::new(db),
        }
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
        let workspace: Option<WorkspaceRecord> = ret.take(0).expect("No workspace found.");
        Ok(ObjectId {
            id: workspace.expect("workspace is None").head,
        })
    }

    pub async fn set_trunk_head(&self, commit_id: &ObjectId) -> Result<(), Error> {
        let _: Vec<WorkspaceRecord> = self
            .conn
            .as_ref()
            .create("workspace")
            .content(WorkspaceRecord {
                name: "trunk".to_string(),
                head: commit_id.id.clone(),
            })
            .await
            .map_err(Error::SurrealError)?;
        Ok(())
    }

    pub async fn update_head(
        &self,
        workspace_id: Option<&str>,
        commit_id: &ObjectId,
    ) -> Result<(), Error> {
        let workspace_name = workspace_id.unwrap_or("trunk");

        self.conn
            .as_ref()
            .query("UPDATE workspace SET name = $wokrspace, head = $head WHERE name = $workspace;")
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

    pub async fn create_workspace(&self, workspace_id: &str) -> Result<(), Error> {
        let trunk_head = self.get_head(None).await?;

        let _created: Vec<WorkspaceRecord> = self
            .conn
            .as_ref()
            .create("workspace")
            .content(WorkspaceRecord {
                name: workspace_id.to_string(),
                head: trunk_head.id,
            })
            .await
            .map_err(Error::SurrealError)?;

        Ok(())
    }

    pub async fn list_workspaces(&self) -> Result<Vec<String>, Error> {
        let mut workspaces = Vec::new();

        let workspace_names: Vec<WorkspaceRecord> = self
            .conn
            .as_ref()
            .select("workspace")
            .await
            .map_err(Error::SurrealError)?;
        for workspace in workspace_names {
            if workspace.name != "trunk" {
                workspaces.push(workspace.name);
            }
        }
        Ok(workspaces)
    }
}

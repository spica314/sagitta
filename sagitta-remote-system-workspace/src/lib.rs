use std::path::PathBuf;

use sagitta_common::sha256::calc_sha256_from_slice;
use sagitta_objects::{ObjectId, SagittaCommitObject, SagittaTreeObject};

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
}

#[derive(Debug)]
pub enum Error {
    WorkspaceAlreadyExists,
    IOError(std::io::Error),
    SerdeCborError(serde_cbor::error::Error),
    Error,
}

impl RemoteSystemWorkspaceManager {
    pub fn new(base_url: PathBuf) -> Self {
        Self {
            base_url: base_url.clone(),
        }
    }

    pub fn get_head(&self, workspace_id: Option<&str>) -> Result<ObjectId, Error> {
        let workspace_path = match workspace_id {
            Some(workspace_id) => self.base_url.join(workspace_id),
            None => self.base_url.join("trunk"),
        };
        let head_path = workspace_path.join("head");
        let head = std::fs::read_to_string(head_path).map_err(Error::IOError)?;
        Ok(ObjectId { id: head })
    }

    pub fn update_head(
        &self,
        workspace_id: Option<&str>,
        commit_id: &ObjectId,
    ) -> Result<(), Error> {
        let workspace_path = match workspace_id {
            Some(workspace_id) => self.base_url.join(workspace_id),
            None => self.base_url.join("trunk"),
        };
        let head_path = workspace_path.join("head");
        std::fs::write(head_path, &commit_id.id).map_err(Error::IOError)?;
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

    pub fn create_workspace(&self, workspace_id: &str) -> Result<(), Error> {
        let workspace_path = self.base_url.join(workspace_id);
        if workspace_path.exists() {
            return Err(Error::WorkspaceAlreadyExists);
        }
        std::fs::create_dir_all(&workspace_path).map_err(Error::IOError)?;

        let trunk_head = self.get_head(None)?;
        self.update_head(Some(workspace_id), &trunk_head)?;

        Ok(())
    }

    pub fn list_workspaces(&self) -> Result<Vec<String>, Error> {
        let mut workspaces = Vec::new();
        for entry in std::fs::read_dir(&self.base_url).map_err(Error::IOError)? {
            let entry = entry.map_err(Error::IOError)?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let workspace = path
                .file_name()
                .expect("The path should not be terminate in `..`.")
                .to_str()
                .expect("The filename should be able to be represented in UTF-8.")
                .to_string();
            if workspace == "trunk" {
                continue;
            }
            workspaces.push(workspace);
        }
        Ok(workspaces)
    }
}

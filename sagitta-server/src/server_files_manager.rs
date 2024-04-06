use std::path::PathBuf;

use sagitta_objects::ObjectId;
use sagitta_objects_store::{file_store::FileStore, sagitta_objects_store::SagittaObjectsStore};

#[derive(Debug, Clone)]
pub struct ServerFilesManager {
    pub base_path: PathBuf,
    pub file_store: FileStore,
}

impl ServerFilesManager {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path: base_path.clone(),
            file_store: FileStore::new(base_path),
        }
    }

    pub fn get_blob(&self, id: &ObjectId) -> Vec<u8> {
        self.file_store.get_blob(id).unwrap()
    }

    pub fn get_trunk_head(&self) -> ObjectId {
        self.file_store.get_trunk_head().unwrap()
    }
}

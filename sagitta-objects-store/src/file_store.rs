use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use sagitta_objects::{ObjectId, SagittaBlobObject, SagittaCommitObject, SagittaTreeObject};
use sha2::{Digest, Sha256};

use crate::sagitta_objects_store::SagittaObjectsStore;

pub struct FileStore {
    root: PathBuf,
}

impl FileStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn save_file(&self, data: &[u8]) -> Result<ObjectId, std::io::Error> {
        let id = self.calc_sha256(data);

        let hierarchy1 = &id[0..2];
        let hierarchy2 = &id[2..4];
        let path = self.root.join(hierarchy1).join(hierarchy2).join(&id);

        // Create the directories if they don't exist
        std::fs::create_dir_all(path.parent().unwrap())?;

        // Write the file
        let mut file = File::create(&path)?;
        let mut writer = brotli::CompressorWriter::new(&mut file, 4096, 11, 22);
        writer.write_all(data)?;

        let id = ObjectId { id };
        Ok(id)
    }

    fn read_file(&self, id: &ObjectId) -> Result<Vec<u8>, std::io::Error> {
        let id = &id.id;
        let hierarchy1 = &id[0..2];
        let hierarchy2 = &id[2..4];
        let path = self.root.join(hierarchy1).join(hierarchy2).join(id);

        let file = File::open(path)?;
        let mut reader = brotli::Decompressor::new(file, 4096);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        Ok(data)
    }

    fn calc_sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

#[derive(Debug)]
pub enum FileStoreError {
    IOError(std::io::Error),
    CborError(serde_cbor::Error),
}

impl SagittaObjectsStore for FileStore {
    type Error = FileStoreError;

    fn save_blob(&self, blob: SagittaBlobObject) -> Result<ObjectId, Self::Error> {
        let mut buf = vec![];
        serde_cbor::to_writer(&mut buf, &blob).map_err(FileStoreError::CborError)?;
        self.save_file(&buf).map_err(FileStoreError::IOError)
    }

    fn get_blob(&self, id: &ObjectId) -> Result<SagittaBlobObject, Self::Error> {
        let buf = self.read_file(id).map_err(FileStoreError::IOError)?;
        let res: SagittaBlobObject =
            serde_cbor::from_reader(buf.as_slice()).map_err(FileStoreError::CborError)?;
        Ok(res)
    }

    fn save_tree(&self, tree: SagittaTreeObject) -> Result<ObjectId, Self::Error> {
        let mut buf = vec![];
        serde_cbor::to_writer(&mut buf, &tree).map_err(FileStoreError::CborError)?;
        self.save_file(&buf).map_err(FileStoreError::IOError)
    }

    fn get_tree(&self, id: &ObjectId) -> Result<SagittaTreeObject, Self::Error> {
        let buf = self.read_file(id).map_err(FileStoreError::IOError)?;
        let res: SagittaTreeObject =
            serde_cbor::from_reader(buf.as_slice()).map_err(FileStoreError::CborError)?;
        Ok(res)
    }

    fn save_commit(&self, commit: SagittaCommitObject) -> Result<ObjectId, Self::Error> {
        let mut buf = vec![];
        serde_cbor::to_writer(&mut buf, &commit).map_err(FileStoreError::CborError)?;
        self.save_file(&buf).map_err(FileStoreError::IOError)
    }

    fn get_commit(&self, id: &ObjectId) -> Result<SagittaCommitObject, Self::Error> {
        let buf = self.read_file(id).map_err(FileStoreError::IOError)?;
        let res: SagittaCommitObject =
            serde_cbor::from_reader(buf.as_slice()).map_err(FileStoreError::CborError)?;
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use fuser::FileType;
    use sagitta_objects::SagittaTreeItem;
    use tempfile::tempdir;

    #[test]
    fn test_save_blob() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().to_path_buf());

        let blob = SagittaBlobObject {
            data: b"Hello, world!".to_vec(),
        };
        let id = store.save_blob(blob).unwrap();

        let blob = store.get_blob(&id).unwrap();
        assert_eq!(blob.data, b"Hello, world!");
    }

    #[test]
    fn test_save_tree() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().to_path_buf());

        let tree = SagittaTreeObject {
            items: vec![
                SagittaTreeItem {
                    name: "file1".to_string(),
                    object_id: ObjectId {
                        id: "1234".to_string(),
                    },
                    size: 10,
                    mtime: std::time::SystemTime::now(),
                    ctime: std::time::SystemTime::now(),
                    kind: FileType::RegularFile,
                    perm: 0o644,
                },
                SagittaTreeItem {
                    name: "file2".to_string(),
                    object_id: ObjectId {
                        id: "5678".to_string(),
                    },
                    size: 20,
                    mtime: std::time::SystemTime::now(),
                    ctime: std::time::SystemTime::now(),
                    kind: FileType::RegularFile,
                    perm: 0o644,
                },
            ],
        };
        let id = store.save_tree(tree).unwrap();

        let tree = store.get_tree(&id).unwrap();
        assert_eq!(tree.items.len(), 2);
        assert_eq!(tree.items[0].name, "file1");
        assert_eq!(tree.items[0].object_id.id, "1234");
        assert_eq!(tree.items[0].size, 10);
        assert_eq!(tree.items[1].name, "file2");
        assert_eq!(tree.items[1].object_id.id, "5678");
        assert_eq!(tree.items[1].size, 20);
    }

    #[test]
    fn test_save_commit() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().to_path_buf());

        let commit = SagittaCommitObject {
            tree_id: ObjectId {
                id: "1234".to_string(),
            },
            parent_commit_id: Some(ObjectId {
                id: "5678".to_string(),
            }),
            message: "Hello, world!".to_string(),
        };
        let id = store.save_commit(commit).unwrap();

        let commit = store.get_commit(&id).unwrap();
        assert_eq!(commit.tree_id.id, "1234");
        assert_eq!(commit.parent_commit_id.unwrap().id, "5678");
        assert_eq!(commit.message, "Hello, world!");
    }
}

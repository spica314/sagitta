use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use sagitta_objects::{ObjectId, SagittaCommitObject, SagittaTreeObject};
use sha2::{Digest, Sha256};

use crate::sagitta_objects_store::SagittaObjectsStore;

#[derive(Debug, Clone)]
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

    fn save_blob(&self, blob: &[u8]) -> Result<ObjectId, Self::Error> {
        self.save_file(blob).map_err(FileStoreError::IOError)
    }

    fn get_blob(&self, id: &ObjectId) -> Result<Vec<u8>, Self::Error> {
        self.read_file(id).map_err(FileStoreError::IOError)
    }

    fn check_blob_exists(&self, id: &ObjectId) -> Result<bool, Self::Error> {
        let id = &id.id;
        let hierarchy1 = &id[0..2];
        let hierarchy2 = &id[2..4];
        let path = self.root.join(hierarchy1).join(hierarchy2).join(id);

        Ok(path.exists())
    }

    fn save_tree(&self, tree: &SagittaTreeObject) -> Result<ObjectId, Self::Error> {
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

    fn save_commit(&self, commit: &SagittaCommitObject) -> Result<ObjectId, Self::Error> {
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

    fn update_trunk_head(&self, commit_id: &ObjectId) -> Result<(), Self::Error> {
        let path = self.root.join("trunk");
        let mut file = File::create(path).map_err(FileStoreError::IOError)?;
        file.write_all(commit_id.id.as_bytes())
            .map_err(FileStoreError::IOError)?;
        Ok(())
    }

    fn get_trunk_head(&self) -> Result<ObjectId, Self::Error> {
        let path = self.root.join("trunk");
        let mut file = File::open(path).map_err(FileStoreError::IOError)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(FileStoreError::IOError)?;
        let id = std::str::from_utf8(&buf).unwrap();
        Ok(ObjectId { id: id.to_string() })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sagitta_objects::*;
    use tempfile::tempdir;

    #[test]
    fn test_object_id_length_is_64() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().to_path_buf());

        let blob = b"Hello, world!".to_vec();
        let id = store.save_blob(&blob).unwrap();

        assert_eq!(id.id.len(), 64);
    }

    #[test]
    fn test_save_blob() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().to_path_buf());

        let blob = b"Hello, world!".to_vec();
        let id = store.save_blob(&blob).unwrap();

        let blob = store.get_blob(&id).unwrap();
        assert_eq!(blob, b"Hello, world!");
    }

    #[test]
    fn test_check_blob_exists() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().to_path_buf());

        let blob = b"Hello, world!".to_vec();
        let id = store.save_blob(&blob).unwrap();

        assert!(store.check_blob_exists(&id).unwrap());
        assert!(!store
            .check_blob_exists(&ObjectId {
                id: "1234".to_string()
            })
            .unwrap());
    }

    #[test]
    fn test_save_tree_file() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().to_path_buf());

        let file = SagittaTreeObject::File(SagittaTreeObjectFile {
            name: "file1".to_string(),
            blob_id: ObjectId {
                id: "1234".to_string(),
            },
            size: 10,
            mtime: std::time::SystemTime::now(),
            ctime: std::time::SystemTime::now(),
            perm: 0o644,
        });

        let id = store.save_tree(&file).unwrap();

        let tree = store.get_tree(&id).unwrap();
        assert_eq!(tree, file);
    }

    #[test]
    fn test_save_tree_dir() {
        let dir = tempdir().unwrap();
        let store = FileStore::new(dir.path().to_path_buf());

        let dir = SagittaTreeObject::Dir(SagittaTreeObjectDir {
            items: vec![ObjectId {
                id: "1234".to_string(),
            }],
            size: 10,
            mtime: std::time::SystemTime::now(),
            ctime: std::time::SystemTime::now(),
            perm: 0o755,
        });

        let id = store.save_tree(&dir).unwrap();

        let tree = store.get_tree(&id).unwrap();
        assert_eq!(tree, dir);
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
        let id = store.save_commit(&commit).unwrap();

        let commit = store.get_commit(&id).unwrap();
        assert_eq!(commit.tree_id.id, "1234");
        assert_eq!(commit.parent_commit_id.unwrap().id, "5678");
        assert_eq!(commit.message, "Hello, world!");
    }
}

use sagitta_objects::{ObjectId, SagittaCommitObject, SagittaTreeObject};

pub trait SagittaObjectsStore {
    type Error;

    fn save_blob(&self, blob: &[u8]) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;

    fn get_blob(&self, id: &ObjectId) -> Result<Vec<u8>, <Self as SagittaObjectsStore>::Error>;

    fn check_blob_exists(
        &self,
        id: &ObjectId,
    ) -> Result<bool, <Self as SagittaObjectsStore>::Error>;

    fn save_tree(
        &self,
        tree: &SagittaTreeObject,
    ) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;

    fn get_tree(
        &self,
        id: &ObjectId,
    ) -> Result<SagittaTreeObject, <Self as SagittaObjectsStore>::Error>;

    fn save_commit(
        &self,
        commit: &SagittaCommitObject,
    ) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;

    fn get_commit(
        &self,
        id: &ObjectId,
    ) -> Result<SagittaCommitObject, <Self as SagittaObjectsStore>::Error>;

    fn update_trunk_head(
        &self,
        commit_id: &ObjectId,
    ) -> Result<(), <Self as SagittaObjectsStore>::Error>;

    fn get_trunk_head(&self) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;
}

use sagitta_objects::{ObjectId, SagittaBlobObject, SagittaCommitObject, SagittaTreeObject};

pub trait SagittaObjectsStore {
    type Error;

    fn save_blob(
        &self,
        blob: SagittaBlobObject,
    ) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;
    fn get_blob(
        &self,
        id: &ObjectId,
    ) -> Result<SagittaBlobObject, <Self as SagittaObjectsStore>::Error>;
    fn save_tree(
        &self,
        tree: SagittaTreeObject,
    ) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;
    fn get_tree(
        &self,
        id: &ObjectId,
    ) -> Result<SagittaTreeObject, <Self as SagittaObjectsStore>::Error>;
    fn save_commit(
        &self,
        commit: SagittaCommitObject,
    ) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;
    fn get_commit(
        &self,
        id: &ObjectId,
    ) -> Result<SagittaCommitObject, <Self as SagittaObjectsStore>::Error>;
}

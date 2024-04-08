use sagitta_objects::{ObjectId, SagittaCommitObject, SagittaTreeObject};

pub trait SagittaObjectsStore {
    type Error;

    fn save_blob(
        &self,
        workspace: Option<String>,
        blob: &[u8],
    ) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;

    fn get_blob(
        &self,
        workspace: Option<String>,
        id: &ObjectId,
    ) -> Result<Vec<u8>, <Self as SagittaObjectsStore>::Error>;

    fn check_blob_exists(
        &self,
        workspace: Option<String>,
        id: &ObjectId,
    ) -> Result<bool, <Self as SagittaObjectsStore>::Error>;

    fn save_tree(
        &self,
        workspace: Option<String>,
        tree: &SagittaTreeObject,
    ) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;

    fn get_tree(
        &self,
        workspace: Option<String>,
        id: &ObjectId,
    ) -> Result<SagittaTreeObject, <Self as SagittaObjectsStore>::Error>;

    fn save_commit(
        &self,
        workspace: Option<String>,
        commit: &SagittaCommitObject,
    ) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;

    fn get_commit(
        &self,
        workspace: Option<String>,
        id: &ObjectId,
    ) -> Result<SagittaCommitObject, <Self as SagittaObjectsStore>::Error>;

    fn workspace_list(&self) -> Result<Vec<String>, <Self as SagittaObjectsStore>::Error>;

    fn workspace_create(
        &self,
        workspace_name: &str,
    ) -> Result<(), <Self as SagittaObjectsStore>::Error>;

    fn update_trunk_head(
        &self,
        commit_id: &ObjectId,
    ) -> Result<(), <Self as SagittaObjectsStore>::Error>;

    fn get_trunk_head(&self) -> Result<ObjectId, <Self as SagittaObjectsStore>::Error>;
}

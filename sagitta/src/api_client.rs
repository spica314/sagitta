use sagitta_api_schema::{
    blob::read::{BlobReadRequest, BlobReadResponse},
    trunk::get_head::{TrunkGetHeadRequest, TrunkGetHeadResponse},
    workspace::{
        create::{WorkspaceCreateRequest, WorkspaceCreateResponse},
        get_head::{WorkspaceGetHeadRequest, WorkspaceGetHeadResponse},
        list::{WorkspaceListRequest, WorkspaceListResponse},
    },
};
use sagitta_objects::ObjectId;

#[derive(Debug, Clone)]
pub struct SagittaApiClient {
    pub base_url: String,
}

#[derive(Debug)]
pub enum SagittaApiClientError {
    Ureq(Box<ureq::Error>),
    IO(Box<std::io::Error>),
    Cbor(Box<serde_cbor::Error>),
}

impl SagittaApiClient {
    pub fn new(base_url: String) -> Self {
        let mut base_url = base_url;
        if base_url.ends_with('/') {
            base_url.pop();
        }
        Self { base_url }
    }

    pub fn trunk_get_head(&self) -> Result<TrunkGetHeadResponse, SagittaApiClientError> {
        let url = format!("{}/trunk/get-head", self.base_url);
        let head_id_res: TrunkGetHeadResponse = ureq::post(&url)
            .send_json(TrunkGetHeadRequest {})
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(head_id_res)
    }

    pub fn workspace_get_head(
        &self,
        workspace_id: &str,
    ) -> Result<WorkspaceGetHeadResponse, SagittaApiClientError> {
        let url = format!("{}/workspace/get-head", self.base_url);
        let head_id_res: WorkspaceGetHeadResponse = ureq::post(&url)
            .send_json(WorkspaceGetHeadRequest {
                workspace_id: workspace_id.to_string(),
            })
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(head_id_res)
    }

    pub fn blob_read(&self, id: &ObjectId) -> Result<BlobReadResponse, SagittaApiClientError> {
        let url = format!("{}/blob/read", self.base_url);
        let commit_res: BlobReadResponse = ureq::post(&url)
            .send_json(BlobReadRequest { id: id.clone() })
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(commit_res)
    }

    pub fn workspace_create(
        &self,
        workspace_name: &str,
    ) -> Result<WorkspaceCreateResponse, SagittaApiClientError> {
        let url = format!("{}/workspace/create", self.base_url);
        let workspace_create_res: WorkspaceCreateResponse = ureq::post(&url)
            .send_json(WorkspaceCreateRequest {
                name: workspace_name.to_string(),
            })
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(workspace_create_res)
    }

    pub fn workspace_list(&self) -> Result<WorkspaceListResponse, SagittaApiClientError> {
        let url = format!("{}/workspace/list", self.base_url);
        let workspace_list_res: WorkspaceListResponse = ureq::post(&url)
            .send_json(WorkspaceListRequest {})
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(workspace_list_res)
    }

    pub fn blob_read_as_commit_object(
        &self,
        id: &ObjectId,
    ) -> Result<sagitta_objects::SagittaCommitObject, SagittaApiClientError> {
        let commit_res = self.blob_read(id)?;
        let commit: sagitta_objects::SagittaCommitObject =
            serde_cbor::from_reader(commit_res.blob.as_slice())
                .map_err(|e| SagittaApiClientError::Cbor(Box::new(e)))?;
        Ok(commit)
    }

    pub fn blob_read_as_tree_object(
        &self,
        id: &ObjectId,
    ) -> Result<sagitta_objects::SagittaTreeObject, SagittaApiClientError> {
        let tree_res = self.blob_read(id)?;
        let tree: sagitta_objects::SagittaTreeObject =
            serde_cbor::from_reader(tree_res.blob.as_slice())
                .map_err(|e| SagittaApiClientError::Cbor(Box::new(e)))?;
        Ok(tree)
    }
}

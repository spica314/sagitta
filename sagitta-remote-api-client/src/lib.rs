use sagitta_remote_api_schema::v2::{
    commit::{V2CommitRequest, V2CommitResponse},
    create_workspace::{V2CreateWorkspaceRequest, V2CreateWorkspaceResponse},
    get_attr::{V2GetAttrRequest, V2GetAttrResponse},
    get_file_blob_id::{V2GetFileBlobIdRequest, V2GetFileBlobIdResponse},
    get_workspaces::{V2GetWorkspacesRequest, V2GetWorkspacesResponse},
    read_blob::{V2ReadBlobRequest, V2ReadBlobResponse},
    read_dir::{V2ReadDirRequest, V2ReadDirResponse},
    sync_files_with_workspace::{
        V2SyncFilesWithWorkspaceRequest, V2SyncFilesWithWorkspaceResponse,
    },
    write_blob::{V2WriteBlobRequest, V2WriteBlobResponse},
};

#[derive(Debug, Clone)]
pub struct SagittaApiClient {
    pub base_url: String,
}

#[derive(Debug)]
pub enum SagittaApiClientError {
    Ureq(Box<ureq::Error>),
    IO(Box<std::io::Error>),
}

impl SagittaApiClient {
    pub fn new(base_url: String) -> Self {
        let mut base_url = base_url;
        if base_url.ends_with('/') {
            base_url.pop();
        }
        Self { base_url }
    }

    pub fn v2_read_dir(
        &self,
        request: V2ReadDirRequest,
    ) -> Result<V2ReadDirResponse, SagittaApiClientError> {
        let url = format!("{}/v2/read-dir", self.base_url);
        let read_dir_res: V2ReadDirResponse = ureq::post(&url)
            .send_json(request)
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(read_dir_res)
    }

    pub fn v2_get_attr(
        &self,
        request: V2GetAttrRequest,
    ) -> Result<V2GetAttrResponse, SagittaApiClientError> {
        let url = format!("{}/v2/get-attr", self.base_url);
        let get_attr_res: V2GetAttrResponse = ureq::post(&url)
            .send_json(request)
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(get_attr_res)
    }

    pub fn v2_get_file_blob_id(
        &self,
        request: V2GetFileBlobIdRequest,
    ) -> Result<V2GetFileBlobIdResponse, SagittaApiClientError> {
        let url = format!("{}/v2/get-file-blob-id", self.base_url);
        let get_file_blob_id_res: V2GetFileBlobIdResponse = ureq::post(&url)
            .send_json(request)
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(get_file_blob_id_res)
    }

    pub fn v2_read_blob_request(
        &self,
        request: V2ReadBlobRequest,
    ) -> Result<V2ReadBlobResponse, SagittaApiClientError> {
        let url = format!("{}/v2/read-blob", self.base_url);
        let read_blob_res: V2ReadBlobResponse = ureq::post(&url)
            .send_json(request)
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(read_blob_res)
    }

    pub fn v2_get_workspaces(
        &self,
        request: V2GetWorkspacesRequest,
    ) -> Result<V2GetWorkspacesResponse, SagittaApiClientError> {
        let url = format!("{}/v2/get-workspaces", self.base_url);
        let get_workspaces_res: V2GetWorkspacesResponse = ureq::post(&url)
            .send_json(request)
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(get_workspaces_res)
    }

    pub fn v2_create_workspace(
        &self,
        request: V2CreateWorkspaceRequest,
    ) -> Result<V2CreateWorkspaceResponse, SagittaApiClientError> {
        let url = format!("{}/v2/create-workspace", self.base_url);
        let create_workspace_res: V2CreateWorkspaceResponse = ureq::post(&url)
            .send_json(request)
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(create_workspace_res)
    }

    pub fn v2_commit(
        &self,
        request: V2CommitRequest,
    ) -> Result<V2CommitResponse, SagittaApiClientError> {
        let url = format!("{}/v2/commit", self.base_url);
        let commit_res: V2CommitResponse = ureq::post(&url)
            .send_json(request)
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(commit_res)
    }

    pub fn v2_write_blob(
        &self,
        request: V2WriteBlobRequest,
    ) -> Result<V2WriteBlobResponse, SagittaApiClientError> {
        let url = format!("{}/v2/write-blob", self.base_url);
        let write_blob_res: V2WriteBlobResponse = ureq::post(&url)
            .send_json(request)
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(write_blob_res)
    }

    pub fn v2_sync_files_with_workspace(
        &self,
        request: V2SyncFilesWithWorkspaceRequest,
    ) -> Result<V2SyncFilesWithWorkspaceResponse, SagittaApiClientError> {
        let url = format!("{}/v2/sync-files-with-workspace", self.base_url);
        let sync_files_with_workspace_res: V2SyncFilesWithWorkspaceResponse = ureq::post(&url)
            .send_json(request)
            .map_err(|e| SagittaApiClientError::Ureq(Box::new(e)))?
            .into_json()
            .map_err(|e| SagittaApiClientError::IO(Box::new(e)))?;
        Ok(sync_files_with_workspace_res)
    }
}

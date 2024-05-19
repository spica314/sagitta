use std::path::PathBuf;

use sagitta_remote_system_db::{
    db::SagittaRemoteSystemDB, CommitRequest, CommitResponse, CreateWorkspaceRequest,
    CreateWorkspaceResponse, GetAttrRequest, GetAttrResponse, GetFileBlobIdRequest,
    GetFileBlobIdResponse, GetWorkspacesRequest, GetWorkspacesResponse, ReadDirRequest,
    ReadDirResponse, SagittaRemoteSystemDBTrait,
};

#[derive(Debug, Clone)]
pub struct RemoteSystemWorkspaceManager {
    base_url: PathBuf,
    db: SagittaRemoteSystemDB,
}

#[derive(Debug)]
pub enum Error {
    WorkspaceAlreadyExists,
    IOError(std::io::Error),
    SerdeCborError(serde_cbor::error::Error),
    Error,
    SagittaRemoteSystemDBError(sagitta_remote_system_db::SagittaRemoteSystemDBError),
}

#[derive(Debug)]
pub struct ReadBlobRequest {
    pub blob_id: String,
}

#[derive(Debug)]
pub enum ReadBlobResponse {
    Found { blob: Vec<u8> },
    NotFound,
}

#[derive(Debug)]
pub struct WriteBlobRequest {
    pub blob: Vec<u8>,
    pub blob_id: String,
}

#[derive(Debug)]
pub struct WriteBlobResponse {
    pub blob_id: String,
}

impl RemoteSystemWorkspaceManager {
    pub async fn new(base_url: PathBuf, db: SagittaRemoteSystemDB) -> Self {
        Self {
            base_url: base_url.clone(),
            db,
        }
    }

    pub fn read_dir(&self, request: ReadDirRequest) -> Result<ReadDirResponse, Error> {
        self.db
            .read_dir(request)
            .map_err(Error::SagittaRemoteSystemDBError)
    }

    pub fn get_attr(&self, request: GetAttrRequest) -> Result<GetAttrResponse, Error> {
        self.db
            .get_attr(request)
            .map_err(Error::SagittaRemoteSystemDBError)
    }

    pub fn get_file_blob_id(
        &self,
        request: GetFileBlobIdRequest,
    ) -> Result<GetFileBlobIdResponse, Error> {
        self.db
            .get_file_blob_id(request)
            .map_err(Error::SagittaRemoteSystemDBError)
    }

    pub fn get_workspaces(
        &self,
        request: GetWorkspacesRequest,
    ) -> Result<GetWorkspacesResponse, Error> {
        self.db
            .get_workspaces(request)
            .map_err(Error::SagittaRemoteSystemDBError)
    }

    pub fn create_workspace(
        &self,
        request: CreateWorkspaceRequest,
    ) -> Result<CreateWorkspaceResponse, Error> {
        self.db
            .create_workspace(request)
            .map_err(Error::SagittaRemoteSystemDBError)
    }

    pub fn read_blob(&self, request: ReadBlobRequest) -> Result<ReadBlobResponse, Error> {
        let hierarchy1 = &request.blob_id[0..2];
        let hierarchy2 = &request.blob_id[2..4];
        let object_path = self
            .base_url
            .join("objects")
            .join(hierarchy1)
            .join(hierarchy2)
            .join(&request.blob_id);
        let r = std::fs::read(object_path).map_err(Error::IOError)?;
        Ok(ReadBlobResponse::Found { blob: r })
    }

    pub fn write_blob(&self, request: WriteBlobRequest) -> Result<WriteBlobResponse, Error> {
        let blob_id = request.blob_id;
        let hierarchy1 = &blob_id[0..2];
        let hierarchy2 = &blob_id[2..4];
        let object_parent_path = self
            .base_url
            .join("objects")
            .join(hierarchy1)
            .join(hierarchy2);
        std::fs::create_dir_all(&object_parent_path).map_err(Error::IOError)?;
        let object_path = object_parent_path.join(&blob_id);
        std::fs::write(object_path, request.blob).map_err(Error::IOError)?;
        Ok(WriteBlobResponse { blob_id })
    }

    pub fn commit(&self, request: CommitRequest) -> Result<CommitResponse, Error> {
        self.db
            .commit(request)
            .map_err(Error::SagittaRemoteSystemDBError)
    }

    pub fn create_or_get_blob(
        &self,
        request: sagitta_remote_system_db::CreateOrGetBlobRequest,
    ) -> Result<sagitta_remote_system_db::CreateOrGetBlobResponse, Error> {
        self.db
            .create_or_get_blob(request)
            .map_err(Error::SagittaRemoteSystemDBError)
    }

    pub fn sync_files_to_workspace(
        &self,
        request: sagitta_remote_system_db::SyncFilesToWorkspaceRequest,
    ) -> Result<sagitta_remote_system_db::SyncFilesToWorkspaceResponse, Error> {
        self.db
            .sync_files_to_workspace(request)
            .map_err(Error::SagittaRemoteSystemDBError)
    }
}

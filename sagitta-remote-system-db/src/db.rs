use crate::{sqlite::SagittaRemoteSystemDBBySqlite, SagittaRemoteSystemDBTrait};

#[derive(Debug, Clone)]
pub enum SagittaRemoteSystemDB {
    Sqlite(SagittaRemoteSystemDBBySqlite),
}

impl SagittaRemoteSystemDBTrait for SagittaRemoteSystemDB {
    fn migration(&self) -> Result<(), crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.migration(),
        }
    }

    fn create_workspace(
        &self,
        request: crate::CreateWorkspaceRequest,
    ) -> Result<crate::CreateWorkspaceResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.create_workspace(request),
        }
    }

    fn get_workspaces(
        &self,
        request: crate::GetWorkspacesRequest,
    ) -> Result<crate::GetWorkspacesResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.get_workspaces(request),
        }
    }

    fn delete_workspace(
        &self,
        request: crate::DeleteWorkspaceRequest,
    ) -> Result<crate::DeleteWorkspaceResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.delete_workspace(request),
        }
    }

    fn create_or_get_blob(
        &self,
        request: crate::CreateOrGetBlobRequest,
    ) -> Result<crate::CreateOrGetBlobResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.create_or_get_blob(request),
        }
    }

    fn search_blob_by_hash(
        &self,
        request: crate::SearchBlobByHashRequest,
    ) -> Result<crate::SearchBlobByHashResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.search_blob_by_hash(request),
        }
    }

    fn get_or_create_file_path(
        &self,
        request: crate::GetOrCreateFilePathRequest,
    ) -> Result<crate::GetOrCreateFilePathResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.get_or_create_file_path(request),
        }
    }

    fn sync_files_to_workspace(
        &self,
        sync_files_to_workspace_request: crate::SyncFilesToWorkspaceRequest,
    ) -> Result<crate::SyncFilesToWorkspaceResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => {
                db.sync_files_to_workspace(sync_files_to_workspace_request)
            }
        }
    }

    fn get_workspace_changelist(
        &self,
        request: crate::GetWorkspaceChangelistRequest,
    ) -> Result<crate::GetWorkspaceChangelistResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.get_workspace_changelist(request),
        }
    }

    fn commit(
        &self,
        request: crate::CommitRequest,
    ) -> Result<crate::CommitResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.commit(request),
        }
    }

    fn get_all_trunk_files(
        &self,
        request: crate::GetAllTrunkFilesRequest,
    ) -> Result<crate::GetAllTrunkFilesResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.get_all_trunk_files(request),
        }
    }

    fn get_commit_history(
        &self,
        request: crate::GetCommitHistoryRequest,
    ) -> Result<crate::GetCommitHistoryResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.get_commit_history(request),
        }
    }

    fn read_dir(
        &self,
        request: crate::ReadDirRequest,
    ) -> Result<crate::ReadDirResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.read_dir(request),
        }
    }

    fn get_attr(
        &self,
        request: crate::GetAttrRequest,
    ) -> Result<crate::GetAttrResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.get_attr(request),
        }
    }

    fn get_file_blob_id(
        &self,
        request: crate::GetFileBlobIdRequest,
    ) -> Result<crate::GetFileBlobIdResponse, crate::SagittaRemoteSystemDBError> {
        match self {
            SagittaRemoteSystemDB::Sqlite(db) => db.get_file_blob_id(request),
        }
    }
}

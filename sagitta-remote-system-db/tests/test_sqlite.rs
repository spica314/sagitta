use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rand_pcg::Pcg64Mcg;
use sagitta_common::clock::Clock;
use sagitta_remote_system_db::{sqlite::SagittaRemoteSystemDBBySqlite, *};
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};
use tempfile::NamedTempFile;

fn setup_db(path: PathBuf) -> SagittaRemoteSystemDBBySqlite {
    let clock = Clock::new_with_fixed_time(
        SystemTime::UNIX_EPOCH + Duration::from_secs(40 * 365 * 24 * 60 * 60),
    );
    let rng = Pcg64Mcg::new(42);
    let rng = ChaCha20Rng::from_rng(rng).unwrap();
    let db = SagittaRemoteSystemDBBySqlite::new(path, rng, clock).unwrap();
    db.migration().unwrap();
    db
}

#[test]
fn test_sqlite_workspace_1() {
    let file = NamedTempFile::new().unwrap();
    let path = file.into_temp_path();
    let path = path.to_path_buf();
    let db = setup_db(path);

    let res1 = db
        .create_workspace(CreateWorkspaceRequest {
            workspace_name: "workspace1".to_string(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res1);
    let workspace1_id = res1.workspace_id;

    let res2 = db
        .get_workspaces(GetWorkspacesRequest {
            contains_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2);

    let res3 = db
        .delete_workspace(DeleteWorkspaceRequest {
            workspace_id: workspace1_id.clone(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3);

    let res4 = db
        .get_workspaces(GetWorkspacesRequest {
            contains_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res4);

    let res5 = db
        .get_workspaces(GetWorkspacesRequest {
            contains_deleted: true,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res5);
}

#[test]
fn test_sqlite_workspace_2() {
    let file = NamedTempFile::new().unwrap();
    let path = file.into_temp_path();
    let path = path.to_path_buf();
    let db = setup_db(path);

    let res1 = db
        .create_or_get_blob(CreateOrGetBlobRequest {
            hash: "hash1".to_string(),
            size: 10,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res1);

    let res2 = db
        .search_blob_by_hash(SearchBlobByHashRequest {
            hash: "hash1".to_string(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2);
}

#[test]
fn test_sqlite_workspace_3() {
    let file = NamedTempFile::new().unwrap();
    let path = file.into_temp_path();
    let path = path.to_path_buf();
    let db = setup_db(path);

    let res1 = db
        .get_or_create_file_path(GetOrCreateFilePathRequest {
            path: vec!["foo".to_string(), "test.txt".to_string()],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res1);

    let res2 = db
        .get_or_create_file_path(GetOrCreateFilePathRequest {
            path: vec!["foo".to_string()],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2);
}

#[test]
fn test_sqlite_workspace_4() {
    let file = NamedTempFile::new().unwrap();
    let path = file.into_temp_path();
    let path = path.to_path_buf();
    let db = setup_db(path);

    let res1 = db
        .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
            workspace_id: "workspace1".to_string(),
            items: vec![
                SyncFilesToWorkspaceRequestItem::UpsertFile {
                    file_path: vec!["foo".to_string(), "test.txt".to_string()],
                    blob_id: "blob1".to_string(),
                    permission: 0o644,
                },
                SyncFilesToWorkspaceRequestItem::UpsertDir {
                    file_path: vec!["bar".to_string()],
                    permission: 0o755,
                },
                SyncFilesToWorkspaceRequestItem::UpsertDir {
                    file_path: vec!["foo".to_string(), "bar".to_string()],
                    permission: 0o755,
                },
            ],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res1);

    let res1_changelist = db
        .get_workspace_changelist(GetWorkspaceChangelistRequest {
            workspace_id: "workspace1".to_string(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res1_changelist);

    let res2 = db
        .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
            workspace_id: "workspace1".to_string(),
            items: vec![SyncFilesToWorkspaceRequestItem::UpsertFile {
                file_path: vec!["foo".to_string(), "test.txt".to_string()],
                blob_id: "blob2".to_string(),
                permission: 0o644,
            }],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2);

    let res2_changelist = db
        .get_workspace_changelist(GetWorkspaceChangelistRequest {
            workspace_id: "workspace1".to_string(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2_changelist);

    let res3 = db
        .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
            workspace_id: "workspace1".to_string(),
            items: vec![
                SyncFilesToWorkspaceRequestItem::DeleteFile {
                    file_path: vec!["foo".to_string(), "test.txt".to_string()],
                },
                SyncFilesToWorkspaceRequestItem::DeleteDir {
                    file_path: vec!["bar".to_string()],
                },
                SyncFilesToWorkspaceRequestItem::DeleteDir {
                    file_path: vec!["foo".to_string(), "bar".to_string()],
                },
            ],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3);

    let res3_changelist = db
        .get_workspace_changelist(GetWorkspaceChangelistRequest {
            workspace_id: "workspace1".to_string(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3_changelist);
}

#[test]
fn test_sqlite_workspace_5() {
    let file = NamedTempFile::new().unwrap();
    let path = file.into_temp_path();
    let path = path.to_path_buf();
    let db = setup_db(path);

    let res1 = db
        .create_workspace(CreateWorkspaceRequest {
            workspace_name: "workspace1".to_string(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res1);
    let workspace_id = res1.workspace_id;

    let blob_id_1 = db
        .create_or_get_blob(CreateOrGetBlobRequest {
            hash: "hash1".to_string(),
            size: 10,
        })
        .unwrap();
    insta::assert_debug_snapshot!(blob_id_1);

    let blob_id_2 = db
        .create_or_get_blob(CreateOrGetBlobRequest {
            hash: "hash2".to_string(),
            size: 20,
        })
        .unwrap();
    insta::assert_debug_snapshot!(blob_id_2);

    let res2 = db
        .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
            workspace_id: workspace_id.clone(),
            items: vec![
                SyncFilesToWorkspaceRequestItem::UpsertFile {
                    file_path: vec!["foo".to_string(), "test.txt".to_string()],
                    blob_id: blob_id_1.blob_id().to_string(),
                    permission: 0o644,
                },
                SyncFilesToWorkspaceRequestItem::UpsertDir {
                    file_path: vec!["bar".to_string()],
                    permission: 0o755,
                },
                SyncFilesToWorkspaceRequestItem::UpsertDir {
                    file_path: vec!["foo".to_string(), "bar".to_string()],
                    permission: 0o755,
                },
            ],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2);

    let res2_changelist = db
        .get_workspace_changelist(GetWorkspaceChangelistRequest {
            workspace_id: workspace_id.clone(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2_changelist);

    let res3 = db
        .commit(CommitRequest {
            workspace_id: workspace_id.clone(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3);

    let res3_files = db.get_all_trunk_files(GetAllTrunkFilesRequest {}).unwrap();
    insta::assert_debug_snapshot!(res3_files);

    let res3_dir_files_1 = db
        .read_dir(ReadDirRequest {
            workspace_id: None,
            file_path: vec![],
            include_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3_dir_files_1);

    let res3_dir_files_2 = db
        .read_dir(ReadDirRequest {
            workspace_id: None,
            file_path: vec!["foo".to_string()],
            include_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3_dir_files_2);

    let res3_dir_files_3 = db
        .read_dir(ReadDirRequest {
            workspace_id: None,
            file_path: vec!["bar".to_string()],
            include_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3_dir_files_3);

    let res4 = db
        .create_workspace(CreateWorkspaceRequest {
            workspace_name: "workspace2".to_string(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res4);
    let workspace_id = res4.workspace_id;

    let res5 = db
        .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
            workspace_id: workspace_id.clone(),
            items: vec![SyncFilesToWorkspaceRequestItem::UpsertFile {
                file_path: vec!["foo".to_string(), "test.txt".to_string()],
                blob_id: blob_id_2.blob_id().to_string(),
                permission: 0o644,
            }],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res5);

    let res5_changelist = db
        .get_workspace_changelist(GetWorkspaceChangelistRequest {
            workspace_id: workspace_id.clone(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res5_changelist);

    let res6 = db
        .commit(CommitRequest {
            workspace_id: workspace_id.clone(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res6);

    let res6_files = db.get_all_trunk_files(GetAllTrunkFilesRequest {}).unwrap();
    insta::assert_debug_snapshot!(res6_files);

    let res7 = db
        .create_workspace(CreateWorkspaceRequest {
            workspace_name: "workspace3".to_string(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res7);
    let workspace_id = res7.workspace_id;

    let res8 = db
        .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
            workspace_id: workspace_id.clone(),
            items: vec![
                SyncFilesToWorkspaceRequestItem::DeleteFile {
                    file_path: vec!["foo".to_string(), "test.txt".to_string()],
                },
                SyncFilesToWorkspaceRequestItem::DeleteDir {
                    file_path: vec!["bar".to_string()],
                },
                SyncFilesToWorkspaceRequestItem::DeleteDir {
                    file_path: vec!["foo".to_string(), "bar".to_string()],
                },
            ],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res8);

    let res8_changelist = db
        .get_workspace_changelist(GetWorkspaceChangelistRequest {
            workspace_id: workspace_id.clone(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res8_changelist);

    let res8_dir_files_1 = db
        .read_dir(ReadDirRequest {
            workspace_id: Some(workspace_id.clone()),
            file_path: vec![],
            include_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res8_dir_files_1);

    let res8_dir_files_1d = db
        .read_dir(ReadDirRequest {
            workspace_id: Some(workspace_id.clone()),
            file_path: vec![],
            include_deleted: true,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res8_dir_files_1d);

    let res8_dir_files_2 = db
        .read_dir(ReadDirRequest {
            workspace_id: Some(workspace_id.clone()),
            file_path: vec!["foo".to_string()],
            include_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res8_dir_files_2);

    let res8_dir_files_2d = db
        .read_dir(ReadDirRequest {
            workspace_id: Some(workspace_id.clone()),
            file_path: vec!["foo".to_string()],
            include_deleted: true,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res8_dir_files_2d);

    let res8_dir_files_3 = db
        .read_dir(ReadDirRequest {
            workspace_id: Some(workspace_id.clone()),
            file_path: vec!["bar".to_string()],
            include_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res8_dir_files_3);

    let res8_dir_files_3d = db
        .read_dir(ReadDirRequest {
            workspace_id: Some(workspace_id.clone()),
            file_path: vec!["bar".to_string()],
            include_deleted: true,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res8_dir_files_3d);

    let res9 = db
        .commit(CommitRequest {
            workspace_id: workspace_id.clone(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res9);

    let res9_files = db.get_all_trunk_files(GetAllTrunkFilesRequest {}).unwrap();
    insta::assert_debug_snapshot!(res9_files);

    let res9_dir_files_1 = db
        .read_dir(ReadDirRequest {
            workspace_id: None,
            file_path: vec![],
            include_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res9_dir_files_1);

    let res9_dir_files_2 = db
        .read_dir(ReadDirRequest {
            workspace_id: None,
            file_path: vec!["foo".to_string()],
            include_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res9_dir_files_2);

    let res9_dir_files_3 = db
        .read_dir(ReadDirRequest {
            workspace_id: None,
            file_path: vec!["bar".to_string()],
            include_deleted: false,
        })
        .unwrap();
    insta::assert_debug_snapshot!(res9_dir_files_3);

    let res10 = db
        .get_commit_history(GetCommitHistoryRequest { take: 10 })
        .unwrap();
    insta::assert_debug_snapshot!(res10);
}

#[test]
fn test_sqlite_workspace_6() {
    let file = NamedTempFile::new().unwrap();
    let path = file.into_temp_path();
    let path = path.to_path_buf();
    let db = setup_db(path);

    let res1 = db
        .create_workspace(CreateWorkspaceRequest {
            workspace_name: "workspace1".to_string(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res1);
    let workspace_id = res1.workspace_id;

    let blob_id_1 = db
        .create_or_get_blob(CreateOrGetBlobRequest {
            hash: "hash1".to_string(),
            size: 10,
        })
        .unwrap();
    insta::assert_debug_snapshot!(blob_id_1);

    let res2 = db
        .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
            workspace_id: workspace_id.clone(),
            items: vec![
                SyncFilesToWorkspaceRequestItem::UpsertFile {
                    file_path: vec!["foo".to_string(), "test.txt".to_string()],
                    blob_id: blob_id_1.blob_id().to_string(),
                    permission: 0o644,
                },
                SyncFilesToWorkspaceRequestItem::UpsertDir {
                    file_path: vec!["bar".to_string()],
                    permission: 0o755,
                },
                SyncFilesToWorkspaceRequestItem::UpsertDir {
                    file_path: vec!["foo".to_string(), "bar".to_string()],
                    permission: 0o755,
                },
            ],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2);

    let res2_attr1 = db
        .get_attr(GetAttrRequest {
            workspace_id: Some(workspace_id.clone()),
            file_path: vec!["foo".to_string(), "test.txt".to_string()],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2_attr1);

    let res2_attr2 = db
        .get_attr(GetAttrRequest {
            workspace_id: Some(workspace_id.clone()),
            file_path: vec!["foo".to_string()],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2_attr2);

    let res2_attr3 = db
        .get_attr(GetAttrRequest {
            workspace_id: Some(workspace_id.clone()),
            file_path: vec!["bar".to_string()],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2_attr3);

    let res2_attr4 = db
        .get_attr(GetAttrRequest {
            workspace_id: Some(workspace_id.clone()),
            file_path: vec!["baz".to_string()],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res2_attr4);

    let res3 = db
        .commit(CommitRequest {
            workspace_id: workspace_id.clone(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3);

    let res3_attr1 = db
        .get_attr(GetAttrRequest {
            workspace_id: None,
            file_path: vec!["foo".to_string(), "test.txt".to_string()],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3_attr1);

    let res3_attr2 = db
        .get_attr(GetAttrRequest {
            workspace_id: None,
            file_path: vec!["foo".to_string()],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3_attr2);

    let res3_attr3 = db
        .get_attr(GetAttrRequest {
            workspace_id: None,
            file_path: vec!["bar".to_string()],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3_attr3);

    let res3_attr4 = db
        .get_attr(GetAttrRequest {
            workspace_id: None,
            file_path: vec!["baz".to_string()],
        })
        .unwrap();
    insta::assert_debug_snapshot!(res3_attr4);
}

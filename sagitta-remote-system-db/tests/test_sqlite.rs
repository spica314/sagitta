use rand_pcg::Pcg64Mcg;
use sagitta_common::clock::Clock;
use sagitta_remote_system_db::{sqlite::SagittaRemoteSystemDBBySqlite, *};
use std::time::{Duration, SystemTime};
use tempfile::NamedTempFile;

#[test]
fn test_sqlite_workspace_1() {
    let file = NamedTempFile::new().unwrap();
    let path = file.into_temp_path();
    let path = path.to_path_buf();
    let clock = Clock::new_with_fixed_time(
        SystemTime::UNIX_EPOCH + Duration::from_secs(40 * 365 * 24 * 60 * 60),
    );
    let rng = Pcg64Mcg::new(42);
    let db = SagittaRemoteSystemDBBySqlite::new(path, rng, clock).unwrap();
    db.migration().unwrap();

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
    let clock = Clock::new_with_fixed_time(
        SystemTime::UNIX_EPOCH + Duration::from_secs(40 * 365 * 24 * 60 * 60),
    );
    let rng = Pcg64Mcg::new(42);
    let db = SagittaRemoteSystemDBBySqlite::new(path, rng, clock).unwrap();
    db.migration().unwrap();

    let res1 = db
        .create_blob(CreateBlobRequest {
            blob_id: "blob1".to_string(),
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
    let clock = Clock::new_with_fixed_time(
        SystemTime::UNIX_EPOCH + Duration::from_secs(40 * 365 * 24 * 60 * 60),
    );
    let rng = Pcg64Mcg::new(42);
    let db = SagittaRemoteSystemDBBySqlite::new(path, rng, clock).unwrap();
    db.migration().unwrap();

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
    let clock = Clock::new_with_fixed_time(
        SystemTime::UNIX_EPOCH + Duration::from_secs(40 * 365 * 24 * 60 * 60),
    );
    let rng = Pcg64Mcg::new(42);
    let db = SagittaRemoteSystemDBBySqlite::new(path, rng, clock).unwrap();
    db.migration().unwrap();

    let res1 = db
        .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
            workspace_id: "workspace1".to_string(),
            items: vec![
                SyncFilesToWorkspaceRequestItem::UpsertFile {
                    file_path: vec!["foo".to_string(), "test.txt".to_string()],
                    blob_id: "blob1".to_string(),
                },
                SyncFilesToWorkspaceRequestItem::UpsertDir {
                    file_path: vec!["bar".to_string()],
                },
                SyncFilesToWorkspaceRequestItem::UpsertDir {
                    file_path: vec!["foo".to_string(), "bar".to_string()],
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
    let clock = Clock::new_with_fixed_time(
        SystemTime::UNIX_EPOCH + Duration::from_secs(40 * 365 * 24 * 60 * 60),
    );
    let rng = Pcg64Mcg::new(42);
    let db = SagittaRemoteSystemDBBySqlite::new(path, rng, clock).unwrap();
    db.migration().unwrap();

    let res1 = db
        .create_workspace(CreateWorkspaceRequest {
            workspace_name: "workspace1".to_string(),
        })
        .unwrap();
    insta::assert_debug_snapshot!(res1);
    let workspace_id = res1.workspace_id;

    let res2 = db
        .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
            workspace_id: workspace_id.clone(),
            items: vec![
                SyncFilesToWorkspaceRequestItem::UpsertFile {
                    file_path: vec!["foo".to_string(), "test.txt".to_string()],
                    blob_id: "blob1".to_string(),
                },
                SyncFilesToWorkspaceRequestItem::UpsertDir {
                    file_path: vec!["bar".to_string()],
                },
                SyncFilesToWorkspaceRequestItem::UpsertDir {
                    file_path: vec!["foo".to_string(), "bar".to_string()],
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

    // let res1_changelist = db
    //     .get_workspace_changelist(GetWorkspaceChangelistRequest {
    //         workspace_id: "workspace1".to_string(),
    //     })
    //     .unwrap();
    // insta::assert_debug_snapshot!(res1_changelist);

    // let res2 = db
    //     .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
    //         workspace_id: "workspace1".to_string(),
    //         items: vec![SyncFilesToWorkspaceRequestItem::UpsertFile {
    //             file_path: vec!["foo".to_string(), "test.txt".to_string()],
    //             blob_id: "blob2".to_string(),
    //         }],
    //     })
    //     .unwrap();
    // insta::assert_debug_snapshot!(res2);

    // let res2_changelist = db
    //     .get_workspace_changelist(GetWorkspaceChangelistRequest {
    //         workspace_id: "workspace1".to_string(),
    //     })
    //     .unwrap();
    // insta::assert_debug_snapshot!(res2_changelist);

    // let res3 = db
    //     .sync_files_to_workspace(SyncFilesToWorkspaceRequest {
    //         workspace_id: "workspace1".to_string(),
    //         items: vec![
    //             SyncFilesToWorkspaceRequestItem::DeleteFile {
    //                 file_path: vec!["foo".to_string(), "test.txt".to_string()],
    //             },
    //             SyncFilesToWorkspaceRequestItem::DeleteDir {
    //                 file_path: vec!["bar".to_string()],
    //             },
    //             SyncFilesToWorkspaceRequestItem::DeleteDir {
    //                 file_path: vec!["foo".to_string(), "bar".to_string()],
    //             },
    //         ],
    //     })
    //     .unwrap();
    // insta::assert_debug_snapshot!(res3);

    // let res3_changelist = db
    //     .get_workspace_changelist(GetWorkspaceChangelistRequest {
    //         workspace_id: "workspace1".to_string(),
    //     })
    //     .unwrap();
    // insta::assert_debug_snapshot!(res3_changelist);
}

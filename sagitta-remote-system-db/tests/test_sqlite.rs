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
    db.migration();

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
    db.migration();

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

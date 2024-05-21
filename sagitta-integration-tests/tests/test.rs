use std::{process::Command, time::SystemTime};

use sagitta::{
    api_client::SagittaApiClient,
    fs::{run_fs, SagittaConfig},
};
use sagitta_common::clock::Clock;
use sagitta_local_system_workspace::LocalSystemWorkspaceManager;
use sagitta_remote_api_schema::v2::{
    commit::V2CommitRequest,
    create_workspace::{V2CreateWorkspaceRequest, V2CreateWorkspaceResponse},
    get_workspaces::V2GetWorkspacesRequest,
    sync_files_with_workspace::{
        V2SyncFilesWithWorkspaceRequest, V2SyncFilesWithWorkspaceRequestItem,
    },
    write_blob::V2WriteBlobRequest,
};
use sagitta_server::api::ServerConfig;
use serial_test::serial;
use tempfile::tempdir;
use tokio::runtime::Builder;

#[test]
#[serial]
fn test_2() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let tempdir1 = tempdir().unwrap();
    let fixed_system_time =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(40 * 365 * 24 * 60 * 60);
    let port = 8082;
    let config = ServerConfig {
        base_path: tempdir1.as_ref().to_path_buf(),
        is_main: false,
        clock: Clock::new_with_fixed_time(fixed_system_time),
        port,
    };

    runtime.spawn(async {
        sagitta_server::api::run_server(config).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));

    let local_system_workspace_base_path = tempdir().unwrap().as_ref().to_path_buf();

    let tempdir2 = tempdir().unwrap();
    let tempdir2_str = tempdir2.as_ref().to_str().unwrap().to_string();
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    std::thread::spawn(move || {
        let config = SagittaConfig {
            base_url: format!("http://localhost:{}", port),
            mountpoint: tempdir2_str,
            uid,
            gid,
            clock: Clock::new_with_fixed_time(fixed_system_time),
            local_system_workspace_base_path,
            debug_sleep_duration: None,
        };
        run_fs(config);
    });
    std::thread::sleep(std::time::Duration::from_secs(1));

    let client = SagittaApiClient::new(format!("http://localhost:{}", port));

    // setup files
    {
        // create workspace
        let workspace_id = client
            .v2_create_workspace(V2CreateWorkspaceRequest {
                name: "workspace1".to_string(),
            })
            .unwrap();
        let workspace_id = match workspace_id {
            V2CreateWorkspaceResponse::Ok { id } => id,
            _ => panic!("unexpected response"),
        };

        // create hello.txt
        let hello_blob_id = client
            .v2_write_blob(V2WriteBlobRequest {
                data: b"Hello, world!\n".to_vec(),
            })
            .unwrap();
        let hello_blob_id = hello_blob_id.blob_id;

        // create hello2.txt
        let hello2_blob_id = client
            .v2_write_blob(V2WriteBlobRequest {
                data: b"Hello, world!!\n".to_vec(),
            })
            .unwrap();
        let hello2_blob_id = hello2_blob_id.blob_id;

        // sync
        client
            .v2_sync_files_with_workspace(V2SyncFilesWithWorkspaceRequest {
                workspace_id: workspace_id.clone(),
                items: vec![
                    V2SyncFilesWithWorkspaceRequestItem::UpsertFile {
                        file_path: vec!["hello.txt".to_string()],
                        blob_id: hello_blob_id.clone(),
                    },
                    V2SyncFilesWithWorkspaceRequestItem::UpsertFile {
                        file_path: vec!["hello_dir".to_string(), "hello2.txt".to_string()],
                        blob_id: hello2_blob_id.clone(),
                    },
                ],
            })
            .unwrap();

        // commit
        client.v2_commit(V2CommitRequest { workspace_id }).unwrap();
    }

    let out0 = Command::new("ls")
        .arg("-lAUgG")
        .current_dir(tempdir2.path())
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out0);

    let mut out1_dir = tempdir2.path().to_path_buf();
    out1_dir.push("trunk");
    let out1 = Command::new("ls")
        .arg("-lAUgG")
        .current_dir(out1_dir)
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out1);

    let mut out2_dir = tempdir2.path().to_path_buf();
    out2_dir.push("trunk");
    let out2 = Command::new("cat")
        .arg("hello.txt")
        .current_dir(out2_dir)
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out2);

    let mut out3_dir = tempdir2.path().to_path_buf();
    out3_dir.push("trunk");
    out3_dir.push("hello_dir");
    let out3 = Command::new("ls")
        .arg("-lAUgG")
        .current_dir(&out3_dir)
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out3);

    let out4 = Command::new("cat")
        .arg("hello2.txt")
        .current_dir(out3_dir)
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out4);
}

#[test]
#[serial]
fn test_3() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let tempdir1 = tempdir().unwrap();
    let fixed_system_time =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(40 * 365 * 24 * 60 * 60);
    let port = 8083;
    let config = ServerConfig {
        base_path: tempdir1.as_ref().to_path_buf(),
        is_main: false,
        clock: Clock::new_with_fixed_time(fixed_system_time),
        port,
    };

    runtime.spawn(async {
        sagitta_server::api::run_server(config).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));

    let client = SagittaApiClient::new(format!("http://localhost:{}", port));
    client
        .v2_create_workspace(V2CreateWorkspaceRequest {
            name: "workspace1".to_string(),
        })
        .unwrap();
    let workspaces = client.v2_get_workspaces(V2GetWorkspacesRequest {}).unwrap();
    insta::assert_debug_snapshot!(workspaces);
}

#[test]
#[serial]
fn test_4() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let tempdir1 = tempdir().unwrap();
    let fixed_system_time =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(40 * 365 * 24 * 60 * 60);
    // port 8084 is used by GitHub Actions
    let port = 8085;
    let config = ServerConfig {
        base_path: tempdir1.as_ref().to_path_buf(),
        is_main: false,
        clock: Clock::new_with_fixed_time(fixed_system_time),
        port,
    };

    runtime.spawn(async {
        sagitta_server::api::run_server(config).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));

    let client = SagittaApiClient::new(format!("http://localhost:{}", port));
    client
        .v2_create_workspace(V2CreateWorkspaceRequest {
            name: "workspace1".to_string(),
        })
        .unwrap();

    // setup files
    {
        // create workspace
        let workspace_id = client
            .v2_create_workspace(V2CreateWorkspaceRequest {
                name: "workspace1".to_string(),
            })
            .unwrap();
        let workspace_id = match workspace_id {
            V2CreateWorkspaceResponse::Ok { id } => id,
            _ => panic!("unexpected response"),
        };

        // create hello.txt
        let hello_blob_id = client
            .v2_write_blob(V2WriteBlobRequest {
                data: b"Hello, world!\n".to_vec(),
            })
            .unwrap();
        let hello_blob_id = hello_blob_id.blob_id;

        // create hello2.txt
        let hello2_blob_id = client
            .v2_write_blob(V2WriteBlobRequest {
                data: b"Hello, world!!\n".to_vec(),
            })
            .unwrap();
        let hello2_blob_id = hello2_blob_id.blob_id;

        // sync
        client
            .v2_sync_files_with_workspace(V2SyncFilesWithWorkspaceRequest {
                workspace_id: workspace_id.clone(),
                items: vec![
                    V2SyncFilesWithWorkspaceRequestItem::UpsertFile {
                        file_path: vec!["hello.txt".to_string()],
                        blob_id: hello_blob_id.clone(),
                    },
                    V2SyncFilesWithWorkspaceRequestItem::UpsertFile {
                        file_path: vec!["hello_dir".to_string(), "hello2.txt".to_string()],
                        blob_id: hello2_blob_id.clone(),
                    },
                ],
            })
            .unwrap();

        // commit
        client.v2_commit(V2CommitRequest { workspace_id }).unwrap();
    }

    let local_system_workspace_base_path = tempdir().unwrap().as_ref().to_path_buf();

    let tempdir2 = tempdir().unwrap();
    let tempdir2_str = tempdir2.as_ref().to_str().unwrap().to_string();
    {
        let local_system_workspace_base_path = local_system_workspace_base_path.clone();
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        std::thread::spawn(move || {
            let config = SagittaConfig {
                base_url: format!("http://localhost:{}", port),
                mountpoint: tempdir2_str,
                uid,
                gid,
                clock: Clock::new_with_fixed_time(fixed_system_time),
                local_system_workspace_base_path: local_system_workspace_base_path.clone(),
                debug_sleep_duration: None,
            };
            run_fs(config);
        });
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let local_system_workspace_manager =
        LocalSystemWorkspaceManager::new(local_system_workspace_base_path.clone());

    local_system_workspace_manager
        .create_cow_file(
            "workspace1",
            &["cow.txt".to_string()],
            b"Hello, copy on write!",
        )
        .unwrap();

    local_system_workspace_manager
        .create_cow_file(
            "workspace1",
            &["cow_dir".to_string(), "cow.txt".to_string()],
            b"Hello, copy on write! (dir)",
        )
        .unwrap();

    let out0 = Command::new("ls")
        .arg("-lAUgG")
        .current_dir(tempdir2.path())
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out0);

    let workspace_path = tempdir2.path().join("workspace1");
    let out1 = Command::new("ls")
        .arg("-lAUgG")
        .current_dir(workspace_path.as_path())
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out1);

    let out2 = Command::new("cat")
        .arg("cow.txt")
        .current_dir(workspace_path.as_path())
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out2);

    let workspace_path = tempdir2.path().join("workspace1").join("cow_dir");
    let out3 = Command::new("ls")
        .arg("-lAUgG")
        .current_dir(workspace_path.as_path())
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out3);

    let out4 = Command::new("cat")
        .arg("cow.txt")
        .current_dir(workspace_path.as_path())
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out4);

    let path_out5 = tempdir2.path().join("workspace1");
    Command::new("bash")
        .arg("-c")
        .arg("echo 'Hello, copy on write! (overwrite)' > cow.txt")
        .current_dir(&path_out5)
        .output()
        .expect("failed to execute process");
    let out5 = Command::new("cat")
        .arg("cow.txt")
        .current_dir(&path_out5)
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out5);

    let path_out6 = tempdir2.path().join("trunk");
    let out6 = Command::new("bash")
        .arg("-c")
        .arg("echo 'Hello (must fail)' > fail.txt")
        .current_dir(&path_out6)
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out6);

    let path_out7 = tempdir2.path().join("workspace1");
    Command::new("bash")
        .arg("-c")
        .arg("echo 'Hello, copy on write! (truncate_test)' > truncate.txt")
        .current_dir(&path_out5)
        .output()
        .expect("failed to execute process");
    Command::new("bash")
        .arg("-c")
        .arg("echo 'Hello, copy on write! (truncate)' > truncate.txt")
        .current_dir(&path_out5)
        .output()
        .expect("failed to execute process");
    let out7 = Command::new("cat")
        .arg("truncate.txt")
        .current_dir(&path_out7)
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out7);
}

#[test]
#[serial]
fn test_5() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let tempdir1 = tempdir().unwrap();
    let fixed_system_time =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(40 * 365 * 24 * 60 * 60);
    // port 8084 is used by GitHub Actions
    let port = 8086;
    let config = ServerConfig {
        base_path: tempdir1.as_ref().to_path_buf(),
        is_main: false,
        clock: Clock::new_with_fixed_time(fixed_system_time),
        port,
    };

    runtime.spawn(async {
        sagitta_server::api::run_server(config).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));

    let client = SagittaApiClient::new(format!("http://localhost:{}", port));
    client
        .v2_create_workspace(V2CreateWorkspaceRequest {
            name: "workspace1".to_string(),
        })
        .unwrap();

    let local_system_workspace_base_path = tempdir().unwrap().as_ref().to_path_buf();

    let tempdir2 = tempdir().unwrap();
    let tempdir2_str = tempdir2.as_ref().to_str().unwrap().to_string();
    {
        let local_system_workspace_base_path = local_system_workspace_base_path.clone();
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        std::thread::spawn(move || {
            let config = SagittaConfig {
                base_url: format!("http://localhost:{}", port),
                mountpoint: tempdir2_str,
                uid,
                gid,
                clock: Clock::new_with_fixed_time(fixed_system_time),
                local_system_workspace_base_path: local_system_workspace_base_path.clone(),
                debug_sleep_duration: None,
            };
            run_fs(config);
        });
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let path_out1 = tempdir2.path().join("workspace1");
    Command::new("mkdir")
        .arg("foo")
        .current_dir(&path_out1)
        .output()
        .expect("failed to execute process");
    let out1 = Command::new("ls")
        .arg("-lAUgG")
        .current_dir(&path_out1)
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out1);
}

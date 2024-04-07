use std::{process::Command, time::SystemTime};

use sagitta::{
    api_client::SagittaApiClient,
    fs::{run_fs, SagittaConfig},
};
use sagitta_objects::SagittaTreeObject;
use sagitta_server::api::ServerConfig;
use serial_test::serial;
use tempfile::tempdir;
use tokio::runtime::Builder;

#[test]
#[serial]
fn test_1() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let tempdir = tempdir().unwrap();
    let fixed_system_time =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(40 * 365 * 24 * 60 * 60);
    let port = 8081;
    let config = ServerConfig {
        base_path: tempdir.as_ref().to_path_buf(),
        clock: sagitta_server::tools::Clock::new_with_fixed_time(fixed_system_time),
        port,
    };

    runtime.spawn(async {
        sagitta_server::api::run_server(config).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));

    let client = SagittaApiClient::new(format!("http://localhost:{}", port));

    let head_id_res = client.trunk_get_head().unwrap();
    let head_id = head_id_res.id;
    insta::assert_debug_snapshot!(head_id);

    let commit = client.blob_read_as_commit_object(&head_id).unwrap();
    insta::assert_debug_snapshot!(commit);

    let dir = client.blob_read_as_tree_object(&commit.tree_id).unwrap();
    insta::assert_debug_snapshot!(dir);

    let SagittaTreeObject::Dir(dir) = dir else {
        panic!()
    };
    let mut xs = vec![];
    for item in &dir.items {
        let file = client.blob_read_as_tree_object(&item.1).unwrap();
        let SagittaTreeObject::File(file) = file else {
            continue;
        };

        let blob_res = client.blob_read(&file.blob_id).unwrap();
        let blob = std::str::from_utf8(blob_res.blob.as_slice()).unwrap();
        insta::assert_debug_snapshot!(blob);

        xs.push((item.0.clone(), blob.to_string()));
    }
    insta::assert_debug_snapshot!(xs);
}

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
        clock: sagitta_server::tools::Clock::new_with_fixed_time(fixed_system_time),
        port,
    };

    runtime.spawn(async {
        sagitta_server::api::run_server(config).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));

    let tempdir2 = tempdir().unwrap();
    let tempdir2_str = tempdir2.as_ref().to_str().unwrap().to_string();
    std::thread::spawn(move || {
        let config = SagittaConfig {
            base_url: format!("http://localhost:{}", port),
            mountpoint: tempdir2_str,
            uid: 0,
            gid: 0,
        };
        run_fs(config);
    });
    std::thread::sleep(std::time::Duration::from_secs(1));

    let out1 = Command::new("ls")
        .arg("-lA")
        .current_dir(tempdir2.path())
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out1);

    let out2 = Command::new("cat")
        .arg("hello.txt")
        .current_dir(tempdir2.path())
        .output()
        .expect("failed to execute process");
    insta::assert_debug_snapshot!(out2);

    let mut out3_dir = tempdir2.path().to_path_buf();
    out3_dir.push("hello_dir");
    let out3 = Command::new("ls")
        .arg("-lA")
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

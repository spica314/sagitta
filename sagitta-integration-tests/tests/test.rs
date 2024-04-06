use std::time::SystemTime;

use sagitta::api_client::SagittaApiClient;
use sagitta_objects::SagittaTreeObject;
use sagitta_server::api::ServerConfig;
use tempfile::tempdir;
use tokio::runtime::Builder;

#[test]
fn test_1() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let tempdir = tempdir().unwrap();
    let fixed_system_time =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(40 * 365 * 24 * 60 * 60);
    let config = ServerConfig {
        base_path: tempdir.as_ref().to_path_buf(),
        clock: sagitta_server::tools::Clock::new_with_fixed_time(fixed_system_time),
    };

    runtime.spawn(async {
        sagitta_server::api::run_server(config).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));

    let client = SagittaApiClient::new("http://localhost:8081".to_string());

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
        let file = client.blob_read_as_tree_object(item).unwrap();
        let SagittaTreeObject::File(file) = file else {
            panic!()
        };

        let blob_res = client.blob_read(&file.blob_id).unwrap();
        let blob = std::str::from_utf8(blob_res.blob.as_slice()).unwrap();
        insta::assert_debug_snapshot!(blob);

        xs.push((file.name.clone(), blob.to_string()));
    }
    insta::assert_debug_snapshot!(xs);
}

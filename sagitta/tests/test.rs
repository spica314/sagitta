use std::time::SystemTime;

use sagitta::api::ServerConfig;
use sagitta_api_schema::{
    blob::read::{BlobReadRequest, BlobReadResponse},
    trunk::get_head::{TrunkGetHeadRequest, TrunkGetHeadResponse},
};
use sagitta_objects::{SagittaCommitObject, SagittaTreeObject};
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
        clock: sagitta::tools::Clock::new_with_fixed_time(fixed_system_time),
    };

    runtime.spawn(async {
        sagitta::api::run_server(config).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));

    let head_id_res: TrunkGetHeadResponse = ureq::post("http://localhost:8081/trunk/get-head")
        .send_json(TrunkGetHeadRequest {})
        .unwrap()
        .into_json()
        .unwrap();
    let head_id = head_id_res.id;
    insta::assert_debug_snapshot!(head_id);

    let commit_res: BlobReadResponse = ureq::post("http://localhost:8081/blob/read")
        .send_json(BlobReadRequest { id: head_id })
        .unwrap()
        .into_json()
        .unwrap();
    let commit: SagittaCommitObject = serde_cbor::from_reader(commit_res.blob.as_slice()).unwrap();
    insta::assert_debug_snapshot!(commit);

    let dir_res: BlobReadResponse = ureq::post("http://localhost:8081/blob/read")
        .send_json(BlobReadRequest { id: commit.tree_id })
        .unwrap()
        .into_json()
        .unwrap();
    let dir: SagittaTreeObject = serde_cbor::from_reader(dir_res.blob.as_slice()).unwrap();
    insta::assert_debug_snapshot!(dir);

    let SagittaTreeObject::Dir(dir) = dir else {
        panic!()
    };
    let mut xs = vec![];
    for item in &dir.items {
        let file_res: BlobReadResponse = ureq::post("http://localhost:8081/blob/read")
            .send_json(BlobReadRequest { id: item.clone() })
            .unwrap()
            .into_json()
            .unwrap();
        let file: SagittaTreeObject = serde_cbor::from_reader(file_res.blob.as_slice()).unwrap();
        let SagittaTreeObject::File(file) = file else {
            panic!()
        };

        let blob_res: BlobReadResponse = ureq::post("http://localhost:8081/blob/read")
            .send_json(BlobReadRequest { id: file.blob_id })
            .unwrap()
            .into_json()
            .unwrap();
        let blob = std::str::from_utf8(blob_res.blob.as_slice()).unwrap();
        insta::assert_debug_snapshot!(blob);

        xs.push((file.name.clone(), blob.to_string()));
    }
    insta::assert_debug_snapshot!(xs);
}

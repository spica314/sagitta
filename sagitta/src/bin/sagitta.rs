use std::{path::PathBuf, str::FromStr};

use sagitta::fs::{run_fs, SagittaConfig};
use sagitta_common::clock::Clock;

fn main() {
    env_logger::init();
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    let config = SagittaConfig {
        base_url: "http://localhost:8081".to_string(),
        mountpoint: "./sagitta-test".to_string(),
        uid,
        gid,
        clock: Clock::new(),
        local_system_workspace_base_path: PathBuf::from_str("./sagitta-test-system").unwrap(),
    };
    run_fs(config);
}

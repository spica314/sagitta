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
    };
    run_fs(config);
}

use sagitta::fs::{run_fs, SagittaConfig};

fn main() {
    env_logger::init();
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    let config = SagittaConfig {
        base_url: "http://localhost:8081".to_string(),
        mountpoint: "./sagitta-test".to_string(),
        uid,
        gid,
    };
    run_fs(config);
}

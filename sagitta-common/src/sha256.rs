use sha2::{Digest, Sha256};

pub fn calc_sha256_from_slice(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

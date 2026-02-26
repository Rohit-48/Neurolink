// File hashing and deduplication module
// TODO: Implement SHA-256 streaming hash and deduplication index

use sha2::{Sha256, Digest};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn compute_file_hash(path: &Path) -> anyhow::Result<String> {
    let mut file = File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192]; // 8KB chunks

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

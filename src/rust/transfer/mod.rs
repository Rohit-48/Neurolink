use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::fs::ReadDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use sha2::{Sha256, Digest};
use tracing::{info, debug};
use chrono::Utc;
use anyhow::Result;
use tempfile::TempDir;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransferError {
    #[error("Transfer not found: {0}")]
    TransferNotFound(String),
    #[error("Chunk out of order: expected {expected}, got {got}")]
    ChunkOutOfOrder { expected: usize, got: usize },
    #[error("Invalid chunk hash")]
    InvalidChunkHash,
    #[error("File too large")]
    FileTooLarge,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMetadata {
    pub id: String,
    pub filename: String,
    pub total_size: u64,
    pub chunk_size: usize,
    pub total_chunks: usize,
    pub batch_id: Option<String>,
    pub created_at: String,
    pub status: TransferStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    InProgress { received_chunks: usize },
    Completed { final_hash: String },
    Failed { reason: String },
}

#[derive(Debug)]
pub struct Transfer {
    pub metadata: TransferMetadata,
    pub temp_dir: TempDir,
    pub received_chunks: HashMap<usize, ChunkInfo>,
}

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub index: usize,
    pub hash: String,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct TransferManager {
    transfers: Arc<Mutex<HashMap<String, Transfer>>>,
    completed_uploads: Arc<Mutex<Vec<CompletedUpload>>>,
    storage_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFile {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    pub name: String,
    pub size: u64,
    pub uploaded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadBatch {
    pub batch_id: String,
    pub uploaded_at: String,
    pub files: Vec<UploadedFile>,
}

#[derive(Debug, Clone)]
pub struct CompletedUpload {
    pub batch_id: String,
    pub name: String,
    pub size: u64,
    pub uploaded_at: String,
}

impl TransferManager {
    pub fn new(storage_path: impl AsRef<Path>) -> Self {
        Self {
            transfers: Arc::new(Mutex::new(HashMap::new())),
            completed_uploads: Arc::new(Mutex::new(Vec::new())),
            storage_path: storage_path.as_ref().to_path_buf(),
        }
    }

    pub async fn init_transfer(
        &self,
        filename: String,
        total_size: u64,
        chunk_size: usize,
        batch_id: Option<String>,
    ) -> Result<String> {
        // Validate chunk_size to prevent division by zero
        if chunk_size == 0 {
            return Err(anyhow::anyhow!("chunk_size must be greater than 0"));
        }

        let transfer_id = format!("trans_{}", Utc::now().timestamp_millis());
        let total_chunks = ((total_size + chunk_size as u64 - 1) / chunk_size as u64) as usize;
        
        info!("Initializing transfer: {} for file: {} ({} chunks)", 
              transfer_id, filename, total_chunks);

        let temp_dir = TempDir::new()?;
        
        let metadata = TransferMetadata {
            id: transfer_id.clone(),
            filename: filename.clone(),
            total_size,
            chunk_size,
            total_chunks,
            batch_id,
            created_at: Utc::now().to_rfc3339(),
            status: TransferStatus::Pending,
        };

        let transfer = Transfer {
            metadata,
            temp_dir,
            received_chunks: HashMap::new(),
        };

        let mut transfers = self.transfers.lock().await;
        transfers.insert(transfer_id.clone(), transfer);

        Ok(transfer_id)
    }

    pub async fn receive_chunk(
        &self,
        transfer_id: &str,
        chunk_index: usize,
        chunk_data: Vec<u8>,
    ) -> Result<String> {
        let mut transfers = self.transfers.lock().await;
        
        let transfer = transfers
            .get_mut(transfer_id)
            .ok_or_else(|| TransferError::TransferNotFound(transfer_id.to_string()))?;

        if chunk_index >= transfer.metadata.total_chunks {
            return Err(TransferError::ChunkOutOfOrder {
                expected: transfer.metadata.total_chunks,
                got: chunk_index,
            }.into());
        }

        // Compute hash
        let mut hasher = Sha256::new();
        hasher.update(&chunk_data);
        let hash = hex::encode(hasher.finalize());

        // Write chunk to temp file
        let chunk_path = transfer.temp_dir.path().join(format!("chunk_{}.tmp", chunk_index));
        let mut file = fs::File::create(&chunk_path).await?;
        file.write_all(&chunk_data).await?;
        file.sync_all().await?;

        debug!("Received chunk {} for transfer {} (hash: {})", 
               chunk_index, transfer_id, &hash[..16]);

        let chunk_info = ChunkInfo {
            index: chunk_index,
            hash: hash.clone(),
            size: chunk_data.len(),
        };

        transfer.received_chunks.insert(chunk_index, chunk_info);
        transfer.metadata.status = TransferStatus::InProgress {
            received_chunks: transfer.received_chunks.len(),
        };

        Ok(hash)
    }

    pub async fn complete_transfer(&self, transfer_id: &str) -> Result<TransferMetadata> {
        let mut transfers = self.transfers.lock().await;
        
        let transfer = transfers
            .get_mut(transfer_id)
            .ok_or_else(|| TransferError::TransferNotFound(transfer_id.to_string()))?;

        // Verify all chunks received
        if transfer.received_chunks.len() != transfer.metadata.total_chunks {
            return Err(TransferError::ChunkOutOfOrder {
                expected: transfer.metadata.total_chunks,
                got: transfer.received_chunks.len(),
            }.into());
        }

        info!("Completing transfer: {}", transfer_id);

        // Reassemble file
        let final_path = self.storage_path.join(&transfer.metadata.filename);
        let mut final_file = fs::File::create(&final_path).await?;

        let mut final_hasher = Sha256::new();

        for i in 0..transfer.metadata.total_chunks {
            let chunk_path = transfer.temp_dir.path().join(format!("chunk_{}.tmp", i));
            let mut chunk_file = fs::File::open(&chunk_path).await?;
            let mut chunk_data = Vec::new();
            chunk_file.read_to_end(&mut chunk_data).await?;
            
            final_file.write_all(&chunk_data).await?;
            final_hasher.update(&chunk_data);
        }

        final_file.sync_all().await?;
        drop(final_file);

        let final_hash = hex::encode(final_hasher.finalize());
        
        transfer.metadata.status = TransferStatus::Completed {
            final_hash: final_hash.clone(),
        };

        info!("Transfer {} completed. File: {} (hash: {})", 
              transfer_id, transfer.metadata.filename, &final_hash[..16]);

        let mut completed_uploads = self.completed_uploads.lock().await;
        completed_uploads.push(CompletedUpload {
            batch_id: transfer
                .metadata
                .batch_id
                .clone()
                .unwrap_or_else(|| format!("single_{}", transfer.metadata.id)),
            name: transfer.metadata.filename.clone(),
            size: transfer.metadata.total_size,
            uploaded_at: Utc::now().to_rfc3339(),
        });

        let metadata = transfer.metadata.clone();
        
        // Remove from active transfers
        transfers.remove(transfer_id);

        Ok(metadata)
    }

    pub async fn get_transfer_status(&self, transfer_id: &str) -> Option<TransferMetadata> {
        let transfers = self.transfers.lock().await;
        transfers.get(transfer_id).map(|t| t.metadata.clone())
    }

    pub async fn cancel_transfer(&self, transfer_id: &str) -> Result<()> {
        let mut transfers = self.transfers.lock().await;
        transfers
            .remove(transfer_id)
            .ok_or_else(|| TransferError::TransferNotFound(transfer_id.to_string()))?;
        info!("Cancelled transfer: {}", transfer_id);
        Ok(())
    }

    pub async fn list_files(&self) -> Result<Vec<SharedFile>> {
        let mut out = Vec::new();
        let mut entries: ReadDir = fs::read_dir(&self.storage_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let meta = entry.metadata().await?;
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let modified_at = meta
                        .modified()
                        .ok()
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_else(|| "unknown".to_string());

                    out.push(SharedFile {
                        name: name.to_string(),
                        size: meta.len(),
                        modified_at,
                    });
                }
            }
        }

        out.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        Ok(out)
    }

    pub async fn list_upload_batches(&self) -> Vec<UploadBatch> {
        let completed_uploads = self.completed_uploads.lock().await;
        let mut grouped: HashMap<String, Vec<CompletedUpload>> = HashMap::new();

        for item in completed_uploads.iter() {
            grouped
                .entry(item.batch_id.clone())
                .or_default()
                .push(item.clone());
        }

        let mut batches: Vec<UploadBatch> = grouped
            .into_iter()
            .map(|(batch_id, mut files)| {
                files.sort_by(|a, b| a.uploaded_at.cmp(&b.uploaded_at));
                let uploaded_at = files
                    .last()
                    .map(|f| f.uploaded_at.clone())
                    .unwrap_or_else(|| Utc::now().to_rfc3339());
                let files = files
                    .into_iter()
                    .map(|f| UploadedFile {
                        name: f.name,
                        size: f.size,
                        uploaded_at: f.uploaded_at,
                    })
                    .collect();

                UploadBatch {
                    batch_id,
                    uploaded_at,
                    files,
                }
            })
            .collect();

        batches.sort_by(|a, b| b.uploaded_at.cmp(&a.uploaded_at));
        batches
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_init_transfer_success() {
        let manager = TransferManager::new("./test_shared");
        let result = manager.init_transfer("test.txt".to_string(), 1024, 512).await;
        assert!(result.is_ok());
        let transfer_id = result.unwrap();
        assert!(transfer_id.starts_with("trans_"));
    }

    #[tokio::test]
    async fn test_init_transfer_zero_chunk_size_fails() {
        let manager = TransferManager::new("./test_shared");
        let result = manager.init_transfer("test.txt".to_string(), 1024, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("chunk_size must be greater than 0"));
    }

    #[tokio::test]
    async fn test_receive_chunk_success() {
        let manager = TransferManager::new("./test_shared");
        let transfer_id = manager.init_transfer("test.txt".to_string(), 1024, 512).await.unwrap();
        
        let chunk_data = vec![0u8; 512];
        let result = manager.receive_chunk(&transfer_id, 0, chunk_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_receive_out_of_range_chunk_fails() {
        let manager = TransferManager::new("./test_shared");
        let transfer_id = manager.init_transfer("test.txt".to_string(), 1024, 512).await.unwrap();
        // File is 1024 bytes with 512 byte chunks = 2 chunks (indices 0 and 1)
        // Index 5 is out of range
        let chunk_data = vec![0u8; 512];
        let result = manager.receive_chunk(&transfer_id, 5, chunk_data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complete_transfer_with_missing_chunks_fails() {
        let manager = TransferManager::new("./test_shared");
        let transfer_id = manager.init_transfer("test.txt".to_string(), 1024, 512).await.unwrap();
        // Only send 1 of 2 chunks
        let chunk_data = vec![0u8; 512];
        manager.receive_chunk(&transfer_id, 0, chunk_data).await.unwrap();
        
        // Try to complete with missing chunk
        let result = manager.complete_transfer(&transfer_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complete_transfer_success() {
        let manager = TransferManager::new("./test_shared");
        let transfer_id = manager.init_transfer("test.txt".to_string(), 1024, 1024).await.unwrap();
        // Send the only chunk
        let chunk_data = vec![0u8; 1024];
        manager.receive_chunk(&transfer_id, 0, chunk_data).await.unwrap();
        
        // Complete should succeed
        let result = manager.complete_transfer(&transfer_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_transfer_status() {
        let manager = TransferManager::new("./test_shared");
        let transfer_id = manager.init_transfer("test.txt".to_string(), 1024, 512).await.unwrap();
        
        let status = manager.get_transfer_status(&transfer_id).await;
        assert!(status.is_some());
    }

    #[tokio::test]
    async fn test_get_nonexistent_transfer_status() {
        let manager = TransferManager::new("./test_shared");
        let status = manager.get_transfer_status("nonexistent").await;
        assert!(status.is_none());
    }
}

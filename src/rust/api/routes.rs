use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::transfer::TransferManager;
use tracing::{info, error};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct InitTransferRequest {
    pub filename: String,
    pub total_size: u64,
    pub chunk_size: usize,
}

#[derive(Serialize)]
pub struct InitTransferResponse {
    pub transfer_id: String,
    pub total_chunks: usize,
}

#[derive(Serialize)]
pub struct ChunkResponse {
    pub chunk_hash: String,
    pub received_count: usize,
    pub total_chunks: usize,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub transfer_id: String,
    pub status: String,
    pub progress: String,
}

pub fn routes(transfer_manager: Arc<TransferManager>) -> Router {
    Router::new()
        .route("/", get(root_page))
        .route("/transfer/init", post(init_transfer))
        .route("/transfer/chunk", post(receive_chunk))
        .route("/transfer/complete", post(complete_transfer))
        .route("/transfer/:id/status", get(get_status))
        .route("/health", get(health_check))
        .with_state(transfer_manager)
}

async fn root_page() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>NeuroLink Server</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; background: #0a0a0a; color: #fff; }
        h1 { color: #00ff88; }
        .status { background: #1a1a1a; padding: 20px; border-radius: 8px; margin: 20px 0; }
        .endpoint { background: #2a2a2a; padding: 10px; margin: 10px 0; border-radius: 4px; font-family: monospace; }
        code { background: #333; padding: 2px 6px; border-radius: 3px; }
    </style>
</head>
<body>
    <h1>NeuroLink v2.0.0 Server</h1>
    <div class="status">
        <h2>Status: Running</h2>
        <p>Rust-powered file transfer service</p>
    </div>
    
    <h2>API Endpoints</h2>
    <div class="endpoint">POST /transfer/init - Initialize upload</div>
    <div class="endpoint">POST /transfer/chunk - Upload chunk</div>
    <div class="endpoint">POST /transfer/complete - Finalize</div>
    <div class="endpoint">GET /transfer/:id/status - Check progress</div>
    <div class="endpoint">GET /health - Health check</div>
    
    <h2>Usage</h2>
    <p>Send files using the CLI:</p>
    <div class="endpoint"><code>neuroshare send file.zip --host localhost --port 3000</code></div>
    <p>For phone access, open <code>http://&lt;your-lan-ip&gt;:3000</code> (not <code>0.0.0.0</code>).</p>
    
    <p>For web UI, run: <code>neurolink --port 3000</code></p>
</body>
</html>
"#)
}

async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse {
        success: true,
        data: Some("healthy".to_string()),
        error: None,
    })
}

async fn init_transfer(
    State(manager): State<Arc<TransferManager>>,
    Json(req): Json<InitTransferRequest>,
) -> impl IntoResponse {
    info!("Init transfer request: {} ({} bytes)", req.filename, req.total_size);

    // Validate chunk_size is not zero to prevent division by zero
    if req.chunk_size == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid chunk_size: must be greater than 0".to_string()),
            }),
        );
    }

    match manager.init_transfer(req.filename, req.total_size, req.chunk_size).await {
        Ok(transfer_id) => {
            let total_chunks = ((req.total_size + req.chunk_size as u64 - 1) / req.chunk_size as u64) as usize;
            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    data: Some(InitTransferResponse {
                        transfer_id,
                        total_chunks,
                    }),
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!("Failed to init transfer: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

async fn receive_chunk(
    State(manager): State<Arc<TransferManager>>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<ChunkResponse>>, StatusCode> {
    let mut transfer_id = None;
    let mut chunk_index = None;
    let mut chunk_data = None;

    while let Some(mut field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "transfer_id" => {
                transfer_id = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            "chunk_index" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                chunk_index = text.parse().ok();
            }
            "chunk" => {
                chunk_data = Some(field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec());
            }
            _ => {}
        }
    }

    let transfer_id = transfer_id.ok_or(StatusCode::BAD_REQUEST)?;
    let chunk_index = chunk_index.ok_or(StatusCode::BAD_REQUEST)?;
    let chunk_data = chunk_data.ok_or(StatusCode::BAD_REQUEST)?;

    match manager.receive_chunk(&transfer_id, chunk_index, chunk_data).await {
        Ok(hash) => {
            if let Some(metadata) = manager.get_transfer_status(&transfer_id).await {
                let received = match &metadata.status {
                    crate::transfer::TransferStatus::InProgress { received_chunks } => *received_chunks,
                    _ => metadata.total_chunks,
                };

                Ok(Json(ApiResponse {
                    success: true,
                    data: Some(ChunkResponse {
                        chunk_hash: hash,
                        received_count: received,
                        total_chunks: metadata.total_chunks,
                    }),
                    error: None,
                }))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            error!("Failed to receive chunk: {}", e);
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn complete_transfer(
    State(manager): State<Arc<TransferManager>>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let transfer_id = req["transfer_id"].as_str().ok_or(StatusCode::BAD_REQUEST)?;

    match manager.complete_transfer(transfer_id).await {
        Ok(metadata) => Ok(Json(ApiResponse {
            success: true,
            data: Some(serde_json::json!({
                "transfer_id": metadata.id,
                "filename": metadata.filename,
                "status": "completed"
            })),
            error: None,
        })),
        Err(e) => {
            error!("Failed to complete transfer: {}", e);
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn get_status(
    State(manager): State<Arc<TransferManager>>,
    axum::extract::Path(transfer_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<StatusResponse>>, StatusCode> {
    match manager.get_transfer_status(&transfer_id).await {
        Some(metadata) => {
            let (status_str, progress) = match &metadata.status {
                crate::transfer::TransferStatus::Pending => ("pending".to_string(), "0%".to_string()),
                crate::transfer::TransferStatus::InProgress { received_chunks } => {
                    let pct = (received_chunks * 100) / metadata.total_chunks;
                    ("in_progress".to_string(), format!("{}%", pct))
                }
                crate::transfer::TransferStatus::Completed { .. } => ("completed".to_string(), "100%".to_string()),
                crate::transfer::TransferStatus::Failed { reason } => ("failed".to_string(), reason.clone()),
            };

            Ok(Json(ApiResponse {
                success: true,
                data: Some(StatusResponse {
                    transfer_id: metadata.id,
                    status: status_str,
                    progress,
                }),
                error: None,
            }))
        }
        None => Ok(Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Transfer not found".to_string()),
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Json;

    #[tokio::test]
    async fn init_transfer_zero_chunk_size_returns_bad_request() {
        let manager = Arc::new(TransferManager::new("./test_shared"));
        let req = InitTransferRequest {
            filename: "test.txt".to_string(),
            total_size: 1024,
            chunk_size: 0,
        };

        let response = init_transfer(State(manager), Json(req)).await.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

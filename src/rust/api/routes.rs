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
        .route("/files", get(list_files))
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
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>NeuroLink Server</title>
    <style>
        :root {
            --bg: #0b1220;
            --panel: #111a2b;
            --accent: #22d3ee;
            --accent-2: #f59e0b;
            --text: #e5eef8;
            --muted: #96a4b8;
            --ok: #10b981;
            --err: #ef4444;
        }
        * { box-sizing: border-box; }
        body {
            margin: 0;
            color: var(--text);
            font-family: "JetBrains Mono", "Fira Code", monospace;
            background:
                radial-gradient(circle at 20% 20%, #1b2845 0%, transparent 35%),
                radial-gradient(circle at 80% 10%, #15313e 0%, transparent 30%),
                linear-gradient(140deg, #060b16, var(--bg));
            min-height: 100vh;
            padding: 24px;
        }
        .wrap { max-width: 920px; margin: 0 auto; }
        .hero {
            background: linear-gradient(135deg, rgba(34,211,238,.15), rgba(245,158,11,.09));
            border: 1px solid rgba(34,211,238,.4);
            border-radius: 16px;
            padding: 20px;
            margin-bottom: 18px;
        }
        h1 { margin: 0 0 8px 0; font-size: 28px; letter-spacing: .3px; }
        .sub { color: var(--muted); margin: 0; }
        .card {
            background: var(--panel);
            border: 1px solid #1f3048;
            border-radius: 14px;
            padding: 16px;
            margin-bottom: 14px;
        }
        label { display: block; margin-bottom: 8px; color: var(--muted); }
        input[type="file"] {
            width: 100%;
            margin-bottom: 12px;
            color: var(--text);
        }
        button {
            background: linear-gradient(135deg, var(--accent), #0ea5e9);
            color: #04121a;
            border: 0;
            border-radius: 10px;
            padding: 10px 14px;
            font-weight: 700;
            cursor: pointer;
        }
        button:disabled { opacity: 0.6; cursor: not-allowed; }
        .muted { color: var(--muted); }
        .row { display: flex; gap: 10px; flex-wrap: wrap; align-items: center; }
        .pill {
            border: 1px solid #284363;
            border-radius: 999px;
            padding: 6px 10px;
            color: var(--muted);
            font-size: 13px;
        }
        .progress {
            width: 100%;
            height: 10px;
            border-radius: 999px;
            background: #1b2940;
            overflow: hidden;
            margin-top: 10px;
        }
        .bar {
            height: 100%;
            width: 0%;
            background: linear-gradient(90deg, var(--accent), var(--accent-2));
            transition: width .15s linear;
        }
        #status { margin-top: 10px; font-size: 14px; color: var(--muted); min-height: 20px; }
        #status.ok { color: var(--ok); }
        #status.err { color: var(--err); }
        .files a { color: var(--accent); text-decoration: none; }
        .files li { margin: 6px 0; }
        code { background: #122136; border-radius: 6px; padding: 2px 6px; }
    </style>
</head>
<body>
    <div class="wrap">
        <section class="hero">
            <h1>NeuroLink Rust UI</h1>
            <p class="sub">Single service mode: upload files from browser, then share links from <code>/shared</code>.</p>
            <div class="row" style="margin-top:10px;">
                <span class="pill">API: <code>/transfer/*</code></span>
                <span class="pill">Health: <code>/health</code></span>
                <span class="pill">Downloads: <code>/shared/&lt;filename&gt;</code></span>
            </div>
        </section>

        <section class="card">
            <label for="fileInput">Choose file</label>
            <input id="fileInput" type="file" />
            <button id="uploadBtn">Upload</button>
            <div class="progress"><div id="bar" class="bar"></div></div>
            <div id="status"></div>
        </section>

        <section class="card">
            <h3 style="margin-top:0;">Files in shared folder</h3>
            <p class="muted">Fetched from <code>/files</code>, each item downloads from <code>/shared/&lt;filename&gt;</code>.</p>
            <ul id="files" class="files"></ul>
        </section>
    </div>

    <script>
        const CHUNK_SIZE = 1024 * 1024;
        const fileInput = document.getElementById('fileInput');
        const uploadBtn = document.getElementById('uploadBtn');
        const bar = document.getElementById('bar');
        const statusEl = document.getElementById('status');
        const filesEl = document.getElementById('files');

        function setStatus(text, kind) {
            statusEl.textContent = text;
            statusEl.className = kind ? kind : '';
        }

        async function refreshFiles() {
            const res = await fetch('/files');
            const json = await res.json();
            if (!res.ok || !json.success || !Array.isArray(json.data)) return;

            if (json.data.length === 0) {
                filesEl.innerHTML = '<li class="muted">No files yet.</li>';
                return;
            }

            filesEl.innerHTML = json.data
                .map(name => `<li><a href="/shared/${encodeURIComponent(name)}" target="_blank" rel="noreferrer">${name}</a></li>`)
                .join('');
        }

        async function uploadFile(file) {
            setStatus('Initializing transfer...', '');
            const initRes = await fetch('/transfer/init', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    filename: file.name,
                    total_size: file.size,
                    chunk_size: CHUNK_SIZE
                })
            });
            const initJson = await initRes.json();
            if (!initRes.ok || !initJson.success || !initJson.data) {
                throw new Error(initJson.error || 'Failed to initialize transfer');
            }

            const transferId = initJson.data.transfer_id;
            const totalChunks = initJson.data.total_chunks;

            for (let idx = 0; idx < totalChunks; idx++) {
                const start = idx * CHUNK_SIZE;
                const end = Math.min(file.size, start + CHUNK_SIZE);
                const chunkBlob = file.slice(start, end);

                const form = new FormData();
                form.append('transfer_id', transferId);
                form.append('chunk_index', idx.toString());
                form.append('chunk', chunkBlob, `${file.name}.part${idx}`);

                const chunkRes = await fetch('/transfer/chunk', { method: 'POST', body: form });
                const chunkJson = await chunkRes.json();
                if (!chunkRes.ok || !chunkJson.success) {
                    throw new Error(chunkJson.error || `Chunk ${idx + 1} failed`);
                }

                const pct = Math.floor(((idx + 1) / totalChunks) * 100);
                bar.style.width = `${pct}%`;
                setStatus(`Uploading... ${pct}%`, '');
            }

            const doneRes = await fetch('/transfer/complete', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ transfer_id: transferId })
            });
            const doneJson = await doneRes.json();
            if (!doneRes.ok || !doneJson.success) {
                throw new Error(doneJson.error || 'Failed to complete transfer');
            }

            const link = `/shared/${encodeURIComponent(file.name)}`;
            setStatus(`Upload complete. File available at ${link}`, 'ok');
            await refreshFiles();
        }

        uploadBtn.addEventListener('click', async () => {
            const file = fileInput.files && fileInput.files[0];
            if (!file) {
                setStatus('Select a file first.', 'err');
                return;
            }

            uploadBtn.disabled = true;
            bar.style.width = '0%';
            setStatus('', '');
            try {
                await uploadFile(file);
            } catch (err) {
                setStatus(err.message || 'Upload failed', 'err');
            } finally {
                uploadBtn.disabled = false;
            }
        });

        refreshFiles();
    </script>
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

async fn list_files(
    State(manager): State<Arc<TransferManager>>,
) -> impl IntoResponse {
    match manager.list_files().await {
        Ok(files) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some(files),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<Vec<String>> {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        ),
    }
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

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
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

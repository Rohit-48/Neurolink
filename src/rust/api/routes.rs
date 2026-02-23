use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::transfer::{SharedFile, TransferManager};
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
    <title>NeuroLink</title>
    <style>
        :root {
            --bg-0: #06070f;
            --bg-1: #0b0f1e;
            --panel: #0f1426;
            --panel-soft: #111a2f;
            --line: #27304b;
            --text: #e8eeff;
            --muted: #8fa0c7;
            --accent: #18f0ff;
            --accent-2: #ffb020;
            --ok: #2ee8a3;
            --err: #ff5d7c;
        }
        * { box-sizing: border-box; }
        body {
            margin: 0;
            color: var(--text);
            font-family: "Sora", "JetBrains Mono", "Fira Code", sans-serif;
            background:
                radial-gradient(1200px 700px at 10% -10%, rgba(24,240,255,.15), transparent 50%),
                radial-gradient(900px 500px at 100% 0%, rgba(255,176,32,.12), transparent 45%),
                linear-gradient(140deg, var(--bg-0), var(--bg-1));
            min-height: 100vh;
            padding: 28px 18px;
            letter-spacing: .01em;
        }
        body::before {
            content: "";
            position: fixed;
            inset: 0;
            pointer-events: none;
            opacity: .15;
            background-image:
                linear-gradient(to right, rgba(143,160,199,.15) 1px, transparent 1px),
                linear-gradient(to bottom, rgba(143,160,199,.15) 1px, transparent 1px);
            background-size: 24px 24px;
            mask-image: radial-gradient(circle at 50% 20%, black, transparent 75%);
        }
        .wrap { max-width: 980px; margin: 0 auto; position: relative; z-index: 1; }
        .hero {
            background: linear-gradient(145deg, rgba(15,20,38,.94), rgba(10,14,26,.9));
            border: 1px solid var(--line);
            border-radius: 18px;
            padding: 20px 22px;
            margin-bottom: 16px;
            box-shadow: 0 20px 50px rgba(2,7,20,.45);
            animation: rise .45s ease-out;
        }
        h1 {
            margin: 0 0 8px 0;
            font-size: clamp(24px, 4vw, 32px);
            font-weight: 700;
            letter-spacing: .02em;
            text-shadow: 0 0 26px rgba(24,240,255,.22);
        }
        .sub { color: var(--muted); margin: 0; line-height: 1.5; }
        .layout {
            display: grid;
            grid-template-columns: 1.2fr 1fr;
            gap: 14px;
        }
        .card {
            background: linear-gradient(155deg, var(--panel-soft), var(--panel));
            border: 1px solid var(--line);
            border-radius: 16px;
            padding: 16px 16px 14px;
            box-shadow: 0 12px 30px rgba(2,7,20,.35);
            animation: rise .55s ease-out;
        }
        .card h3 {
            margin: 0 0 10px 0;
            font-size: 15px;
            text-transform: uppercase;
            letter-spacing: .08em;
            color: #c8d4f6;
        }
        label {
            display: block;
            margin-bottom: 8px;
            color: var(--muted);
            font-size: 13px;
            text-transform: uppercase;
            letter-spacing: .08em;
        }
        input[type="file"] {
            width: 100%;
            margin-bottom: 10px;
            color: var(--text);
            background: #0a0f20;
            border: 1px dashed #314168;
            border-radius: 10px;
            padding: 10px;
        }
        button {
            background: linear-gradient(95deg, var(--accent), #57f6ff);
            color: #001319;
            border: 0;
            border-radius: 10px;
            padding: 10px 15px;
            font-weight: 800;
            letter-spacing: .04em;
            text-transform: uppercase;
            cursor: pointer;
            transition: transform .15s ease, box-shadow .15s ease, filter .15s ease;
            box-shadow: 0 0 0 rgba(24,240,255,0);
        }
        button:hover {
            transform: translateY(-1px);
            box-shadow: 0 10px 24px rgba(24,240,255,.3);
            filter: brightness(1.03);
        }
        button:disabled { opacity: 0.6; cursor: not-allowed; }
        .actions { display: flex; gap: 8px; align-items: center; margin-top: 2px; }
        .ghost {
            background: transparent;
            color: var(--text);
            border: 1px solid #2f3a5d;
            box-shadow: none;
        }
        .dropzone {
            border: 1px dashed #33456f;
            border-radius: 12px;
            padding: 12px;
            margin-bottom: 10px;
            background: rgba(11,17,35,.55);
            transition: border-color .15s ease, background .15s ease;
        }
        .dropzone.active {
            border-color: var(--accent);
            background: rgba(24,240,255,.08);
        }
        .muted { color: var(--muted); }
        .row { display: flex; gap: 8px; flex-wrap: wrap; align-items: center; margin-top: 12px; }
        .pill {
            border: 1px solid #33456f;
            border-radius: 999px;
            padding: 6px 11px;
            color: var(--muted);
            font-size: 12px;
            background: rgba(18,26,48,.55);
        }
        .progress {
            width: 100%;
            height: 9px;
            border-radius: 999px;
            background: #121a30;
            overflow: hidden;
            margin-top: 10px;
            border: 1px solid #243458;
        }
        .bar {
            height: 100%;
            width: 0%;
            background: linear-gradient(90deg, var(--accent), var(--accent-2));
            transition: width .14s linear;
            box-shadow: 0 0 20px rgba(24,240,255,.45);
        }
        #status { margin-top: 10px; font-size: 14px; color: var(--muted); min-height: 20px; line-height: 1.45; }
        #status.ok { color: var(--ok); }
        #status.err { color: var(--err); }
        .files { list-style: none; margin: 0; padding: 0; max-height: 340px; overflow: auto; }
        .files li { margin: 0; border-bottom: 1px solid #202c4a; }
        .files li:last-child { border-bottom: 0; }
        .files a {
            color: #7ff7ff;
            text-decoration: none;
            display: grid;
            grid-template-columns: 1fr auto;
            gap: 8px;
            align-items: center;
            padding: 10px 2px;
            transition: color .15s ease, padding-left .15s ease;
        }
        .files a:hover { color: var(--accent); padding-left: 8px; }
        .file-meta { color: var(--muted); font-size: 12px; }
        code {
            background: rgba(16,26,48,.85);
            border: 1px solid #2a3a63;
            border-radius: 7px;
            padding: 2px 6px;
            font-family: "JetBrains Mono", "Fira Code", monospace;
            font-size: .92em;
        }
        @keyframes rise {
            from { opacity: 0; transform: translateY(6px); }
            to { opacity: 1; transform: translateY(0); }
        }
        @media (max-width: 860px) {
            .layout { grid-template-columns: 1fr; }
            .hero, .card { padding: 14px; border-radius: 14px; }
            body { padding: 16px 12px; }
        }
    </style>
</head>
<body>
    <div class="wrap">
        <section class="hero">
            <h1>NeuroLink</h1>
            <p class="sub">Minimal cyberpunk file transfer. Upload from browser, share from <code>/shared</code>.</p>
            <div class="row">
                <span class="pill">API: <code>/transfer/*</code></span>
                <span class="pill">Health: <code>/health</code></span>
                <span class="pill">Downloads: <code>/shared/&lt;filename&gt;</code></span>
            </div>
        </section>

        <div class="layout">
            <section class="card">
                <h3>Upload</h3>
                <label for="fileInput">Choose file</label>
                <div id="dropzone" class="dropzone">
                    <input id="fileInput" type="file" />
                    <div class="muted">Drop file here or click to browse.</div>
                </div>
                <div class="actions">
                    <button id="uploadBtn">Upload</button>
                    <button id="refreshBtn" class="ghost" type="button">Refresh</button>
                </div>
                <div class="progress"><div id="bar" class="bar"></div></div>
                <div id="status"></div>
            </section>

            <section class="card">
                <h3>Shared Files</h3>
                <p class="muted">Click any file to download from <code>/shared</code>.</p>
                <ul id="files" class="files"></ul>
            </section>
        </div>
    </div>

    <script>
        const CHUNK_SIZE = 1024 * 1024;
        const fileInput = document.getElementById('fileInput');
        const dropzone = document.getElementById('dropzone');
        const uploadBtn = document.getElementById('uploadBtn');
        const refreshBtn = document.getElementById('refreshBtn');
        const bar = document.getElementById('bar');
        const statusEl = document.getElementById('status');
        const filesEl = document.getElementById('files');
        let selectedFile = null;

        function setStatus(text, kind) {
            statusEl.textContent = text;
            statusEl.className = kind ? kind : '';
        }

        async function refreshFiles() {
            const res = await fetch('/files');
            const json = await res.json();
            if (!res.ok || !json.success || !Array.isArray(json.data)) {
                filesEl.innerHTML = '<li class="muted">Failed to load files.</li>';
                return;
            }

            if (json.data.length === 0) {
                filesEl.innerHTML = '<li class="muted">No files yet.</li>';
                return;
            }

            filesEl.innerHTML = json.data
                .map(file => {
                    const kb = file.size < 1024 ? `${file.size} B` : `${(file.size / 1024).toFixed(1)} KB`;
                    const ts = file.modified_at === 'unknown'
                        ? 'unknown'
                        : new Date(file.modified_at).toLocaleString();
                    return `<li><a href="/shared/${encodeURIComponent(file.name)}" target="_blank" rel="noreferrer">
                        <span>${file.name}</span>
                        <span class="file-meta">${kb} · ${ts}</span>
                    </a></li>`;
                })
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
            const file = selectedFile || (fileInput.files && fileInput.files[0]);
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

        refreshBtn.addEventListener('click', refreshFiles);

        fileInput.addEventListener('change', () => {
            selectedFile = fileInput.files && fileInput.files[0] ? fileInput.files[0] : null;
            if (selectedFile) setStatus(`${selectedFile.name} selected`, '');
        });

        dropzone.addEventListener('dragover', (e) => {
            e.preventDefault();
            dropzone.classList.add('active');
        });
        dropzone.addEventListener('dragleave', () => dropzone.classList.remove('active'));
        dropzone.addEventListener('drop', (e) => {
            e.preventDefault();
            dropzone.classList.remove('active');
            if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length > 0) {
                selectedFile = e.dataTransfer.files[0];
                setStatus(`${selectedFile.name} selected`, '');
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
            Json(ApiResponse::<Vec<SharedFile>> {
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

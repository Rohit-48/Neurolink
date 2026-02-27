use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::transfer::{SharedFile, TransferManager, UploadBatch};
use tokio::process::Command;
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
    pub batch_id: Option<String>,
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
        .route("/uploads", get(list_uploads))
        .route("/download/batch/:batch_id", get(download_batch))
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
    <title>NeuroLinkd</title>
    <style>
        :root {
            --bg: #f7f7f4;
            --ink: #111111;
            --muted: #666666;
            --line: #d9d9d3;
            --panel: #ffffff;
            --ok: #1f1f1f;
            --err: #3b3b3b;
        }
        * { box-sizing: border-box; }
        body {
            margin: 0;
            font-family: "Styrene B", "Avenir Next", "Inter", system-ui, sans-serif;
            color: var(--ink);
            background: radial-gradient(circle at top, #ffffff 0%, var(--bg) 55%);
            min-height: 100vh;
            padding: 30px 16px 50px;
            letter-spacing: 0.01em;
        }
        .wrap { max-width: 920px; margin: 0 auto; }
        .hero {
            border-top: 2px solid var(--ink);
            padding-top: 18px;
            margin-bottom: 22px;
        }
        h1 {
            margin: 0;
            font-family: "Tiempos Headline", "Iowan Old Style", "Times New Roman", serif;
            font-size: clamp(34px, 6vw, 58px);
            font-weight: 500;
            letter-spacing: -0.02em;
            line-height: 0.98;
        }
        .sub {
            margin: 12px 0 0;
            font-size: 16px;
            color: var(--muted);
            max-width: 680px;
            line-height: 1.6;
        }
        .meta {
            margin-top: 14px;
            display: flex;
            gap: 8px;
            flex-wrap: wrap;
        }
        .pill {
            border: 1px solid var(--line);
            border-radius: 999px;
            padding: 5px 11px;
            font-size: 12px;
            color: #2c2c2c;
            background: #fff;
        }
        .card {
            background: var(--panel);
            border: 1px solid var(--line);
            border-radius: 16px;
            padding: 18px;
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.04);
        }
        .card + .card { margin-top: 16px; }
        .title {
            margin: 0 0 6px;
            font-size: 13px;
            text-transform: uppercase;
            letter-spacing: 0.12em;
            color: #4f4f4f;
        }
        .hint {
            margin: 0 0 14px;
            font-size: 14px;
            color: var(--muted);
        }
        .dropzone {
            border: 1px dashed #c9c9c3;
            border-radius: 12px;
            padding: 14px;
            margin-bottom: 12px;
            background: #fcfcfb;
            color: #444;
            font-size: 14px;
        }
        .dropzone.active {
            border-color: #999;
            background: #f2f2ee;
        }
        .actions {
            display: flex;
            gap: 10px;
            flex-wrap: wrap;
        }
        button, .link-btn {
            border-radius: 11px;
            border: 1px solid #c7c7c1;
            padding: 10px 13px;
            font-size: 13px;
            font-weight: 600;
            cursor: pointer;
            background: #fff;
            color: #111;
        }
        button.primary {
            background: #111;
            border-color: #111;
            color: #fff;
        }
        button:disabled {
            opacity: 0.55;
            cursor: not-allowed;
        }
        .progress {
            margin-top: 12px;
            height: 8px;
            border-radius: 999px;
            background: #ededea;
            overflow: hidden;
        }
        .bar {
            width: 0%;
            height: 100%;
            background: linear-gradient(90deg, #202020, #545454);
            transition: width .15s linear;
        }
        #status {
            min-height: 20px;
            margin-top: 10px;
            color: #404040;
            font-size: 14px;
        }
        #status.ok { color: var(--ok); }
        #status.err { color: var(--err); }
        #selection {
            margin: 5px 0 0;
            color: var(--muted);
            font-size: 13px;
        }
        .files {
            list-style: none;
            margin: 0;
            padding: 0;
            border-top: 1px solid var(--line);
        }
        .files li {
            border-bottom: 1px solid var(--line);
            padding: 12px 0;
        }
        .batch-head {
            display: flex;
            justify-content: space-between;
            align-items: center;
            gap: 8px;
            color: var(--muted);
            font-size: 12px;
            margin-bottom: 8px;
        }
        .file-row {
            display: flex;
            justify-content: space-between;
            gap: 10px;
            align-items: center;
            padding: 7px 0;
        }
        .file-actions {
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .mini-btn {
            border-radius: 8px;
            border: 1px solid #c7c7c1;
            padding: 4px 8px;
            font-size: 11px;
            text-decoration: none;
            color: #111;
            background: #fff;
        }
        a.file-link {
            color: #121212;
            text-decoration: none;
            font-size: 14px;
            overflow-wrap: anywhere;
        }
        a.file-link:hover { text-decoration: underline; }
        .size { color: var(--muted); font-size: 12px; }
        .hidden-input { display: none; }
        @media (max-width: 760px) {
            body { padding: 22px 12px 34px; }
            .card { padding: 14px; }
        }
    </style>
</head>
<body>
    <div class="wrap">
        <section class="hero">
            <h1>NeuroLinkd</h1>
            <p class="sub">Fast local transfers with clean batch uploads. Pick a folder, upload once, share single files or a full batch zip.</p>
            <div class="meta">
                <span class="pill">Rust Runtime (neurolinkd)</span>
                <span class="pill">Batch Download: ZIP</span>
                <span class="pill">API: /transfer/*</span>
            </div>
        </section>

        <section class="card">
            <h2 class="title">Upload</h2>
            <p class="hint">Primary flow: click Upload Folder, then Start Upload.</p>
            <div id="dropzone" class="dropzone">Drop files/folder here or use the buttons below.</div>
            <input id="folderInput" class="hidden-input" type="file" webkitdirectory directory multiple />
            <input id="fileInput" class="hidden-input" type="file" multiple />
            <div class="actions">
                <button id="pickFolderBtn" class="primary" type="button">Upload Folder</button>
                <button id="pickFilesBtn" type="button">Select Files</button>
                <button id="startUploadBtn" type="button">Start Upload</button>
                <button id="refreshBtn" type="button">Refresh</button>
            </div>
            <div class="progress"><div id="bar" class="bar"></div></div>
            <div id="status"></div>
            <p id="selection"></p>
        </section>

        <section class="card">
            <h2 class="title">Upload Batches</h2>
            <p class="hint">Newest batches first. Every upload run creates one batch.</p>
            <ul id="files" class="files"></ul>
        </section>
    </div>

    <script>
        const CHUNK_SIZE = 1024 * 1024;
        const folderInput = document.getElementById('folderInput');
        const fileInput = document.getElementById('fileInput');
        const pickFolderBtn = document.getElementById('pickFolderBtn');
        const pickFilesBtn = document.getElementById('pickFilesBtn');
        const startUploadBtn = document.getElementById('startUploadBtn');
        const refreshBtn = document.getElementById('refreshBtn');
        const dropzone = document.getElementById('dropzone');
        const bar = document.getElementById('bar');
        const statusEl = document.getElementById('status');
        const selectionEl = document.getElementById('selection');
        const filesEl = document.getElementById('files');
        let selectedFiles = [];

        function setStatus(text, kind) {
            statusEl.className = kind || '';
            statusEl.textContent = text;
        }

        function formatBytes(size) {
            if (size < 1024) return `${size} B`;
            if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
            return `${(size / (1024 * 1024)).toFixed(2)} MB`;
        }

        function updateSelection() {
            if (selectedFiles.length === 0) {
                selectionEl.textContent = 'No files selected';
                return;
            }
            const total = selectedFiles.reduce((n, f) => n + f.size, 0);
            selectionEl.textContent = `${selectedFiles.length} files selected · ${formatBytes(total)}`;
        }

        function setFilesFromList(list) {
            selectedFiles = Array.from(list || []);
            updateSelection();
        }

        async function refreshFiles() {
            const res = await fetch('/uploads');
            const json = await res.json();

            if (!res.ok || !json.success || !Array.isArray(json.data)) {
                filesEl.innerHTML = '<li>Failed to load uploads.</li>';
                return;
            }

            if (json.data.length === 0) {
                filesEl.innerHTML = '<li>No uploads yet.</li>';
                return;
            }

            filesEl.innerHTML = json.data.map((batch) => {
                const when = new Date(batch.uploaded_at).toLocaleString();
                const items = batch.files.map((file) => `
                    <div class="file-row">
                        <a class="file-link" href="/shared/${encodeURIComponent(file.name)}" target="_blank" rel="noreferrer">${file.name}</a>
                        <div class="file-actions">
                            <span class="size">${formatBytes(file.size)}</span>
                            <a class="mini-btn" href="/shared/${encodeURIComponent(file.name)}" download="${file.name}">Download</a>
                        </div>
                    </div>
                `).join('');
                return `
                    <li>
                        <div class="batch-head">
                            <span>${when} · ${batch.files.length} file(s)</span>
                            <a class="link-btn" href="/download/batch/${encodeURIComponent(batch.batch_id)}">Download ZIP</a>
                        </div>
                        ${items}
                    </li>
                `;
            }).join('');
        }

        async function uploadSingleFile(file, batchId, doneBytes, totalBytes) {
            const initRes = await fetch('/transfer/init', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    filename: file.name,
                    total_size: file.size,
                    chunk_size: CHUNK_SIZE,
                    batch_id: batchId
                })
            });
            const initJson = await initRes.json();
            if (!initRes.ok || !initJson.success || !initJson.data) {
                throw new Error(initJson.error || 'Init failed');
            }

            const transferId = initJson.data.transfer_id;
            const totalChunks = initJson.data.total_chunks;
            for (let idx = 0; idx < totalChunks; idx++) {
                const chunkBlob = file.slice(idx * CHUNK_SIZE, Math.min(file.size, (idx + 1) * CHUNK_SIZE));
                const form = new FormData();
                form.append('transfer_id', transferId);
                form.append('chunk_index', idx.toString());
                form.append('chunk', chunkBlob, `${file.name}.part${idx}`);
                const chunkRes = await fetch('/transfer/chunk', { method: 'POST', body: form });
                const chunkJson = await chunkRes.json();
                if (!chunkRes.ok || !chunkJson.success) throw new Error(chunkJson.error || `Chunk ${idx + 1} failed`);
                const uploaded = doneBytes + Math.min(file.size, (idx + 1) * CHUNK_SIZE);
                bar.style.width = `${Math.floor((uploaded / totalBytes) * 100)}%`;
            }

            const doneRes = await fetch('/transfer/complete', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ transfer_id: transferId })
            });
            const doneJson = await doneRes.json();
            if (!doneRes.ok || !doneJson.success) throw new Error(doneJson.error || 'Complete failed');
        }

        async function uploadBatch() {
            if (selectedFiles.length === 0) {
                setStatus('Select files or a folder first.', 'err');
                return;
            }

            startUploadBtn.disabled = true;
            bar.style.width = '0%';
            const batchId = `batch_${Date.now()}`;
            const totalBytes = selectedFiles.reduce((n, f) => n + f.size, 0);
            let doneBytes = 0;
            try {
                for (let i = 0; i < selectedFiles.length; i++) {
                    const file = selectedFiles[i];
                    setStatus(`Uploading ${i + 1}/${selectedFiles.length}: ${file.name}`);
                    await uploadSingleFile(file, batchId, doneBytes, totalBytes);
                    doneBytes += file.size;
                }
                setStatus(`Batch upload complete (${selectedFiles.length} files)`, 'ok');
                await refreshFiles();
            } catch (err) {
                setStatus(err.message || 'Upload failed', 'err');
            } finally {
                startUploadBtn.disabled = false;
            }
        }

        pickFolderBtn.addEventListener('click', () => folderInput.click());
        pickFilesBtn.addEventListener('click', () => fileInput.click());
        startUploadBtn.addEventListener('click', uploadBatch);
        refreshBtn.addEventListener('click', refreshFiles);

        folderInput.addEventListener('change', () => setFilesFromList(folderInput.files));
        fileInput.addEventListener('change', () => setFilesFromList(fileInput.files));

        dropzone.addEventListener('dragover', (e) => {
            e.preventDefault();
            dropzone.classList.add('active');
        });
        dropzone.addEventListener('dragleave', () => dropzone.classList.remove('active'));
        dropzone.addEventListener('drop', (e) => {
            e.preventDefault();
            dropzone.classList.remove('active');
            if (e.dataTransfer?.files?.length) setFilesFromList(e.dataTransfer.files);
        });

        updateSelection();
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

async fn list_uploads(
    State(manager): State<Arc<TransferManager>>,
) -> impl IntoResponse {
    let uploads = manager.list_upload_batches().await;
    (
        StatusCode::OK,
        Json(ApiResponse::<Vec<UploadBatch>> {
            success: true,
            data: Some(uploads),
            error: None,
        }),
    )
}

async fn download_batch(
    State(manager): State<Arc<TransferManager>>,
    Path(batch_id): Path<String>,
) -> Response {
    let files = manager.files_for_batch(&batch_id).await;
    if files.is_empty() {
        return (StatusCode::NOT_FOUND, "Batch not found").into_response();
    }

    let storage_path = manager.storage_path();
    let mut cmd = Command::new("zip");
    cmd.arg("-q").arg("-").current_dir(storage_path);
    for file in &files {
        cmd.arg(&file.name);
    }

    let output = match cmd.output().await {
        Ok(output) if output.status.success() => output.stdout,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build zip archive: {}", stderr),
            )
                .into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to run zip: {}", err),
            )
                .into_response();
        }
    };

    let mut response = Response::new(Body::from(output));
    *response.status_mut() = StatusCode::OK;
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/zip"));

    let disposition = format!("attachment; filename=\"upload-{}.zip\"", batch_id);
    if let Ok(v) = HeaderValue::from_str(&disposition) {
        response.headers_mut().insert(header::CONTENT_DISPOSITION, v);
    }
    response
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

    match manager
        .init_transfer(req.filename, req.total_size, req.chunk_size, req.batch_id)
        .await
    {
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
            batch_id: None,
        };

        let response = init_transfer(State(manager), Json(req)).await.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

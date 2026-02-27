import express from "express";
import multer from "multer";
import { createReadStream, createWriteStream } from "fs";
import {
  promises as fs,
  existsSync,
} from "fs";
import os from "os";
import path from "path";
import archiver from "archiver";
import { lookup } from "mime-types";

const upload = multer({ storage: multer.memoryStorage() });

function apiResponse({ success, data = null, error = null }) {
  return { success, data, error };
}

function safeName(filename) {
  const name = path.basename(filename || "");
  if (!name || name === "." || name === "..") return null;
  return name;
}

function formatIso(dateLike) {
  try {
    return new Date(dateLike).toISOString();
  } catch {
    return new Date().toISOString();
  }
}

function uiHtml() {
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>NeuroLink (Express)</title>
  <style>
    :root{
      --bg-0:#080b14; --bg-1:#0d1426;
      --surface:#111b33; --surface-2:#0f182f;
      --line:#2b3d66; --line-soft:#22314f;
      --text:#eaf0ff; --muted:#93a3c9;
      --accent:#25f2ff; --accent-2:#6bf8ff; --accent-warm:#ffb02d;
      --ok:#2fe9a5; --err:#ff5f84;
      --shadow:0 16px 36px rgba(2,8,24,.42);
      --radius:16px;
    }
    *{box-sizing:border-box}
    body{
      margin:0;
      color:var(--text);
      font-family:"Sora","Inter",system-ui,sans-serif;
      background:
        radial-gradient(1000px 600px at 10% -10%, rgba(37,242,255,.16), transparent 55%),
        radial-gradient(800px 500px at 110% -15%, rgba(255,176,45,.14), transparent 52%),
        linear-gradient(145deg,var(--bg-0),var(--bg-1));
      min-height:100vh;
      padding:26px 16px 34px;
      letter-spacing:.01em;
    }
    body::before{
      content:"";
      position:fixed; inset:0; pointer-events:none;
      opacity:.12;
      background-image:
        linear-gradient(to right, rgba(147,163,201,.22) 1px, transparent 1px),
        linear-gradient(to bottom, rgba(147,163,201,.22) 1px, transparent 1px);
      background-size:30px 30px;
      mask-image:radial-gradient(circle at 50% 18%, black, transparent 70%);
      -webkit-mask-image:radial-gradient(circle at 50% 18%, black, transparent 70%);
    }
    .wrap{max-width:1040px; margin:0 auto; position:relative; z-index:1}
    .hero,.card{
      background:linear-gradient(160deg,var(--surface),var(--surface-2));
      border:1px solid var(--line);
      border-radius:var(--radius);
      box-shadow:var(--shadow);
    }
    .hero{padding:20px; margin-bottom:14px}
    .layout{display:grid; grid-template-columns:1.08fr .92fr; gap:12px}
    .card{padding:15px}
    h1{margin:0; font-size:clamp(26px,4vw,38px); line-height:1.02}
    h3{margin:0 0 11px; font-size:13px; text-transform:uppercase; letter-spacing:.1em; color:#ced9f8}
    .hero-sub{margin:.55rem 0 0; color:var(--muted); font-size:14px}
    .chip-row{display:flex; flex-wrap:wrap; gap:8px; margin-top:11px}
    .chip{
      border:1px solid #3b4d7a;
      border-radius:999px;
      padding:6px 11px;
      background:rgba(18,29,55,.66);
      color:var(--muted);
      font-size:12px;
    }
    .drop{
      border:1px dashed #435886;
      border-radius:12px;
      padding:12px;
      margin-bottom:9px;
      background:rgba(12,18,36,.58);
      transition:border-color .16s ease, background .16s ease;
    }
    .drop.active{border-color:var(--accent); background:rgba(37,242,255,.08)}
    input[type=file]{width:100%; color:var(--text); margin-bottom:8px}
    .actions{display:flex; gap:8px}
    button{
      border:0;
      border-radius:10px;
      padding:9px 12px;
      font-weight:700;
      cursor:pointer;
      transition:transform .12s ease, box-shadow .12s ease, filter .12s ease, border-color .12s ease;
    }
    button:hover{transform:translateY(-1px); filter:brightness(1.03)}
    .primary{
      background:linear-gradient(95deg,var(--accent),var(--accent-2));
      color:#001319;
      box-shadow:0 10px 24px rgba(37,242,255,.32);
    }
    .ghost{
      background:transparent;
      color:var(--text);
      border:1px solid #3a4a75;
    }
    .progress{
      height:9px;
      background:#192543;
      border:1px solid #2e4069;
      border-radius:999px;
      overflow:hidden;
      margin-top:10px;
    }
    .bar{
      height:100%;
      width:0%;
      background:linear-gradient(90deg,var(--accent),var(--accent-warm));
      transition:width .12s linear;
    }
    .muted{color:var(--muted)}
    #status{min-height:20px; margin-top:8px; font-size:14px}
    #selection{font-size:13px}
    .ok{color:var(--ok)}
    .err{color:var(--err)}
    .files{
      list-style:none;
      margin:0;
      padding:0;
      max-height:460px;
      overflow:auto;
      border:1px solid var(--line-soft);
      border-radius:12px;
      background:rgba(9,14,28,.48);
    }
    .files li{
      border-bottom:1px solid var(--line-soft);
      padding:10px 11px;
    }
    .files li:last-child{border-bottom:0}
    .row{display:flex; justify-content:space-between; align-items:center; gap:8px}
    .meta{color:var(--muted); font-size:12px}
    a{
      color:#9af9ff;
      text-decoration:none;
      transition:color .12s ease;
    }
    a:hover{color:var(--accent)}
    @media (max-width:860px){
      .layout{grid-template-columns:1fr}
      .hero,.card{padding:13px}
      .actions{flex-wrap:wrap}
    }
  </style>
</head>
<body>
  <div class="wrap">
    <section class="hero">
      <h1>NeuroLink</h1>
      <p class="hero-sub">Modern local file transfer with batch uploads, archive download, and chunk extraction.</p>
      <div class="chip-row">
        <span class="chip">GET /health</span>
        <span class="chip">GET /uploads</span>
        <span class="chip">GET /download/batch/:id</span>
      </div>
    </section>
    <div class="layout">
      <section class="card">
        <h3>Upload</h3>
        <div id="drop" class="drop">
          <input id="fileInput" type="file" multiple />
          <div class="muted">Drop files here or browse.</div>
        </div>
        <div class="actions">
          <button id="uploadBtn" class="primary">Upload Batch</button>
          <button id="refreshBtn" class="ghost" type="button">Refresh</button>
        </div>
        <div class="progress"><div id="bar" class="bar"></div></div>
        <div id="status"></div>
        <div id="selection" class="muted"></div>
      </section>
      <section class="card">
        <h3>Upload Batches</h3>
        <ul id="files" class="files"></ul>
      </section>
    </div>
  </div>
  <script>
    const CHUNK_SIZE = 1024 * 1024;
    const fileInput = document.getElementById("fileInput");
    const drop = document.getElementById("drop");
    const uploadBtn = document.getElementById("uploadBtn");
    const refreshBtn = document.getElementById("refreshBtn");
    const bar = document.getElementById("bar");
    const statusEl = document.getElementById("status");
    const selectionEl = document.getElementById("selection");
    const filesEl = document.getElementById("files");
    let selectedFiles = [];

    const setStatus = (text, kind = "") => { statusEl.className = kind; statusEl.textContent = text; };
    const formatBytes = (n) => n < 1024 ? \`\${n} B\` : n < 1024*1024 ? \`\${(n/1024).toFixed(1)} KB\` : \`\${(n/(1024*1024)).toFixed(2)} MB\`;
    const updateSelection = () => {
      if (!selectedFiles.length) { selectionEl.textContent = "No files selected"; return; }
      const total = selectedFiles.reduce((a, f) => a + f.size, 0);
      selectionEl.textContent = \`\${selectedFiles.length} file(s) selected · \${formatBytes(total)}\`;
    };

    async function refreshFiles() {
      const res = await fetch("/uploads");
      const json = await res.json();
      if (!res.ok || !json.success || !Array.isArray(json.data)) {
        filesEl.innerHTML = '<li class="muted">Failed to load uploads.</li>'; return;
      }
      if (json.data.length === 0) {
        filesEl.innerHTML = '<li class="muted">No uploads yet.</li>'; return;
      }
      filesEl.innerHTML = json.data.map((batch) => {
        const when = new Date(batch.uploaded_at).toLocaleString();
        const items = batch.files.map((file) => \`
          <div class="row">
            <a href="/shared/\${encodeURIComponent(file.name)}" target="_blank" rel="noreferrer">\${file.name}</a>
            <div class="meta">
              \${formatBytes(file.size)}
              <button class="ghost chunk-btn" data-file="\${encodeURIComponent(file.name)}" style="margin-left:6px;padding:3px 7px;font-size:11px">Chunk</button>
            </div>
          </div>\`).join("");
        return \`<li>
          <div class="row meta"><span>\${when} · \${batch.files.length} file(s)</span>
            <a class="ghost" style="padding:4px 8px;text-decoration:none;color:inherit" href="/download/batch/\${encodeURIComponent(batch.batch_id)}">Download Batch</a>
          </div>
          \${items}
        </li>\`;
      }).join("");

      filesEl.querySelectorAll(".chunk-btn").forEach((btn) => {
        btn.addEventListener("click", (e) => {
          e.preventDefault();
          const encoded = btn.dataset.file;
          const idxRaw = prompt("Chunk index (0-based):", "0");
          if (idxRaw == null) return;
          const sizeRaw = prompt("Chunk size in bytes:", "1048576");
          if (sizeRaw == null) return;
          const idx = parseInt(idxRaw, 10);
          const size = parseInt(sizeRaw, 10);
          if (Number.isNaN(idx) || Number.isNaN(size) || idx < 0 || size <= 0) { setStatus("Invalid chunk values", "err"); return; }
          window.open(\`/download/chunk/\${encoded}?index=\${idx}&chunk_size=\${size}\`, "_blank");
        });
      });
    }

    async function uploadFile(file, batchId, doneBytes, totalBytes) {
      const initRes = await fetch("/transfer/init", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ filename: file.name, total_size: file.size, chunk_size: CHUNK_SIZE, batch_id: batchId })
      });
      const initJson = await initRes.json();
      if (!initRes.ok || !initJson.success || !initJson.data) throw new Error(initJson.error || "Init failed");
      const transferId = initJson.data.transfer_id;
      const totalChunks = initJson.data.total_chunks;
      for (let idx = 0; idx < totalChunks; idx++) {
        const chunkBlob = file.slice(idx * CHUNK_SIZE, Math.min(file.size, (idx + 1) * CHUNK_SIZE));
        const form = new FormData();
        form.append("transfer_id", transferId);
        form.append("chunk_index", String(idx));
        form.append("chunk", chunkBlob, \`\${file.name}.part\${idx}\`);
        const chunkRes = await fetch("/transfer/chunk", { method: "POST", body: form });
        const chunkJson = await chunkRes.json();
        if (!chunkRes.ok || !chunkJson.success) throw new Error(chunkJson.error || \`Chunk \${idx + 1} failed\`);
        const uploaded = doneBytes + Math.min(file.size, (idx + 1) * CHUNK_SIZE);
        bar.style.width = \`\${Math.floor((uploaded / totalBytes) * 100)}%\`;
      }
      const doneRes = await fetch("/transfer/complete", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ transfer_id: transferId })
      });
      const doneJson = await doneRes.json();
      if (!doneRes.ok || !doneJson.success) throw new Error(doneJson.error || "Complete failed");
    }

    uploadBtn.addEventListener("click", async () => {
      if (!selectedFiles.length) { setStatus("Select files first", "err"); return; }
      uploadBtn.disabled = true;
      bar.style.width = "0%";
      const totalBytes = selectedFiles.reduce((n, f) => n + f.size, 0);
      const batchId = \`batch_\${Date.now()}\`;
      let doneBytes = 0;
      try {
        for (let i = 0; i < selectedFiles.length; i++) {
          const file = selectedFiles[i];
          setStatus(\`Uploading \${i + 1}/\${selectedFiles.length}: \${file.name}\`);
          await uploadFile(file, batchId, doneBytes, totalBytes);
          doneBytes += file.size;
        }
        setStatus(\`Batch upload complete (\${selectedFiles.length} file(s))\`, "ok");
        await refreshFiles();
      } catch (err) {
        setStatus(err.message || "Upload failed", "err");
      } finally {
        uploadBtn.disabled = false;
      }
    });

    refreshBtn.addEventListener("click", refreshFiles);
    fileInput.addEventListener("change", () => { selectedFiles = Array.from(fileInput.files || []); updateSelection(); });
    drop.addEventListener("dragover", (e) => { e.preventDefault(); drop.classList.add("active"); });
    drop.addEventListener("dragleave", () => drop.classList.remove("active"));
    drop.addEventListener("drop", (e) => {
      e.preventDefault(); drop.classList.remove("active");
      if (e.dataTransfer?.files?.length) { selectedFiles = Array.from(e.dataTransfer.files); updateSelection(); }
    });
    updateSelection();
    refreshFiles();
  </script>
</body>
</html>`;
}

export async function startNeurolink({ port = 3000, storage = "./shared" } = {}) {
  const app = express();
  app.use(express.json({ limit: "25mb" }));

  if (!existsSync(storage)) {
    await fs.mkdir(storage, { recursive: true });
  }

  const transfers = new Map();
  const uploads = [];

  app.get("/", (_req, res) => res.type("html").send(uiHtml()));
  app.get("/health", (_req, res) => res.json(apiResponse({ success: true, data: "healthy" })));
  app.use("/shared", express.static(storage));

  app.get("/files", async (_req, res) => {
    try {
      const dir = await fs.readdir(storage);
      const files = [];
      for (const name of dir) {
        const filePath = path.join(storage, name);
        const st = await fs.stat(filePath);
        if (st.isFile()) files.push({ name, size: st.size, modified_at: formatIso(st.mtime) });
      }
      files.sort((a, b) => b.modified_at.localeCompare(a.modified_at));
      res.json(apiResponse({ success: true, data: files }));
    } catch (e) {
      res.status(500).json(apiResponse({ success: false, error: e.message }));
    }
  });

  app.get("/uploads", (_req, res) => {
    const grouped = new Map();
    for (const u of uploads) {
      if (!grouped.has(u.batch_id)) grouped.set(u.batch_id, []);
      grouped.get(u.batch_id).push(u);
    }
    const batches = [...grouped.entries()].map(([batch_id, files]) => {
      files.sort((a, b) => a.uploaded_at.localeCompare(b.uploaded_at));
      return {
        batch_id,
        uploaded_at: files[files.length - 1]?.uploaded_at ?? new Date().toISOString(),
        files: files.map((f) => ({ name: f.name, size: f.size, uploaded_at: f.uploaded_at })),
      };
    });
    batches.sort((a, b) => b.uploaded_at.localeCompare(a.uploaded_at));
    res.json(apiResponse({ success: true, data: batches }));
  });

  app.get("/download/batch/:batch_id", async (req, res) => {
    const batchId = req.params.batch_id;
    const files = uploads.filter((u) => u.batch_id === batchId);
    if (files.length === 0) {
      res.status(404).send("Batch not found");
      return;
    }

    res.setHeader("Content-Type", "application/zip");
    res.setHeader("Content-Disposition", `attachment; filename="upload-${batchId}.zip"`);
    const archive = archiver("zip", { zlib: { level: 9 } });
    archive.on("error", () => res.status(500).end());
    archive.pipe(res);
    for (const file of files) {
      const filePath = path.join(storage, file.name);
      if (existsSync(filePath)) archive.file(filePath, { name: file.name });
    }
    archive.finalize();
  });

  app.get("/download/chunk/:filename", async (req, res) => {
    const name = safeName(decodeURIComponent(req.params.filename));
    const chunkSize = Number.parseInt(String(req.query.chunk_size || "0"), 10);
    const index = Number.parseInt(String(req.query.index || "0"), 10);
    if (!name || !Number.isInteger(chunkSize) || chunkSize <= 0 || !Number.isInteger(index) || index < 0) {
      res.status(400).send("Invalid chunk request");
      return;
    }

    const filePath = path.join(storage, name);
    try {
      const stat = await fs.stat(filePath);
      const start = index * chunkSize;
      if (start >= stat.size) {
        res.status(400).send("Chunk index out of range");
        return;
      }
      const end = Math.min(stat.size - 1, start + chunkSize - 1);
      res.setHeader("Content-Type", "application/octet-stream");
      res.setHeader("Content-Disposition", `attachment; filename="${name}.part${index}"`);
      createReadStream(filePath, { start, end }).pipe(res);
    } catch {
      res.status(404).send("File not found");
    }
  });

  app.post("/transfer/init", (req, res) => {
    const filename = safeName(req.body?.filename);
    const totalSize = Number(req.body?.total_size);
    const chunkSize = Number(req.body?.chunk_size);
    const batchId = req.body?.batch_id || null;
    if (!filename || !Number.isFinite(totalSize) || totalSize < 0 || !Number.isFinite(chunkSize) || chunkSize <= 0) {
      res.status(400).json(apiResponse({ success: false, error: "Invalid init payload" }));
      return;
    }

    const transferId = `trans_${Date.now()}_${Math.random().toString(16).slice(2, 8)}`;
    const totalChunks = Math.ceil(totalSize / chunkSize);
    const tempDir = path.join(os.tmpdir(), transferId);
    transfers.set(transferId, {
      transferId,
      filename,
      totalSize,
      chunkSize,
      totalChunks,
      batchId,
      tempDir,
      received: new Set(),
    });

    fs.mkdir(tempDir, { recursive: true }).then(() => {
      res.json(apiResponse({ success: true, data: { transfer_id: transferId, total_chunks: totalChunks } }));
    }).catch((e) => {
      res.status(500).json(apiResponse({ success: false, error: e.message }));
    });
  });

  app.post("/transfer/chunk", upload.single("chunk"), async (req, res) => {
    const transferId = req.body?.transfer_id;
    const chunkIndex = Number.parseInt(String(req.body?.chunk_index ?? ""), 10);
    const transfer = transfers.get(transferId);
    if (!transfer || !req.file || !Number.isInteger(chunkIndex) || chunkIndex < 0 || chunkIndex >= transfer.totalChunks) {
      res.status(400).json(apiResponse({ success: false, error: "Invalid chunk payload" }));
      return;
    }
    const chunkPath = path.join(transfer.tempDir, `chunk_${chunkIndex}.tmp`);
    try {
      await fs.writeFile(chunkPath, req.file.buffer);
      transfer.received.add(chunkIndex);
      res.json(apiResponse({
        success: true,
        data: { chunk_hash: "", received_count: transfer.received.size, total_chunks: transfer.totalChunks },
      }));
    } catch (e) {
      res.status(500).json(apiResponse({ success: false, error: e.message }));
    }
  });

  app.post("/transfer/complete", async (req, res) => {
    const transferId = req.body?.transfer_id;
    const transfer = transfers.get(transferId);
    if (!transfer) {
      res.status(404).json(apiResponse({ success: false, error: "Transfer not found" }));
      return;
    }
    if (transfer.received.size !== transfer.totalChunks) {
      res.status(400).json(apiResponse({ success: false, error: "Missing chunks" }));
      return;
    }

    const outPath = path.join(storage, transfer.filename);
    try {
      await fs.mkdir(path.dirname(outPath), { recursive: true });
      const ws = createWriteStream(outPath);
      for (let i = 0; i < transfer.totalChunks; i += 1) {
        const chunkPath = path.join(transfer.tempDir, `chunk_${i}.tmp`);
        const data = await fs.readFile(chunkPath);
        ws.write(data);
      }
      ws.end();
      await new Promise((resolve, reject) => {
        ws.on("finish", resolve);
        ws.on("error", reject);
      });

      uploads.push({
        batch_id: transfer.batchId || `single_${transferId}`,
        name: transfer.filename,
        size: transfer.totalSize,
        uploaded_at: new Date().toISOString(),
      });
      await fs.rm(transfer.tempDir, { recursive: true, force: true });
      transfers.delete(transferId);
      res.json(apiResponse({ success: true, data: { transfer_id: transferId, filename: transfer.filename, status: "completed" } }));
    } catch (e) {
      res.status(500).json(apiResponse({ success: false, error: e.message }));
    }
  });

  app.get("/transfer/:id/status", (req, res) => {
    const transfer = transfers.get(req.params.id);
    if (!transfer) {
      res.status(404).json(apiResponse({ success: false, error: "Transfer not found" }));
      return;
    }
    const pct = transfer.totalChunks === 0 ? 0 : Math.floor((transfer.received.size * 100) / transfer.totalChunks);
    res.json(apiResponse({
      success: true,
      data: {
        transfer_id: transfer.transferId,
        status: transfer.received.size === transfer.totalChunks ? "completed" : "in_progress",
        progress: `${pct}%`,
      },
    }));
  });

  app.listen(port, "0.0.0.0", () => {
    console.log(`NeuroLink Express listening on http://0.0.0.0:${port}`);
    console.log(`Local URL: http://localhost:${port}`);
    console.log(`Storage: ${storage}`);
  });
}

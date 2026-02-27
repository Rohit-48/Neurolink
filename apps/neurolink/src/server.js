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
      --bg:#f7f7f4;
      --ink:#111111;
      --muted:#666666;
      --line:#d9d9d3;
      --panel:#ffffff;
      --ok:#1f1f1f;
      --err:#3b3b3b;
    }
    *{box-sizing:border-box}
    body{
      margin:0;
      color:var(--ink);
      font-family:"Styrene B","Avenir Next","Inter",system-ui,sans-serif;
      background:radial-gradient(circle at top, #ffffff 0%, var(--bg) 55%);
      min-height:100vh;
      padding:30px 16px 50px;
      letter-spacing:0.01em;
    }
    .wrap{max-width:920px; margin:0 auto}
    .hero{
      border-top:2px solid var(--ink);
      padding-top:18px;
      margin-bottom:22px;
    }
    h1{
      margin:0;
      font-family:"Tiempos Headline","Iowan Old Style","Times New Roman",serif;
      font-size:clamp(34px,6vw,58px);
      font-weight:500;
      letter-spacing:-0.02em;
      line-height:.98;
    }
    .sub{
      margin:12px 0 0;
      color:var(--muted);
      font-size:16px;
      max-width:680px;
      line-height:1.6;
    }
    .meta{margin-top:14px;display:flex;gap:8px;flex-wrap:wrap}
    .pill{
      border:1px solid var(--line);
      border-radius:999px;
      padding:5px 11px;
      font-size:12px;
      color:#2c2c2c;
      background:#fff;
    }
    .card{
      background:var(--panel);
      border:1px solid var(--line);
      border-radius:16px;
      padding:18px;
      box-shadow:0 8px 24px rgba(0,0,0,.04);
    }
    .card + .card{margin-top:16px}
    .title{margin:0 0 6px;font-size:13px;text-transform:uppercase;letter-spacing:.12em;color:#4f4f4f}
    .hint{margin:0 0 14px;font-size:14px;color:var(--muted)}
    .drop{
      border:1px dashed #c9c9c3;
      border-radius:12px;
      padding:14px;
      margin-bottom:12px;
      background:#fcfcfb;
      color:#444;
      font-size:14px;
    }
    .drop.active{border-color:#999;background:#f2f2ee}
    .actions{display:flex;gap:10px;flex-wrap:wrap}
    button,.link-btn{
      border-radius:11px;
      border:1px solid #c7c7c1;
      padding:10px 13px;
      font-size:13px;
      font-weight:600;
      cursor:pointer;
      background:#fff;
      color:#111;
    }
    .primary{background:#111;border-color:#111;color:#fff}
    button:disabled{opacity:.55;cursor:not-allowed}
    .progress{margin-top:12px;height:8px;border-radius:999px;background:#ededea;overflow:hidden}
    .bar{height:100%;width:0%;background:linear-gradient(90deg,#202020,#545454);transition:width .15s linear}
    #status{min-height:20px;margin-top:10px;color:#404040;font-size:14px}
    #selection{margin:5px 0 0;color:var(--muted);font-size:13px}
    .ok{color:var(--ok)}
    .err{color:var(--err)}
    .files{list-style:none;margin:0;padding:0;border-top:1px solid var(--line)}
    .files li{border-bottom:1px solid var(--line);padding:12px 0}
    .batch-head{display:flex;justify-content:space-between;align-items:center;gap:8px;color:var(--muted);font-size:12px;margin-bottom:8px}
    .file-row{display:flex;justify-content:space-between;gap:10px;align-items:center;padding:7px 0}
    a.file-link{color:#121212;text-decoration:none;font-size:14px;overflow-wrap:anywhere}
    a.file-link:hover{text-decoration:underline}
    .size{color:var(--muted);font-size:12px}
    .hidden-input{display:none}
    @media (max-width:760px){body{padding:22px 12px 34px}.card{padding:14px}}
  </style>
</head>
<body>
  <div class="wrap">
    <section class="hero">
      <h1>NeuroLink</h1>
      <p class="sub">Fast local transfers with clean batch uploads. Pick a folder, upload once, share single files or a full batch zip.</p>
      <div class="meta">
        <span class="pill">Express Runtime</span>
        <span class="pill">Batch Download: ZIP</span>
        <span class="pill">API: /transfer/*</span>
      </div>
    </section>

    <section class="card">
      <h2 class="title">Upload</h2>
      <p class="hint">Primary flow: click Upload Folder, then Start Upload.</p>
      <div id="drop" class="drop">Drop files/folder here or use the buttons below.</div>
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
      <div id="selection"></div>
    </section>

    <section class="card">
      <h2 class="title">Upload Batches</h2>
      <p class="hint">Newest batches first. Every upload run creates one batch.</p>
      <ul id="files" class="files"></ul>
    </section>
  </div>
  <script>
    const CHUNK_SIZE = 1024 * 1024;
    const folderInput = document.getElementById("folderInput");
    const fileInput = document.getElementById("fileInput");
    const pickFolderBtn = document.getElementById("pickFolderBtn");
    const pickFilesBtn = document.getElementById("pickFilesBtn");
    const startUploadBtn = document.getElementById("startUploadBtn");
    const drop = document.getElementById("drop");
    const refreshBtn = document.getElementById("refreshBtn");
    const bar = document.getElementById("bar");
    const statusEl = document.getElementById("status");
    const selectionEl = document.getElementById("selection");
    const filesEl = document.getElementById("files");
    let selectedFiles = [];

    const setStatus = (text, kind = "") => { statusEl.className = kind; statusEl.textContent = text; };
    const formatBytes = (n) => n < 1024 ? (n + " B") : n < 1024 * 1024 ? ((n / 1024).toFixed(1) + " KB") : ((n / (1024 * 1024)).toFixed(2) + " MB");
    const updateSelection = () => {
      if (!selectedFiles.length) { selectionEl.textContent = "No files selected"; return; }
      const total = selectedFiles.reduce((a, f) => a + f.size, 0);
      selectionEl.textContent = selectedFiles.length + " files selected · " + formatBytes(total);
    };
    const setFilesFromList = (list) => { selectedFiles = Array.from(list || []); updateSelection(); };

    async function refreshFiles() {
      const res = await fetch("/uploads");
      const json = await res.json();
      if (!res.ok || !json.success || !Array.isArray(json.data)) {
        filesEl.innerHTML = "<li>Failed to load uploads.</li>";
        return;
      }
      if (json.data.length === 0) {
        filesEl.innerHTML = "<li>No uploads yet.</li>";
        return;
      }

      filesEl.innerHTML = json.data.map((batch) => {
        const when = new Date(batch.uploaded_at).toLocaleString();
        const items = batch.files.map((file) =>
          '<div class="file-row">' +
            '<a class="file-link" href="/shared/' + encodeURIComponent(file.name) + '" target="_blank" rel="noreferrer">' + file.name + "</a>" +
            '<span class="size">' + formatBytes(file.size) + "</span>" +
          "</div>"
        ).join("");

        return (
          "<li>" +
            '<div class="batch-head"><span>' + when + " · " + batch.files.length + ' file(s)</span>' +
              '<a class="link-btn" href="/download/batch/' + encodeURIComponent(batch.batch_id) + '">Download ZIP</a>' +
            "</div>" +
            items +
          "</li>"
        );
      }).join("");
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
        form.append("chunk", chunkBlob, file.name + ".part" + idx);
        const chunkRes = await fetch("/transfer/chunk", { method: "POST", body: form });
        const chunkJson = await chunkRes.json();
        if (!chunkRes.ok || !chunkJson.success) throw new Error(chunkJson.error || ("Chunk " + (idx + 1) + " failed"));
        const uploaded = doneBytes + Math.min(file.size, (idx + 1) * CHUNK_SIZE);
        bar.style.width = Math.floor((uploaded / totalBytes) * 100) + "%";
      }

      const doneRes = await fetch("/transfer/complete", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ transfer_id: transferId })
      });
      const doneJson = await doneRes.json();
      if (!doneRes.ok || !doneJson.success) throw new Error(doneJson.error || "Complete failed");
    }

    async function uploadBatch() {
      if (!selectedFiles.length) { setStatus("Select files or a folder first.", "err"); return; }
      startUploadBtn.disabled = true;
      bar.style.width = "0%";
      const totalBytes = selectedFiles.reduce((n, f) => n + f.size, 0);
      const batchId = "batch_" + Date.now();
      let doneBytes = 0;
      try {
        for (let i = 0; i < selectedFiles.length; i++) {
          const file = selectedFiles[i];
          setStatus("Uploading " + (i + 1) + "/" + selectedFiles.length + ": " + file.name);
          await uploadFile(file, batchId, doneBytes, totalBytes);
          doneBytes += file.size;
        }
        setStatus("Batch upload complete (" + selectedFiles.length + " file(s))", "ok");
        await refreshFiles();
      } catch (err) {
        setStatus(err.message || "Upload failed", "err");
      } finally {
        startUploadBtn.disabled = false;
      }
    }

    pickFolderBtn.addEventListener("click", () => folderInput.click());
    pickFilesBtn.addEventListener("click", () => fileInput.click());
    startUploadBtn.addEventListener("click", uploadBatch);
    refreshBtn.addEventListener("click", refreshFiles);
    folderInput.addEventListener("change", () => setFilesFromList(folderInput.files));
    fileInput.addEventListener("change", () => setFilesFromList(fileInput.files));

    drop.addEventListener("dragover", (e) => { e.preventDefault(); drop.classList.add("active"); });
    drop.addEventListener("dragleave", () => drop.classList.remove("active"));
    drop.addEventListener("drop", (e) => {
      e.preventDefault();
      drop.classList.remove("active");
      if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length) setFilesFromList(e.dataTransfer.files);
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

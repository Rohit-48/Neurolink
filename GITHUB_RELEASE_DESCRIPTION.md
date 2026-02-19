## NeuroLink v1.0.0 - Foundation Release 🚀

First stable release of NeuroLink - Local network file sharing made simple.

### What's New

**Core Features:**
- 📁 **Web Interface** - Drag-and-drop file sharing from any browser
- 🔍 **Device Discovery** - Automatic mDNS/Bonjour device detection  
- 💻 **Interactive CLI** - Menu-driven terminal interface
- 📦 **Session Organization** - Files grouped by upload sessions
- 🏷️ **Smart Categorization** - Auto-sort (Photos, Videos, Files)

**Supported File Types:**
- Images: jpg, png, gif, webp, svg, raw, psd
- Videos: mp4, mov, avi, mkv, webm
- Documents: pdf, doc, txt, md
- Archives: zip, rar, 7z
- Code files: js, ts, py, html, css

### Installation

```bash
npm install -g neurolink
```

Or from source:
```bash
git clone https://github.com/Rohit-48/Neurolink.git
cd Neurolink
npm install && npm run build
npm link
```

### Quick Start

```bash
# Start server
neurolink

# Access web UI
# Open http://localhost:3000

# Send files
neuroshare send file.pdf
```

### Architecture

This release includes the **Node.js foundation** with a **Rust microservice** ready for v2:

- **Node.js (Port 3000)**: Web UI, discovery, sessions
- **Rust (Port 3030)**: High-performance chunked transfers

```
Node.js                    Rust (Preview)
├─ Express server          ├─ Axum HTTP server
├─ Web UI                  ├─ Chunked file transfer
├─ mDNS discovery          ├─ SHA-256 verification
└─ Session management      └─ Progress tracking
```

### API Endpoints

**Node.js:**
- `GET /` - Web interface
- `GET /api/files` - List files
- `POST /api/upload` - Upload file

**Rust (Optional):**
- `POST /transfer/init` - Initialize chunked transfer
- `POST /transfer/chunk` - Upload chunk
- `GET /health` - Health check

### Coming in v2.0

- File deduplication (SHA-256)
- Bandwidth optimization
- Real-time sync (file watcher)
- Desktop app (Tauri)
- P2P mesh networking

See [ROADMAP.md](ROADMAP.md) for details.

### System Requirements

- Node.js 18+
- Same local network (WiFi/LAN)
- Firewall port 3000 open

### Documentation

- [Full README](README.md)
- [Release Notes](RELEASE-v1.0.0.md)
- [Rust Implementation](RUST_IMPLEMENTATION.md)
- [Testing Guide](rust-service/TESTING.md)

### Security

⚠️ Designed for trusted local networks only. No authentication by default.

---

**Full Changelog**: Compare with previous releases

**Contributors**: Thank you to everyone who contributed to this release!

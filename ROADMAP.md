# NeuroLink Release Roadmap

## Current Status

**Version 1.0.0** - Stable Release ✅

---

## Release v1.0.0 (Current)

### What's Included

#### Core Features
- ✅ **Web Interface** - Browser-based file upload/download
- ✅ **Session-Based Grouping** - Files organized by upload sessions (5-min window)
- ✅ **File Categorization** - Automatic sorting (Photos, Videos, Files)
- ✅ **Device Discovery** - mDNS/Bonjour for automatic device detection
- ✅ **Interactive CLI** - Menu-driven interface for server management
- ✅ **File Management** - Upload, download, delete individual files
- ✅ **Batch Operations** - Download all files or per-session ZIP archives
- ✅ **Mobile Responsive** - Works on phones and tablets

#### Node.js Service (`neurolink`)
- Express server with Hono framework
- Web UI with drag-and-drop upload
- mDNS service advertisement
- REST API for file operations
- Session tracking and metadata

#### Supported File Types
- **Images**: jpg, png, gif, webp, svg, bmp, tiff, raw, psd
- **Videos**: mp4, mov, avi, mkv, wmv, flv, webm, m4v
- **Audio**: mp3, wav, flac, m4a, aac, ogg, wma
- **Documents**: pdf, doc, docx, txt, md, xls, xlsx, ppt
- **Archives**: zip, rar, 7z, tar, gz
- **Code**: js, ts, py, html, css, json, and more

#### API Endpoints (Node.js)
- `GET /` - Web interface
- `GET /api/files` - List all files
- `GET /api/files/grouped` - Files by session
- `GET /api/files/:name` - Download file
- `POST /api/upload` - Upload file (multipart)
- `DELETE /api/files/:name` - Delete file
- `GET /api/download-all` - Download ZIP
- `GET /api/download-session/:timestamp` - Session ZIP

#### CLI Commands
```bash
# Start server with interactive menu
neurolink

# Send files to discovered device
neuroshare send file.pdf

# List available devices
neuroshare devices
```

#### Configuration
- Port: 3000 (default)
- Shared directory: ./shared
- Device name: System hostname
- Environment variables: NEUROLINK_PORT, NEUROLINK_DIR, NEUROLINK_NAME

### Known Limitations (v1)
- Single-threaded file transfers
- No file deduplication
- No compression
- No bandwidth optimization
- No real-time sync
- No P2P/mesh networking

---

## Release v2.0.0 (Planned)

### Architecture Changes
- **Hybrid Stack**: Node.js (UI/API) + Rust (Performance Engine)
- **Microservices**: Separate services on different ports
- **API Gateway**: Node.js proxies to Rust for heavy operations

### New Features

#### 1. Rust Microservice (`neurolinkd`) 🦀
**Status**: Partially implemented, needs integration

**Features**:
- ✅ Chunked parallel file transfers (1MB chunks)
- ✅ SHA-256 hash verification per chunk
- ✅ Concurrent chunk uploads
- ✅ Async I/O with Tokio
- ✅ Progress tracking with progress bars
- ✅ REST API for transfer lifecycle
- ✅ CLI tool (`neuroshare` in Rust)

**API Endpoints**:
- `POST /transfer/init` - Initialize chunked transfer
- `POST /transfer/chunk` - Upload chunk (multipart)
- `POST /transfer/complete` - Finalize transfer
- `GET /transfer/:id/status` - Check progress
- `GET /health` - Health check

**Performance Improvements**:
- 2-5x faster for large files
- Lower memory usage
- Better CPU efficiency
- Concurrent upload support

#### 2. File Deduplication (#4)
**Status**: Foundation ready, needs completion

**Implementation**:
- SHA-256 hash index for all files
- Before saving: check if hash exists
- If duplicate: return reference, discard new file
- Lightweight in-memory hash table
- Startup scanner to rebuild index
- Streaming hash computation (memory efficient)

**Benefits**:
- Save disk space
- Faster subsequent uploads
- Prevent duplicate files

#### 3. Tauri Desktop Application (#6)
**Status**: Not started

**Features**:
- Native desktop app (Windows, macOS, Linux)
- System tray icon
- Native file picker
- Desktop notifications
- Auto-start on boot
- No browser needed

**Tech Stack**:
- Tauri (Rust backend)
- React/Vue frontend
- Secure IPC communication

#### 4. File System Watcher (#7)
**Status**: Dependencies included, needs implementation

**Features**:
- Watch shared directory for changes
- Real-time sync across devices
- Detect: create, modify, delete
- Debounce rapid changes
- Auto-trigger transfers

**Use Cases**:
- Dropbox-like sync
- Backup folders
- Collaborative editing

#### 5. Adaptive Bandwidth Optimizer (#8)
**Status**: Not started

**Features**:
- Measure latency and throughput
- Dynamic chunk size adjustment
- Parallel stream optimization
- Feedback loop algorithm
- CPU/memory aware tuning

**Algorithm**:
- Increase parallelism when stable
- Reduce when latency spikes
- Gradual adjustment (avoid oscillation)

#### 6. LAN Mesh Mode / P2P (#10) 🔥
**Status**: Not started - Major feature

**Features**:
- Peer-to-peer architecture
- UDP broadcast discovery
- Gossip protocol for metadata
- Conflict resolution (timestamp-based)
- No central server required

**Architecture**:
```
┌─────────┐      ┌─────────┐      ┌─────────┐
│  Node A │◄────►│  Node B │◄────►│  Node C │
└─────────┘      └─────────┘      └─────────┘
      ▲                ▲                ▲
      └────────────────┴────────────────┘
              Gossip Protocol
```

**Benefits**:
- True decentralization
- No single point of failure
- Works without internet
- Scales with device count

### Breaking Changes (v2)

#### Port Changes
- Node.js: 3000 (unchanged)
- Rust service: 3030 (new)
- Tauri app: Internal port (auto)

#### API Changes
- `/api/upload` → Proxy to Rust service OR direct to Rust
- New `/transfer/*` endpoints (Rust)
- Backward compatibility layer needed

#### CLI Changes
- `neuroshare` rewritten in Rust
- New flags: `--chunk-size`, `--concurrent`
- Better progress bars
- Static binary distribution

### Migration Path (v1 → v2)

#### Phase 1: Side-by-Side
1. Install v2 alongside v1
2. Run both services
3. Test compatibility
4. Gradual migration

#### Phase 2: Integration
1. Node.js proxies uploads to Rust
2. Web UI uses Rust for large files
3. Small files stay on Node.js
4. Transparent to users

#### Phase 3: Full v2
1. Optional Rust service for performance
2. Node.js for simple deployments
3. User chooses stack

---

## Implementation Priority

### High Priority (Core v2)
1. ✅ Chunked transfer engine (Rust) - DONE
2. ✅ neuroshare CLI (Rust) - DONE
3. 🔲 File deduplication - READY TO IMPLEMENT
4. 🔲 Integration layer (Node.js ↔ Rust) - NEXT

### Medium Priority (Enhancements)
5. 🔲 File system watcher
6. 🔲 Bandwidth optimizer
7. 🔲 Compression (zstd)

### Low Priority (Advanced)
8. 🔲 Tauri desktop app
9. 🔲 LAN mesh / P2P mode

---

## Technical Debt

### v1 Cleanup
- [ ] Remove old src/cli.ts (moved to src/cli/)
- [ ] Consolidate TypeScript types
- [ ] Add comprehensive tests
- [ ] Documentation cleanup

### v2 Preparation
- [ ] Define API contract between services
- [ ] Create integration tests
- [ ] Docker compose setup
- [ ] CI/CD pipeline

---

## Version Timeline

```
v1.0.0 (Current) ────► v1.1.0 ────► v2.0.0-alpha ────► v2.0.0
    │                      │              │                │
    │                      │              │                │
Current              Bug fixes     Rust service    Production
release              Minor         integration     ready
                     features      Testing
```

### Estimated Timeline
- **v1.1.0**: 2-4 weeks (bug fixes, minor features)
- **v2.0.0-alpha**: 6-8 weeks (Rust integration)
- **v2.0.0**: 10-12 weeks (production ready)

---

## Contributing

### v1 Contributions
- Bug fixes welcome
- Minor features (UI improvements)
- Documentation
- Testing

### v2 Contributions
- Rust expertise needed
- Architecture discussions required
- Feature design review
- Performance benchmarking

---

## Questions?

See individual feature files:
- `rust-service/README.md` - Rust service details
- `RUST_IMPLEMENTATION.md` - Implementation notes
- `TESTING.md` - Testing guide

Or open an issue on GitHub.

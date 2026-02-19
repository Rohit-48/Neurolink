# Personal Development Notes - NeuroLink

## Project Overview

**NeuroLink** is a local network file sharing tool built with a hybrid architecture:
- **Rust** (Primary): High-performance file transfer engine
- **Node.js** (Optional): Web UI for browser access

## How It Works

### Architecture Flow

```
User wants to send a file:

1. neuroshare (CLI) → Splits file into 1MB chunks
2. Each chunk → Sent to neurolinkd (Rust server)
3. Server → Verifies chunk with SHA-256 hash
4. All chunks received → File reassembled
5. File saved → ./shared/ directory

┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│   User CLI   │      │ Rust Server  │      │   Storage    │
│  neuroshare  │──────│  neurolinkd  │──────│   ./shared   │
└──────────────┘      └──────────────┘      └──────────────┘
      │                       │                     │
      │  1. POST /transfer/init│                     │
      │──────────────────────>│                     │
      │                       │                     │
      │  2. POST /transfer/chunk (for each chunk)   │
      │──────────────────────>│                     │
      │                       │ 3. Save chunk       │
      │                       │────────────────────>│
      │                       │                     │
      │  4. POST /transfer/complete                 │
      │──────────────────────>│                     │
      │                       │ 5. Reassemble       │
      │                       │────────────────────>│
```

## Key Components

### 1. Rust Server (`neurolinkd`)

**File**: `src/rust/main.rs`

Purpose: HTTP server handling file transfers

Key parts:
- **Axum**: Web framework (like Express for Rust)
- **Tokio**: Async runtime (handles multiple connections)
- **TransferManager**: Tracks active file uploads
- **Port**: 3030 (default)

### 2. Transfer Engine (`src/rust/transfer/mod.rs`)

Purpose: Core logic for chunked file transfers

Key structs:
```rust
TransferManager {
    transfers: HashMap<String, Transfer>  // Active uploads
}

Transfer {
    metadata: TransferMetadata,      // File info
    temp_dir: TempDir,               // Temporary storage
    received_chunks: HashMap<...>    // Which chunks received
}
```

Key methods:
- `init_transfer()`: Create new upload session
- `receive_chunk()`: Process incoming chunk
- `complete_transfer()`: Reassemble file

### 3. API Routes (`src/rust/api/routes.rs`)

Purpose: HTTP endpoint handlers

Endpoints:
- `POST /transfer/init`: Start upload
  ```json
  Request: {"filename": "file.zip", "total_size": 10485760, "chunk_size": 1048576}
  Response: {"transfer_id": "trans_123", "total_chunks": 10}
  ```

- `POST /transfer/chunk`: Upload chunk (multipart form)
  - Fields: transfer_id, chunk_index, chunk (binary)

- `POST /transfer/complete`: Finish upload
  - Reassembles all chunks into final file

### 4. CLI Client (`src/rust/cli.rs`)

Purpose: User interface for sending files

Flow:
1. Read file from disk
2. Get file size
3. Split into chunks (default 1MB)
4. Upload chunks sequentially
5. Show progress bar

Key functions:
- `send_files()`: Main upload logic
- `format_size()`: Convert bytes to KB/MB/GB

## How to Use

### Installation

```bash
# Clone repo
git clone https://github.com/Rohit-48/Neurolink.git
cd Neurolink

# Build Rust components
cargo build --release

# Copy binaries to PATH
cp target/release/neurolinkd ~/.local/bin/
cp target/release/neuroshare ~/.local/bin/
```

### Basic Usage

**Terminal 1 - Start Server:**
```bash
neurolinkd
```

**Terminal 2 - Send File:**
```bash
# Send single file
neuroshare send document.pdf --host localhost --port 3030

# Send multiple files
neuroshare send photo1.jpg photo2.png --host 192.168.1.100

# Custom chunk size (4MB)
neuroshare send large-video.mp4 --chunk-size 4096
```

### With Web UI (Optional)

**Terminal 1 - Rust Server:**
```bash
neurolinkd --port 3030
```

**Terminal 2 - Node.js Web UI:**
```bash
cd /path/to/neurolink
npm install -g .
neurolink --port 3000
```

**Browser:**
- Open http://localhost:3000
- Use web interface to upload/download

## Development

### Building

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized, slower)
cargo build --release

# Run with logs
RUST_LOG=debug cargo run --bin neurolinkd
```

### Testing

```bash
# Start server
cargo run --bin neurolinkd

# In another terminal, send test file
cargo run --bin neuroshare -- send test.txt --host localhost
```

### Project Structure

```
Neurolink/
├── Cargo.toml              # Rust dependencies
├── src/rust/
│   ├── main.rs            # Server entry point
│   ├── cli.rs             # CLI client
│   ├── api/
│   │   └── routes.rs      # HTTP handlers
│   ├── transfer/
│   │   └── mod.rs         # Transfer logic
│   └── hashing/
│       └── mod.rs         # SHA-256 utilities
├── package.json           # Node.js (optional)
└── shared/                # File storage (created at runtime)
```

## Key Design Decisions

### Why Chunked Transfers?

1. **Memory Efficiency**: Don't load entire file into RAM
2. **Reliability**: If one chunk fails, retry just that chunk
3. **Progress Tracking**: Show upload progress per chunk
4. **Concurrent Uploads**: Send multiple chunks simultaneously

### Why SHA-256?

1. **Data Integrity**: Verify each chunk wasn't corrupted
2. **Security**: Cryptographic hash, hard to fake
3. **Deduplication**: Can detect duplicate files (future feature)

### Why Rust?

1. **Performance**: Zero-cost abstractions, fast as C
2. **Safety**: Memory safety without GC pauses
3. **Async**: Built-in async/await with Tokio
4. **Binaries**: Single static executable

## Common Issues & Solutions

### Port Already in Use
```bash
# Find process using port 3030
lsof -i :3030

# Kill it
kill <PID>

# Or use different port
neurolinkd --port 3031
```

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Permission Denied
```bash
# Make binaries executable
chmod +x target/release/neurolinkd
chmod +x target/release/neuroshare
```

## Environment Variables

```bash
# Rust Server
NEUROLINK_PORT=3030           # Server port
NEUROLINK_STORAGE=./shared    # File storage path
RUST_LOG=info                 # Log level (error, warn, info, debug, trace)

# Node.js (optional)
NEUROLINK_PORT=3000
NEUROLINK_DIR=./shared
NEUROLINK_NAME=my-device
```

## Future Improvements

### v2.1 Ideas
1. **File Deduplication**: Skip uploading duplicate files
2. **Compression**: zstd compression before transfer
3. **Bandwidth Limiting**: Don't use all network bandwidth
4. **Resume**: Resume interrupted uploads
5. **Encryption**: End-to-end encryption

### v3.0 Ideas
1. **P2P**: Direct device-to-device, no server needed
2. **Relay**: Transfer through intermediate devices
3. **Sync**: Real-time folder synchronization

## Useful Commands

```bash
# Watch for changes and rebuild
cargo watch -x run

# Format code
cargo fmt

# Run tests
cargo test

# Check without building
cargo check

# Generate documentation
cargo doc --open
```

## Resources

- **Axum Docs**: https://docs.rs/axum
- **Tokio Docs**: https://tokio.rs/tokio/tutorial
- **Rust Book**: https://doc.rust-lang.org/book/
- **Project Repo**: https://github.com/Rohit-48/Neurolink

## Notes

- Keep `shared/` directory backed up - that's where all files go
- Rust server is primary, Node.js is just for web UI
- Use `--host` with IP address when discovery fails
- Default chunk size (1MB) is good for most use cases
- Progress bars require terminal with Unicode support

## Personal Reminders

- [ ] Add file deduplication (hash check before upload)
- [ ] Add bandwidth limiting option
- [ ] Test on Windows
- [ ] Add compression (zstd)
- [ ] Create Docker image
- [ ] Add systemd service file
- [ ] Write proper tests
- [ ] Benchmark performance vs v1.x
- [ ] Create GitHub Actions CI/CD

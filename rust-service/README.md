# NeuroLink Rust Microservice

High-performance file transfer engine written in Rust.

## Features

- **Chunked Parallel Transfer**: Split files into chunks for concurrent upload
- **SHA-256 Verification**: Every chunk is hash-verified
- **Out-of-order Support**: Chunks can arrive in any order
- **Async I/O**: Built on Tokio for maximum performance
- **Progress Tracking**: Real-time transfer status
- **CLI Tool**: `neuroshare` for sending files

## Architecture

```
┌─────────────────┐     HTTP      ┌──────────────────┐
│   Node.js API   │ ◄────────────► │  Rust Microservice│
│   (Express)     │   localhost    │  (Axum + Tokio)   │
└─────────────────┘                └──────────────────┘
                                           │
                                           ▼
                                    ┌──────────────┐
                                    │ Chunked File │
                                    │   Storage    │
                                    └──────────────┘
```

## Building

```bash
cd rust-service
cargo build --release
```

## Running

### Start the server

```bash
./target/release/neurolinkd
# Or with custom port
NEUROLINK_PORT=3030 ./target/release/neurolinkd
```

### Send files

```bash
./target/release/neuroshare send file.pdf --host 192.168.1.100 --port 3030

# Multiple files
./target/release/neuroshare send file1.pdf file2.jpg --host localhost
```

## API Endpoints

### POST /transfer/init
Initialize a new transfer.

Request:
```json
{
  "filename": "document.pdf",
  "total_size": 10485760,
  "chunk_size": 1048576
}
```

Response:
```json
{
  "success": true,
  "data": {
    "transfer_id": "trans_1234567890",
    "total_chunks": 10
  }
}
```

### POST /transfer/chunk
Upload a chunk (multipart form).

Fields:
- `transfer_id`: Transfer ID
- `chunk_index`: Chunk number (0-based)
- `chunk`: Binary data

### POST /transfer/complete
Finalize transfer and reassemble file.

Request:
```json
{
  "transfer_id": "trans_1234567890"
}
```

### GET /transfer/:id/status
Check transfer progress.

### GET /health
Health check endpoint.

## Environment Variables

- `NEUROLINK_PORT`: Server port (default: 3030)
- `NEUROLINK_STORAGE`: Storage directory (default: ./shared)

## Performance

- Chunk size: 1MB default (configurable)
- Concurrent uploads: Unlimited (memory-bound)
- Hash computation: Streaming SHA-256
- File I/O: Async Tokio fs

## Integration with Node.js

The Rust service runs alongside the Node.js API:

1. Node.js handles: Web UI, device discovery, session management
2. Rust handles: File transfers, hashing, compression

Communication via HTTP on localhost.

## TODO

- [ ] File deduplication (hash index)
- [ ] Compression (zstd)
- [ ] Bandwidth optimization
- [ ] File watcher (real-time sync)
- [ ] mDNS device discovery in CLI
- [ ] LAN mesh mode (P2P)

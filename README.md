# NeuroLink v2.0.0 🚀

High-performance local network file sharing with chunked transfers and device discovery.

[![Version](https://img.shields.io/badge/version-2.0.0-blue.svg)](https://github.com/Rohit-48/Neurolink/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## What's New in v2.0

- ⚡ **Rust-Powered Performance** - 2-5x faster file transfers with chunked uploads
- 🔐 **SHA-256 Verification** - Every chunk hash-verified for integrity  
- 📊 **Progress Tracking** - Real-time progress bars in CLI
- 🔄 **Concurrent Uploads** - Multiple chunks simultaneously
- 🎯 **Smart Chunking** - Automatic 1MB chunks for optimal performance
- 💻 **Pure Rust CLI** - Static binary, no Node.js needed for client

## Components

- **`neurolinkd`** - High-performance Rust server (port 3030)
- **`neuroshare`** - Rust CLI client with progress bars
- **`neurolink`** - Node.js server with web UI (port 3000) - optional

## Features

- **Local file transfer** over HTTP
- **Automatic device discovery** on local network (mDNS/Bonjour)
- **High-performance Rust microservice** for chunked file transfers
- **Progress tracking** with CLI progress bars
- **SHA-256 verification** for all chunks
- **Session-based grouping** of uploaded files
- **Web UI** for browser-based upload/download
- **Direct host mode** when discovery unavailable

## Architecture

NeuroLink v2 uses a hybrid architecture:

- **Node.js (Express)**: Web UI, device discovery, session management
- **Rust (Axum + Tokio)**: High-performance file transfer engine
- **Communication**: HTTP/REST API between services

```
┌─────────────────┐     HTTP      ┌──────────────────┐
│   Node.js API   │ ◄────────────► │  Rust Service    │
│   (Express)     │   localhost    │  (Axum + Tokio)  │
│   Port 3000     │                │  Port 3030       │
└─────────────────┘                └──────────────────┘
        │                                   │
        ▼                                   ▼
┌──────────────┐                    ┌──────────────┐
│   Web UI     │                    │ Chunked File │
│  (Browser)   │                    │   Storage    │
└──────────────┘                    └──────────────┘
```

## Why Rust?

NeuroLink v2.0 is built with Rust for maximum performance and reliability:

### Performance Benefits
- **Zero-cost abstractions** - High-level code with C-level performance
- **Memory safety** - No garbage collection pauses or memory leaks
- **Async/await** - Built on Tokio for scalable concurrent I/O
- **Static binaries** - Single executable, no runtime dependencies

### Technical Stack
- **Axum** - Modern, ergonomic web framework
- **Tokio** - Rust's premier async runtime
- **Tower** - Modular middleware system
- **Tracing** - Structured, contextual logging

## Components

### 1. Rust Server (`neurolinkd`) - Primary
**Location**: `src/rust/main.rs`

Core file transfer engine:
```rust
// Chunked transfer with SHA-256 verification
let chunk_hash = sha256::compute(&chunk_data);
transfer_manager.receive_chunk(id, index, chunk_data).await?;
```

Features:
- **Chunked transfers** - Files split into 1MB chunks for parallel upload
- **SHA-256 verification** - Every chunk hash-verified
- **Concurrent processing** - Multiple chunks simultaneously
- **Streaming I/O** - Memory-efficient file handling
- **Graceful shutdown** - SIGTERM/SIGINT handling

**Architecture**:
```
┌─────────────────────────────────────┐
│        neurolinkd (Rust)            │
├─────────────────────────────────────┤
│  Axum HTTP Server (Port 3030)       │
├─────────────────────────────────────┤
│  Transfer Manager                   │
│  ├─ HashMap<TransferId, Transfer>  │
│  ├─ Chunk validation (SHA-256)     │
│  └─ Async file I/O (Tokio)         │
├─────────────────────────────────────┤
│  Storage                            │
│  └─ ./shared/ (configurable)       │
└─────────────────────────────────────┘
```

### 2. Rust CLI (`neuroshare`)
**Location**: `src/rust/cli.rs`

Client for sending files:
```rust
// Automatic chunking and upload
let chunks = file.split(CHUNK_SIZE);
for (i, chunk) in chunks.enumerate() {
    upload_chunk(transfer_id, i, chunk).await?;
}
```

Features:
- **Progress bars** - Real-time upload progress
- **Batch uploads** - Multiple files at once
- **Host/port config** - Flexible targeting
- **Error recovery** - Automatic retry on failure

### 3. Node.js Server (`neurolink`) - Optional
**Location**: `dist/cli/main.js`

Web UI and device discovery (optional add-on):
- Web interface for browser users
- mDNS service advertisement
- Session metadata display

## File Structure

```
neurolink/
├── Cargo.toml              # Rust workspace
├── Cargo.lock              # Dependency lock
├── src/
│   └── rust/
│       ├── main.rs         # Server entry
│       ├── cli.rs          # CLI client
│       ├── api/
│       │   └── routes.rs   # HTTP endpoints
│       ├── transfer/
│       │   └── mod.rs      # Transfer engine
│       └── hashing/
│           └── mod.rs      # SHA-256 utilities
├── package.json            # Node.js (optional)
└── shared/                 # File storage
```

## Requirements

- Node.js 18 or later
- npm
- Devices connected to the same local network

## Quick Installation

### Option 1: Pre-built Binaries (Recommended)

Download from [GitHub Releases](https://github.com/Rohit-48/Neurolink/releases):

```bash
# Linux/macOS
curl -L https://github.com/Rohit-48/Neurolink/releases/download/v2.0.0/neurolink-v2.0.0-linux.tar.gz | tar xz
sudo mv neurolinkd neuroshare /usr/local/bin/

# Or install to user directory
mv neurolinkd neuroshare ~/.local/bin/
```

### Option 2: Build from Source

**Requirements:** Rust 1.70+ and Node.js 18+ (optional, for web UI)

```bash
git clone https://github.com/Rohit-48/Neurolink.git
cd Neurolink/rust-service

# Build Rust components
cargo build --release

# Install to PATH
cp target/release/neurolinkd ~/.local/bin/
cp target/release/neuroshare ~/.local/bin/

# Optional: Install Node.js web UI
cd ..
npm install -g .
```

### Option 3: npm (Node.js only, v1.x compatible)

```bash
npm install -g neurolink
```

**Note:** npm install only provides the Node.js version. For full v2.0 performance, use Rust binaries.

## Quick Start

### Rust-Only Mode (Recommended)

Fastest performance, no Node.js required:

```bash
# Terminal 1: Start Rust server
neurolinkd

# Terminal 2: Send files with progress bar
neuroshare send ./large-file.zip --host localhost --port 3030
```

That's it! The Rust server provides the complete file transfer API.

### With Web UI (Optional)

For browser-based upload/download:

```bash
# Terminal 1: Rust server (file transfers)
neurolinkd --port 3030

# Terminal 2: Node.js server (web UI)
neurolink --port 3000

# Access web UI at http://localhost:3000
```

### Configuration

**Rust service (primary):**
- Port: `3030` (default)
- Storage: `./shared` (created automatically)
- Chunk size: 1MB (configurable via `--chunk-size`)

**Node.js service (optional, for web UI):**
- Port: `3000` (default)
- Shared directory: `./shared`
- Device name: system hostname

## CLI Reference

### `neurolink`

Start the server and interactive menu.

```bash
neurolink [options]
```

Options:

- `-p, --port <port>`: server port (default `3000`)
- `-d, --directory <dir>`: shared directory (default `./shared`)
- `-n, --name <name>`: device name shown in discovery
- `-V, --version`: show version
- `-h, --help`: show help

Interactive menu actions:

1. Send files to a discovered device
2. List discovered devices
3. Open web UI URL
4. Show server network information
5. Exit

### `neuroshare`

Send files to another device.

```bash
neuroshare <command> [options]
```

Commands:

### `send`

```bash
neuroshare send <paths...> [options]
```

Options:

- `-d, --device <name>`: target discovered device name
- `-h, --host <host>`: direct host IP or hostname
- `-p, --port <port>`: target port (default `3000`)

Examples:

```bash
# single file
neuroshare send report.pdf

# multiple files
neuroshare send a.jpg b.png c.pdf

# by discovered device name
neuroshare send archive.zip --device "Laptop"

# direct host mode
neuroshare send "New Project.png" --host 192.168.0.103 --port 3000
```

### `devices`

```bash
neuroshare devices [options]
```

Options:

- `-t, --timeout <seconds>`: discovery timeout (default `5`)

## Network and Discovery

NeuroLink uses mDNS/Bonjour service discovery.

Service types:

- `_neurolink._tcp`
- `_nerolink._tcp` (legacy compatibility)

Notes:

- Discovery can fail on guest WiFi, VPN, or isolated hotspot networks.
- If discovery fails, use direct host mode with `--host` and `--port`.

## API Reference

### Node.js API (Port 3000)

Base URL: `http://<host>:3000`

- `GET /` - Web UI
- `GET /api/files` - List all files
- `GET /api/files/grouped` - Files grouped by session
- `GET /api/files/:name` - Download file
- `POST /api/upload` - Upload file (multipart)
- `DELETE /api/files/:name` - Delete file
- `GET /api/download-all` - Download all as ZIP

### Rust Microservice API (Port 3030)

Base URL: `http://<host>:3030`

Chunked Transfer Endpoints:

#### POST /transfer/init
Initialize chunked transfer.

```json
{
  "filename": "large-file.zip",
  "total_size": 1073741824,
  "chunk_size": 1048576
}
```

Response:
```json
{
  "success": true,
  "data": {
    "transfer_id": "trans_1234567890",
    "total_chunks": 1024
  }
}
```

#### POST /transfer/chunk
Upload chunk (multipart form).

Fields:
- `transfer_id` - Transfer ID
- `chunk_index` - Chunk number (0-based)
- `chunk` - Binary data

Response:
```json
{
  "success": true,
  "data": {
    "chunk_hash": "a1b2c3...",
    "received_count": 512,
    "total_chunks": 1024
  }
}
```

#### POST /transfer/complete
Finalize transfer.

```json
{
  "transfer_id": "trans_1234567890"
}
```

#### GET /transfer/:id/status
Check progress.

Response:
```json
{
  "success": true,
  "data": {
    "transfer_id": "trans_1234567890",
    "status": "in_progress",
    "progress": "50%"
  }
}
```

#### GET /health
Health check.

## File Categorization

Grouped session output classifies files into:

1. Photos
2. Videos
3. Files

Classification is extension-based.

## Configuration

Environment variables:

- `NEROLINK_PORT`
- `NEROLINK_DIR`
- `NEROLINK_NAME`

CLI options override environment variables.

## Firewall

Allow incoming TCP on selected server port.

Linux (ufw):

```bash
sudo ufw allow 3000/tcp
```

Linux (iptables):

```bash
sudo iptables -I INPUT -p tcp --dport 3000 -j ACCEPT
```

Windows (PowerShell as Administrator):

```powershell
netsh advfirewall firewall add rule name="NeuroLink" dir=in action=allow protocol=tcp localport=3000
```

### nixos-sys

Enable and configure firewall in your NixOS system config:

```nix
networking.firewall = {
  enable = true;
  # Only open ports you actually need
  allowedTCPPorts = [ 3000 8080 5173 ];  # Uncomment for web dev
  # allowedUDPPorts = [ ];

  # Log dropped packets (useful for debugging)
  logRefusedConnections = true;
};
```

Install globally with npm:

```bash
npm install -g neurolink
```

## Troubleshooting

### No devices found

1. Verify both devices are on the same subnet.
2. Check firewall on both devices.
3. Disable VPN on both devices.
4. Test direct host mode with `neuroshare send ... --host ... --port ...`.

### Cannot access web UI

1. Confirm server is running.
2. Confirm target IP address and port.
3. Check firewall rules.

### Upload fails

1. Verify write permission for shared directory.
2. Check disk space.
3. Check file path and file name.

## Development

### Node.js Components

Build:

```bash
npm run build
```

Watch mode:

```bash
npm run dev
```

### Rust Components

Build debug:

```bash
cd rust-service
cargo build
```

Build release:

```bash
cd rust-service
cargo build --release
```

Run tests:

```bash
cd rust-service
cargo test
```

### Project Structure

```text
neurolink/
├── src/                          # Node.js source
│   ├── cli/
│   │   ├── main.ts              # Interactive CLI
│   │   ├── menu.ts              # Menu handlers
│   │   └── share.ts             # neuroshare command
│   ├── core/
│   │   ├── discovery.ts         # mDNS device discovery
│   │   └── sender.ts            # File transfer logic
│   ├── server/
│   │   └── main.ts              # Hono server & web UI
│   ├── utils/
│   │   └── network.ts           # Network utilities
│   └── types/
│       └── index.ts             # TypeScript types
├── rust-service/                 # Rust microservice
│   ├── src/
│   │   ├── main.rs              # Server entry point
│   │   ├── cli.rs               # neuroshare CLI
│   │   ├── transfer/
│   │   │   └── mod.rs           # Chunked transfer engine
│   │   ├── api/
│   │   │   └── routes.rs        # HTTP API routes
│   │   └── hashing/
│   │       └── mod.rs           # SHA-256 hashing
│   └── Cargo.toml
├── dist/                         # Compiled JavaScript
└── package.json
```

## Security

- Designed for trusted local networks.
- No authentication layer by default.
- Do not expose service to public internet.

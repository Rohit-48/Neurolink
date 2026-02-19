# NeuroLink

NeuroLink is a local network file sharing tool for desktop and mobile devices.
It includes:

- `neurolink`: server process with interactive menu and web UI
- `neuroshare`: client CLI for sending files from terminal

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

## Components

### 1. Node.js Server (`neurolink`)
- Interactive CLI menu
- Web interface (HTML/CSS/JS)
- mDNS device discovery
- Session management
- File metadata

### 2. Rust Microservice (`neurolinkd`)
- Chunked file transfers
- SHA-256 hash verification
- Async I/O with Tokio
- Progress tracking
- Compression ready

### 3. CLI Client (`neuroshare`)
- Send files from terminal
- Progress bars
- Host/port selection
- Batch file upload

## Requirements

- Node.js 18 or later
- npm
- Devices connected to the same local network

## Installation

### Prerequisites

- Node.js 18+ and npm
- Rust toolchain (for Rust microservice)

### Install Rust components

```bash
cd rust-service
cargo build --release

# Copy binaries to PATH
cp target/release/neurolinkd ~/.local/bin/
cp target/release/neuroshare ~/.local/bin/
```

### Global install from npm

```bash
npm install -g neurolink
```

### Local development install

```bash
git clone <repository-url>
cd neurolink

# Install Node.js components
npm install
npm run build
npm link

# Install Rust components
cd rust-service
cargo build --release
```

## Quick Start

### Option 1: Node.js Server (Full Stack)

Start the Node.js server with web UI and discovery:

```bash
neurolink
```

Open web UI: `http://<server-ip>:3000`

### Option 2: Rust Microservice (Performance Mode)

Start the high-performance Rust file transfer service:

```bash
# Terminal 1: Start Rust service
neurolinkd

# Terminal 2: Send files with progress bar
neuroshare send ./large-file.zip --host localhost --port 3030
```

### Complete Stack (Recommended)

For best performance, run both services:

```bash
# Terminal 1: Node.js (Web UI + Discovery)
neurolink --port 3000

# Terminal 2: Rust service (File Transfer)
NEUROLINK_PORT=3030 neurolinkd
```

The Node.js service handles the web UI and device discovery, while the Rust service handles high-performance file transfers via chunked uploads.

### Configuration

**Node.js service:**
- Port: `3000` (default)
- Shared directory: `./shared`
- Device name: system hostname

**Rust service:**
- Port: `3030` (default)
- Storage: `./shared`
- Chunk size: 1MB (configurable)

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

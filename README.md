# NeuroLink

NeuroLink is a local network file sharing tool for desktop and mobile devices.
It includes:

- `neurolink`: server process with interactive menu and web UI
- `neuroshare`: client CLI for sending files from terminal

## Features

- Local file transfer over HTTP
- Automatic device discovery on local network
- Direct host mode when discovery is unavailable
- Browser-based upload and download UI
- Session-based grouping of uploaded files

## Requirements

- Node.js 18 or later
- npm
- Devices connected to the same local network

## Installation

### Global install from npm

```bash
npm install -g neurolink
```

### Local development install

```bash
git clone <repository-url>
cd neurolink
npm install
npm run build
npm link
```

## Quick Start

### 1. Start server

```bash
neurolink
```

Default settings:

- Port: `3000`
- Shared directory: `./shared`
- Device name: system hostname

### 2. Open web UI

Open in browser:

```text
http://<server-ip>:3000
```

### 3. Send files from terminal

```bash
neuroshare send ./file.pdf
```

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

## Web API

Base URL:

```text
http://<server-ip>:<port>
```

Endpoints:

- `GET /`: web UI
- `GET /api/files`: list files
- `GET /api/files/grouped`: list files grouped by upload session
- `GET /api/files/:name`: download one file
- `POST /api/upload`: upload one file (multipart form field: `file`)
- `DELETE /api/files/:name`: delete one file
- `GET /api/download-all`: download all files as zip
- `GET /api/download-session/:timestamp`: download one upload session as zip

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

Build:

```bash
npm run build
```

Watch mode:

```bash
npm run dev
```

Project layout:

```text
neurolink/
  src/
    cli.ts
    share.ts
    server.ts
    discovery.ts
    sender.ts
    network.ts
  dist/
  package.json
```

## Security

- Designed for trusted local networks.
- No authentication layer by default.
- Do not expose service to public internet.

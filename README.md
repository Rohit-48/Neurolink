# NeroLink

Local network file sharing with device discovery, web interface, and CLI tools.

## Overview

NeroLink enables peer-to-peer file sharing across devices on the same local network. It provides both a web-based interface accessible via browsers and command-line tools for programmatic file transfers.

## Installation

### Global Installation

```bash
npm install -g nerolink
```

### Local Installation

```bash
git clone <repository-url>
cd nerolink
npm install
npm run build
npm link
```

## Quick Start

### Start the Server

```bash
nerolink
```

This starts the file sharing server on port 3000 and opens an interactive menu.

### Send Files

```bash
neroshare send /path/to/file.pdf
```

## Architecture

NeroLink operates on a client-server model where:

- **Server (nerolink)**: Receives files, hosts web interface, advertises presence on network
- **Client (neroshare)**: Discovers servers and sends files

### Network Requirements

- All devices must be on the same local network (WiFi/LAN)
- Firewall must allow connections on the configured port (default: 3000)
- mDNS/Bonjour service discovery for automatic device detection

## Command Line Interface

### nerolink

Start the file sharing server with interactive mode.

```bash
nerolink [options]
```

**Options:**

- `-p, --port <port>`: Port to run server on (default: 3000)
- `-d, --directory <dir>`: Directory to share files from (default: ./shared)
- `-n, --name <name>`: Device name for discovery (default: system hostname)
- `-V, --version`: Display version number
- `-h, --help`: Display help

**Interactive Commands:**

When running without arguments, nerolink presents an interactive menu:

1. **Send files to a device**: Select discovered device and send files
2. **List available devices**: Show all nerolink instances on network
3. **Open Web UI**: Launch browser to web interface
4. **Show server info**: Display network URLs and configuration
5. **Exit**: Stop server and exit

### neroshare

Send files to devices on the network.

```bash
neroshare <command> [options]
```

#### Commands

**send**

Send files or directories to a target device.

```bash
neroshare send <paths...> [options]
```

**Options:**

- `-d, --device <name>`: Target device name (auto-discover if not specified)
- `-h, --host <host>`: Target host IP address (bypasses discovery)
- `-p, --port <port>`: Target port (default: 3000)

**Examples:**

```bash
# Send single file
neroshare send document.pdf

# Send multiple files
neroshare send photo1.jpg photo2.png document.pdf

# Send to specific device by name
neroshare send file.zip --device "My-Laptop"

# Send to specific IP address
neroshare send video.mp4 --host 192.168.1.100 --port 3000
```

**devices**

List available devices on the network.

```bash
neroshare devices [options]
```

**Options:**

- `-t, --timeout <seconds>`: Discovery timeout in seconds (default: 5)

## Web Interface

Access the web interface by navigating to:

```
http://<server-ip>:<port>
```

Example:
```
http://192.168.1.100:3000
```

### Features

- **Drag and Drop Upload**: Drag files directly into browser window
- **File Organization**: Files grouped by upload sessions (5-minute window)
- **Category Sorting**: Photos, videos, and other files sorted separately
- **Session Management**: Download all files from specific upload sessions
- **Mobile Responsive**: Optimized for phones and tablets

### Supported File Types

The web interface supports all file types including:

- **Images**: jpg, jpeg, png, gif, webp, svg, bmp, tiff, raw, psd
- **Videos**: mp4, mov, avi, mkv, wmv, flv, webm, m4v, mpg, mpeg
- **Audio**: mp3, wav, flac, m4a, aac, ogg, wma
- **Documents**: pdf, doc, docx, txt, md, xls, xlsx, ppt, pptx
- **Archives**: zip, rar, 7z, tar, gz
- **Code**: js, ts, py, html, css, json, xml, and more

## REST API

### Endpoints

#### GET /
Returns the web interface HTML.

#### GET /api/files
List all shared files.

**Response:**
```json
{
  "files": [
    {
      "name": "example.pdf",
      "size": 1048576,
      "modified": "2026-02-13T10:00:00.000Z",
      "type": "application/pdf"
    }
  ]
}
```

#### GET /api/files/grouped
Get files organized by upload sessions.

**Response:**
```json
{
  "sessions": [
    {
      "timestamp": "2026-02-13T10:00:00.000Z",
      "files": [
        {
          "name": "photo.jpg",
          "size": 2048000,
          "modified": "2026-02-13T10:00:00.000Z",
          "type": "image/jpeg"
        }
      ]
    }
  ]
}
```

#### GET /api/files/:name
Download a specific file.

#### POST /api/upload
Upload a file.

**Request:** Multipart form data with file field

**Response:**
```json
{
  "success": true,
  "message": "File uploaded successfully",
  "file": "uploaded-file.pdf"
}
```

#### DELETE /api/files/:name
Delete a specific file.

#### GET /api/download-all
Download all files as a ZIP archive.

#### GET /api/download-session/:timestamp
Download all files from a specific session as ZIP.

## Device Discovery

NeroLink uses mDNS/Bonjour (zero-configuration networking) to discover devices:

- Service Type: `_nerolink._tcp`
- Automatic discovery within local network
- No manual IP configuration required
- Works across platforms (Linux, macOS, Windows)

## Session Grouping

Files uploaded within a 5-minute window are grouped into a session. This allows:

- Logical organization of batch uploads
- Separate download per session
- Time-based file management

Sessions are displayed in reverse chronological order (newest first).

## File Categories

Within each session, files are sorted into categories:

1. **Photos**: Image files (jpg, png, gif, etc.)
2. **Videos**: Video files (mp4, mov, avi, etc.)
3. **Files**: All other file types

## Configuration

### Environment Variables

- `NEROLINK_PORT`: Default port (overridden by --port)
- `NEROLINK_DIR`: Default directory (overridden by --directory)
- `NEROLINK_NAME`: Default device name (overridden by --name)

### Firewall Configuration

Allow incoming connections on the chosen port:

**Linux (UFW):**
```bash
sudo ufw allow 3000/tcp
```

**Linux (iptables):**
```bash
sudo iptables -I INPUT -p tcp --dport 3000 -j ACCEPT
```

**Windows (PowerShell Admin):**
```powershell
netsh advfirewall firewall add rule name="NeroLink" dir=in action=allow protocol=tcp localport=3000
```

## Troubleshooting

### No devices found

1. Verify both devices are on the same network
2. Check firewall settings
3. Ensure nerolink is running on target device
4. Try using IP address directly: `neroshare send file.pdf --host 192.168.x.x`

### Cannot connect to web interface

1. Verify server is running: `nerolink`
2. Check firewall allows port 3000
3. Try different port: `nerolink --port 8080`
4. Verify IP address is correct

### File upload fails

1. Check disk space on server
2. Verify write permissions on shared directory
3. Check file size limits (no limit by default)

## Development

### Build

```bash
npm run build
```

### Development Mode

```bash
npm run dev
```

### Project Structure

```
nerolink/
├── src/
│   ├── cli.ts          # Interactive CLI
│   ├── share.ts        # Send CLI
│   ├── server.ts       # Hono server and web UI
│   ├── discovery.ts    # mDNS device discovery
│   ├── sender.ts       # File transfer logic
│   └── network.ts      # Network utilities
├── dist/               # Compiled JavaScript
└── package.json
```

### Dependencies

**Runtime:**
- hono: Web framework
- @hono/node-server: Node.js server adapter
- archiver: ZIP file creation
- bonjour: mDNS service discovery
- inquirer: Interactive CLI prompts
- commander: CLI argument parsing
- qrcode: QR code generation
- mime-types: MIME type detection

**Development:**
- TypeScript: Type checking and compilation
- tsx: TypeScript execution for development

## Security Considerations

- NeroLink is designed for trusted local networks only
- No authentication mechanism (intentionally simple)
- Files are shared as-is without encryption in transit
- Do not expose to public internet
- Use firewall rules to restrict access if needed

## Platform Support

- **Linux**: Full support (tested on Ubuntu, Fedora, Arch)
- **macOS**: Full support (Bonjour included)
- **Windows**: Full support (Bonjour service required)
- **Mobile**: Web interface only (no CLI support)

## Browser Compatibility

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Mobile Safari (iOS 14+)
- Chrome Mobile (Android 10+)

## License

MIT

## Contributing

Contributions are welcome. Please ensure:

1. Code follows existing style
2. All tests pass
3. Documentation is updated
4. Commit messages are descriptive

## Changelog

### 2.0.0

- Initial release
- Interactive CLI mode
- Device discovery via mDNS
- Web interface with session grouping
- File categorization (photos, videos, files)
- Session-based download
- Mobile responsive design
- Dark theme UI

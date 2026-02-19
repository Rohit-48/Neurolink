# NeuroLink v1.0.0 Release Notes

**Release Date**: February 19, 2026  
**Codename**: Foundation  
**Status**: Stable ✅

---

## Overview

NeuroLink v1.0.0 is the first stable release of our local network file sharing tool. Built with Node.js and TypeScript, it provides a complete solution for sharing files across devices on the same network.

## What's New

### Core Features

#### 1. Web-Based File Sharing
- Modern, responsive web interface
- Drag-and-drop file upload
- Mobile-friendly design
- Works on any device with a browser
- No app installation required on clients

#### 2. Session-Based Organization
- Files grouped by upload time (5-minute window)
- Visual separation between upload batches
- Time-based file management
- Easy identification of related files

#### 3. Automatic File Categorization
Files automatically sorted into categories:
- **Photos**: jpg, png, gif, webp, svg, bmp, tiff, raw, psd
- **Videos**: mp4, mov, avi, mkv, wmv, flv, webm
- **Files**: Everything else (docs, archives, code, etc.)

#### 4. Device Discovery
- Automatic device detection via mDNS/Bonjour
- No manual IP configuration
- Real-time device list updates
- Works across platforms (Linux, macOS, Windows)

#### 5. Interactive CLI
Beautiful terminal interface with:
- Menu-driven navigation
- Device selection
- File sending with discovery
- Server information display
- One-command web UI opening

#### 6. Batch Operations
- Download individual files
- Download all files as ZIP
- Download entire sessions as ZIP
- Multi-file upload support

## Installation

### npm (Global)
```bash
npm install -g neurolink
```

### From Source
```bash
git clone <repository>
cd neurolink
npm install
npm run build
npm link
```

## Quick Start

```bash
# Start server
neurolink

# Access web UI
# Open browser to http://localhost:3000

# Send files from terminal
neuroshare send file.pdf
```

## System Requirements

- **Node.js**: 18.x or higher
- **npm**: 8.x or higher
- **Network**: All devices on same LAN
- **Firewall**: Port 3000 open (or configured)

## Commands

### neurolink
Start the server with interactive menu.

```bash
neurolink [options]

Options:
  -p, --port <port>      Server port (default: 3000)
  -d, --directory <dir>  Shared directory (default: ./shared)
  -n, --name <name>      Device name (default: hostname)
  -v, --version          Show version
  -h, --help             Show help
```

### neuroshare
Send files to devices.

```bash
# Send to discovered device
neuroshare send file.pdf

# Send to specific host
neuroshare send file.pdf --host 192.168.1.100 --port 3000

# Send multiple files
neuroshare send a.jpg b.png c.pdf

# List devices
neuroshare devices --timeout 10
```

## API Reference

### Base URL
```
http://<host>:<port>/
```

### Endpoints

#### Web Interface
- `GET /` - Main web UI

#### File Operations
- `GET /api/files` - List all files with metadata
- `GET /api/files/grouped` - Files grouped by session
- `GET /api/files/:name` - Download specific file
- `POST /api/upload` - Upload file (multipart/form-data)
- `DELETE /api/files/:name` - Delete file

#### Batch Operations
- `GET /api/download-all` - Download all files as ZIP
- `GET /api/download-session/:timestamp` - Download session as ZIP

## Configuration

### Environment Variables
```bash
NEUROLINK_PORT=3000        # Server port
NEUROLINK_DIR=./shared     # Storage directory
NEUROLINK_NAME=mydevice    # Device name
```

### CLI Options Override
Command-line options take precedence over environment variables.

## Security Notes

⚠️ **Important**:
- Designed for trusted local networks only
- No authentication by default
- Files shared as-is without encryption
- Do not expose to public internet
- Use firewall rules to restrict access

## Performance

### Benchmarks
- Upload speed: Network limited (typically 50-100 MB/s on Gigabit)
- Concurrent uploads: Limited by Node.js single-thread
- Memory usage: ~100-200 MB typical
- Startup time: < 2 seconds

### Optimizations
- Static file serving
- Efficient file streaming
- Minimal dependencies
- Clean architecture

## Known Issues

### v1.0.0 Limitations
1. **Single-threaded**: File transfers block on large files
2. **No deduplication**: Duplicate files stored multiple times
3. **No compression**: Files transferred uncompressed
4. **No resume**: Interrupted uploads restart from beginning
5. **Discovery issues**: May fail on guest networks or VPNs

### Workarounds
- Use `--host` flag for direct IP when discovery fails
- Split very large files manually
- Clear shared directory periodically to save space

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+
- Mobile Safari (iOS 14+)
- Chrome Mobile (Android 10+)

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux | ✅ Full | Tested on Ubuntu, Fedora, Arch |
| macOS | ✅ Full | Bonjour included |
| Windows | ✅ Full | Bonjour service required |
| iOS | ⚠️ Web only | Via browser |
| Android | ⚠️ Web only | Via browser |

## Troubleshooting

### Common Issues

**No devices found**
```bash
# Use direct IP instead
neuroshare send file.pdf --host 192.168.1.100
```

**Port already in use**
```bash
# Use different port
neurolink --port 8080
```

**Cannot access web UI**
```bash
# Check firewall
sudo ufw allow 3000/tcp
# or
sudo iptables -I INPUT -p tcp --dport 3000 -j ACCEPT
```

**Upload fails**
- Check disk space
- Verify write permissions
- Check file path

## Development

### Build
```bash
npm run build
```

### Watch Mode
```bash
npm run dev
```

### Project Structure
```
src/
├── cli/
│   ├── main.ts       # CLI entry
│   ├── menu.ts       # Interactive menu
│   └── share.ts      # Send command
├── core/
│   ├── discovery.ts  # mDNS discovery
│   └── sender.ts     # File transfer
├── server/
│   └── main.ts       # Web server
├── utils/
│   └── network.ts    # Network utilities
└── types/
    └── index.ts      # TypeScript types
```

## Migration from Beta

If upgrading from beta versions:
1. Backup your `./shared` directory
2. Uninstall old version: `npm uninstall -g neurolink`
3. Install v1.0.0: `npm install -g neurolink`
4. Restore files to new shared directory

## Future Roadmap

See `ROADMAP.md` for planned features in v2.0.0:
- Rust microservice for performance
- File deduplication
- Desktop app (Tauri)
- Real-time sync
- P2P mesh networking

## Credits

- Built with Node.js, TypeScript, Hono
- mDNS via Bonjour
- Inspired by AirDrop and Snapdrop

## License

MIT License - See LICENSE file

## Support

- GitHub Issues: <repository>/issues
- Documentation: README.md, ROADMAP.md
- Community: Discussions on GitHub

---

**Thank you for using NeuroLink!** 🚀

For feature requests or bug reports, please open an issue on GitHub.

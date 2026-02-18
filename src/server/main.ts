import { Hono } from 'hono';
import { serve } from '@hono/node-server';
import { serveStatic } from '@hono/node-server/serve-static';
import { readFile, readdir, stat, unlink, writeFile, mkdir } from 'fs/promises';
import { createReadStream, existsSync, createWriteStream } from 'fs';
import { join, basename, extname } from 'path';
import { lookup } from 'mime-types';
import { styleText } from 'util';
import { getNetworkInfo, troubleshootConnection, generateQRCode } from '../utils/network.js';
import os from 'os';
import archiver from 'archiver';

interface FileInfo {
  name: string;
  size: number;
  modified: string;
  type: string;
}

export async function createServer(uploadDir: string, port: number) {
  const app = new Hono();

  // Ensure upload directory exists
  if (!existsSync(uploadDir)) {
    await mkdir(uploadDir, { recursive: true });
  }

  // API Routes
  app.get('/api/files', async (c) => {
    try {
      const files = await readdir(uploadDir);
      const fileInfos: FileInfo[] = [];

      for (const file of files) {
        const filePath = join(uploadDir, file);
        const stats = await stat(filePath);
        
        if (stats.isFile()) {
          fileInfos.push({
            name: file,
            size: stats.size,
            modified: stats.mtime.toISOString(),
            type: lookup(file) || 'application/octet-stream'
          });
        }
      }

      return c.json({ files: fileInfos });
    } catch (error) {
      return c.json({ error: 'Failed to list files' }, 500);
    }
  });

  // Get files grouped by upload sessions and sorted by category
  // IMPORTANT: This must come BEFORE /api/files/:name to avoid being matched as a filename
  app.get('/api/files/grouped', async (c) => {
    try {
      const files = await readdir(uploadDir);
      const fileInfos: FileInfo[] = [];

      for (const file of files) {
        const filePath = join(uploadDir, file);
        const stats = await stat(filePath);
        
        if (stats.isFile()) {
          fileInfos.push({
            name: file,
            size: stats.size,
            modified: stats.mtime.toISOString(),
            type: lookup(file) || 'application/octet-stream'
          });
        }
      }

      // Sort by modified time (newest first)
      fileInfos.sort((a, b) => new Date(b.modified).getTime() - new Date(a.modified).getTime());

      // Group files into sessions (files uploaded within 5 minutes of each other)
      const SESSION_THRESHOLD = 5 * 60 * 1000; // 5 minutes
      const sessions: { timestamp: string; files: FileInfo[] }[] = [];
      
      for (const file of fileInfos) {
        const fileTime = new Date(file.modified).getTime();
        
        // Check if this file belongs to an existing session
        let addedToSession = false;
        for (const session of sessions) {
          const sessionTime = new Date(session.timestamp).getTime();
          if (Math.abs(fileTime - sessionTime) <= SESSION_THRESHOLD) {
            session.files.push(file);
            addedToSession = true;
            break;
          }
        }
        
        // If not added to any session, create new session
        if (!addedToSession) {
          sessions.push({
            timestamp: file.modified,
            files: [file]
          });
        }
      }

      // Sort files within each session by category
      const getCategory = (filename: string) => {
        const ext = filename.split('.').pop()?.toLowerCase() || '';
        const imageExts = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp', 'tiff', 'tif', 'ico', 'raw', 'psd', 'ai', 'eps'];
        const videoExts = ['mp4', 'mov', 'avi', 'mkv', 'wmv', 'flv', 'webm', 'm4v', 'mpg', 'mpeg', '3gp', 'ogv'];
        
        if (imageExts.includes(ext)) return 'photos';
        if (videoExts.includes(ext)) return 'videos';
        return 'files';
      };

      const sortedSessions = sessions.map(session => {
        const categorized = {
          photos: session.files.filter(f => getCategory(f.name) === 'photos'),
          videos: session.files.filter(f => getCategory(f.name) === 'videos'),
          files: session.files.filter(f => getCategory(f.name) === 'files')
        };
        
        return {
          timestamp: session.timestamp,
          files: [...categorized.photos, ...categorized.videos, ...categorized.files]
        };
      });

      return c.json({ sessions: sortedSessions });
    } catch (error) {
      return c.json({ error: 'Failed to list files' }, 500);
    }
  });

  app.get('/api/files/:name', async (c) => {
    const name = c.req.param('name');
    const filePath = join(uploadDir, decodeURIComponent(name));

    try {
      const fileStat = await stat(filePath);
      
      if (!fileStat.isFile()) {
        return c.json({ error: 'Not found' }, 404);
      }

      const mimeType = lookup(filePath) || 'application/octet-stream';
      const stream = createReadStream(filePath);

      c.header('Content-Type', mimeType);
      c.header('Content-Length', fileStat.size.toString());
      c.header('Content-Disposition', `attachment; filename="${basename(filePath)}"`);

      return new Response(stream as any);
    } catch (error) {
      return c.json({ error: 'File not found' }, 404);
    }
  });

  app.post('/api/upload', async (c) => {
    try {
      const formData = await c.req.formData();
      const file = formData.get('file') as File;

      if (!file) {
        return c.json({ error: 'No file provided' }, 400);
      }

      const buffer = Buffer.from(await file.arrayBuffer());
      const filePath = join(uploadDir, file.name);
      
      await writeFile(filePath, buffer);

      return c.json({ 
        success: true, 
        message: 'File uploaded successfully',
        file: file.name 
      });
    } catch (error) {
      return c.json({ error: 'Upload failed' }, 500);
    }
  });

  app.delete('/api/files/:name', async (c) => {
    const name = c.req.param('name');
    const filePath = join(uploadDir, decodeURIComponent(name));

    try {
      await unlink(filePath);
      return c.json({ success: true, message: 'File deleted' });
    } catch (error) {
      return c.json({ error: 'Failed to delete file' }, 500);
    }
  });

  // Download all files as ZIP
  app.get('/api/download-all', async (c) => {
    try {
      const files = await readdir(uploadDir);
      const validFiles: string[] = [];
      
      for (const file of files) {
        const filePath = join(uploadDir, file);
        const stats = await stat(filePath);
        if (stats.isFile()) {
          validFiles.push(file);
        }
      }

      if (validFiles.length === 0) {
        return c.json({ error: 'No files to download' }, 404);
      }

      // Create ZIP archive
      const archive = archiver('zip', { zlib: { level: 9 } });
      const chunks: Buffer[] = [];

      archive.on('data', (chunk) => chunks.push(chunk));
      archive.on('error', (err) => {
        console.error('Archive error:', err);
      });

      // Add files to archive
      for (const file of validFiles) {
        const filePath = join(uploadDir, file);
        archive.file(filePath, { name: file });
      }

      await archive.finalize();

      const buffer = Buffer.concat(chunks);
      
      c.header('Content-Type', 'application/zip');
      c.header('Content-Disposition', 'attachment; filename="shared-files.zip"');
      c.header('Content-Length', buffer.length.toString());

      return new Response(buffer);
    } catch (error) {
      console.error('Download all error:', error);
      return c.json({ error: 'Failed to create archive' }, 500);
    }
  });

  // Download files from a specific session
  app.get('/api/download-session/:timestamp', async (c) => {
    try {
      const timestamp = c.req.param('timestamp');
      const sessionTime = new Date(decodeURIComponent(timestamp)).getTime();
      const SESSION_THRESHOLD = 5 * 60 * 1000; // 5 minutes
      
      const files = await readdir(uploadDir);
      const sessionFiles: string[] = [];
      
      for (const file of files) {
        const filePath = join(uploadDir, file);
        const stats = await stat(filePath);
        if (stats.isFile()) {
          const fileTime = stats.mtime.getTime();
          if (Math.abs(fileTime - sessionTime) <= SESSION_THRESHOLD) {
            sessionFiles.push(file);
          }
        }
      }

      if (sessionFiles.length === 0) {
        return c.json({ error: 'No files found for this session' }, 404);
      }

      // Create ZIP archive
      const archive = archiver('zip', { zlib: { level: 9 } });
      const chunks: Buffer[] = [];

      archive.on('data', (chunk) => chunks.push(chunk));
      archive.on('error', (err) => {
        console.error('Archive error:', err);
      });

      // Add files to archive
      for (const file of sessionFiles) {
        const filePath = join(uploadDir, file);
        archive.file(filePath, { name: file });
      }

      await archive.finalize();

      const buffer = Buffer.concat(chunks);
      
      c.header('Content-Type', 'application/zip');
      c.header('Content-Disposition', `attachment; filename="upload-session-${timestamp}.zip"`);
      c.header('Content-Length', buffer.length.toString());

      return new Response(buffer);
    } catch (error) {
      console.error('Download session error:', error);
      return c.json({ error: 'Failed to create archive' }, 500);
    }
  });

  // UI Route - Serve the web interface
  app.get('/', (c) => {
    return c.html(getUIHtml());
  });

  return { app, port };
}

export async function startServer(uploadDir: string, port: number) {
  const { app } = await createServer(uploadDir, port);
  
  serve({
    fetch: app.fetch,
    port
  });

  console.log('\n' + styleText(['bold', 'green'], '📁 Local File Share Server Started'));
  console.log('\n' + styleText('gray', 'Upload directory: ') + styleText('cyan', uploadDir));

  // Get and display network info
  const addresses = getNetworkInfo(port);

  if (addresses.length === 0) {
    console.log('\n⚠️  No network interfaces found!');
    console.log('   Connect to WiFi and try again.');
  } else {
    console.log('\n' + styleText(['bold'], '📱 Access from your phone:'));
    
    // Find WiFi address for QR code
    const wifiAddress = addresses.find(a => !a.name.includes('tailscale') && !a.name.includes('tun'));
    
    if (wifiAddress) {
      console.log('\n' + styleText(['bold', 'cyan'], 'Scan QR code with your phone:'));
      const qrCode = await generateQRCode(wifiAddress.url);
      console.log(qrCode);
      console.log(styleText('gray', 'URL: ') + styleText(['bold', 'yellow'], wifiAddress.url) + '\n');
    }
    
    addresses.forEach(({ name, url }) => {
      console.log('  ' + styleText('blue', '➜ ') + styleText('gray', name + ': ') + styleText('yellow', url));
    });
  }

  // Show troubleshooting
  troubleshootConnection(port, addresses);
  
  console.log('\n' + styleText('gray', 'Press Ctrl+C to stop'));
  console.log();
}

function getUIHtml(): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
  <meta name="theme-color" content="#000000">
  <title>File Share</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    
    :root {
      --bg: #000000;
      --surface: #111111;
      --surface-hover: #1a1a1a;
      --border: #333333;
      --text: #ffffff;
      --text-secondary: #888888;
      --accent: #ffffff;
    }
    
    html {
      -webkit-text-size-adjust: 100%;
      touch-action: manipulation;
    }
    
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      padding: 1rem;
      line-height: 1.5;
      -webkit-font-smoothing: antialiased;
      -moz-osx-font-smoothing: grayscale;
    }
    
    .container {
      max-width: 800px;
      margin: 0 auto;
    }
    
    header {
      text-align: center;
      padding: 2rem 0 1.5rem;
      border-bottom: 1px solid var(--border);
      margin-bottom: 1.5rem;
    }
    
    h1 {
      font-size: 1.5rem;
      font-weight: 600;
      letter-spacing: -0.02em;
      text-transform: uppercase;
    }
    
    .subtitle {
      color: var(--text-secondary);
      font-size: 0.875rem;
      margin-top: 0.25rem;
    }
    
    .upload-section {
      background: var(--surface);
      border: 2px dashed var(--border);
      border-radius: 12px;
      padding: 2rem 1.5rem;
      text-align: center;
      cursor: pointer;
      transition: all 0.2s ease;
      margin-bottom: 1.5rem;
    }
    
    .upload-section:hover, .upload-section.dragover {
      border-color: var(--accent);
      background: var(--surface-hover);
    }
    
    .upload-icon {
      width: 48px;
      height: 48px;
      margin: 0 auto 1rem;
      stroke: var(--text);
    }
    
    .upload-text {
      font-size: 1rem;
      font-weight: 500;
    }
    
    .upload-hint {
      color: var(--text-secondary);
      font-size: 0.875rem;
      margin-top: 0.5rem;
    }
    
    .file-input { display: none; }
    
    .files-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1rem;
      padding: 0 0.5rem;
    }
    
    .section-title {
      font-size: 0.75rem;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.1em;
      color: var(--text-secondary);
    }
    
    .btn-download-all {
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.5rem 1rem;
      font-size: 0.8125rem;
    }
    
    .btn-download-all svg {
      flex-shrink: 0;
    }
    
    .file-list {
      background: var(--surface);
      border-radius: 12px;
      overflow: hidden;
    }
    
    .file-item {
      display: flex;
      align-items: center;
      padding: 1rem;
      border-bottom: 1px solid var(--border);
      transition: background 0.15s;
    }
    
    .file-item:last-child {
      border-bottom: none;
    }
    
    .file-item:active {
      background: var(--surface-hover);
    }
    
    .file-icon {
      width: 40px;
      height: 40px;
      border: 1px solid var(--border);
      border-radius: 8px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 1.25rem;
      flex-shrink: 0;
      margin-right: 0.875rem;
    }
    
    .file-info {
      flex: 1;
      min-width: 0;
    }
    
    .file-name {
      font-size: 0.9375rem;
      font-weight: 500;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
    
    .file-meta {
      color: var(--text-secondary);
      font-size: 0.8125rem;
      margin-top: 0.125rem;
    }
    
    .file-actions {
      display: flex;
      gap: 0.5rem;
      margin-left: 0.75rem;
    }
    
    .btn {
      padding: 0.5rem 0.875rem;
      border: 1px solid var(--border);
      border-radius: 6px;
      background: transparent;
      color: var(--text);
      font-size: 0.8125rem;
      font-weight: 500;
      cursor: pointer;
      transition: all 0.15s;
      text-decoration: none;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 36px;
      white-space: nowrap;
    }
    
    .btn:hover, .btn:active {
      border-color: var(--accent);
      background: var(--surface-hover);
    }
    
    .btn-icon {
      padding: 0.5rem;
      min-width: 36px;
    }
    
    .progress-bar {
      height: 2px;
      background: var(--border);
      margin-top: 1rem;
      overflow: hidden;
      display: none;
    }
    
    .progress-bar.active { display: block; }
    
    .progress-fill {
      height: 100%;
      background: var(--accent);
      width: 0%;
      transition: width 0.3s ease;
    }
    
    .empty-state {
      text-align: center;
      padding: 3rem 1.5rem;
      color: var(--text-secondary);
    }
    
    .empty-icon {
      width: 48px;
      height: 48px;
      margin: 0 auto 1rem;
      stroke: var(--text-secondary);
      opacity: 0.5;
    }
    
    .toast {
      position: fixed;
      bottom: 1.5rem;
      left: 50%;
      transform: translateX(-50%) translateY(100px);
      background: var(--surface);
      color: var(--text);
      padding: 0.875rem 1.5rem;
      border-radius: 8px;
      border: 1px solid var(--border);
      font-size: 0.9375rem;
      font-weight: 500;
      opacity: 0;
      transition: all 0.3s ease;
      z-index: 1000;
      white-space: nowrap;
    }
    
    .toast.show {
      transform: translateX(-50%) translateY(0);
      opacity: 1;
    }
    
    /* Mobile optimizations */
    @media (max-width: 480px) {
      body {
        padding: 0.75rem;
      }
      
      header {
        padding: 1.5rem 0 1rem;
      }
      
      h1 {
        font-size: 1.25rem;
      }
      
      .upload-section {
        padding: 1.5rem 1rem;
      }
      
      .upload-icon {
        width: 40px;
        height: 40px;
      }
      
      .file-item {
        padding: 0.875rem;
      }
      
      .file-icon {
        width: 36px;
        height: 36px;
        font-size: 1rem;
      }
      
      .file-actions {
        gap: 0.375rem;
      }
      
      .btn {
        padding: 0.4375rem 0.75rem;
        font-size: 0.75rem;
        min-height: 32px;
      }
    }
    
    /* Session grouping */
    .session {
      margin-bottom: 1.5rem;
    }
    
    .session-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 0.75rem 1rem;
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 8px 8px 0 0;
      border-bottom: none;
    }
    
    .session-time {
      font-size: 0.8125rem;
      font-weight: 600;
      color: var(--text);
    }
    
    .session-count {
      font-size: 0.75rem;
      color: var(--text-secondary);
    }
    
    .session-files {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 0 0 8px 8px;
      overflow: hidden;
    }
    
    .category-divider {
      padding: 0.5rem 1rem;
      background: var(--surface-hover);
      border-bottom: 1px solid var(--border);
      font-size: 0.75rem;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--text-secondary);
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }
    
    .category-divider:first-child {
      border-radius: 0;
    }
    
    .btn-sm {
      padding: 0.375rem 0.625rem;
      font-size: 0.75rem;
      min-height: 28px;
    }
    
    /* Prevent text selection on mobile */
    .upload-section, .btn {
      -webkit-user-select: none;
      user-select: none;
      -webkit-touch-callout: none;
    }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>File Share</h1>
      <div class="subtitle">Local Network Transfer</div>
    </header>
    
    <div class="upload-section" id="uploadArea">
      <svg class="upload-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
        <polyline points="17 8 12 3 7 8"></polyline>
        <line x1="12" y1="3" x2="12" y2="15"></line>
      </svg>
      <div class="upload-text">Tap to upload files</div>
      <div class="upload-hint">or drag and drop</div>
      <input type="file" class="file-input" id="fileInput" multiple>
      <div class="progress-bar" id="progressBar">
        <div class="progress-fill" id="progressFill"></div>
      </div>
    </div>

    <div class="files-header">
      <div class="section-title">Shared Files</div>
      <button class="btn btn-download-all" id="downloadAllBtn" style="display: none;">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
          <polyline points="7 10 12 15 17 10"></polyline>
          <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
        Download All
      </button>
    </div>
    <div class="file-list" id="fileList">
      <div class="empty-state">
        <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
          <polyline points="13 2 13 9 20 9"></polyline>
        </svg>
        <div>No files yet</div>
      </div>
    </div>
  </div>

  <div class="toast" id="toast"></div>

  <script>
    const uploadArea = document.getElementById('uploadArea');
    const fileInput = document.getElementById('fileInput');
    const fileList = document.getElementById('fileList');
    const progressBar = document.getElementById('progressBar');
    const progressFill = document.getElementById('progressFill');
    const toast = document.getElementById('toast');
    const downloadAllBtn = document.getElementById('downloadAllBtn');

    const fileIcons = {
      'image': '🖼️',
      'video': '🎬',
      'audio': '🎵',
      'pdf': '📄',
      'archive': '📦',
      'code': '💻',
      'spreadsheet': '📊',
      'presentation': '📽️',
      'executable': '⚙️',
      'font': '🔤',
      'vector': '🎨',
      'cad': '🏗️',
      '3d': '🧊',
      'disk': '💿',
      'database': '🗄️',
      'ebook': '📚',
      'cert': '🔐',
      'config': '⚙️',
      'doc': '📝',
      'default': '📎'
    };

    function getFileIcon(filename) {
      const ext = filename.split('.').pop().toLowerCase();
      const typeMap = {
        // Images
        'jpg': 'image', 'jpeg': 'image', 'png': 'image', 'gif': 'image', 'webp': 'image',
        'svg': 'vector', 'bmp': 'image', 'tiff': 'image', 'tif': 'image', 'ico': 'image',
        'raw': 'image', 'psd': 'image', 'ai': 'vector', 'eps': 'vector',
        // Videos
        'mp4': 'video', 'mov': 'video', 'avi': 'video', 'mkv': 'video', 'wmv': 'video',
        'flv': 'video', 'webm': 'video', 'm4v': 'video', 'mpg': 'video', 'mpeg': 'video',
        '3gp': 'video', 'ogv': 'video',
        // Audio
        'mp3': 'audio', 'wav': 'audio', 'flac': 'audio', 'm4a': 'audio', 'aac': 'audio',
        'ogg': 'audio', 'wma': 'audio', 'opus': 'audio', 'aiff': 'audio',
        // Documents
        'pdf': 'pdf',
        'doc': 'doc', 'docx': 'doc', 'odt': 'doc', 'rtf': 'doc',
        'txt': 'doc', 'md': 'doc', 'tex': 'doc',
        // Spreadsheets
        'xls': 'spreadsheet', 'xlsx': 'spreadsheet', 'ods': 'spreadsheet', 'csv': 'spreadsheet',
        'tsv': 'spreadsheet', 'numbers': 'spreadsheet',
        // Presentations
        'ppt': 'presentation', 'pptx': 'presentation', 'odp': 'presentation', 'key': 'presentation',
        // Archives
        'zip': 'archive', 'rar': 'archive', '7z': 'archive', 'tar': 'archive', 'gz': 'archive',
        'bz2': 'archive', 'xz': 'archive', 'lz': 'archive', 'zst': 'archive', 'lz4': 'archive',
        // Code
        'js': 'code', 'ts': 'code', 'jsx': 'code', 'tsx': 'code', 'py': 'code', 'rb': 'code',
        'php': 'code', 'java': 'code', 'cpp': 'code', 'c': 'code', 'h': 'code', 'hpp': 'code',
        'go': 'code', 'rs': 'code', 'swift': 'code', 'kt': 'code', 'scala': 'code',
        'html': 'code', 'htm': 'code', 'css': 'code', 'scss': 'code', 'sass': 'code', 'less': 'code',
        'json': 'code', 'xml': 'code', 'yaml': 'code', 'yml': 'code', 'toml': 'code',
        'sql': 'code', 'sh': 'code', 'bash': 'code', 'zsh': 'code', 'fish': 'code',
        'ps1': 'code', 'bat': 'code', 'cmd': 'code',
        // Executables
        'exe': 'executable', 'msi': 'executable', 'dmg': 'executable', 'pkg': 'executable',
        'deb': 'executable', 'rpm': 'executable', 'appimage': 'executable', 'snap': 'executable',
        'apk': 'executable', 'ipa': 'executable',
        // Fonts
        'ttf': 'font', 'otf': 'font', 'woff': 'font', 'woff2': 'font', 'eot': 'font',
        // CAD
        'dwg': 'cad', 'dxf': 'cad', 'step': 'cad', 'stp': 'cad', 'iges': 'cad', 'igs': 'cad',
        // 3D
        'stl': '3d', 'obj': '3d', 'fbx': '3d', 'blend': '3d', '3ds': '3d', 'dae': '3d',
        'gltf': '3d', 'glb': '3d',
        // Disk images
        'iso': 'disk', 'img': 'disk', 'vmdk': 'disk', 'qcow2': 'disk', 'vdi': 'disk',
        // Databases
        'db': 'database', 'sqlite': 'database', 'mdb': 'database', 'accdb': 'database',
        // Ebooks
        'epub': 'ebook', 'mobi': 'ebook', 'azw3': 'ebook',
        // Certificates
        'pem': 'cert', 'crt': 'cert', 'cer': 'cert', 'key': 'cert', 'p12': 'cert', 'pfx': 'cert',
        // Config
        'conf': 'config', 'cfg': 'config', 'ini': 'config', 'env': 'config', 'properties': 'config'
      };
      return fileIcons[typeMap[ext] || 'default'];
    }

    uploadArea.addEventListener('click', () => fileInput.click());
    
    fileInput.addEventListener('change', (e) => {
      const files = e.target.files;
      if (files.length > 0) uploadFiles(files);
    });

    downloadAllBtn.addEventListener('click', () => {
      window.location.href = '/api/download-all';
    });

    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
      uploadArea.addEventListener(eventName, preventDefaults, false);
    });

    function preventDefaults(e) {
      e.preventDefault();
      e.stopPropagation();
    }

    uploadArea.addEventListener('dragenter', () => uploadArea.classList.add('dragover'));
    uploadArea.addEventListener('dragleave', () => uploadArea.classList.remove('dragover'));
    uploadArea.addEventListener('drop', (e) => {
      uploadArea.classList.remove('dragover');
      const files = e.dataTransfer.files;
      if (files.length > 0) uploadFiles(files);
    });

    async function uploadFiles(files) {
      progressBar.classList.add('active');
      
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        const formData = new FormData();
        formData.append('file', file);

        try {
          progressFill.style.width = \`\${((i + 1) / files.length) * 100}%\`;
          
          const response = await fetch('/api/upload', {
            method: 'POST',
            body: formData
          });

          if (!response.ok) throw new Error('Upload failed');
        } catch (error) {
          showToast(\`Failed: \${file.name}\`, 'error');
        }
      }

      setTimeout(() => {
        progressBar.classList.remove('active');
        progressFill.style.width = '0%';
        fileInput.value = '';
      }, 300);

      showToast(\`\${files.length} file\${files.length > 1 ? 's' : ''} uploaded\`);
      loadFiles();
    }

    async function loadFiles() {
      try {
        const response = await fetch('/api/files/grouped');
        const data = await response.json();
        renderGroupedFiles(data.sessions);
      } catch (error) {
        showToast('Failed to load files', 'error');
      }
    }

    function getCategory(filename) {
      const ext = filename.split('.').pop().toLowerCase();
      const imageExts = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp', 'tiff', 'tif', 'ico', 'raw', 'psd', 'ai', 'eps'];
      const videoExts = ['mp4', 'mov', 'avi', 'mkv', 'wmv', 'flv', 'webm', 'm4v', 'mpg', 'mpeg', '3gp', 'ogv'];
      
      if (imageExts.includes(ext)) return 'photos';
      if (videoExts.includes(ext)) return 'videos';
      return 'files';
    }

    function formatTimeAgo(timestamp) {
      const date = new Date(timestamp);
      const now = new Date();
      const diffMs = now - date;
      const diffMins = Math.floor(diffMs / 60000);
      const diffHours = Math.floor(diffMs / 3600000);
      const diffDays = Math.floor(diffMs / 86400000);

      if (diffMins < 1) return 'Just now';
      if (diffMins < 60) return diffMins + 'm ago';
      if (diffHours < 24) return diffHours + 'h ago';
      if (diffDays === 1) return 'Yesterday';
      if (diffDays < 7) return diffDays + ' days ago';
      return date.toLocaleDateString();
    }

    function renderGroupedFiles(sessions) {
      if (!sessions || sessions.length === 0) {
        fileList.innerHTML = \`
          <div class="empty-state">
            <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
              <polyline points="13 2 13 9 20 9"></polyline>
            </svg>
            <div>No files yet</div>
          </div>
        \`;
        downloadAllBtn.style.display = 'none';
        return;
      }

      downloadAllBtn.style.display = 'inline-flex';

      const totalFiles = sessions.reduce((sum, s) => sum + s.files.length, 0);

      fileList.innerHTML = sessions.map((session, index) => {
        const sessionNum = sessions.length - index;
        const timeLabel = formatTimeAgo(session.timestamp);
        
        // Group files by category
        const photos = session.files.filter(f => getCategory(f.name) === 'photos');
        const videos = session.files.filter(f => getCategory(f.name) === 'videos');
        const files = session.files.filter(f => getCategory(f.name) === 'files');

        let filesHtml = '';
        
        if (photos.length > 0) {
          filesHtml += \`
            <div class="category-divider">🖼️ Photos (\${photos.length})</div>
            \${photos.map(file => renderFileItem(file)).join('')}
          \`;
        }
        
        if (videos.length > 0) {
          filesHtml += \`
            <div class="category-divider">🎬 Videos (\${videos.length})</div>
            \${videos.map(file => renderFileItem(file)).join('')}
          \`;
        }
        
        if (files.length > 0) {
          filesHtml += \`
            <div class="category-divider">📄 Files (\${files.length})</div>
            \${files.map(file => renderFileItem(file)).join('')}
          \`;
        }

        return \`
          <div class="session">
            <div class="session-header">
              <span class="session-time">Upload #\${sessionNum} · \${timeLabel}</span>
              <div style="display: flex; align-items: center; gap: 0.75rem;">
                <span class="session-count">\${session.files.length} file\${session.files.length > 1 ? 's' : ''}</span>
                <a href="/api/download-session/\${encodeURIComponent(session.timestamp)}" class="btn btn-sm" download title="Download all files in this session">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align: middle; margin-right: 4px;">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                    <polyline points="7 10 12 15 17 10"></polyline>
                    <line x1="12" y1="15" x2="12" y2="3"></line>
                  </svg>
                  All
                </a>
              </div>
            </div>
            <div class="session-files">
              \${filesHtml}
            </div>
          </div>
        \`;
      }).join('');
    }

    function renderFileItem(file) {
      return \`
        <div class="file-item">
          <div class="file-icon">\${getFileIcon(file.name)}</div>
          <div class="file-info">
            <div class="file-name">\${escapeHtml(file.name)}</div>
            <div class="file-meta">\${formatSize(file.size)}</div>
          </div>
          <div class="file-actions">
            <a href="/api/files/\${encodeURIComponent(file.name)}" class="btn" download>Download</a>
            <button class="btn btn-icon" onclick="deleteFile('\${escapeHtml(file.name)}')">✕</button>
          </div>
        </div>
      \`;
    }

    async function deleteFile(name) {
      if (!confirm(\`Delete "\${name}"?\`)) return;

      try {
        const response = await fetch(\`/api/files/\${encodeURIComponent(name)}\`, {
          method: 'DELETE'
        });

        if (!response.ok) throw new Error('Delete failed');

        showToast('File deleted');
        loadFiles();
      } catch (error) {
        showToast('Delete failed', 'error');
      }
    }

    function formatSize(bytes) {
      if (bytes === 0) return '0 B';
      const sizes = ['B', 'KB', 'MB', 'GB'];
      const i = Math.floor(Math.log(bytes) / Math.log(1024));
      return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
    }

    function escapeHtml(text) {
      const div = document.createElement('div');
      div.textContent = text;
      return div.innerHTML;
    }

    function showToast(message, type) {
      toast.textContent = message;
      toast.className = 'toast' + (type === 'error' ? ' error' : '');
      toast.classList.add('show');
      setTimeout(() => toast.classList.remove('show'), 2500);
    }

    loadFiles();
    setInterval(loadFiles, 5000);
  </script>
</body>
</html>`;
}

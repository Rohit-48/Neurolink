import { createReadStream, statSync } from 'fs';
import { basename } from 'path';
import FormData from 'form-data';
import fetch from 'node-fetch';
import { styleText } from 'util';

export interface SendOptions {
  host: string;
  port: number;
  files: string[];
  onProgress?: (fileName: string, progress: number) => void;
  onComplete?: (fileName: string) => void;
  onError?: (fileName: string, error: Error) => void;
}

export async function sendFiles(options: SendOptions): Promise<void> {
  const { host, port, files, onProgress, onComplete, onError } = options;
  const url = `http://${host}:${port}/api/upload`;

  for (const filePath of files) {
    try {
      const fileName = basename(filePath);
      const stats = statSync(filePath);
      const fileSize = stats.size;

      console.log(styleText('gray', `Sending ${fileName} (${formatSize(fileSize)})...`));

      const formData = new FormData();
      formData.append('file', createReadStream(filePath), {
        filename: fileName,
        contentType: 'application/octet-stream',
        knownLength: fileSize
      });

      const response = await fetch(url, {
        method: 'POST',
        body: formData,
        headers: formData.getHeaders()
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      
      if (onComplete) {
        onComplete(fileName);
      }
      
      console.log(styleText('green', `✓ ${fileName} sent successfully`));
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      if (onError) {
        onError(filePath, err);
      }
      console.log(styleText('red', `✗ Failed to send ${basename(filePath)}: ${err.message}`));
    }
  }
}

export async function sendToDevice(device: { host: string; port: number }, filePaths: string[]): Promise<void> {
  console.log(`\n${styleText('bold', 'Sending to:')} ${styleText('cyan', `${device.host}:${device.port}`)}\n`);
  
  await sendFiles({
    host: device.host,
    port: device.port,
    files: filePaths
  });
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
}

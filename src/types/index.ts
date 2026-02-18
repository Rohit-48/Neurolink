export interface DeviceInfo {
  name: string;
  host: string;
  port: number;
  addresses: string[];
  txt: Record<string, string>;
}

export interface FileInfo {
  name: string;
  size: number;
  modified: string;
  type: string;
}

export interface ServerConfig {
  port: number;
  directory: string;
  deviceName: string;
}

export interface SendOptions {
  host: string;
  port: number;
  files: string[];
  onProgress?: (fileName: string, progress: number) => void;
  onComplete?: (fileName: string) => void;
  onError?: (fileName: string, error: Error) => void;
}

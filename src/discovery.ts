import bonjour from 'bonjour';
import { EventEmitter } from 'events';

const SERVICE_TYPE = 'nerolink';
const SERVICE_PROTOCOL = 'tcp';

export class DeviceDiscovery extends EventEmitter {
  private bonjour: bonjour.Bonjour;
  private browser: bonjour.Browser | null = null;
  private service: bonjour.Service | null = null;
  private devices: Map<string, DeviceInfo> = new Map();

  constructor() {
    super();
    this.bonjour = bonjour();
  }

  startAdvertising(name: string, port: number) {
    this.service = this.bonjour.publish({
      name: name,
      type: SERVICE_TYPE,
      protocol: SERVICE_PROTOCOL,
      port: port,
      txt: {
        version: '2.0.0',
        platform: process.platform
      }
    });

    this.service.on('up', () => {
      console.log(`📡 Advertising as "${name}" on port ${port}`);
    });
  }

  startDiscovery() {
    this.browser = this.bonjour.find({ type: SERVICE_TYPE, protocol: SERVICE_PROTOCOL });

    this.browser.on('up', (service: any) => {
      const device: DeviceInfo = {
        name: service.name,
        host: service.host || service.addresses?.[0] || 'unknown',
        port: service.port,
        addresses: service.addresses || [],
        txt: service.txt || {}
      };

      this.devices.set(service.name, device);
      this.emit('deviceUp', device);
    });

    this.browser.on('down', (service: any) => {
      this.devices.delete(service.name);
      this.emit('deviceDown', service.name);
    });
  }

  getDevices(): DeviceInfo[] {
    return Array.from(this.devices.values());
  }

  stop() {
    if (this.browser) {
      this.browser.stop();
    }
    if (this.service) {
      this.service.stop();
    }
    this.bonjour.destroy();
  }
}

export interface DeviceInfo {
  name: string;
  host: string;
  port: number;
  addresses: string[];
  txt: Record<string, string>;
}

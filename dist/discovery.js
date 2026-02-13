import bonjour from 'bonjour';
import { EventEmitter } from 'events';
const SERVICE_TYPE = 'nerolink';
const SERVICE_PROTOCOL = 'tcp';
export class DeviceDiscovery extends EventEmitter {
    bonjour;
    browser = null;
    service = null;
    devices = new Map();
    constructor() {
        super();
        this.bonjour = bonjour();
    }
    startAdvertising(name, port) {
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
        this.browser.on('up', (service) => {
            const device = {
                name: service.name,
                host: service.host || service.addresses?.[0] || 'unknown',
                port: service.port,
                addresses: service.addresses || [],
                txt: service.txt || {}
            };
            this.devices.set(service.name, device);
            this.emit('deviceUp', device);
        });
        this.browser.on('down', (service) => {
            this.devices.delete(service.name);
            this.emit('deviceDown', service.name);
        });
    }
    getDevices() {
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

import bonjour from 'bonjour';
import { EventEmitter } from 'events';
const SERVICE_TYPES = ['neurolink', 'nerolink'];
const SERVICE_PROTOCOL = 'tcp';
export class DeviceDiscovery extends EventEmitter {
    bonjour;
    browsers = [];
    services = [];
    devices = new Map();
    constructor() {
        super();
        this.bonjour = bonjour();
    }
    startAdvertising(name, port) {
        for (const serviceType of SERVICE_TYPES) {
            const service = this.bonjour.publish({
                name: name,
                type: serviceType,
                protocol: SERVICE_PROTOCOL,
                port: port,
                txt: {
                    version: '2.0.0',
                    platform: process.platform
                }
            });
            service.on('up', () => {
                if (serviceType === SERVICE_TYPES[0]) {
                    console.log(`📡 Advertising as "${name}" on port ${port}`);
                }
            });
            this.services.push(service);
        }
    }
    startDiscovery() {
        const makeKey = (service) => `${service.name}|${service.host || service.addresses?.[0] || 'unknown'}|${service.port}`;
        for (const serviceType of SERVICE_TYPES) {
            const browser = this.bonjour.find({ type: serviceType, protocol: SERVICE_PROTOCOL });
            browser.on('up', (service) => {
                const key = makeKey(service);
                if (this.devices.has(key)) {
                    return;
                }
                const device = {
                    name: service.name,
                    host: service.host || service.addresses?.[0] || 'unknown',
                    port: service.port,
                    addresses: service.addresses || [],
                    txt: service.txt || {}
                };
                this.devices.set(key, device);
                this.emit('deviceUp', device);
            });
            browser.on('down', (service) => {
                const key = makeKey(service);
                if (!this.devices.has(key)) {
                    return;
                }
                this.devices.delete(key);
                this.emit('deviceDown', service.name);
            });
            this.browsers.push(browser);
        }
    }
    getDevices() {
        return Array.from(this.devices.values());
    }
    stop() {
        for (const browser of this.browsers) {
            browser.stop();
        }
        for (const service of this.services) {
            service.stop();
        }
        this.browsers = [];
        this.services = [];
        this.bonjour.destroy();
    }
}

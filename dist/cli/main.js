#!/usr/bin/env node
import { Command } from 'commander';
import { styleText } from 'util';
import { resolve } from 'path';
import os from 'os';
import { createServer } from '../server/main.js';
import { DeviceDiscovery } from '../core/discovery.js';
import { showMainMenu } from './menu.js';
const program = new Command();
program
    .name('neurolink')
    .description('Local network file sharing with device discovery')
    .version('2.0.0')
    .option('-p, --port <port>', 'Port to run the server on', '3000')
    .option('-d, --directory <dir>', 'Directory to share files from', './shared')
    .option('-n, --name <name>', 'Device name', os.hostname())
    .action(async (options) => {
    await interactiveMode(options);
});
function printElephantBanner() {
    const art = [
        "                          _.-- ,.--.",
        "                        .'   .'    /",
        "                        | @       |'..--------._",
        "                       /      \\._/              '.",
        "                      /  .-.-                     \\",
        "                     (  /    \\                     \\",
        "                      \\\\      '.                  | #",
        "                       \\\\       \\   -.           /",
        "                        :\\       |    )._____.'   \\",
        "                         \"       |   /  \\  |  \\    )",
        "                                 |   |./'  :__ \\.-'",
        "                                 '--'"
    ];
    const palette = ['cyan', 'blueBright', 'magentaBright', 'yellowBright'];
    console.log('');
    art.forEach((line, idx) => {
        const color = palette[idx % palette.length];
        console.log(styleText(color, line));
    });
    console.log(styleText(['bold', 'greenBright'], '                 NEUROLINK - Painted Elephant Mode'));
    console.log('');
}
async function interactiveMode(options) {
    const port = parseInt(options.port);
    const directory = resolve(options.directory);
    const deviceName = options.name;
    printElephantBanner();
    // Start the server
    const { app } = await createServer(directory, port);
    const { serve } = await import('@hono/node-server');
    serve({
        fetch: app.fetch,
        port
    });
    console.log(styleText(['bold'], '                 N E U R O L I N K   v2.0.0'));
    console.log(styleText('gray', '            Local Network File Sharing'));
    console.log('');
    console.log(styleText('gray', 'Directory: ') + styleText('cyan', directory));
    console.log(styleText('gray', 'Port: ') + styleText('cyan', port.toString()));
    console.log(styleText('gray', 'Device: ') + styleText('cyan', deviceName));
    // Start device discovery
    const discovery = new DeviceDiscovery();
    discovery.startAdvertising(deviceName, port);
    discovery.startDiscovery();
    discovery.on('deviceUp', (device) => {
        console.log('\n' + styleText('blue', '-> ') + styleText('yellow', `Found device: ${device.name} at ${device.host}:${device.port}`));
    });
    discovery.on('deviceDown', (name) => {
        console.log('\n' + styleText('gray', `Device disconnected: ${name}`));
    });
    // Show menu
    await showMainMenu(discovery, port, directory);
}
program.parse();

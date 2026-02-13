import { Command } from 'commander';
import inquirer from 'inquirer';
import ora from 'ora';
import { styleText } from 'util';
import { exec } from 'child_process';
import { promisify } from 'util';
import { createServer, startServer } from './server.js';
import { DeviceDiscovery } from './discovery.js';
import { sendToDevice } from './sender.js';
import { resolve } from 'path';
import os from 'os';

const execAsync = promisify(exec);

const program = new Command();

program
  .name('nerolink')
  .description('Local network file sharing with device discovery')
  .version('2.0.0')
  .option('-p, --port <port>', 'Port to run the server on', '3000')
  .option('-d, --directory <dir>', 'Directory to share files from', './shared')
  .option('-n, --name <name>', 'Device name', os.hostname())
  .action(async (options) => {
    await interactiveMode(options);
  });

async function interactiveMode(options: any) {
  const port = parseInt(options.port);
  const directory = resolve(options.directory);
  const deviceName = options.name;

  // Start the server
  const { app } = await createServer(directory, port);
  const { serve } = await import('@hono/node-server');
  
  serve({
    fetch: app.fetch,
    port
  });
  
  console.log('\n' + styleText(['bold', 'green'], '📡 NeroLink Server Started'));
  console.log(styleText('gray', 'Directory: ') + styleText('cyan', directory));
  console.log(styleText('gray', 'Port: ') + styleText('cyan', port.toString()));
  console.log(styleText('gray', 'Device: ') + styleText('cyan', deviceName));

  // Start device discovery
  const discovery = new DeviceDiscovery();
  discovery.startAdvertising(deviceName, port);
  discovery.startDiscovery();

  discovery.on('deviceUp', (device) => {
    console.log('\n' + styleText('blue', '↳ ') + styleText('yellow', `Found device: ${device.name} at ${device.host}:${device.port}`));
  });

  discovery.on('deviceDown', (name) => {
    console.log('\n' + styleText('gray', `Device disconnected: ${name}`));
  });

  // Show menu
  await showMainMenu(discovery, port, directory);
}

async function showMainMenu(discovery: DeviceDiscovery, port: number, directory: string) {
  const choices = [
    { name: '📤 Send files to a device', value: 'send' },
    { name: '📋 List available devices', value: 'list' },
    { name: '🌐 Open Web UI', value: 'web' },
    { name: 'ℹ️  Show server info', value: 'info' },
    { name: '❌ Exit', value: 'exit' }
  ];

  while (true) {
    const inquirerMod = await import('inquirer');
    const { action } = await inquirerMod.default.prompt([{
      type: 'list',
      name: 'action',
      message: 'What would you like to do?',
      choices,
      loop: false
    }] as any);

    switch (action) {
      case 'send':
        await handleSend(discovery);
        break;
      case 'list':
        await handleList(discovery);
        break;
      case 'web':
        await handleOpenWeb(port);
        break;
      case 'info':
        await handleShowInfo(port, directory);
        break;
      case 'exit':
        console.log('\n' + styleText('gray', 'Shutting down...'));
        discovery.stop();
        process.exit(0);
    }
  }
}

async function handleSend(discovery: DeviceDiscovery) {
  const devices = discovery.getDevices();
  
  if (devices.length === 0) {
    console.log('\n' + styleText('yellow', '⚠️  No devices found. Make sure other devices are running nerolink.'));
    return;
  }

  const deviceChoices = devices.map((d, i) => ({
    name: `${d.name} (${d.host}:${d.port})`,
    value: d
  }));

  const inquirerMod = await import('inquirer');
  const { selectedDevice } = await inquirerMod.default.prompt([{
    type: 'list',
    name: 'selectedDevice',
    message: 'Select a device to send files to:',
    choices: deviceChoices
  }] as any);

  const { filePaths } = await inquirer.prompt([{
    type: 'input',
    name: 'filePaths',
    message: 'Enter file paths (comma-separated):',
    filter: (input: string) => input.split(',').map(p => p.trim()).filter(p => p)
  }]);

  if (filePaths.length === 0) {
    console.log(styleText('yellow', 'No files specified.'));
    return;
  }

  await sendToDevice(selectedDevice, filePaths);
}

async function handleList(discovery: DeviceDiscovery) {
  const devices = discovery.getDevices();
  
  console.log('\n' + styleText(['bold'], '📱 Available Devices:'));
  console.log(styleText('gray', '─'.repeat(50)));

  if (devices.length === 0) {
    console.log(styleText('gray', 'No devices found.'));
  } else {
    devices.forEach((device, i) => {
      console.log(`${i + 1}. ${styleText('cyan', device.name)}`);
      console.log(`   Host: ${styleText('yellow', device.host)}`);
      console.log(`   Port: ${styleText('yellow', device.port.toString())}`);
      console.log(`   Addresses: ${device.addresses.join(', ')}`);
      console.log();
    });
  }
  
  console.log(styleText('gray', '─'.repeat(50)));
}

async function handleOpenWeb(port: number) {
  const url = `http://localhost:${port}`;
  
  console.log('\n' + styleText('blue', '🌐 Opening ') + styleText('cyan', url));
  
  try {
    const platform = process.platform;
    const cmd = platform === 'darwin' ? 'open' : platform === 'win32' ? 'start' : 'xdg-open';
    await execAsync(`${cmd} ${url}`);
  } catch (error) {
    console.log(styleText('gray', `Please open ${url} manually in your browser.`));
  }
}

async function handleShowInfo(port: number, directory: string) {
  const interfaces = os.networkInterfaces();
  
  console.log('\n' + styleText(['bold'], 'ℹ️  Server Information:'));
  console.log(styleText('gray', '─'.repeat(50)));
  console.log(`Directory: ${styleText('cyan', directory)}`);
  console.log(`Port: ${styleText('cyan', port.toString())}`);
  console.log('\nAccess URLs:');
  
  for (const [name, addrs] of Object.entries(interfaces)) {
    for (const iface of addrs || []) {
      if (iface.family === 'IPv4' && !iface.internal) {
        console.log(`  ${styleText('blue', '➜')} http://${styleText('yellow', iface.address)}:${port}`);
      }
    }
  }
  
  console.log(styleText('gray', '─'.repeat(50)));
}

program.parse();

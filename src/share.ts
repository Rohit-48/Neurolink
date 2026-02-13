#!/usr/bin/env node
import { Command } from 'commander';
import { styleText } from 'util';
import { DeviceDiscovery } from './discovery.js';
import { sendToDevice } from './sender.js';
import { resolve } from 'path';
import { statSync } from 'fs';
import ora from 'ora';

const program = new Command();

program
  .name('neroshare')
  .description('Send files to devices on your local network')
  .version('2.0.0');

program
  .command('send')
  .description('Send files or folders to a device')
  .argument('<paths...>', 'Files or folders to send')
  .option('-d, --device <name>', 'Target device name (auto-discover if not specified)')
  .option('-h, --host <host>', 'Target host (bypass discovery)')
  .option('-p, --port <port>', 'Target port', '3000')
  .action(async (paths: string[], options) => {
    await sendCommand(paths, options);
  });

program
  .command('devices')
  .description('List available devices on the network')
  .option('-t, --timeout <seconds>', 'Discovery timeout', '5')
  .action(async (options) => {
    await listDevicesCommand(parseInt(options.timeout));
  });

async function sendCommand(paths: string[], options: any) {
  // Resolve all paths
  const filePaths: string[] = [];
  
  for (const path of paths) {
    const resolvedPath = resolve(path);
    try {
      const stats = statSync(resolvedPath);
      if (stats.isFile()) {
        filePaths.push(resolvedPath);
      } else if (stats.isDirectory()) {
        console.log(styleText('yellow', `Note: ${path} is a directory. Currently only files are supported.`));
      }
    } catch (error) {
      console.log(styleText('red', `Error: Cannot access ${path}`));
      process.exit(1);
    }
  }

  if (filePaths.length === 0) {
    console.log(styleText('red', 'Error: No valid files to send.'));
    process.exit(1);
  }

  console.log(styleText('bold', '\n📤 NeroShare'));
  console.log(styleText('gray', `Sending ${filePaths.length} file(s)...\n`));

  // Determine target device
  let targetDevice: { host: string; port: number } | null = null;

  if (options.host) {
    // Use specified host and port
    targetDevice = {
      host: options.host,
      port: parseInt(options.port)
    };
  } else if (options.device) {
    // Discover and find by name
    const spinner = ora('Discovering devices...').start();
    const discovery = new DeviceDiscovery();
    discovery.startDiscovery();

    // Wait a bit for discovery
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    const devices = discovery.getDevices();
    const device = devices.find(d => d.name.toLowerCase() === options.device.toLowerCase());
    
    discovery.stop();
    spinner.stop();

    if (!device) {
      console.log(styleText('red', `\nError: Device "${options.device}" not found.`));
      console.log(styleText('gray', 'Run "neroshare devices" to see available devices.'));
      process.exit(1);
    }

    targetDevice = device;
  } else {
    // Interactive device selection
    const spinner = ora('Discovering devices...').start();
    const discovery = new DeviceDiscovery();
    discovery.startDiscovery();

    await new Promise(resolve => setTimeout(resolve, 3000));
    
    const devices = discovery.getDevices();
    discovery.stop();
    spinner.stop();

    if (devices.length === 0) {
      console.log(styleText('red', '\nError: No devices found on the network.'));
      console.log(styleText('gray', 'Make sure the target device is running nerolink.'));
      process.exit(1);
    }

    if (devices.length === 1) {
      targetDevice = devices[0];
      console.log(styleText('green', `\nFound device: ${devices[0].name}`));
    } else {
      const inquirer = await import('inquirer');
      const { selectedDevice } = await inquirer.default.prompt([{
        type: 'list',
        name: 'selectedDevice',
        message: 'Select a device:',
        choices: devices.map(d => ({
          name: `${d.name} (${d.host}:${d.port})`,
          value: d
        }))
      }]);
      targetDevice = selectedDevice;
    }
  }

  // Send files
  if (!targetDevice) {
    console.log(styleText('red', 'Error: No target device selected.'));
    process.exit(1);
  }
  
  await sendToDevice(targetDevice, filePaths);
  
  console.log('\n' + styleText('green', '✓ All files sent!'));
}

async function listDevicesCommand(timeoutSeconds: number) {
  console.log(styleText('bold', '\n📱 Discovering devices...'));
  console.log(styleText('gray', `Scanning for ${timeoutSeconds} seconds...\n`));

  const spinner = ora('Searching...').start();
  const discovery = new DeviceDiscovery();
  discovery.startDiscovery();

  // Wait for specified timeout
  await new Promise(resolve => setTimeout(resolve, timeoutSeconds * 1000));
  
  const devices = discovery.getDevices();
  discovery.stop();
  spinner.stop();

  if (devices.length === 0) {
    console.log(styleText('yellow', 'No devices found.'));
    console.log(styleText('gray', 'Make sure other devices are running nerolink.'));
  } else {
    console.log(styleText(['bold'], `Found ${devices.length} device(s):\n`));
    
    devices.forEach((device, i) => {
      console.log(`${i + 1}. ${styleText('cyan', device.name)}`);
      console.log(`   Host: ${styleText('yellow', device.host)}:${device.port}`);
      console.log(`   Addresses: ${device.addresses.join(', ')}`);
      console.log();
    });
  }
}

program.parse();

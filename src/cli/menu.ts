import inquirer from 'inquirer';
import { styleText } from 'util';
import { exec } from 'child_process';
import { promisify } from 'util';
import { DeviceDiscovery } from '../core/discovery.js';
import { sendToDevice } from '../core/sender.js';

const execAsync = promisify(exec);

export async function showMainMenu(discovery: DeviceDiscovery, port: number, directory: string) {
  const choices = [
    { name: 'Send files to a device', value: 'send' },
    { name: 'List available devices', value: 'list' },
    { name: 'Open Web UI', value: 'web' },
    { name: 'Show server info', value: 'info' },
    { name: 'Exit', value: 'exit' }
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
        await handleExit(discovery, directory);
        break;
    }
  }
}

async function handleSend(discovery: DeviceDiscovery) {
  const devices = discovery.getDevices();
  
  if (devices.length === 0) {
    console.log('\n' + styleText('yellow', 'Warning: No devices found. Make sure other devices are running neurolink.'));
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

  const { filePaths } = await inquirerMod.default.prompt([{
    type: 'input',
    name: 'filePaths',
    message: 'Enter file paths (comma-separated):',
    filter: (input: string) => input.split(',').map(p => p.trim()).filter(p => p)
  }] as any);

  if (filePaths.length === 0) {
    console.log(styleText('yellow', 'No files specified.'));
    return;
  }

  await sendToDevice(selectedDevice, filePaths);
}

async function handleList(discovery: DeviceDiscovery) {
  const devices = discovery.getDevices();
  
  console.log('\n' + styleText(['bold'], 'Available Devices:'));
  console.log(styleText('gray', '-'.repeat(50)));

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
  
  console.log(styleText('gray', '-'.repeat(50)));
}

async function handleOpenWeb(port: number) {
  const url = `http://localhost:${port}`;
  
  console.log('\n' + styleText('blue', 'Opening ') + styleText('cyan', url));
  
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
  
  console.log('\n' + styleText(['bold'], 'Server Information:'));
  console.log(styleText('gray', '-'.repeat(50)));
  console.log(`Directory: ${styleText('cyan', directory)}`);
  console.log(`Port: ${styleText('cyan', port.toString())}`);
  console.log('\nAccess URLs:');
  
  for (const [name, addrs] of Object.entries(interfaces)) {
    for (const iface of addrs || []) {
      if (iface.family === 'IPv4' && !iface.internal) {
        console.log(`  ${styleText('blue', '->')} http://${styleText('yellow', iface.address)}:${port}`);
      }
    }
  }
  
  console.log(styleText('gray', '-'.repeat(50)));
}

async function handleExit(discovery: DeviceDiscovery, directory: string) {
  const inquirerMod = await import('inquirer');
  
  const { clearFiles } = await inquirerMod.default.prompt([{
    type: 'confirm',
    name: 'clearFiles',
    message: 'Clear all shared files before exiting?',
    default: false
  }] as any);
  
  if (clearFiles) {
    const { readdir, unlink } = await import('fs/promises');
    const { join } = await import('path');
    
    try {
      const files = await readdir(directory);
      let deletedCount = 0;
      
      for (const file of files) {
        const filePath = join(directory, file);
        try {
          await unlink(filePath);
          deletedCount++;
        } catch (e) {
          // Skip files that can't be deleted
        }
      }
      
      console.log(styleText('green', `Cleared ${deletedCount} files from shared directory`));
    } catch (error) {
      console.log(styleText('yellow', 'Warning: Could not clear shared files'));
    }
  }
  
  console.log('\n' + styleText('gray', 'Shutting down...'));
  discovery.stop();
  process.exit(0);
}

import os from 'os';

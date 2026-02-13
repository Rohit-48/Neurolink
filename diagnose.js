#!/usr/bin/env node
import { getNetworkInfo, troubleshootConnection } from './dist/network.js';
import { styleText } from 'util';
import os from 'os';
import { execSync } from 'child_process';

const PORT = process.argv[2] || '3000';

console.log(styleText(['bold', 'blue'], '\n🔧 Network Diagnostic Tool'));
console.log(styleText('gray', '═'.repeat(50)));

// Check if server is running
console.log('\n📡 Checking if server is running on port ' + PORT + '...');
try {
  const result = execSync(`ss -tlnp | grep :${PORT} || netstat -tlnp 2>/dev/null | grep :${PORT} || echo "NOT_FOUND"`, {
    encoding: 'utf-8',
    timeout: 2000
  });
  
  if (result.includes('NOT_FOUND')) {
    console.log('❌ No server found on port ' + PORT);
    console.log('   Start the server with: npm start');
    process.exit(1);
  } else {
    console.log('✅ Server is running on port ' + PORT);
  }
} catch (e) {
  console.log('⚠️  Could not check server status');
}

// Get network info
const addresses = getNetworkInfo(parseInt(PORT));

// Check firewall
try {
  const ufwStatus = execSync('sudo ufw status 2>/dev/null || echo "not_installed"', { 
    encoding: 'utf-8',
    timeout: 1000 
  });
  
  console.log('\n🛡️  Firewall Status:');
  if (ufwStatus.includes('not_installed')) {
    console.log('   UFW not installed (checking iptables...)');
  } else if (ufwStatus.includes('Status: active')) {
    console.log('   UFW is ACTIVE');
    console.log('   You may need to allow port ' + PORT + ':');
    console.log('   sudo ufw allow ' + PORT + '/tcp');
  } else {
    console.log('   UFW is inactive');
  }
} catch {
  console.log('   Could not check firewall status');
}

// Show troubleshooting
troubleshootConnection(parseInt(PORT), addresses);

console.log('\n' + styleText('green', '💡 Quick Test:'));
console.log('   1. Open a browser on this computer and go to:');
console.log(`      http://localhost:${PORT}`);
console.log('   2. If that works, try from your phone using one of the URLs above');
console.log('   3. Make sure your phone is on the same WiFi network (192.168.0.x)');
console.log('   4. Temporarily disable mobile data on your phone\n');

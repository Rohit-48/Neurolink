import { styleText } from 'util';
import { execSync } from 'child_process';
import os from 'os';
import QRCode from 'qrcode';
export async function generateQRCode(url) {
    try {
        // Generate QR code as ASCII art
        const qr = await QRCode.toString(url, {
            type: 'terminal',
            small: true
        });
        return qr;
    }
    catch (error) {
        return '';
    }
}
export function getNetworkInfo(port) {
    const interfaces = os.networkInterfaces();
    const addresses = [];
    console.log('\n' + styleText(['bold'], '🌐 Network Interface Check:'));
    console.log(styleText('gray', '─'.repeat(50)));
    for (const [name, addrs] of Object.entries(interfaces)) {
        for (const iface of addrs || []) {
            if (iface.family === 'IPv4' && !iface.internal) {
                const url = `http://${iface.address}:${port}`;
                addresses.push({ name, ip: iface.address, url });
                // Determine interface type
                let type = '🔌 Ethernet';
                if (name.startsWith('wl') || name.startsWith('wi') || name.includes('wifi')) {
                    type = '📶 WiFi';
                }
                else if (name.includes('tailscale') || name.includes('tun')) {
                    type = '🔒 VPN';
                }
                console.log(`${type} ${styleText('cyan', name)}`);
                console.log(`   IP: ${styleText('yellow', iface.address)}`);
                console.log(`   URL: ${styleText('green', url)}`);
                console.log();
            }
        }
    }
    return addresses;
}
export function troubleshootConnection(port, addresses) {
    console.log(styleText(['bold'], '🔍 Troubleshooting:'));
    console.log(styleText('gray', '─'.repeat(50)));
    // Check if server is accessible from localhost
    try {
        execSync(`curl -s -o /dev/null -w "%{http_code}" http://localhost:${port}`, {
            timeout: 2000,
            stdio: 'pipe'
        });
        console.log('✅ Server responding on localhost');
    }
    catch {
        console.log('❌ Server not responding on localhost');
    }
    // Check firewall
    console.log('\n📋 Common fixes:');
    console.log('1. Ensure phone is on SAME WiFi network (not mobile data)');
    console.log('2. Try disabling mobile data on phone temporarily');
    console.log('3. Check if your router has "AP Isolation" enabled (disables device-to-device communication)');
    console.log('4. Try pinging your phone from computer:');
    if (addresses.length > 0) {
        console.log(`   ping ${addresses[0].ip}`);
    }
    console.log('\n5. Firewall commands (run with sudo):');
    console.log(`   sudo ufw allow ${port}/tcp`);
    console.log(`   sudo iptables -I INPUT -p tcp --dport ${port} -j ACCEPT`);
    console.log('\n6. Try a different port:');
    console.log('   npm start -- -p 8080');
}
